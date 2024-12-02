use crate::config::{auth::AuthConfig, RalineConfig};
use crate::views::oauth::OAuthQuery;
use crate::model::prelude::UserOauth;
use crate::model::sea_orm_active_enums::{UserGender, UserType};
use crate::model::users::Entity as Users;
use crate::model::{user_oauth, users};
use crate::utils::jwt::{self, Claims, OptionalClaims};
use anyhow::Context;
use askama_axum::IntoResponse;
use just_auth::{qq, AuthUser, GenericAuthAction};
use sea_orm::sqlx::types::chrono::DateTime;
use sea_orm::{ActiveModelTrait, QueryFilter, Set};
use sea_orm::{ColumnTrait, EntityTrait};
use spring::{config::ConfigRef, plugin::service::Service};
use spring_sea_orm::DbConn;
use spring_web::axum::body::Body;
use spring_web::axum::http::Response;
use spring_web::axum::response::Redirect;
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
    pub fn get_auth_server(&self, ty: &str, query: OAuthQuery) -> Result<AuthServer> {
        let server_url = &self.raline.server_url;
        let redirect_url = match query.redirect {
            None => format!("{server_url}/api/oauth/{ty}/callback"),
            Some(redirect) => format!("{server_url}/api/oauth/{ty}/callback?redirect={redirect}"),
        };
        match ty {
            "qq" => Ok(AuthServer::QQ(
                qq::AuthorizationServer::builder()
                    .client_id(&self.auth.qq.client_id)
                    .client_secret(&self.auth.qq.client_secret.clone().unwrap_or_default())
                    .redirect_uri(redirect_url)
                    .build(),
                self,
            )),
            "wechat" => Ok(AuthServer::Wechat(
                just_auth::wechat_open::AuthorizationServer::builder()
                    .client_id(&self.auth.wechat.client_id)
                    .client_secret(&self.auth.wechat.client_secret.clone().unwrap_or_default())
                    .redirect_uri(redirect_url)
                    .build(),
                self,
            )),
            "weibo" => Ok(AuthServer::Weibo(
                just_auth::weibo::AuthorizationServer::builder()
                    .client_id(&self.auth.weibo.client_id)
                    .client_secret(&self.auth.weibo.client_secret.clone().unwrap_or_default())
                    .redirect_uri(redirect_url)
                    .build(),
                self,
            )),
            "github" => Ok(AuthServer::Github(
                just_auth::github::AuthorizationServer::builder()
                    .client_id(&self.auth.github.client_id)
                    .client_secret(&self.auth.github.client_secret.clone().unwrap_or_default())
                    .redirect_uri(redirect_url)
                    .build(),
                self,
            )),
            "twitter" => Ok(AuthServer::Twitter(
                just_auth::twitter::AuthorizationServer::builder()
                    .client_id(&self.auth.twitter.client_id)
                    .client_secret(&self.auth.twitter.client_secret.clone().unwrap_or_default())
                    .redirect_uri(redirect_url)
                    .build(),
                self,
            )),
            _ => Err(KnownWebError::bad_request(&format!(
                "type is not defined:{ty}"
            )))?,
        }
    }

    async fn save<S, F>(
        &self,
        provider: &str,
        server: &S,
        query: &str,
        claims: OptionalClaims,
        user_mapping: F,
    ) -> Result<Response<Body>>
    where
        S: GenericAuthAction,
        F: FnOnce(&AuthUser) -> users::ActiveModel,
    {
        let auth_query: OAuthQuery = serde_urlencoded::from_str(query)
            .with_context(|| format!("decode query failed:{query}"))?;
        let user = server.login(query).await.context("login failed")?;
        let model = UserOauth::find()
            .filter(
                user_oauth::Column::Provider
                    .eq(provider)
                    .and(user_oauth::Column::ProviderId.eq(user.user_id.clone())),
            )
            .one(&self.db)
            .await
            .context("find user oauth failed")?;
        let expires_at = DateTime::from_timestamp_millis(user.expires_in)
            .map(|t| t.naive_local())
            .unwrap_or_default();
        match model {
            Some(m) => {
                let user_in_db = Users::find_by_id(m.user_id)
                    .one(&self.db)
                    .await
                    .with_context(|| format!("query user failed:{}", m.user_id))?
                    .expect(&format!("user#{} not exists", m.user_id));
                user_oauth::ActiveModel {
                    id: Set(m.id),
                    access_token: Set(user.access_token),
                    refresh_token: Set(user.refresh_token),
                    expires_at: Set(expires_at),
                    ..Default::default()
                }
                .save(&self.db)
                .await
                .context("save user oauth failed")?;

                let token = jwt::encode(Claims::new(&user_in_db))?;

                let redirect = match auth_query.redirect {
                    Some(redirect) => {
                        if redirect.contains("?") {
                            format!("{redirect}&token={token}")
                        } else {
                            format!("{redirect}?token={token}")
                        }
                    }
                    None => {
                        if claims.is_none() {
                            return Ok(().into_response());
                        } else {
                            format!("/ui/profile?token={token}")
                        }
                    }
                };
                return Ok(Redirect::to(&redirect).into_response());
            }
            None => {
                let mut active_model = user_oauth::ActiveModel {
                    provider: Set(provider.into()),
                    expires_at: Set(expires_at),
                    ..Default::default()
                };
                let token = match &*claims {
                    Some(claims) => {
                        active_model.user_id = Set(claims.uid);
                        jwt::encode(claims.clone())?
                    }
                    None => {
                        let user_in_db = user_mapping(&user)
                            .insert(&self.db)
                            .await
                            .context("save user failed")?;
                        active_model.user_id = Set(user_in_db.id);
                        jwt::encode(Claims::new(&user_in_db))?
                    }
                };
                active_model.provider_id = Set(user.user_id);
                active_model.access_token = Set(user.access_token);
                active_model.refresh_token = Set(user.refresh_token);
                active_model
                    .save(&self.db)
                    .await
                    .context("save user oauth failed")?;

                let redirect = match auth_query.redirect {
                    Some(redirect) => {
                        if redirect.contains("?") {
                            format!("{redirect}&token={token}")
                        } else {
                            format!("{redirect}?token={token}")
                        }
                    }
                    None => {
                        if claims.is_none() {
                            return Ok(().into_response());
                        } else {
                            format!("/ui/profile?token={token}")
                        }
                    }
                };
                return Ok(Redirect::to(&redirect).into_response());
            }
        }
    }
}

pub enum AuthServer<'a> {
    QQ(qq::AuthorizationServer, &'a AuthService),
    Weibo(just_auth::weibo::AuthorizationServer, &'a AuthService),
    Wechat(just_auth::wechat_open::AuthorizationServer, &'a AuthService),
    Github(just_auth::github::AuthorizationServer, &'a AuthService),
    Twitter(just_auth::twitter::AuthorizationServer, &'a AuthService),
}

impl<'a> AuthServer<'a> {
    pub async fn authorize(&self) -> Result<String> {
        let state = "";
        Ok(match self {
            Self::QQ(server, _) => server.authorize(state).await,
            Self::Weibo(server, _) => server.authorize(state).await,
            Self::Wechat(server, _) => server.authorize(state).await,
            Self::Github(server, _) => server.authorize(state).await,
            Self::Twitter(server, _) => server.authorize(state).await,
        }
        .context("get auth url failed")?)
    }

    pub async fn login(&self, query: String, claims: OptionalClaims) -> Result<impl IntoResponse> {
        match self {
            Self::QQ(server, service) => {
                service
                    .save("qq", server, &query, claims, |u| users::ActiveModel {
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
                    .await
            }
            Self::Weibo(server, service) => {
                service
                    .save("weibo", server, &query, claims, |u| users::ActiveModel {
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
                    .await
            }
            Self::Wechat(server, service) => {
                service
                    .save("wechat", server, &query, claims, |u| users::ActiveModel {
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
                            .map(|v| {
                                if v.as_i64() == Some(1) {
                                    UserGender::Male
                                } else if v.as_i64() == Some(2) {
                                    UserGender::Female
                                } else {
                                    UserGender::Unknown
                                }
                            })
                            .unwrap_or_else(|| UserGender::Unknown)),
                        ..Default::default()
                    })
                    .await
            }
            Self::Github(server, service) => {
                service
                    .save("github", server, &query, claims, |u| users::ActiveModel {
                        r#type: Set(UserType::Normal),
                        username: Set(u.name.clone()),
                        avatar: Set(u.extra.get("avatar_url").map(|v| v.to_string())),
                        gender: Set(UserGender::Unknown),
                        ..Default::default()
                    })
                    .await
            }
            Self::Twitter(server, service) => {
                service
                    .save("twitter", server, &query, claims, |u| users::ActiveModel {
                        r#type: Set(UserType::Normal),
                        username: Set(u.name.clone()),
                        avatar: Set(u.extra.get("profile_image_url").map(|v| v.to_string())),
                        gender: Set(UserGender::Unknown),
                        ..Default::default()
                    })
                    .await
            }
        }
    }
}
