use axum::{
    body::Body,
    extract::{State, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject (user or session id)
    exp: usize,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Check API Key
    if let Some(key_header) = headers.get("X-API-Key") {
        if let Ok(key_str) = key_header.to_str() {
            if state.config.api_keys.iter().any(|k| k == key_str) {
                return Ok(next.run(request).await);
            }
        }
    }

    // 2. Check JWT
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let secret = state.config.auth_secret.as_bytes();
                
                // Validate token
                let validation = Validation::default();
                let decoding_key = DecodingKey::from_secret(secret);
                // Decode
                if decode::<Claims>(token, &decoding_key, &validation).is_ok() {
                    return Ok(next.run(request).await);
                }
            }
        }
    }


    // 3. Fallback: Unauthorized
    Err(StatusCode::UNAUTHORIZED)
}
