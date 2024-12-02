use serde::Deserialize;
use spring::config::Configurable;

#[derive(Clone, Deserialize, Configurable)]
#[config_prefix = "auth"]
pub struct AuthConfig {
    pub qq: AuthConfigDefine,
    pub weibo: AuthConfigDefine,
    pub wechat: AuthConfigDefine,
    pub github: AuthConfigDefine,
    pub twitter: AuthConfigDefine,
}

#[derive(Clone, Deserialize)]
pub struct AuthConfigDefine {
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
}
