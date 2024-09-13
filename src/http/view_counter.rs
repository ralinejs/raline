use crate::model::prelude::ViewCounter;
use crate::{dto::view_counter::ViewCountQuery, model::view_counter};
use anyhow::Context;
use sea_orm::{DeriveColumn, EntityTrait, EnumIter, QueryFilter, QuerySelect};
use spring_sea_orm::DbConn;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::Result,
    extractor::{Component, Query},
    get, post,
};

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    Times,
}

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
        .column_as(view_counter::Column::Times, QueryAs::Times)
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
    

    Ok(Json(true))
}
