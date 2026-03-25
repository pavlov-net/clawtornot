use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::api::rate_limit::{self, RateLimiter};
use crate::error::AppError;
use crate::models::agent;

#[derive(Clone)]
pub struct AuthAgent(pub agent::Agent);

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn auth_middleware(
    State(pool): State<SqlitePool>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(AppError::unauthorized)?;

    let hash = hash_api_key(auth_header);
    let found = agent::find_by_api_key_hash(&pool, &hash)
        .await?
        .ok_or_else(AppError::unauthorized)?;

    if let Some(limiter) = req.extensions().get::<RateLimiter>() {
        if let Err(retry_after) = limiter.check_general(&hash).await {
            return Ok(rate_limit::rate_limit_response(retry_after, "Rate limit exceeded"));
        }
    }

    req.extensions_mut().insert(AuthAgent(found));
    Ok(next.run(req).await)
}
