pub mod akismet;
pub mod comrak;
pub mod mail;
pub mod auth;

use serde::Deserialize;
use spring::config::Configurable;
use std::net::IpAddr;

#[derive(Clone, Deserialize, Configurable)]
#[config_prefix = "raline"]
pub struct RalineConfig {
    pub site_url: String,
    #[serde(default)]
    pub site_name: String,
    pub server_url: String,
    #[serde(default)]
    pub disallow_ips: Vec<IpAddr>,
    #[serde(default = "default_ip_qps")]
    pub ip_qps: u64,
    #[serde(default)]
    pub audit: bool,
    #[serde(default)]
    pub disable_user_agent: bool,
    #[serde(default)]
    pub disable_region: bool,
    #[serde(default)]
    pub forbidden_words: Vec<String>,
    pub recaptcha_v3_key: Option<String>,
    pub turnstile_key: Option<String>,
}

fn default_ip_qps() -> u64 {
    60
}
