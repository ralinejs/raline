use std::collections::HashMap;

use spring_web::axum::response::{IntoResponse, Redirect};
use spring_web::error::{KnownWebError, Result};
use spring_web::extractor::{Config, Query};
use spring_web::get;

use crate::config::RalineConfig;

type MapQuery = Query<HashMap<String, String>>;

#[get("/api/oauth")]
async fn oauth(
    Query(params): MapQuery,
    Config(config): Config<RalineConfig>,
) -> Result<impl IntoResponse> {
    let code = params.get("code");
    let ty = params
        .get("type")
        .ok_or_else(|| KnownWebError::bad_request("type is empty"))?;
    let redirect = params.get("redirect");
    match code {
        None => {
            let server_url = config.server_url;

            return Ok(Redirect::to(&format!("")));
        }
        Some(code) => Ok(Redirect::to(&format!(""))),
    }
}
