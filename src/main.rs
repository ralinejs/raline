mod config;
mod dto;
mod http;
mod model;
mod plugins;
mod service;
mod utils;

use plugins::{akismet::AkismetPlugin, uaparser::UAParserPlugin};
use spring::App;
use spring_mail::MailPlugin;
use spring_redis::RedisPlugin;
use spring_sea_orm::SeaOrmPlugin;
use spring_web::{WebConfigurator, WebPlugin};
use xdb::searcher_init;

// Init translations for current crate.
rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() {
    let xdb_filepath = "./data/ip2region.xdb";
    searcher_init(Some(xdb_filepath.to_owned()));
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(SeaOrmPlugin)
        .add_plugin(MailPlugin)
        .add_plugin(RedisPlugin)
        .add_plugin(AkismetPlugin)
        .add_plugin(UAParserPlugin)
        .add_router(http::router())
        .run()
        .await
}
