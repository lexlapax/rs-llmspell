use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    api_key: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate API Key
    if state.config.api_keys.iter().any(|k| *k == payload.api_key) {
        // Generate JWT
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 3600; // 1 hour expiration

        let claims = Claims {
            sub: "user".to_owned(),
            exp: expiration,
        };

        let secret = state.config.auth_secret.as_bytes();
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(LoginResponse { token }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
