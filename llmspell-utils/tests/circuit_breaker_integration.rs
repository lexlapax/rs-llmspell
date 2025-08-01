//! ABOUTME: Integration tests for the circuit breaker pattern implementation
//! ABOUTME: Tests state transitions, thresholds, recovery, and per-service configuration

use llmspell_utils::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerManager, CircuitState, ServicePresets,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
#[tokio::test]
async fn test_circuit_breaker_basic_flow() {
    let config = CircuitBreakerConfig::new()
        .with_failure_threshold(3)
        .with_reset_timeout(Duration::from_millis(200))
        .with_success_threshold(2);

    let breaker = CircuitBreaker::new(config);

    // Initially closed
    assert_eq!(breaker.current_state().await, CircuitState::Closed);

    // Successful operations keep circuit closed
    for _ in 0..5 {
        let result = breaker
            .execute(|| Box::pin(async { Ok::<_, std::io::Error>(42) }))
            .await;
        assert!(result.is_ok());
    }
    assert_eq!(breaker.current_state().await, CircuitState::Closed);

    // Failures open the circuit
    for _ in 0..3 {
        let result = breaker
            .execute(|| {
                Box::pin(async {
                    Err::<i32, _>(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "simulated failure",
                    ))
                })
            })
            .await;
        assert!(result.is_err());
    }
    assert_eq!(breaker.current_state().await, CircuitState::Open);

    // Circuit open rejects requests
    let result = breaker
        .execute(|| Box::pin(async { Ok::<_, std::io::Error>(42) }))
        .await;
    assert!(result.is_err());

    // Wait for reset timeout
    sleep(Duration::from_millis(250)).await;

    // Circuit should transition to half-open
    let result = breaker
        .execute(|| Box::pin(async { Ok::<_, std::io::Error>(42) }))
        .await;
    assert!(result.is_ok());
    assert_eq!(breaker.current_state().await, CircuitState::HalfOpen);

    // Success in half-open closes circuit
    let result = breaker
        .execute(|| Box::pin(async { Ok::<_, std::io::Error>(42) }))
        .await;
    assert!(result.is_ok());
    assert_eq!(breaker.current_state().await, CircuitState::Closed);
}
#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let config = CircuitBreakerConfig::new()
        .with_failure_threshold(2)
        .with_reset_timeout(Duration::from_millis(100))
        .with_success_threshold(2);

    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        breaker.record_failure().await;
    }
    assert_eq!(breaker.current_state().await, CircuitState::Open);

    // Wait for reset
    sleep(Duration::from_millis(150)).await;

    // First request transitions to half-open
    assert!(breaker.allow_request().await.is_ok());
    assert_eq!(breaker.current_state().await, CircuitState::HalfOpen);

    // Failure in half-open reopens immediately
    breaker.record_failure().await;
    assert_eq!(breaker.current_state().await, CircuitState::Open);
}
#[tokio::test]
async fn test_circuit_breaker_manager() {
    let manager = CircuitBreakerManager::default();

    // Configure specific services
    manager
        .configure_service("critical-api", ServicePresets::critical_service())
        .await;
    manager
        .configure_service("high-volume-api", ServicePresets::high_volume())
        .await;

    // Execute operations on different services
    let result = manager
        .execute("critical-api", || {
            Box::pin(async { Ok::<_, std::io::Error>(true) })
        })
        .await;
    assert!(result.is_ok());

    let result = manager
        .execute("high-volume-api", || {
            Box::pin(async { Ok::<_, std::io::Error>(true) })
        })
        .await;
    assert!(result.is_ok());

    // Get metrics
    let metrics = manager.all_metrics().await;
    assert!(metrics.contains_key("critical-api"));
    assert!(metrics.contains_key("high-volume-api"));
}
#[tokio::test]
async fn test_circuit_breaker_concurrent_access() {
    let breaker = Arc::new(CircuitBreaker::new(
        CircuitBreakerConfig::new()
            .with_failure_threshold(5)
            .with_success_threshold(3),
    ));

    let success_count = Arc::new(AtomicU32::new(0));
    let failure_count = Arc::new(AtomicU32::new(0));

    // Spawn concurrent tasks
    let mut tasks = vec![];
    for i in 0..10 {
        let breaker = Arc::clone(&breaker);
        let success_count = Arc::clone(&success_count);
        let failure_count = Arc::clone(&failure_count);

        let task = tokio::spawn(async move {
            // Even tasks succeed, odd tasks fail
            if i % 2 == 0 {
                let result = breaker
                    .execute(move || Box::pin(async move { Ok::<_, std::io::Error>(i) }))
                    .await;
                if result.is_ok() {
                    success_count.fetch_add(1, Ordering::SeqCst);
                }
            } else {
                let result = breaker
                    .execute(|| {
                        Box::pin(async {
                            Err::<i32, _>(std::io::Error::new(std::io::ErrorKind::Other, "failure"))
                        })
                    })
                    .await;
                if result.is_err() {
                    failure_count.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }

    // Check metrics
    let metrics = breaker.metrics().await;
    assert!(metrics.total_allowed > 0);
    assert_eq!(
        metrics.total_successes + metrics.total_failures,
        success_count.load(Ordering::SeqCst) as u64 + failure_count.load(Ordering::SeqCst) as u64
    );
}
#[tokio::test]
async fn test_circuit_breaker_alert_handler() {
    let alerts = Arc::new(RwLock::new(Vec::<String>::new()));
    let alerts_clone = Arc::clone(&alerts);

    let config = CircuitBreakerConfig::new()
        .with_failure_threshold(2)
        .with_alert_handler(move |message| {
            let alerts = Arc::clone(&alerts_clone);
            tokio::spawn(async move {
                alerts.write().await.push(message);
            });
        });

    let breaker = CircuitBreaker::new(config);

    // Trigger circuit open
    breaker.record_failure().await;
    breaker.record_failure().await;

    // Give time for alert handler
    sleep(Duration::from_millis(50)).await;

    let alerts = alerts.read().await;
    assert_eq!(alerts.len(), 1);
    assert!(alerts[0].contains("Circuit opened"));
}
#[tokio::test]
async fn test_service_presets() {
    // Test HTTP API preset
    let http_config = ServicePresets::http_api();
    assert_eq!(http_config.failure_threshold_count, 5);
    assert_eq!(http_config.reset_timeout, Duration::from_secs(30));

    // Test database preset
    let db_config = ServicePresets::database();
    assert_eq!(db_config.failure_threshold_count, 3);
    assert_eq!(db_config.reset_timeout, Duration::from_secs(60));

    // Test critical service preset
    let critical_config = ServicePresets::critical_service();
    assert_eq!(critical_config.failure_threshold_count, 2);
    assert_eq!(critical_config.reset_timeout, Duration::from_secs(300));
}
#[tokio::test]
async fn test_circuit_breaker_metrics() {
    let breaker = CircuitBreaker::new(
        CircuitBreakerConfig::new()
            .with_failure_threshold(3)
            .with_success_threshold(2),
    );

    // Record some operations - mostly successes for healthy state
    for _ in 0..19 {
        breaker.allow_request().await.ok();
        breaker.record_success().await;
    }

    // One failure
    breaker.allow_request().await.ok();
    breaker.record_failure().await;

    let metrics = breaker.metrics().await;
    assert_eq!(metrics.total_allowed, 20);
    assert_eq!(metrics.total_successes, 19);
    assert_eq!(metrics.total_failures, 1);
    assert_eq!(metrics.success_rate(), 95.0);
    // 95% is neither healthy (>95%) nor degraded (<90%)
    assert!(!metrics.is_healthy());
    assert!(!metrics.is_degraded());

    // One more success to become healthy
    breaker.allow_request().await.ok();
    breaker.record_success().await;

    let metrics = breaker.metrics().await;
    assert!(metrics.success_rate() > 95.0);
    assert!(metrics.is_healthy());
}
#[tokio::test]
async fn test_force_state() {
    let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

    // Force to open
    breaker.force_state(CircuitState::Open).await;
    assert_eq!(breaker.current_state().await, CircuitState::Open);
    assert!(breaker.allow_request().await.is_err());

    // Force to half-open
    breaker.force_state(CircuitState::HalfOpen).await;
    assert_eq!(breaker.current_state().await, CircuitState::HalfOpen);
    assert!(breaker.allow_request().await.is_ok());

    // Force back to closed
    breaker.force_state(CircuitState::Closed).await;
    assert_eq!(breaker.current_state().await, CircuitState::Closed);
}
#[tokio::test]
async fn test_open_circuits_tracking() {
    let manager = CircuitBreakerManager::default();

    // Configure services
    manager
        .configure_service(
            "service1",
            CircuitBreakerConfig::new().with_failure_threshold(1),
        )
        .await;
    manager
        .configure_service(
            "service2",
            CircuitBreakerConfig::new().with_failure_threshold(1),
        )
        .await;
    manager
        .configure_service(
            "service3",
            CircuitBreakerConfig::new().with_failure_threshold(1),
        )
        .await;

    // Open some circuits
    let breaker1 = manager.get_or_create("service1").await;
    breaker1.record_failure().await;

    let breaker3 = manager.get_or_create("service3").await;
    breaker3.record_failure().await;

    // Check open circuits
    let open = manager.open_circuits().await;
    assert_eq!(open.len(), 2);
    assert!(open.contains(&"service1".to_string()));
    assert!(open.contains(&"service3".to_string()));
}

use tokio::sync::RwLock;

// Example of using circuit breaker with actual HTTP client
#[cfg(feature = "rate-limiting-http")]
#[allow(dead_code)]
async fn example_http_with_circuit_breaker() -> Result<(), Box<dyn std::error::Error>> {
    let manager = CircuitBreakerManager::with_default_config(ServicePresets::http_api);

    // Execute HTTP request with circuit breaker protection
    let response = manager
        .execute("api.example.com", || {
            Box::pin(async {
                let client = reqwest::Client::new();
                client
                    .get("https://api.example.com/data")
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await?
                    .text()
                    .await
            })
        })
        .await;

    match response {
        Ok(body) => println!("Response: {}", body),
        Err(e) => eprintln!("Request failed: {}", e),
    }

    // Check circuit metrics
    let metrics = manager.all_metrics().await;
    if let Some(api_metrics) = metrics.get("api.example.com") {
        println!("Success rate: {:.2}%", api_metrics.success_rate());
        println!("Circuit state: {:?}", api_metrics.current_state);
    }

    Ok(())
}
