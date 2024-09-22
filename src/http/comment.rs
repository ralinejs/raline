use std::net::IpAddr;
use std::time::Duration;

use crate::config::RalineConfig;
use crate::dto::comment::{
    AddCommentReq, AdminCommentQuery, AdminListResp, CommentQueryResp, CommentResp,
    CountCommentQuery, CountResp, ListCommentQuery, ListResp, Owner,
};
use crate::dto::Urls;
use crate::model::sea_orm_active_enums::UserType;
use crate::model::{prelude::*, users};
use crate::plugins::akismet::Akismet;
use crate::utils::jwt::Claims;
use crate::{
    dto::comment::CommentQueryReq,
    model::{comments, sea_orm_active_enums::CommentStatus},
    utils::jwt::OptionalClaims,
};
use anyhow::Context;
use axum_client_ip::SecureClientIp;
use itertools::Itertools;
use regex::Regex;
use sea_orm::sqlx::types::chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde_json::json;
use spring_sea_orm::DbConn;
use spring_web::error::KnownWebError;
use spring_web::extractor::Config;
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
    Config(config): Config<RalineConfig>,
) -> Result<Json<CommentQueryResp>> {
    match req {
        CommentQueryReq::Count(q) => get_comment_count(&q, &db, &claims)
            .await
            .map(|r| Json(r.into())),
        CommentQueryReq::List(q) => get_comment_list(&q, &db, &claims, &config)
            .await
            .map(|r| Json(r.into())),
        CommentQueryReq::Admin(q) => get_admin_comment_list(&q, &db, &claims, &config)
            .await
            .map(|r| Json(r.into())),
        _ => todo!(),
    }
}

async fn get_admin_comment_list(
    q: &AdminCommentQuery,
    db: &DbConn,
    optional_claims: &OptionalClaims,
    config: &RalineConfig,
) -> Result<AdminListResp> {
    let claims = match &**optional_claims {
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

    let uids = comments.iter().filter_map(|c| c.user_id).collect_vec();
    let users = Users::find()
        .filter(users::Column::Id.is_in(uids))
        .all(db)
        .await
        .context("query users failed")?;

    Ok(AdminListResp {
        page: q.page,
        total_pages: total / q.size,
        page_size: q.size,
        spam_count,
        waiting_count,
        data: compute_comments(comments, &vec![], &users, config, optional_claims).await,
    })
}

async fn get_comment_list(
    q: &ListCommentQuery,
    db: &DbConn,
    claims: &OptionalClaims,
    config: &RalineConfig,
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
    let count = Comments::find()
        .filter(filter.clone())
        .count(db)
        .await
        .context("count comments failed")?;

    let filter = filter.and(comments::Column::Id.gt(q.offset));

    let (column, order) = q.sort_by.into_column_order();
    let root_comments = Comments::find()
        .filter(filter.clone().and(comments::Column::Rid.is_null()))
        .order_by(column, order)
        .limit(q.limit)
        .all(db)
        .await
        .context("find root comments failed")?;

    let rids = root_comments.iter().map(|c| c.id).collect_vec();

    let children = Comments::find()
        .filter(filter.and(comments::Column::Rid.is_in(rids)))
        .all(db)
        .await
        .context("find children comments failed")?;

    let comments = vec![children, root_comments.clone()].concat();

    let uids = comments.iter().filter_map(|c| c.user_id).collect_vec();
    let users = Users::find()
        .filter(users::Column::Id.is_in(uids))
        .all(db)
        .await
        .context("query users failed")?;

    Ok(ListResp {
        count,
        data: compute_comments(root_comments, &comments, &users, config, claims).await,
    })
}

async fn compute_comments(
    roots: Vec<comments::Model>,
    children: &Vec<comments::Model>,
    users: &Vec<users::Model>,
    config: &RalineConfig,
    login_user: &OptionalClaims,
) -> Vec<CommentResp> {
    let mut data = Vec::new();
    for c in roots {
        let mut cr = CommentResp::format(&c, users, config, login_user).await;
        for cc in children.iter() {
            if cc.rid == Some(cr.object_id) {
                let ccr = CommentResp::format(cc, users, config, login_user).await;
                cr.children.push(ccr);
            }
        }
        data.push(cr);
    }
    data
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
    claims: OptionalClaims,
    Config(config): Config<RalineConfig>,
    SecureClientIp(client_ip): SecureClientIp,
    Component(db): Component<DbConn>,
    Component(akismet): Component<Akismet>,
    Json(body): Json<AddCommentReq>,
) -> Result<impl IntoResponse> {
    let mut data = body.clone().into_active_model();
    data.ip = Set(client_ip.to_string());
    data.user_id = Set(claims.as_ref().map(|c| c.uid));

    if let Some(pid) = &body.pid {
        match &body.at {
            None => Err(KnownWebError::bad_request("at required with pid"))?,
            Some(at) => {
                let comment = &body.comment;
                data.content = Set(format!("[@{at}](#{pid}): {comment}"))
            }
        }
    }
    tracing::debug!("Post Comment initial Data: {:?}", &body);

    let mut status = CommentStatus::Approved;
    if let Some(Claims { ty, .. }) = &*claims {
        if *ty == UserType::Admin {
            status = check_comment(&body, &client_ip, &config, &db, &akismet).await?
        }
    }
    data.status = Set(status);

    let c = data.insert(&db).await.context("insert comment failed")?;
    let resp = CommentResp::format(&c, &vec![], &config, &claims).await;
    Ok(Json(json!({"data": resp})))
}

async fn check_comment(
    comment: &AddCommentReq,
    client_ip: &IpAddr,
    config: &RalineConfig,
    db: &DbConn,
    akismet: &Akismet,
) -> Result<CommentStatus> {
    if config.disallow_ips.contains(&client_ip) {
        tracing::debug!("Comment IP {} is in disallowIPList", &client_ip);
        Err(KnownWebError::forbidden("禁止访问"))?;
    }
    tracing::debug!("Comment IP {} check OK!", &client_ip);

    let duplicate_count = Comments::find()
        .filter(
            comments::Column::Url
                .eq(comment.url.clone())
                .and(comments::Column::Mail.eq(comment.mail.clone()))
                .and(comments::Column::Link.eq(comment.link.clone()))
                .and(comments::Column::Nick.eq(comment.nick.clone()))
                .and(comments::Column::Content.eq(comment.comment.clone())),
        )
        .count(db)
        .await
        .context("check duplicate content failed")?;

    if duplicate_count > 0 {
        tracing::debug!("The comment author had post same comment content before",);

        Err(KnownWebError::bad_request("Duplicate Content"))?;
    }
    tracing::debug!("Comment duplicate check OK!");

    let ns = Local::now().naive_local() - Duration::from_secs(config.ip_qps);
    let ip_comment_count = Comments::find()
        .filter(
            comments::Column::CreatedAt
                .gt(ns)
                .and(comments::Column::Ip.eq(client_ip.to_string())),
        )
        .count(db)
        .await
        .context("check ip comments failed")?;
    if ip_comment_count > 0 {
        tracing::debug!("The author has posted in {} seconds", config.ip_qps);
        Err(KnownWebError::bad_request("Comment too fast!"))?;
    }
    tracing::debug!("Comment post frequency check OK!");

    let mut status = if config.audit {
        CommentStatus::Waiting
    } else {
        CommentStatus::Approved
    };
    tracing::debug!("Comment initial status is {:?}", status);

    /* Akismet */
    if status == CommentStatus::Approved {
        match akismet.check_comment(&client_ip, &comment).await {
            Err(e) => tracing::warn!("akismet error:{}", e),
            Ok(spam) => {
                if spam {
                    status = CommentStatus::Spam;
                }
            }
        }
    }
    tracing::debug!("Comment akismet check result: {:?}", status);

    /* KeyWord Filter */
    if config.forbidden_words.len() > 0 {
        let regex = format!("({})", config.forbidden_words.iter().join("|"));
        let regex = Regex::new(&regex)
            .with_context(|| format!("forbidden_words regex parse failed:{}", regex))?;
        if regex.is_match(&comment.comment) {
            status = CommentStatus::Spam;
        }
    }
    tracing::debug!("Comment keyword check result: {:?}", status);

    Ok(status)
}

#[put("/comment/:id")]
async fn update_comment(
    claims: OptionalClaims,
    Component(db): Component<DbConn>,
    Path(id): Path<i64>,
    Json(body): Json<CommentQueryReq>,
) -> Result<impl IntoResponse> {
    Ok("")
}
