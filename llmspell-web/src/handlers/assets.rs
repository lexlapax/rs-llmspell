use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    let asset = Assets::get(&path);

    if let Some(content) = asset {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        return Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data))
            .unwrap()
            .into_response();
    }

    // SPA fallback: serve index.html for unknown routes (excluding API)
    if let Some(index) = Assets::get("index.html") {
         return Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(index.data))
            .unwrap()
            .into_response();
    }

    (StatusCode::NOT_FOUND, "Index not found").into_response()
}
