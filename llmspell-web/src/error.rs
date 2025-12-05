use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            WebError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            WebError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            WebError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        };

        let body = Json(ErrorResponse {
            error: status.canonical_reason().unwrap_or("Unknown").to_string(),
            message,
            details: None,
        });

        (status, body).into_response()
    }
}

// Helper implementations
impl From<anyhow::Error> for WebError {
    fn from(err: anyhow::Error) -> Self {
        WebError::Internal(err.to_string())
    }
}

impl From<String> for WebError {
    fn from(err: String) -> Self {
        WebError::Internal(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_web_error_into_response() {
        let error = WebError::NotFound("Resource not found".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body_json["error"], "Not Found");
        assert_eq!(body_json["message"], "Resource not found");
    }

    #[tokio::test]
    async fn test_anyhow_conversion() {
        let anyhow_err = anyhow::anyhow!("Something went wrong");
        let web_error: WebError = anyhow_err.into();

        match web_error {
            WebError::Internal(msg) => assert_eq!(msg, "Something went wrong"),
            _ => panic!("Expected Internal error"),
        }
    }
}
