use crate::{
    config::mail::Email,
    dto::user::{
        RegisterReq, ResetPasswdReq, SendEmailReq, SetNameReq, UserResp, UserRespWithToken,
        ValidateCodeEmailTemplate,
    },
    model::{
        prelude::Users,
        sea_orm_active_enums::{UserGender, UserType},
        users,
    },
    utils::{
        jwt::{self, Claims},
        mail,
        validate_code::{gen_validate_code, get_validate_code},
    },
};
use anyhow::Context;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use spring_mail::Mailer;
use spring_redis::Redis;
use spring_sea_orm::DbConn;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::{KnownWebError, Result},
    extractor::Component,
};
use spring_web::{extractor::Config, patch, post};

#[post("/user")]
async fn register(
    Component(mut redis): Component<Redis>,
    Component(db): Component<DbConn>,
    Json(body): Json<RegisterReq>,
) -> Result<Json<UserResp>> {
    let code = get_validate_code(&mut redis, &body.email).await?;

    match code {
        None => return Err(KnownWebError::bad_request("验证码已过期"))?,
        Some(code) => {
            if code != body.validate_code {
                return Err(KnownWebError::bad_request("验证码错误"))?;
            }
        }
    }

    let user = Users::find()
        .filter(users::Column::Email.eq(&body.email))
        .one(&db)
        .await
        .context("select user from db failed")?;
    if user.is_some() {
        return Err(KnownWebError::bad_request("邮箱已被注册"))?;
    }
    let user = users::ActiveModel {
        id: NotSet,
        username: Set(body.name),
        email: Set(Some(body.email)),
        password: Set(Some(body.password)),
        gender: Set(UserGender::Unknown),
        r#type: Set(UserType::Normal),
        mfa: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .context("user insert failed")?;

    Ok(Json(user.into()))
}

#[post("/user/register-validate-code")]
async fn register_validate_code(
    Component(mut redis): Component<Redis>,
    Component(mailer): Component<Mailer>,
    Config(email): Config<Email>,
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

#[post("/user/reset-validate-code")]
async fn reset_validate_code(
    Component(mut redis): Component<Redis>,
    Component(mailer): Component<Mailer>,
    Config(email): Config<Email>,
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

#[post("/user/password")]
async fn reset_password(
    Component(mut redis): Component<Redis>,
    Component(db): Component<DbConn>,
    Json(req): Json<ResetPasswdReq>,
) -> Result<impl IntoResponse> {
    let code = get_validate_code(&mut redis, &req.email)
        .await?
        .ok_or_else(|| KnownWebError::bad_request("验证码已过期"))?;

    if code != req.validate_code {
        Err(KnownWebError::bad_request("验证码错误"))?;
    }

    let u = Users::find()
        .filter(users::Column::Email.eq(&req.email))
        .one(&db)
        .await
        .with_context(|| format!("query user by email failed: {}", req.email))?
        .ok_or_else(|| KnownWebError::not_found("用户不存在"))?;

    let u = users::ActiveModel {
        id: Set(u.id),
        password: Set(Some(req.password)),
        ..Default::default()
    }
    .update(&db)
    .await
    .with_context(|| format!("user#{} change password failed", u.id))?;

    let claims = Claims::new(u.id);
    let token = jwt::encode(claims)?;

    Ok(Json(UserRespWithToken::new(u, token)))
}

#[patch("/user/name")]
async fn set_name(
    claims: Claims,
    Component(db): Component<DbConn>,
    Json(req): Json<SetNameReq>,
) -> Result<impl IntoResponse> {
    let u = Users::find_by_id(claims.uid)
        .one(&db)
        .await
        .with_context(|| format!("query user by id#{} failed", claims.uid))?
        .ok_or_else(|| KnownWebError::not_found("用户不存在"))?;

    let u = users::ActiveModel {
        id: Set(u.id),
        username: Set(req.name),
        ..Default::default()
    }
    .update(&db)
    .await
    .with_context(|| format!("change name for user#{} failed", u.id))?;

    tracing::debug!("user#{} change name success", u.id);

    Ok(Json(true))
}
