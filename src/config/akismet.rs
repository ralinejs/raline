use serde::Deserialize;
use spring::config::Configurable;

#[derive(Deserialize, Configurable)]
#[config_prefix = "akismet"]
pub struct AkismetConfig {
    #[serde(default = "default_key")]
    pub akismet_key: String,
}

fn default_key() -> String {
    "70542d86693e".to_string()
}
