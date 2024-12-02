use delegate_attr::delegate;
use spring::{app::AppBuilder, async_trait, plugin::Plugin};
use std::sync::Arc;
use uaparser::{Parser, UserAgent, UserAgentParser, OS};

pub struct UAParserPlugin;

#[async_trait]
impl Plugin for UAParserPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let parser =
            UserAgentParser::from_yaml("./data/ua-regexes.yaml").expect("Parser creation failed");
        app.add_component(UAParser(Arc::new(parser)));
    }
}

#[derive(Debug, Clone)]
pub struct UAParser(Arc<UserAgentParser>);

#[delegate(self.0)]
impl Parser for UAParser {
    fn parse<'a>(&self, user_agent: &'a str) -> uaparser::Client<'a> {}

    fn parse_device<'a>(&self, user_agent: &'a str) -> uaparser::Device<'a> {}

    fn parse_os<'a>(&self, user_agent: &'a str) -> uaparser::OS<'a> {}

    fn parse_user_agent<'a>(&self, user_agent: &'a str) -> uaparser::UserAgent<'a> {}
}

pub trait ToStringExt {
    fn to_string(&self) -> String;
}

impl<'a> ToStringExt for UserAgent<'a> {
    fn to_string(&self) -> String {
        let Self {
            family,
            major,
            minor,
            patch,
        } = self;
        let mut string = format!("{family}");
        if let Some(major) = major {
            string = string + " " + major;
        }
        if let Some(minor) = minor {
            string = string + "." + minor;
        }
        if let Some(patch) = patch {
            string = string + "." + patch;
        }
        string
    }
}

impl<'a> ToStringExt for OS<'a> {
    fn to_string(&self) -> String {
        let Self {
            family,
            major,
            minor,
            patch,
            ..
        } = self;
        let mut string = format!("{family}");
        if let Some(major) = major {
            string = string + " " + major;
        }
        if let Some(minor) = minor {
            string = string + "." + minor;
        }
        if let Some(patch) = patch {
            string = string + "." + patch;
        }
        string
    }
}
