use crate::model::users;
use askama::Template;
use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct AuthenticationToken {
    pub email: String,
    pub passwd: String,
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
    pub passwd: String,

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
    pub passwd: String,
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
    pub created: DateTime,
    pub modified: DateTime,
    pub name: String,
    pub email: Option<String>,
    pub locked: bool,
    pub last_login: Option<String>,
}

impl From<users::Model> for UserResp {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            created: user.created_at,
            modified: user.updated_at,
            name: user.username,
            email: user.email,
            last_login: user.last_login,
        }
    }
}

#[derive(Template)]
#[template(path = "mail/validate_code.html")]
pub struct ValidateCodeEmailTemplate<'a> {
    pub tip: &'a str,
    pub code: &'a str,
}
