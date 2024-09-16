use crate::{
    dto::user::{AuthenticationToken, UserRespWithToken},
    model::{prelude::Users, users},
    utils::jwt::{self, Claims},
};
use anyhow::Context;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use spring_sea_orm::DbConn;
use spring_web::{axum::response::IntoResponse, post};
use spring_web::{
    axum::Json,
    error::{KnownWebError, Result},
    extractor::Component,
};

#[post("/token")]
async fn login(
    Component(db): Component<DbConn>,
    Json(body): Json<AuthenticationToken>,
) -> Result<impl IntoResponse> {
    let user = Users::find()
        .filter(users::Column::Email.eq(&body.email))
        .one(&db)
        .await
        .context("query db failed")?
        .ok_or_else(|| KnownWebError::unauthorized("用户不存在，请先注册"))?;

    match &user.password {
        Some(password) => {
            if password != &body.password {
                Err(KnownWebError::unauthorized("密码错误"))?;
            }
        }
        None => Err(KnownWebError::unauthorized(
            "该账号未初始化密码，请尝试其他方式登录",
        ))?,
    }

    let claims = Claims::new(user.id);
    let token = jwt::encode(claims)?;

    Ok(Json(UserRespWithToken::new(user, token)))
}
