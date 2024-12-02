use crate::views::oauth::OAuthQuery;
use crate::service::auth::AuthService;
use crate::utils::jwt::OptionalClaims;
use spring_web::axum::response::{IntoResponse, Redirect};
use spring_web::error::Result;
use spring_web::extractor::{Component, Path, Query, RawQuery};
use spring_web::get;

#[get("/api/oauth/:ty/render")]
async fn oauth_render(
    Path(ty): Path<String>,
    Component(auth): Component<AuthService>,
    Query(auth_query): Query<OAuthQuery>,
) -> Result<impl IntoResponse> {
    let auth_server = auth.get_auth_server(&ty, auth_query)?;
    Ok(Redirect::to(&auth_server.authorize().await?))
}

#[get("/api/oauth/:ty/callback")]
async fn oauth_callback(
    claims: OptionalClaims,
    Path(ty): Path<String>,
    RawQuery(query): RawQuery,
    Query(auth_query): Query<OAuthQuery>,
    Component(auth): Component<AuthService>,
) -> Result<impl IntoResponse> {
    let auth_server = auth.get_auth_server(&ty, auth_query)?;
    auth_server.login(query.unwrap_or_default(), claims).await
}
