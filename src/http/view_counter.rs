use crate::dto::view_counter::{ColumnQueryAs, SetViewCount};
use crate::model::prelude::ViewCounter;
use crate::{dto::view_counter::ViewCountQuery, model::view_counter};
use anyhow::Context;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use spring_sea_orm::DbConn;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::Result,
    extractor::{Component, Query},
    get, post,
};
use std::collections::HashMap;

#[get("/view")]
async fn get_view_count(
    Component(db): Component<DbConn>,
    Query(req): Query<ViewCountQuery>,
) -> Result<impl IntoResponse> {
    if req.path.is_empty() {
        let result = req.types.into_iter().map(|ty| (ty, 0)).collect();
        return Ok(Json(vec![result]));
    }
    let result = ViewCounter::find()
        .filter(view_counter::Column::Url.is_in(&req.path))
        .all(&db)
        .await
        .context("query view counter failed")?;

    let item_map: HashMap<String, view_counter::Model> =
        result.into_iter().map(|vc| (vc.url.clone(), vc)).collect();

    let ty_count: usize = req.types.len();
    let mut result = vec![];
    for url in req.path {
        let mut counter = HashMap::<ColumnQueryAs, i32>::with_capacity(ty_count);
        for field in &req.types {
            counter.insert(
                field.clone(),
                item_map
                    .get(&url)
                    .map(|vc| field.get(vc))
                    .unwrap_or_default(),
            );
        }
        result.push(counter);
    }
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
