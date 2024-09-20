use crate::dto::comment::{CommentQueryResp, CountCommentQuery, CountResp};
use crate::dto::Urls;
use crate::model::prelude::Comments;
use crate::{
    dto::comment::CommentQueryReq,
    model::{comments, sea_orm_active_enums::CommentStatus},
    utils::jwt::OptionalClaims,
};
use anyhow::Context;
use itertools::Itertools;
use sea_orm::{
    ColumnTrait, EntityOrSelect, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    TransactionTrait,
};
use spring_sea_orm::DbConn;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::Result,
    extractor::{Component, Path, Query},
    get, post, put,
};

#[get("/comment")]
async fn get_comment(
    claims: OptionalClaims,
    Component(db): Component<DbConn>,
    Query(req): Query<CommentQueryReq>,
) -> Result<Json<CommentQueryResp>> {
    match req {
        CommentQueryReq::Count(q) => get_comment_count(&q, &db, &claims)
            .await
            .map(|r| Json(r.into())),
        _ => todo!(),
    }
}

async fn get_comment_count(
    q: &CountCommentQuery,
    db: &DbConn,
    claims: &OptionalClaims,
) -> Result<CountResp> {
    let filter = match &**claims {
        None => comments::Column::Status.eq(CommentStatus::Approved),
        Some(c) => comments::Column::Status
            .eq(CommentStatus::Approved)
            .or(comments::Column::UserId.eq(c.uid)),
    };
    match &q.url {
        Urls::Single(url) => {
            let filter = filter.and(comments::Column::Url.eq(url));
            let count = Comments::find()
                .filter(filter)
                .count(db)
                .await
                .context("query comment count failed")?;
            Ok(count.into())
        }
        Urls::List(urls) => {
            let filter = filter.and(comments::Column::Url.is_in(urls));
            let count: Vec<(String, u64)> = Comments::find()
                .select_only()
                .column_as(comments::Column::Url, "url")
                .column_as(comments::Column::Id.count(), "count")
                .filter(filter)
                .group_by(comments::Column::Url)
                .into_tuple()
                .all(db)
                .await
                .context("query comment count failed")?;
            let count = urls
                .iter()
                .map(|u| {
                    count
                        .iter()
                        .filter(|(url, _)| url == u)
                        .last()
                        .map(|(_, count)| *count)
                        .unwrap_or_default()
                })
                .collect_vec();
            Ok(count.into())
        }
    }
}

#[post("/comment")]
async fn add_comment(
    Component(db): Component<DbConn>,
    Json(body): Json<CommentQueryReq>,
) -> Result<impl IntoResponse> {
    Ok("")
}

#[put("/comment/:id")]
async fn update_comment(
    Component(db): Component<DbConn>,
    Path(id): Path<i64>,
    Json(body): Json<CommentQueryReq>,
) -> Result<impl IntoResponse> {
    Ok("")
}
