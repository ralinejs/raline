mod comment;
mod token;
mod user;
mod view_counter;

use askama::Template;
use axum_client_ip::SecureClientIpSource;
use spring_web::{
    axum::{
        body,
        middleware::{self, Next},
        response::{IntoResponse, Response},
    },
    extractor::{Config, Request},
    get, routes, Router,
};

use crate::config::RalineConfig;

pub fn router() -> Router {
    spring_web::handler::auto_router()
        .layer(middleware::from_fn(problem_middleware))
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
}

async fn problem_middleware(request: Request, next: Next) -> Response {
    let uri = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let msg = response.into_body();
        let msg = body::to_bytes(msg, usize::MAX)
            .await
            .expect("server body read failed");
        let msg = String::from_utf8(msg.to_vec()).expect("read body to string failed");
        problemdetails::new(status)
            .with_instance(uri)
            .with_title(status.canonical_reason().unwrap_or("error"))
            .with_detail(msg)
            .into_response()
    } else {
        response
    }
}
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    server_url: String,
    recaptcha_v3_key: String,
    turnstile_key: String,
}

#[get("/")]
async fn index(Config(config): Config<RalineConfig>) -> impl IntoResponse {
    IndexTemplate {
        server_url: config.site_url,
        recaptcha_v3_key: config.recaptcha_v3_key.unwrap_or_default(),
        turnstile_key: config.turnstile_key.unwrap_or_default(),
    }
}

#[derive(Template)]
#[template(path = "ui.html")]
struct UITemplate {
    site_url: String,
    site_name: String,
    server_url: String,
}

#[routes]
#[get("/ui")]
#[get("/ui/")]
#[get("/ui/:sub")]
async fn ui(Config(config): Config<RalineConfig>) -> impl IntoResponse {
    let RalineConfig {
        site_url,
        server_url,
        site_name,
        ..
    } = config;
    UITemplate {
        site_url: site_url,
        site_name: site_name,
        server_url: format!("{server_url}/api/"),
    }
}
