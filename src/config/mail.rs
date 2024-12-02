use serde::Deserialize;
use spring::config::Configurable;

#[derive(Deserialize, Configurable)]
#[config_prefix = "email"]
pub struct EmailConfig {
    pub from: String,
}