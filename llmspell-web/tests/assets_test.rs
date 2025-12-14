use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_serve_index_html() {
    let app = Router::new().fallback(llmspell_web::handlers::assets::static_handler);

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html"));
}

#[tokio::test]
async fn test_spa_fallback() {
    let app = Router::new().fallback(llmspell_web::handlers::assets::static_handler);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/random-page")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html"));
    // Verify it serves index.html content (simple check)
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("<!doctype html>") || body_str.contains("<html"));
}

#[tokio::test]
async fn test_api_exclusion() {
    let app = Router::new().fallback(llmspell_web::handlers::assets::static_handler);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/some-resource")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_serve_static_file() {
    let app = Router::new().fallback(llmspell_web::handlers::assets::static_handler);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/vite.svg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("image/svg+xml"));
}
