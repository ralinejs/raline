mod config;
mod dto;
mod http;
mod model;
mod utils;

use spring::App;
use spring_mail::MailPlugin;
use spring_redis::RedisPlugin;
use spring_sea_orm::SeaOrmPlugin;
use spring_web::{WebConfigurator, WebPlugin};

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(SeaOrmPlugin)
        .add_plugin(MailPlugin)
        .add_plugin(RedisPlugin)
        .add_router(http::router())
        .run()
        .await
}
