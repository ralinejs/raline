use crate::views::comment::{AddCommentReq, CommentQueryResp, CommentUpdateReq};
use crate::router::Locale;
use crate::model::sea_orm_active_enums::UserType;
use crate::model::{comments, prelude::*};
use crate::service::comment::CommentService;
use crate::{views::comment::CommentQueryReq, utils::jwt::OptionalClaims};
use anyhow::Context;
use axum_client_ip::SecureClientIp;
use rust_i18n::t;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use spring_sea_orm::DbConn;
use spring_web::delete;
use spring_web::error::KnownWebError;
use spring_web::{
    axum::{response::IntoResponse, Json},
    error::Result,
    extractor::{Component, Path, Query},
    get, post, put,
};

#[get("/api/comment")]
async fn get_comment(
    claims: OptionalClaims,
    Component(user_service): Component<CommentService>,
    Query(req): Query<CommentQueryReq>,
) -> Result<Json<CommentQueryResp>> {
    match req {
        CommentQueryReq::Count(q) => user_service
            .get_comment_count(&q, &claims)
            .await
            .map(|r| Json(r.into())),
        CommentQueryReq::List(q) => user_service
            .get_comment_list(&q, &claims)
            .await
            .map(|r| Json(r.into())),
        CommentQueryReq::Admin(q) => user_service
            .get_admin_comment_list(&q, &claims)
            .await
            .map(|r| Json(r.into())),
        CommentQueryReq::Recent(q) => user_service
            .get_recent_comment_list(&q, &claims)
            .await
            .map(|r| Json(r.into())),
    }
}

#[post("/api/comment")]
async fn add_comment(
    claims: OptionalClaims,
    Component(comment_service): Component<CommentService>,
    SecureClientIp(client_ip): SecureClientIp,
    Json(body): Json<AddCommentReq>,
) -> Result<impl IntoResponse> {
    let comment = comment_service.add_comment(claims, client_ip, body).await?;
    Ok(Json(json!({"data": comment})))
}

#[put("/api/comment/:id")]
async fn update_comment(
    optional_claims: OptionalClaims,
    Component(comment_service): Component<CommentService>,
    Path(id): Path<i32>,
    Json(body): Json<CommentUpdateReq>,
) -> Result<impl IntoResponse> {
    let comment = comment_service
        .update_comment(optional_claims, id, body)
        .await?;

    Ok(Json(json!({"data": comment})))
}

#[delete("/api/comment/:id")]
async fn delete_comment(
    claims: OptionalClaims,
    Component(db): Component<DbConn>,
    Path(id): Path<i32>,
    Locale(lang): Locale,
) -> Result<impl IntoResponse> {
    let c = Comments::find_by_id(id)
        .one(&db)
        .await
        .context("find comment failed")?;

    let c = match c {
        None => Err(KnownWebError::not_found(t!("not_found", locale = lang)))?,
        Some(c) => c,
    };

    let uid = claims.clone().map(|c| c.uid);
    if c.user_id != uid && claims.clone().map(|c| c.ty) != Some(UserType::Admin) {
        Err(KnownWebError::forbidden(t!("no_permission", locale = lang)))?;
    }

    let effect = Comments::delete_many()
        .filter(
            comments::Column::Id
                .eq(id)
                .or(comments::Column::Pid.eq(id))
                .or(comments::Column::Rid.eq(id)),
        )
        .exec(&db)
        .await
        .context("delete comment failed")?;
    let success = effect.rows_affected > 0;
    Ok(Json(json!({"data":success})))
}
