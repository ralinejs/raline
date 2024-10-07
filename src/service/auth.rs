use crate::config::{auth::AuthConfig, RalineConfig};
use crate::model::prelude::UserOauth;
use crate::model::sea_orm_active_enums::{UserGender, UserType};
use crate::model::{user_oauth, users};
use anyhow::Context;
use just_auth::{qq, AuthUser, GenericAuthAction};
use sea_orm::sqlx::types::chrono::DateTime;
use sea_orm::{ActiveModelTrait, QueryFilter, Set};
use sea_orm::{ColumnTrait, EntityTrait};
use spring::{config::ConfigRef, plugin::service::Service};
use spring_sea_orm::DbConn;
use spring_web::error::KnownWebError;
use spring_web::error::Result;

#[derive(Clone, Service)]
pub struct AuthService {
    #[component]
    db: DbConn,
    raline: ConfigRef<RalineConfig>,
    auth: ConfigRef<AuthConfig>,
}

impl AuthService {
    pub fn get_auth_server(&self, ty: &str) -> Result<AuthServer> {
        let server_url = &self.raline.server_url;
        match ty {
            "qq" => Ok(AuthServer::QQ(
                qq::AuthorizationServer::builder()
                    .client_id(&self.auth.qq.client_id)
                    .client_secret(&self.auth.qq.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
                self,
            )),
            "weibo" => Ok(AuthServer::Weibo(
                just_auth::weibo::AuthorizationServer::builder()
                    .client_id(&self.auth.weibo.client_id)
                    .client_secret(&self.auth.weibo.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
                self,
            )),
            "github" => Ok(AuthServer::Github(
                just_auth::github::AuthorizationServer::builder()
                    .client_id(&self.auth.github.client_id)
                    .client_secret(&self.auth.github.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
                self,
            )),
            "twitter" => Ok(AuthServer::Twitter(
                just_auth::twitter::AuthorizationServer::builder()
                    .client_id(&self.auth.twitter.client_id)
                    .client_secret(&self.auth.twitter.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
                self,
            )),
            _ => Err(KnownWebError::bad_request(&format!(
                "type is not defined:{ty}"
            )))?,
        }
    }
}

pub enum AuthServer<'a> {
    QQ(qq::AuthorizationServer, &'a AuthService),
    Weibo(just_auth::weibo::AuthorizationServer, &'a AuthService),
    Github(just_auth::github::AuthorizationServer, &'a AuthService),
    Twitter(just_auth::twitter::AuthorizationServer, &'a AuthService),
}

impl<'a> AuthServer<'a> {
    pub async fn authorize(&self) -> Result<String> {
        let state = "";
        Ok(match self {
            Self::QQ(server, _) => server.authorize(state).await,
            Self::Weibo(server, _) => server.authorize(state).await,
            Self::Github(server, _) => server.authorize(state).await,
            Self::Twitter(server, _) => server.authorize(state).await,
        }
        .context("get auth url failed")?)
    }

    pub async fn login(&self, query: String) -> Result<()> {
        match self {
            Self::QQ(server, service) => {
                save(server, service, &query, |u| users::ActiveModel {
                    r#type: Set(UserType::Normal),
                    username: Set(u.name.clone()),
                    avatar: Set(u
                        .extra
                        .get("figureurl")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())),
                    gender: Set(u
                        .extra
                        .get("gender")
                        .and_then(|v| v.as_str().map(UserGender::from_string))
                        .unwrap_or_else(|| UserGender::Unknown)),
                    ..Default::default()
                })
                .await?;
            }
            Self::Weibo(server, service) => {
                save(server, service, &query, |u| users::ActiveModel {
                    r#type: Set(UserType::Normal),
                    username: Set(u.name.clone()),
                    avatar: Set(u
                        .extra
                        .get("headimgurl")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())),
                    gender: Set(u
                        .extra
                        .get("sex")
                        .and_then(|v| v.as_str().map(UserGender::from_string))
                        .unwrap_or_else(|| UserGender::Unknown)),
                    ..Default::default()
                })
                .await?;
            }
            Self::Github(server, service) => {
                save(server, service, &query, |u| users::ActiveModel {
                    r#type: Set(UserType::Normal),
                    username: Set(u.name.clone()),
                    avatar: Set(u.extra.get("avatar_url").map(|v| v.to_string())),
                    gender: Set(UserGender::Unknown),
                    ..Default::default()
                })
                .await?;
            }
            Self::Twitter(server, service) => {
                save(server, service, &query, |u| users::ActiveModel {
                    r#type: Set(UserType::Normal),
                    username: Set(u.name.clone()),
                    avatar: Set(u.extra.get("profile_image_url").map(|v| v.to_string())),
                    gender: Set(UserGender::Unknown),
                    ..Default::default()
                })
                .await?;
            }
        }
        Ok(())
    }
}

async fn save<S, F>(server: &S, service: &AuthService, query: &str, user_mapping: F) -> Result<()>
where
    S: GenericAuthAction,
    F: FnOnce(&AuthUser) -> users::ActiveModel,
{
    let user = server.login(query).await.context("login failed")?;
    let model = UserOauth::find()
        .filter(
            user_oauth::Column::Provider
                .eq("qq")
                .and(user_oauth::Column::ProviderId.eq(user.user_id.clone())),
        )
        .one(&service.db)
        .await
        .context("find user oauth failed")?;
    let expires_at = DateTime::from_timestamp_millis(user.expires_in)
        .map(|t| t.naive_local())
        .unwrap_or_default();
    match model {
        Some(m) => user_oauth::ActiveModel {
            id: Set(m.id),
            access_token: Set(user.access_token),
            refresh_token: Set(user.refresh_token),
            expires_at: Set(expires_at),
            ..Default::default()
        },
        None => {
            // save user
            user_mapping(&user)
                .save(&service.db)
                .await
                .context("save user failed")?;
            user_oauth::ActiveModel {
                provider: Set("qq".into()),
                provider_id: Set(user.user_id),
                access_token: Set(user.access_token),
                refresh_token: Set(user.refresh_token),
                expires_at: Set(expires_at),
                ..Default::default()
            }
        }
    }
    .save(&service.db)
    .await
    .context("save user oauth failed")?;
    Ok(())
}
