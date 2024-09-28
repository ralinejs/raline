use crate::dto::view_counter::{ColumnQueryAs, SetViewCount};
use crate::model::prelude::ViewCounter;
use crate::{dto::view_counter::ViewCountQuery, model::view_counter};
use anyhow::Context;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use spring_sea_orm::DbConn;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::Result,
    extractor::{Component, Query},
    get, post,
};
use std::collections::HashMap;

#[get("/api/view")]
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

#[post("/api/view")]
async fn post_view_count(
    Component(db): Component<DbConn>,
    Json(req): Json<SetViewCount>,
) -> Result<impl IntoResponse> {
    let count = ViewCounter::increase_by_path(&db, &req)
        .await
        .context("increase view count failed")?;

    let count = match req.r#type {
        ColumnQueryAs::Times => json!({"times":count.times}),
        ColumnQueryAs::Reaction0 => json!({"reaction0":count.reaction0}),
        ColumnQueryAs::Reaction1 => json!({"reaction1":count.reaction1}),
        ColumnQueryAs::Reaction2 => json!({"reaction2":count.reaction2}),
        ColumnQueryAs::Reaction3 => json!({"reaction3":count.reaction3}),
        ColumnQueryAs::Reaction4 => json!({"reaction4":count.reaction4}),
        ColumnQueryAs::Reaction5 => json!({"reaction5":count.reaction5}),
        ColumnQueryAs::Reaction6 => json!({"reaction6":count.reaction6}),
        ColumnQueryAs::Reaction7 => json!({"reaction7":count.reaction7}),
        ColumnQueryAs::Reaction8 => json!({"reaction8":count.reaction8}),
    };
    Ok(Json(vec![count]))
}
