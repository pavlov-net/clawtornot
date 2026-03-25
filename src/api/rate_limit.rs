use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

type Bucket = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

#[derive(Clone)]
pub struct RateLimiter {
    general: Bucket,
    voting: Bucket,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            general: Arc::new(Mutex::new(HashMap::new())),
            voting: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check_general(&self, key_hash: &str) -> Result<(), u64> {
        Self::check(&self.general, key_hash, 60, 60).await
    }

    pub async fn check_voting(&self, key_hash: &str) -> Result<(), u64> {
        Self::check(&self.voting, key_hash, 30, 3600).await
    }

    async fn check(bucket: &Bucket, key: &str, max: u32, window_secs: u64) -> Result<(), u64> {
        let mut map = bucket.lock().await;
        let now = Instant::now();
        let entry = map.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1).as_secs() >= window_secs {
            *entry = (1, now);
            Ok(())
        } else if entry.0 >= max {
            let retry_after = window_secs - now.duration_since(entry.1).as_secs();
            Err(retry_after)
        } else {
            entry.0 += 1;
            Ok(())
        }
    }
}

pub fn rate_limit_response(retry_after: u64, message: &str) -> Response {
    let mut resp = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response();
    resp.headers_mut().insert(
        "retry-after",
        HeaderValue::from_str(&retry_after.to_string()).unwrap(),
    );
    resp
}
