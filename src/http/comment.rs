use crate::dto::comment::CommentQueryReq;
use sea_orm::TransactionTrait;
use spring_sea_orm::DbConn;
use spring_web::{
    axum::response::IntoResponse,
    error::Result,
    extractor::{Component, Query},
    get,
};

#[get("/comment")]
async fn get_comment(
    Component(db): Component<DbConn>,
    Query(req): Query<CommentQueryReq>,
) -> Result<impl IntoResponse> {
    Ok("")
}
