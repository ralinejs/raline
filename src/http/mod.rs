mod comment;
mod token;
mod user;
mod view_counter;

use axum_client_ip::SecureClientIpSource;
use spring_web::{
    axum::{
        body,
        middleware::{self, Next},
        response::{IntoResponse, Response},
    },
    extractor::Request,
    Router,
};

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
