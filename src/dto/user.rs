use crate::model::{
    sea_orm_active_enums::{UserGender, UserType},
    users,
};
use askama::Template;
use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct AuthenticationToken {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterReq {
    #[validate(length(max = 30, message = "用户名不能超过30个字符"))]
    pub name: String,

    #[validate(
        email(message = "邮箱格式不正确"),
        length(max = 60, message = "邮箱过长")
    )]
    pub email: String,

    #[validate(length(max = 32, message = "密码过长"))]
    pub password: String,

    #[validate(length(max = 8, message = "验证码过长"))]
    pub validate_code: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ResetPasswdReq {
    #[validate(
        email(message = "邮箱格式不正确"),
        length(max = 60, message = "邮箱过长")
    )]
    pub email: String,
    #[validate(length(max = 32, message = "密码过长"))]
    pub password: String,
    #[validate(length(max = 8, message = "验证码过长"))]
    pub validate_code: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct SendEmailReq {
    #[validate(
        email(message = "邮箱格式不正确"),
        length(max = 60, message = "邮箱过长")
    )]
    pub email: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct SetNameReq {
    #[validate(length(max = 30, message = "用户名不能超过30个字符"))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResp {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub gender: UserGender,
    pub r#type: UserType,
    pub avatar: Option<String>,
    pub mfa: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl From<users::Model> for UserResp {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            name: user.username,
            email: user.email,
            gender: user.gender,
            r#type: user.r#type,
            avatar: user.avatar,
            mfa: user.mfa,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserRespWithToken {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub gender: UserGender,
    pub r#type: UserType,
    pub avatar: Option<String>,
    pub mfa: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub token: String,
}

impl UserRespWithToken {
    pub fn new<S: Into<String>>(user: users::Model, token: S) -> Self {
        Self {
            id: user.id,
            name: user.username,
            email: user.email,
            gender: user.gender,
            r#type: user.r#type,
            avatar: user.avatar,
            mfa: user.mfa,
            created_at: user.created_at,
            updated_at: user.updated_at,
            token: token.into(),
        }
    }
}

#[derive(Template)]
#[template(path = "mail/validate_code.html")]
pub struct ValidateCodeEmailTemplate<'a> {
    pub tip: &'a str,
    pub code: &'a str,
}
