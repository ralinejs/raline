use spring_sea_orm::DbConn;
use spring_web::{axum::response::IntoResponse, error::Result, extractor::Component, get};

#[get("/comment")]
async fn get_comment(Component(db): Component<DbConn>) -> Result<impl IntoResponse> {
    Ok("")
}
