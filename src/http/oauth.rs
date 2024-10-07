use crate::service::auth::AuthService;
use spring_web::axum::response::{IntoResponse, Redirect};
use spring_web::error::Result;
use spring_web::extractor::{Component, Path, RawQuery};
use spring_web::get;

#[get("/api/oauth/:ty/render")]
async fn oauth_render(
    Path(ty): Path<String>,
    Component(auth): Component<AuthService>,
) -> Result<impl IntoResponse> {
    let auth_server = auth.get_auth_server(&ty)?;
    Ok(Redirect::to(&auth_server.authorize().await?))
}

#[get("/api/oauth/:ty/callback")]
async fn oauth_callback(
    Path(ty): Path<String>,
    RawQuery(query): RawQuery,
    Component(auth): Component<AuthService>,
) -> Result<impl IntoResponse> {
    let auth_server = auth.get_auth_server(&ty)?;
    auth_server.login(query.unwrap_or_default()).await?;
    Ok("")
}
