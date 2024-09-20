use crate::dto::view_counter::SetViewCount;
use crate::dto::Urls;
use crate::model::prelude::ViewCounter;
use crate::{dto::view_counter::ViewCountQuery, model::view_counter};
use anyhow::Context;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, DeriveColumn, EntityTrait, EnumIter, QueryFilter, QuerySelect};
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
    let filter = match path {
        Urls::Single(path) => view_counter::Column::Url.eq(path).into_condition(),
        Urls::List(paths) => view_counter::Column::Url.is_in(paths).into_condition(),
    };
    let result: Vec<i32> = ViewCounter::find()
        .select_only()
        .column_as(view_counter::Column::Times, QueryAs::Times)
        .filter(filter)
        .into_values::<_, QueryAs>()
        .all(&db)
        .await
        .context("query view counter failed")?;

    Ok(Json(result))
}

#[post("/view")]
async fn post_view_count(
    Component(db): Component<DbConn>,
    Json(req): Json<SetViewCount>,
) -> Result<impl IntoResponse> {
    let count = ViewCounter::increase_by_path(&db, req.path)
        .await
        .context("increase view count failed")?;

    Ok(Json(count))
}
