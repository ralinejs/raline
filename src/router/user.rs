use super::Locale;
use crate::{
    config::mail::EmailConfig,
    views::user::{
        RegisterReq, ResetPasswdReq, SendEmailReq, UpdateUserReq, UserQuery, UserResp,
        UserRespWithToken, ValidateCodeEmailTemplate,
    },
    model::{
        prelude::Users,
        sea_orm_active_enums::{UserGender, UserType},
        users,
    },
    utils::{
        avatar::avatar_url,
        jwt::{self, Claims, OptionalClaims},
        mail,
        validate_code::{gen_validate_code, get_validate_code},
    },
};
use anyhow::Context;
use rust_i18n::t;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use spring_mail::Mailer;
use spring_redis::Redis;
use spring_sea_orm::{
    pagination::{Pagination, PaginationExt},
    DbConn,
};
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::{KnownWebError, Result},
    extractor::{Component, Query},
    get, put,
};
use spring_web::{extractor::Config, post};

#[get("/api/user")]
async fn get_users(
    claims: OptionalClaims,
    Component(db): Component<DbConn>,
    Query(q): Query<UserQuery>,
    Locale(lang): Locale,
) -> Result<impl IntoResponse> {
    match &*claims {
        None => Err(KnownWebError::forbidden(t!("no_permission", locale = lang)))?,
        Some(c) => c,
    };
    let p = Pagination {
        page: if q.page > 0 { q.page - 1 } else { 0 },
        size: q.size,
    };
    let page = match q.email {
        Some(email) => Users::find()
            .filter(users::Column::Email.eq(email))
            .page(&db, p)
            .await
            .context("fetch user failed")?,
        None => Users::find()
            .page(&db, p)
            .await
            .context("fetch user failed")?,
    };
    Ok(Json(page))
}

#[post("/api/user")]
async fn register(
    Component(mut redis): Component<Redis>,
    Component(db): Component<DbConn>,
    Locale(lang): Locale,
    Json(body): Json<RegisterReq>,
) -> Result<Json<UserResp>> {
    let code = get_validate_code(&mut redis, &body.email).await?;

    match code {
        None => Err(KnownWebError::bad_request(t!(
            "expired_code",
            locale = lang
        )))?,
        Some(code) => {
            if code != body.validate_code {
                Err(KnownWebError::bad_request(t!("error_code", locale = lang)))?
            }
        }
    }

    let user = Users::find()
        .filter(users::Column::Email.eq(&body.email))
        .one(&db)
        .await
        .context("select user from db failed")?;
    if user.is_some() {
        Err(KnownWebError::bad_request(t!(
            "user_registered",
            locale = lang
        )))?
    }
    let avatar = avatar_url(&body.name, &body.email);
    let user = users::ActiveModel {
        id: NotSet,
        username: Set(body.name),
        email: Set(Some(body.email)),
        password: Set(Some(body.password)),
        gender: Set(UserGender::Unknown),
        r#type: Set(UserType::Normal),
        mfa: Set(false),
        avatar: Set(Some(avatar)),
        ..Default::default()
    }
    .insert(&db)
    .await
    .context("user insert failed")?;

    Ok(Json(user.into()))
}

#[post("/api/user/register-validate-code")]
async fn register_validate_code(
    Component(mut redis): Component<Redis>,
    Component(mailer): Component<Mailer>,
    Config(email): Config<EmailConfig>,
    Json(body): Json<SendEmailReq>,
) -> Result<impl IntoResponse> {
    let code = gen_validate_code(&mut redis, &body.email).await?;

    let template = ValidateCodeEmailTemplate {
        tip: "欢迎您注册我们的服务，您的注册验证码(5分钟内有效)是：",
        code: code.as_str(),
    };
    let from = email.from;
    let to = body.email;
    let success = mail::send_mail(&mailer, &from, &to, "注册验证码", &template).await?;

    Ok(Json(success))
}

#[post("/api/user/reset-validate-code")]
async fn reset_validate_code(
    Component(mut redis): Component<Redis>,
    Component(mailer): Component<Mailer>,
    Config(email): Config<EmailConfig>,
    Json(body): Json<SendEmailReq>,
) -> Result<impl IntoResponse> {
    let code = gen_validate_code(&mut redis, &body.email).await?;

    let template = ValidateCodeEmailTemplate {
        tip: "请确认您是否需要重置密码，重置密码请在系统中输入以下验证码(5分钟内有效)：",
        code: code.as_str(),
    };
    let from = email.from;
    let to = body.email;
    let success = mail::send_mail(&mailer, &from, &to, "重置密码的验证码", &template).await?;

    Ok(Json(success))
}

#[post("/api/user/password")]
async fn reset_password(
    Component(mut redis): Component<Redis>,
    Component(db): Component<DbConn>,
    Locale(lang): Locale,
    Json(req): Json<ResetPasswdReq>,
) -> Result<impl IntoResponse> {
    let code = get_validate_code(&mut redis, &req.email)
        .await?
        .ok_or_else(|| KnownWebError::bad_request(t!("expired_code", locale = lang)))?;

    if code != req.validate_code {
        Err(KnownWebError::bad_request(t!("error_code", locale = lang)))?
    }

    let u = Users::find()
        .filter(users::Column::Email.eq(&req.email))
        .one(&db)
        .await
        .with_context(|| format!("query user by email failed: {}", req.email))?
        .ok_or_else(|| KnownWebError::not_found(t!("user_not_exists", locale = lang)))?;

    let u = users::ActiveModel {
        id: Set(u.id),
        password: Set(Some(req.password)),
        ..Default::default()
    }
    .update(&db)
    .await
    .with_context(|| format!("user#{} change password failed", u.id))?;

    let claims = Claims::new(&u);
    let token = jwt::encode(claims)?;

    Ok(Json(UserRespWithToken::new(u, token)))
}

#[put("/api/user")]
async fn update_user(
    claims: Claims,
    Component(db): Component<DbConn>,
    Locale(lang): Locale,
    Json(req): Json<UpdateUserReq>,
) -> Result<impl IntoResponse> {
    let u = Users::find_by_id(claims.uid)
        .one(&db)
        .await
        .with_context(|| format!("query user by id#{} failed", claims.uid))?
        .ok_or_else(|| KnownWebError::not_found(t!("user_not_exists", locale = lang)))?;

    let u = users::ActiveModel {
        id: Set(u.id),
        username: match req.name {
            Some(name) => Set(name),
            None => NotSet,
        },
        gender: match req.gender {
            Some(gender) => Set(gender),
            None => NotSet,
        },
        password: match req.password {
            Some(password) => Set(Some(password)),
            None => NotSet,
        },
        avatar: match req.avatar {
            Some(avatar) => Set(Some(avatar)),
            None => NotSet,
        },
        ..Default::default()
    }
    .update(&db)
    .await
    .with_context(|| format!("change name for user#{} failed", u.id))?;

    tracing::debug!("user#{} change name success", u.id);

    Ok(Json(UserResp::from(u)))
}
