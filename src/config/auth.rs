use serde::Deserialize;
use spring::config::Configurable;

#[derive(Clone, Deserialize, Configurable)]
#[config_prefix = "auth"]
pub struct AuthConfig {
    pub qq: AuthConfigRef,
    pub weibo: AuthConfigRef,
    pub github: AuthConfigRef,
    pub twitter: AuthConfigRef,
}

#[derive(Clone, Deserialize)]
pub struct AuthConfigRef {
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
}
