use crate::config::akismet::AkismetConfig;
use crate::config::RalineConfig;
use crate::views::comment::AddCommentReq;
use anyhow::Context;
use instant_akismet::{AkismetClient, AkismetOptions, CheckResult, Comment};
use spring::app::AppBuilder;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::Plugin;
use spring_web::error::Result;
use std::net::IpAddr;
use std::sync::Arc;

pub struct AkismetPlugin;

#[derive(Clone)]
pub enum Akismet {
    Disable,
    Enable(Arc<AkismetClient>),
}

#[async_trait]
impl Plugin for AkismetPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let akismet_config = app
            .get_config::<AkismetConfig>()
            .expect("akismet plugin config load failed");

        let raline_config = app
            .get_config::<RalineConfig>()
            .expect("raline plugin config load failed");

        let akismet = Self::create_client(akismet_config, raline_config);

        app.add_component(akismet);
    }
}

impl AkismetPlugin {
    fn create_client(config: AkismetConfig, raline: RalineConfig) -> Akismet {
        if config.akismet_key == "false" {
            return Akismet::Disable;
        }
        let client = reqwest::Client::default();
        let options = AkismetOptions::default();
        let akismet = AkismetClient::new(raline.site_url, config.akismet_key, client, options);
        Akismet::Enable(Arc::new(akismet))
    }
}

impl Akismet {
    /// return true is spam
    pub async fn check_comment(&self, ip: &IpAddr, comment: &AddCommentReq) -> Result<bool> {
        let akismet = match self {
            Self::Disable => return Ok(false),
            Self::Enable(akismet) => akismet,
        };

        let blog = akismet.blog.clone() + "/" + &comment.url;
        let ip_str = ip.to_string();
        let mut c = Comment::new(&blog, &ip_str);
        c = c.comment_content(&comment.comment);
        if let Some(author) = &comment.nick {
            c = c.comment_author(&author);
        }
        if let Some(mail) = &comment.mail {
            c = c.comment_author_email(&mail);
        }
        let r = akismet
            .check_comment(c)
            .await
            .context("akismet check comment failed")?;
        Ok(r == CheckResult::Spam)
    }
}
