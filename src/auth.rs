use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use warp::{
    header::headers_cloned,
    http::HeaderValue,
    hyper::{header::AUTHORIZATION, HeaderMap},
    Filter, Rejection,
};

use crate::{errors::Error, models::band::BandInterface};
const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"kahloriz";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub bands: Vec<BandInterface>,
    pub id_user: i32,
    pub exp: i64,
}

pub fn create_jwt(id_user: i32, bands: Vec<BandInterface>) -> Result<String> {
    let exp = match Utc::now().checked_add_signed(Duration::hours(2)) {
        Some(t) => t.timestamp(),
        None => return Err(anyhow!("Invalid timestamp")),
    };
    let c = Claims {
        sub: id_user.to_string(),
        bands,
        id_user,
        exp,
    };

    let header = Header::new(Algorithm::HS512);
    Ok(encode(&header, &c, &EncodingKey::from_secret(JWT_SECRET))?)
}

pub async fn extract_jwt(
    headers: HeaderMap<HeaderValue>,
) -> std::result::Result<Claims, Rejection> {
    let h = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(warp::reject::custom(Error::NoAuthHeader)),
    };
    fn inter(h: &HeaderValue) -> Result<Claims> {
        let auth = std::str::from_utf8(h.as_bytes())?;

        if !auth.starts_with(BEARER) {
            Err(anyhow!("Wrong auth header"))
        } else {
            Ok(decode::<Claims>(
                auth.trim_start_matches(BEARER),
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(Algorithm::HS512),
            )?
            .claims)
        }
    }

    inter(h).map_err(|_| warp::reject::custom(Error::WrongAuthHeader))
}

pub fn with_jwt() -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    headers_cloned().and_then(extract_jwt)
}
