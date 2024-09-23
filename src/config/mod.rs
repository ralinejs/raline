pub mod akismet;
pub mod mail;
pub mod comrak;

use serde::Deserialize;
use spring::config::Configurable;
use std::net::IpAddr;

#[derive(Deserialize, Configurable)]
#[config_prefix = "raline"]
pub struct RalineConfig {
    pub site_url: String,
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
}

fn default_ip_qps() -> u64 {
    60
}
