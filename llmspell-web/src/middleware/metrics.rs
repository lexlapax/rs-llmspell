use axum::{extract::Request, middleware::Next, response::Response};
use metrics::{counter, histogram};
use std::time::Instant;

pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    histogram!("http_request_duration_seconds", "path" => path.clone(), "method" => method.to_string(), "status" => status.clone()).record(latency);
    counter!("http_requests_total", "path" => path, "method" => method.to_string(), "status" => status).increment(1);

    response
}
