use crate::config::ip2region::Ip2RegionConfig;
use spring::{app::AppBuilder, async_trait, config::ConfigRegistry, plugin::Plugin};
use xdb::searcher_init;

pub struct Ip2RegionPlugin;

#[async_trait]
impl Plugin for Ip2RegionPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<Ip2RegionConfig>()
            .expect("ip2region config is invalid");
        searcher_init(Some(config.db_path));
    }
}
