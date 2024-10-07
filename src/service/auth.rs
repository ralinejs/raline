use crate::config::{auth::AuthConfig, RalineConfig};
use anyhow::Context;
use just_auth::AuthAction;
use spring::{config::ConfigRef, plugin::service::Service};
use spring_web::error::KnownWebError;
use spring_web::error::Result;

#[derive(Clone, Service)]
pub struct AuthService {
    raline: ConfigRef<RalineConfig>,
    auth: ConfigRef<AuthConfig>,
}

impl AuthService {
    pub fn get_auth_server(&self, ty: &str) -> Result<AuthServer> {
        let server_url = &self.raline.server_url;
        match ty {
            "qq" => Ok(AuthServer::QQ(
                just_auth::qq::AuthorizationServer::builder()
                    .client_id(&self.auth.qq.client_id)
                    .client_secret(&self.auth.qq.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
            )),
            "weibo" => Ok(AuthServer::Weibo(
                just_auth::weibo::AuthorizationServer::builder()
                    .client_id(&self.auth.weibo.client_id)
                    .client_secret(&self.auth.weibo.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
            )),
            "github" => Ok(AuthServer::Github(
                just_auth::github::AuthorizationServer::builder()
                    .client_id(&self.auth.github.client_id)
                    .client_secret(&self.auth.github.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
            )),
            "twitter" => Ok(AuthServer::Twitter(
                just_auth::twitter::AuthorizationServer::builder()
                    .client_id(&self.auth.twitter.client_id)
                    .client_secret(&self.auth.twitter.client_secret.clone().unwrap_or_default())
                    .redirect_uri(format!("{server_url}/api/oauth/{ty}/callback"))
                    .build(),
            )),
            _ => Err(KnownWebError::bad_request(&format!(
                "type is not defined:{ty}"
            )))?,
        }
    }
}

pub enum AuthServer {
    QQ(just_auth::qq::AuthorizationServer),
    Weibo(just_auth::weibo::AuthorizationServer),
    Github(just_auth::github::AuthorizationServer),
    Twitter(just_auth::twitter::AuthorizationServer),
}

impl AuthServer {
    pub async fn authorize(&self) -> Result<String> {
        let state = "";
        Ok(match self {
            Self::QQ(server) => server.authorize(state).await,
            Self::Weibo(server) => server.authorize(state).await,
            Self::Github(server) => server.authorize(state).await,
            Self::Twitter(server) => server.authorize(state).await,
        }
        .context("get auth url failed")?)
    }

    pub async fn login(&self, callback_query: String) {
        
    }
}
