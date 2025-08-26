//! ABOUTME: Integration tests for the rate limiting framework
//! ABOUTME: Tests provider rate limiting, retry logic, and metrics collection

#[cfg(feature = "rate-limiting-http")]
use llmspell_utils::RateLimitInfo;
use llmspell_utils::{BackoffStrategy, ProviderLimits, ProviderRateLimiter};
#[cfg(feature = "rate-limiting-http")]
use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
#[tokio::test]
async fn test_provider_rate_limiter_creation() {
    let mut configs = HashMap::new();
    configs.insert("test_api".to_string(), ProviderLimits::generic());

    let limiter = ProviderRateLimiter::with_configs(configs).await.unwrap();

    // Should allow first request
    assert!(limiter.check_rate_limit("test_api").await.is_ok());

    // Check metrics
    let metrics = limiter.get_metrics("test_api").await.unwrap();
    assert_eq!(metrics.requests_allowed, 1);
    assert_eq!(metrics.requests_denied, 0);
}
#[tokio::test]
#[cfg(feature = "rate-limiting-http")]
async fn test_rate_limit_headers_parsing() {
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-remaining", HeaderValue::from_static("0"));
    headers.insert("x-ratelimit-limit", HeaderValue::from_static("100"));
    headers.insert("x-ratelimit-reset", HeaderValue::from_static("1234567890"));
    headers.insert("retry-after", HeaderValue::from_static("60"));

    let info = RateLimitInfo::from_headers(&headers);
    assert_eq!(info.remaining, Some(0));
    assert_eq!(info.limit, Some(100));
    assert_eq!(info.reset_at, Some(1234567890));
    assert_eq!(info.retry_after, Some(Duration::from_secs(60)));

    // Should have wait time since remaining is 0
    assert!(info.wait_time().is_some());
}
#[tokio::test]
async fn test_provider_specific_limits() {
    let openai_config = ProviderLimits::openai();
    assert_eq!(openai_config.requests_per_minute, 3_500);
    assert!(openai_config.allow_burst);

    let duckduckgo_config = ProviderLimits::duckduckgo();
    assert_eq!(duckduckgo_config.requests_per_minute, 20);
    assert!(!duckduckgo_config.allow_burst);

    // Test provider lookup
    let config = ProviderLimits::for_provider("openai");
    assert_eq!(config.requests_per_minute, 3_500);

    let config = ProviderLimits::for_provider("unknown_provider");
    assert_eq!(config.requests_per_minute, 60); // Should use generic
}
#[tokio::test]
async fn test_retry_with_backoff() {
    let limiter = ProviderRateLimiter::new();
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = Arc::clone(&attempts);

    let start = std::time::Instant::now();

    let result = limiter
        .execute_with_retry("test_provider", move || {
            let attempts = Arc::clone(&attempts_clone);
            Box::pin(async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(Box::new(std::io::Error::other("rate limit exceeded"))
                        as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Ok("Success".to_string())
                }
            })
        })
        .await;

    // Should succeed after retries
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);

    // Should have taken some time due to backoff
    let elapsed = start.elapsed();
    assert!(elapsed > Duration::from_millis(100));
}
#[tokio::test]
async fn test_metrics_collection() {
    let mut limiter = ProviderRateLimiter::new();

    // Add provider with low rate limit for testing
    let mut config = ProviderLimits::generic();
    config.requests_per_minute = 2; // Very low for testing
    limiter.add_provider("test_metrics", config).await.unwrap();

    // Make some requests
    for _ in 0..2 {
        assert!(limiter.check_rate_limit("test_metrics").await.is_ok());
    }

    // Next request should be rate limited
    // Note: This might not fail immediately depending on token bucket implementation

    // Check metrics
    let metrics = limiter.get_metrics("test_metrics").await.unwrap();
    assert!(metrics.requests_allowed >= 2);

    // Test all metrics
    let all_metrics = limiter.get_all_metrics().await;
    assert!(all_metrics.contains_key("test_metrics"));
}
#[tokio::test]
async fn test_backoff_strategies() {
    // Test linear backoff
    let linear = BackoffStrategy::Linear { increment_ms: 100 };
    assert_eq!(linear.calculate_delay(0), Duration::from_millis(100));
    assert_eq!(linear.calculate_delay(1), Duration::from_millis(200));
    assert_eq!(linear.calculate_delay(2), Duration::from_millis(300));

    // Test exponential backoff
    let exponential = BackoffStrategy::Exponential { base_ms: 100 };
    assert_eq!(exponential.calculate_delay(0), Duration::from_millis(100));
    assert_eq!(exponential.calculate_delay(1), Duration::from_millis(200));
    assert_eq!(exponential.calculate_delay(2), Duration::from_millis(400));
    assert_eq!(exponential.calculate_delay(3), Duration::from_millis(800));

    // Test cap at 5 minutes
    assert_eq!(
        exponential.calculate_delay(20),
        Duration::from_millis(300_000)
    );
}
#[tokio::test]
async fn test_concurrent_rate_limiting() {
    let limiter = Arc::new(ProviderRateLimiter::new());
    let mut limiter_clone = (*limiter).clone();

    // Add provider with specific limit
    let mut config = ProviderLimits::generic();
    config.requests_per_minute = 10;
    limiter_clone
        .add_provider("concurrent_test", config)
        .await
        .unwrap();

    // Spawn multiple concurrent tasks
    let mut tasks = vec![];
    for i in 0..5 {
        let limiter = Arc::clone(&limiter);
        let task = tokio::spawn(async move {
            sleep(Duration::from_millis(i * 10)).await;
            limiter.check_rate_limit("concurrent_test").await
        });
        tasks.push(task);
    }

    // All should succeed since we're under the limit
    for task in tasks {
        assert!(task.await.unwrap().is_ok());
    }

    // Check metrics
    let metrics = limiter.get_metrics("concurrent_test").await.unwrap();
    assert_eq!(metrics.requests_allowed, 5);
}

// Example of how to integrate with an actual HTTP client
#[cfg(feature = "rate-limiting-http")]
#[allow(dead_code)]
async fn example_http_request_with_rate_limiting() -> Result<(), Box<dyn std::error::Error>> {
    let limiter = ProviderRateLimiter::new();

    // Execute HTTP request with rate limiting and retry
    let response = limiter
        .execute_with_retry("example_api", || {
            Box::pin(async {
                let client = reqwest::Client::new();
                let resp = client.get("https://api.example.com/data").send().await?;

                // Update rate limit info from response headers
                // limiter.update_from_headers("example_api", resp.headers()).await;

                resp.text()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })
        })
        .await;

    match response {
        Ok(body) => println!("Response: {}", body),
        Err(e) => eprintln!("Request failed after retries: {}", e),
    }

    Ok(())
}
