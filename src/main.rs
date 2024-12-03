mod config;
mod model;
mod plugins;
mod router;
mod service;
mod utils;
mod views;

use plugins::{akismet::AkismetPlugin, ip2region::Ip2RegionPlugin, uaparser::UAParserPlugin};
use spring::App;
use spring_mail::MailPlugin;
use spring_opentelemetry::{
    KeyValue, OpenTelemetryPlugin, ResourceConfigurator, SERVICE_NAME, SERVICE_VERSION,
};
use spring_redis::RedisPlugin;
use spring_sea_orm::SeaOrmPlugin;
use spring_web::{WebConfigurator, WebPlugin};

// Init translations for current crate.
rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() {
    App::new()
        .opentelemetry_attrs([
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ])
        .add_plugin(WebPlugin)
        .add_plugin(OpenTelemetryPlugin)
        .add_plugin(SeaOrmPlugin)
        .add_plugin(MailPlugin)
        .add_plugin(RedisPlugin)
        .add_plugin(AkismetPlugin)
        .add_plugin(UAParserPlugin)
        .add_plugin(Ip2RegionPlugin)
        .add_router(router::router())
        .run()
        .await
}
