use std::ops::Deref;

use crate::model::sea_orm_active_enums::UserType;
use crate::model::users;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use spring_web::async_trait;
use spring_web::axum::http::header;
use spring_web::axum::http::request::Parts;
use spring_web::axum::RequestPartsExt;
use spring_web::error::{KnownWebError, Result, WebError};
use spring_web::extractor::FromRequestParts;

lazy_static! {
    static ref DECODE_KEY: DecodingKey =
        DecodingKey::from_rsa_pem(include_bytes!("./keys/public.key"))
            .expect("public key parse failed");
    static ref ENCODE_KEY: EncodingKey =
        EncodingKey::from_rsa_pem(include_bytes!("./keys/private.key"))
            .expect("private key parse failed");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub uid: i32,
    pub ty: UserType,
    pub mail: Option<String>,
    exp: u64,
}

impl Claims {
    pub fn new(u: &users::Model) -> Self {
        Self {
            uid: u.id,
            ty: u.r#type.clone(),
            mail: u.email.clone(),
            exp: jsonwebtoken::get_current_timestamp() + 360 * 24 * 60 * 60 * 1000,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| KnownWebError::unauthorized("invalid token"))?;
        // Decode the user data
        let claims = decode(bearer.token())?;

        Ok(claims)
    }
}

pub struct OptionalClaims(Option<Claims>);

impl OptionalClaims {
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl Deref for OptionalClaims {
    type Target = Option<Claims>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for OptionalClaims
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        if !parts.headers.contains_key(header::AUTHORIZATION) {
            return Ok(Self(None));
        }
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| KnownWebError::unauthorized("invalid token"))?;
        // Decode the user data
        let claims = decode(bearer.token())?;

        Ok(Self(Some(claims)))
    }
}

/// JWT encode
pub fn encode(claims: Claims) -> Result<String> {
    let header = Header::new(Algorithm::RS256);

    let token = jsonwebtoken::encode::<Claims>(&header, &claims, &ENCODE_KEY)
        .map_err(|_| KnownWebError::internal_server_error("Token created error"))?;

    Ok(token)
}

/// JWT decode
pub fn decode(token: &str) -> Result<Claims> {
    let validation = Validation::new(Algorithm::RS256);
    let token_data =
        jsonwebtoken::decode::<Claims>(&token, &DECODE_KEY, &validation).map_err(|e| {
            tracing::error!("{:?}", e);
            KnownWebError::unauthorized("invalid token")
        })?;
    Ok(token_data.claims)
}
