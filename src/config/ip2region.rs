use serde::Deserialize;
use spring::config::Configurable;

#[derive(Clone, Deserialize, Configurable)]
#[config_prefix = "ip2region"]
pub struct Ip2RegionConfig {
    pub db_path: String,
}

fn default_db_path() -> String {
    "./data/ip2region.xdb".to_string()
}
