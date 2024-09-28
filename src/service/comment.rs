use crate::config::comrak::ComrakConfig;
use crate::config::RalineConfig;
use crate::dto::comment::{
    AddCommentReq, AdminCommentQuery, AdminListResp, CommentResp, CommentUpdateReq,
    CountCommentQuery, ListCommentQuery, ListResp, Owner, RecentCommentQuery, ToStringExt,
};
use crate::model::sea_orm_active_enums::UserType;
use crate::model::{prelude::*, users};
use crate::plugins::akismet::Akismet;
use crate::utils::jwt::Claims;
use crate::{
    model::{comments, sea_orm_active_enums::CommentStatus},
    utils::jwt::OptionalClaims,
};
use anyhow::Context;
use comrak::markdown_to_html;
use itertools::Itertools;
use regex::Regex;
use sea_orm::sqlx::types::chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use spring::config::ConfigRef;
use spring::plugin::service::Service;
use spring_sea_orm::DbConn;
use spring_web::error::KnownWebError;
use spring_web::error::Result;
use std::cmp::max;
use std::net::IpAddr;
use std::ops::Deref;
use std::time::Duration;
use uaparser::Client;

#[derive(Clone, Service)]
pub struct CommentService {
    #[component]
    db: DbConn,
    #[component]
    akismet: Akismet,
    raline: ConfigRef<RalineConfig>,
    comrak: ConfigRef<ComrakConfig>,
}

impl CommentService {
    pub async fn get_recent_comment_list(
        &self,
        q: &RecentCommentQuery,
        optional_claims: &OptionalClaims,
    ) -> Result<Vec<CommentResp>> {
        let filter = match &**optional_claims {
            None => comments::Column::Status.eq(CommentStatus::Approved),
            Some(c) => {
                if c.ty == UserType::Admin {
                    comments::Column::Status.ne(CommentStatus::Deleted)
                } else {
                    comments::Column::Status
                        .eq(CommentStatus::Approved)
                        .or(comments::Column::UserId.eq(c.uid))
                }
            }
        };

        let comments = Comments::find()
            .filter(filter)
            .order_by_desc(comments::Column::CreatedAt)
            .limit(q.count)
            .all(&self.db)
            .await
            .context("find comments page failed")?;

        let uids = comments.iter().filter_map(|c| c.user_id).collect_vec();
        let users = Users::find()
            .filter(users::Column::Id.is_in(uids))
            .all(&self.db)
            .await
            .context("query users failed")?;

        let comments = self
            .compute_comments(comments, &vec![], &users, optional_claims)
            .await;
        Ok(comments)
    }

    pub async fn get_admin_comment_list(
        &self,
        q: &AdminCommentQuery,
        optional_claims: &OptionalClaims,
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
            .count(&self.db)
            .await
            .context("count comments failed")?;

        let spam_count = Comments::find()
            .filter(comments::Column::Status.eq(CommentStatus::Spam))
            .count(&self.db)
            .await
            .context("count comments failed")?;

        let waiting_count = Comments::find()
            .filter(comments::Column::Status.eq(CommentStatus::Waiting))
            .count(&self.db)
            .await
            .context("count comments failed")?;

        let comments = Comments::find()
            .filter(filter)
            .order_by_desc(comments::Column::CreatedAt)
            .paginate(&self.db, q.size)
            .fetch_page(max(q.page - 1, 0))
            .await
            .context("find comments page failed")?;

        let uids = comments.iter().filter_map(|c| c.user_id).collect_vec();
        let users = Users::find()
            .filter(users::Column::Id.is_in(uids))
            .all(&self.db)
            .await
            .context("query users failed")?;

        Ok(AdminListResp {
            page: q.page,
            total_pages: total / q.size,
            page_size: q.size,
            spam_count,
            waiting_count,
            data: self
                .compute_comments(comments, &vec![], &users, optional_claims)
                .await,
        })
    }

    pub async fn get_comment_list(
        &self,
        q: &ListCommentQuery,
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
        let count = Comments::find()
            .filter(filter.clone())
            .count(&self.db)
            .await
            .context("count comments failed")?;

        let filter = filter.and(comments::Column::Id.gt(q.offset));

        let (column, order) = q.sort_by.into_column_order();
        let root_comments = Comments::find()
            .filter(filter.clone().and(comments::Column::Rid.is_null()))
            .order_by(column, order)
            .limit(q.limit)
            .all(&self.db)
            .await
            .context("find root comments failed")?;

        let rids = root_comments.iter().map(|c| c.id).collect_vec();

        let children = Comments::find()
            .filter(filter.and(comments::Column::Rid.is_in(rids)))
            .all(&self.db)
            .await
            .context("find children comments failed")?;

        let comments = vec![children, root_comments.clone()].concat();

        let uids = comments.iter().filter_map(|c| c.user_id).collect_vec();
        let users = Users::find()
            .filter(users::Column::Id.is_in(uids))
            .all(&self.db)
            .await
            .context("query users failed")?;

        Ok(ListResp {
            count,
            data: self
                .compute_comments(root_comments, &comments, &users, claims)
                .await,
        })
    }

    pub async fn get_comment_count(
        &self,
        q: &CountCommentQuery,
        claims: &OptionalClaims,
    ) -> Result<Vec<i64>> {
        let filter = match &**claims {
            None => comments::Column::Status.eq(CommentStatus::Approved),
            Some(c) => comments::Column::Status
                .eq(CommentStatus::Approved)
                .or(comments::Column::UserId.eq(c.uid)),
        };

        let filter = filter.and(comments::Column::Url.is_in(&q.url));
        let count: Vec<(String, i64)> = Comments::find()
            .select_only()
            .column_as(comments::Column::Url, "url")
            .column_as(comments::Column::Id.count(), "count")
            .filter(filter)
            .group_by(comments::Column::Url)
            .into_tuple()
            .all(&self.db)
            .await
            .context("query comment count failed")?;
        let count = q
            .url
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
        Ok(count)
    }

    pub async fn add_comment(
        &self,
        claims: OptionalClaims,
        client_ip: IpAddr,
        body: AddCommentReq,
    ) -> Result<CommentResp> {
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
                status = self.check_comment(&body, &client_ip).await?
            }
        }
        data.status = Set(status);

        let c = data
            .insert(&self.db)
            .await
            .context("insert comment failed")?;
        let comment = self.format_comment(&c, &vec![], &claims).await;
        Ok(comment)
    }

    pub async fn update_comment(
        &self,
        optional_claims: OptionalClaims,
        id: i32,
        body: CommentUpdateReq,
    ) -> Result<CommentResp> {
        let c = Comments::find_by_id(id)
            .one(&self.db)
            .await
            .context("find comment failed")?;

        let c = match c {
            None => Err(KnownWebError::not_found("not found"))?,
            Some(c) => c,
        };
        let mut ac = comments::ActiveModel {
            id: Set(c.id),
            ..Default::default()
        };
        if let Some(like) = body.like {
            match like {
                true => ac.star = Set(max(c.star + 1, 0)),
                false => ac.star = Set(max(c.star - 1, 0)),
            }
        }

        let c = match &*optional_claims {
            None => match body.is_empty() {
                false => Err(KnownWebError::forbidden("forbidden"))?,
                true => {
                    let c = ac.update(&self.db).await.context("update comment failed")?;
                    self.format_comment(&c, &vec![], &optional_claims).await
                }
            },
            Some(claims) => {
                if body.is_empty() {
                    let c = ac.update(&self.db).await.context("update comment failed")?;
                    self.format_comment(&c, &vec![], &optional_claims).await
                } else {
                    if c.user_id != Some(claims.uid) || UserType::Admin != claims.ty {
                        Err(KnownWebError::forbidden("forbidden"))?;
                    }
                    let c = body
                        .update_active_model(ac, claims.ty.clone())
                        .update(&self.db)
                        .await
                        .context("update comment failed")?;
                    let u = Users::find_by_id(claims.uid)
                        .one(&self.db)
                        .await
                        .context("find user by id failed")?
                        .expect("用户不存在");
                    self.format_comment(&c, &vec![u], &optional_claims).await
                }
            }
        };

        Ok(c)
    }

    async fn check_comment(
        &self,
        comment: &AddCommentReq,
        client_ip: &IpAddr,
    ) -> Result<CommentStatus> {
        if self.raline.disallow_ips.contains(&client_ip) {
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
            .count(&self.db)
            .await
            .context("check duplicate content failed")?;

        if duplicate_count > 0 {
            tracing::debug!("The comment author had post same comment content before",);

            Err(KnownWebError::bad_request("Duplicate Content"))?;
        }
        tracing::debug!("Comment duplicate check OK!");

        let ns = Local::now().naive_local() - Duration::from_secs(self.raline.ip_qps);
        let ip_comment_count = Comments::find()
            .filter(
                comments::Column::CreatedAt
                    .gt(ns)
                    .and(comments::Column::Ip.eq(client_ip.to_string())),
            )
            .count(&self.db)
            .await
            .context("check ip comments failed")?;
        if ip_comment_count > 0 {
            tracing::debug!("The author has posted in {} seconds", self.raline.ip_qps);
            Err(KnownWebError::bad_request("Comment too fast!"))?;
        }
        tracing::debug!("Comment post frequency check OK!");

        let mut status = if self.raline.audit {
            CommentStatus::Waiting
        } else {
            CommentStatus::Approved
        };
        tracing::debug!("Comment initial status is {:?}", status);

        /* Akismet */
        if status == CommentStatus::Approved {
            match self.akismet.check_comment(&client_ip, &comment).await {
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
        if self.raline.forbidden_words.len() > 0 {
            let regex = format!("({})", self.raline.forbidden_words.iter().join("|"));
            let regex = Regex::new(&regex)
                .with_context(|| format!("forbidden_words regex parse failed:{}", regex))?;
            if regex.is_match(&comment.comment) {
                status = CommentStatus::Spam;
            }
        }
        tracing::debug!("Comment keyword check result: {:?}", status);

        Ok(status)
    }

    async fn compute_comments(
        &self,
        roots: Vec<comments::Model>,
        children: &Vec<comments::Model>,
        users: &Vec<users::Model>,
        login_user: &OptionalClaims,
    ) -> Vec<CommentResp> {
        let mut data = Vec::new();
        for c in roots {
            let mut cr = self.format_comment(&c, users, login_user).await;
            for cc in children.iter() {
                if cc.rid == Some(cr.object_id) {
                    let ccr = self.format_comment(cc, users, login_user).await;
                    cr.children.push(ccr);
                }
            }
            data.push(cr);
        }
        data
    }

    async fn format_comment(
        &self,
        c: &comments::Model,
        users: &Vec<users::Model>,
        login_user: &OptionalClaims,
    ) -> CommentResp {
        let RalineConfig {
            disable_user_agent,
            disable_region,
            ..
        } = &*self.raline;
        let client: Option<Client> = if *disable_user_agent { None } else { None };
        let is_admin = match &**login_user {
            None => false,
            Some(u) => u.ty == UserType::Admin,
        };
        let addr = if is_admin || !disable_region {
            None
        } else {
            None
        };
        let comrak_opts = self.comrak.deref().into();
        let comment_html = markdown_to_html(&c.content, &comrak_opts);
        let comment_html = ammonia::clean(&*comment_html);
        let orig = if login_user.is_none() {
            None
        } else {
            Some(c.content.to_owned())
        };
        let user = users.iter().find(|u| c.user_id == Some(u.id));
        CommentResp {
            url: c.url.to_owned(),
            status: c.status.to_owned(),
            comment: comment_html,
            inserted_at: c.created_at,
            link: c.link.to_owned(),
            nick: user.map(|u| u.username.clone()).or(c.nick.to_owned()),
            mail: user.and_then(|u| u.email.clone()).or(c.mail.to_owned()),
            r#type: user.map(|u| u.r#type.clone()),
            avatar: user.and_then(|u| u.avatar.clone()).unwrap_or_default(),
            pid: c.pid,
            rid: c.rid,
            user_id: c.user_id,
            sticky: c.sticky,
            like: c.star,
            object_id: c.id,
            level: 0,
            browser: client
                .clone()
                .map(|c| c.user_agent.to_string())
                .unwrap_or_default(),
            os: client.map(|c| c.os.to_string()).unwrap_or_default(),
            orig,
            addr,
            time: c.created_at.and_utc().timestamp_micros(),
            children: Default::default(),
        }
    }
}
