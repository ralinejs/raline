use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthQuery {
    pub redirect: Option<String>,
}
