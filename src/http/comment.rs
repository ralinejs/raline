use crate::dto::comment::{
    AdminCommentQuery, AdminListResp, CommentQueryResp, CountCommentQuery, CountResp,
    ListCommentQuery, ListResp, Owner,
};
use crate::dto::Urls;
use crate::model::prelude::Comments;
use crate::model::sea_orm_active_enums::UserType;
use crate::{
    dto::comment::CommentQueryReq,
    model::{comments, sea_orm_active_enums::CommentStatus},
    utils::jwt::OptionalClaims,
};
use anyhow::Context;
use itertools::Itertools;
use sea_orm::{
    ColumnTrait, EntityOrSelect, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    TransactionTrait,
};
use spring_sea_orm::DbConn;
use spring_web::error::KnownWebError;
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
        CommentQueryReq::List(q) => get_comment_list(&q, &db, &claims)
            .await
            .map(|r| Json(r.into())),
        CommentQueryReq::Admin(q) => get_admin_comment_list(&q, &db, &claims)
            .await
            .map(|r| Json(r.into())),
        _ => todo!(),
    }
}

async fn get_admin_comment_list(
    q: &AdminCommentQuery,
    db: &DbConn,
    claims: &OptionalClaims,
) -> Result<AdminListResp> {
    let claims = match &**claims {
        None => return Err(KnownWebError::forbidden("没有权限"))?,
        Some(claims) => claims,
    };
    let mut filter = comments::Column::Status.eq(q.status.clone());
    filter = match q.owner {
        Owner::All => filter,
        Owner::Mine => {
            let user_filter = match &claims.mail {
                Some(mail) => comments::Column::UserId
                    .eq(claims.uid)
                    .or(comments::Column::Mail.eq(mail)),
                None => comments::Column::UserId.eq(claims.uid),
            };
            filter.and(user_filter)
        }
    };
    if let Some(keyword) = &q.keyword {
        filter = filter.and(comments::Column::Content.like(format!("%{keyword}%")));
    }

    let total = Comments::find()
        .filter(filter.clone())
        .count(db)
        .await
        .context("count comments failed")?;

    let spam_count = Comments::find()
        .filter(comments::Column::Status.eq(CommentStatus::Spam))
        .count(db)
        .await
        .context("count comments failed")?;

    let waiting_count = Comments::find()
        .filter(comments::Column::Status.eq(CommentStatus::Waiting))
        .count(db)
        .await
        .context("count comments failed")?;

    let comments = Comments::find()
        .filter(filter)
        .paginate(db, q.size)
        .fetch_page(std::cmp::max(q.page - 1, 0))
        .await
        .context("find comments page failed")?;

    Ok(AdminListResp {
        page: q.page,
        total_pages: total / q.size,
        page_size: q.size,
        spam_count,
        waiting_count,
        data: comments,
    })
}

async fn get_comment_list(
    q: &ListCommentQuery,
    db: &DbConn,
    claims: &OptionalClaims,
) -> Result<ListResp> {
    let filter = comments::Column::Url.eq(&q.path);
    let filter = match &**claims {
        None => filter.and(comments::Column::Status.eq(CommentStatus::Approved)),
        Some(c) => {
            if c.ty == UserType::Admin {
                filter.and(comments::Column::Status.ne(CommentStatus::Deleted))
            } else {
                filter.and(
                    comments::Column::Status
                        .eq(CommentStatus::Approved)
                        .or(comments::Column::UserId.eq(c.uid)),
                )
            }
        }
    };
    let total = Comments::find()
        .filter(filter.clone())
        .count(db)
        .await
        .context("count comments failed")?;

    let filter = filter.and(comments::Column::Id.gt(q.offset));

    let (column, order) = q.sort_by.into_column_order();
    let data = Comments::find()
        .filter(filter)
        .order_by(column, order)
        .limit(q.limit)
        .all(db)
        .await
        .context("find comments failed")?;

    Ok(ListResp { total, data })
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
