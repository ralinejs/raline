use crate::{
    views::user::{AuthenticationToken, UserResp, UserRespWithToken},
    router::Locale,
    model::{prelude::Users, users},
    utils::jwt::{self, Claims},
};
use anyhow::Context;
use rust_i18n::t;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use spring_sea_orm::DbConn;
use spring_web::{axum::response::IntoResponse, get, post};
use spring_web::{
    axum::Json,
    error::{KnownWebError, Result},
    extractor::Component,
};

#[post("/api/token")]
async fn login(
    Locale(lang): Locale,
    Component(db): Component<DbConn>,
    Json(body): Json<AuthenticationToken>,
) -> Result<impl IntoResponse> {
    let user = Users::find()
        .filter(users::Column::Email.eq(&body.email))
        .one(&db)
        .await
        .context("query db failed")?
        .ok_or_else(|| KnownWebError::unauthorized(t!("user_not_exists", locale = lang)))?;

    match &user.password {
        Some(password) => {
            if password != &body.password {
                Err(KnownWebError::unauthorized(t!(
                    "error_password",
                    locale = lang
                )))?;
            }
        }
        None => Err(KnownWebError::unauthorized(t!(
            "not_password",
            locale = lang
        )))?,
    }

    let claims = Claims::new(&user);
    let token = jwt::encode(claims)?;

    Ok(Json(UserRespWithToken::new(user, token)))
}

#[get("/api/token")]
async fn current_user(
    claims: Claims,
    Locale(lang): Locale,
    Component(db): Component<DbConn>,
) -> Result<impl IntoResponse> {
    let user = Users::find_by_id(claims.uid)
        .one(&db)
        .await
        .with_context(|| format!("find user by id#{}", claims.uid))?
        .ok_or_else(|| KnownWebError::not_found(t!("user_not_exists", locale = lang)))?;

    Ok(Json(UserResp::from(user)))
}

#[get("/api/token/2fa")]
async fn mfa() -> Result<impl IntoResponse> {
    Ok(Json(json!({"x":"x"})))
}
