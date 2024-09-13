use crate::model::prelude::ViewCounter;
use crate::{dto::view_counter::ViewCountQuery, model::view_counter};
use anyhow::Context;
use sea_orm::sqlx::query::QueryAs;
use sea_orm::{EntityTrait, QueryFilter, QuerySelect};
use spring_sea_orm::DbConn;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::Result,
    extractor::{Component, Query},
    get, post,
};

#[get("/view")]
async fn get_view_count(
    Component(db): Component<DbConn>,
    Query(req): Query<ViewCountQuery>,
) -> Result<impl IntoResponse> {
    let path = match req.path {
        None => return Ok(Json(vec![0])),
        Some(path) => path,
    };
    let result: Vec<i32> = ViewCounter::find()
        .select_only()
        .column(view_counter::Column::Times)
        .filter(path)
        .into_values::<_, QueryAs>()
        .all(&db)
        .await
        .context("query view counter failed")?;

    Ok(Json(result))
}

#[post("/view")]
async fn post_view_count(
    Component(db): Component<DbConn>,
    Json(req): Json<ViewCountQuery>,
) -> Result<impl IntoResponse> {
    // let u = users::ActiveModel {
    //     id: Set(u.id),
    //     username: Set(req.name),
    //     ..Default::default()
    // }
    // .update(&db)
    // .await
    // .with_context(|| format!("change name for user#{} failed", u.id))?;

    Ok(Json(true))
}
