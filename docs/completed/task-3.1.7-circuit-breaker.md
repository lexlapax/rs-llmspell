# Task 3.1.7: Circuit Breaker Implementation

## Overview

The Circuit Breaker pattern provides fault tolerance for external service calls by preventing cascading failures. It monitors for failures and automatically opens/closes based on configurable thresholds.

**Important Note**: This framework is implemented as a **utility in llmspell-utils**, not as a tool in llmspell-tools. This is because:
- It's infrastructure used by both tools and providers
- It's not directly callable by agents
- It doesn't implement the Tool trait
- It provides shared fault tolerance across the entire system

## Architecture

### Core Components

1. **CircuitBreaker**: Core implementation for a single service
2. **CircuitBreakerManager**: Manages multiple circuit breakers per service
3. **CircuitState**: Three states - Closed (normal), Open (rejecting), HalfOpen (testing)
4. **CircuitBreakerConfig**: Configurable thresholds and timeouts
5. **CircuitMetrics**: Real-time metrics and monitoring

### Design Principles

- **State Machine**: Clear transitions between Closed → Open → HalfOpen → Closed
- **Configurable Thresholds**: Failure count, percentage, time windows
- **Automatic Recovery**: Self-healing with configurable test periods
- **Per-Service Configuration**: Different settings for different services
- **Metrics & Alerting**: Real-time monitoring and alert capabilities

## Implementation Details

### Circuit States

```rust
pub enum CircuitState {
    Closed,    // Normal operation - requests allowed
    Open,      // Failure threshold exceeded - requests rejected
    HalfOpen,  // Testing recovery - limited requests allowed
}
```

### State Transitions

1. **Closed → Open**: When failure threshold is exceeded
2. **Open → HalfOpen**: After reset timeout expires
3. **HalfOpen → Closed**: When success threshold is met
4. **HalfOpen → Open**: On any failure during recovery

### Configuration Options

```rust
pub struct CircuitBreakerConfig {
    // Failure thresholds
    pub failure_threshold_count: u32,
    pub failure_threshold_percentage: Option<f32>,
    pub failure_window: Option<Duration>,
    
    // Recovery settings
    pub reset_timeout: Duration,
    pub success_threshold_count: u32,
    pub test_request_count: u32,
    
    // Alerting
    pub alert_handler: Option<AlertHandler>,
}
```

## Usage Examples

### Basic Circuit Breaker

```rust
use llmspell_utils::{CircuitBreaker, CircuitBreakerConfig};

// Create circuit breaker with custom config
let config = CircuitBreakerConfig::new()
    .with_failure_threshold(5)
    .with_reset_timeout(Duration::from_secs(60))
    .with_success_threshold(3);

let breaker = CircuitBreaker::new(config);

// Execute operation with protection
let result = breaker.execute(|| {
    Box::pin(async {
        // Your external service call here
        make_api_call().await
    })
}).await;
```

### Using Circuit Breaker Manager

```rust
use llmspell_utils::{CircuitBreakerManager, ServicePresets};

// Create manager with default configuration
let manager = CircuitBreakerManager::new();

// Configure specific services
manager.configure_service(
    "critical-api",
    ServicePresets::critical_service()
).await;

// Execute with automatic circuit breaker
let result = manager.execute("api.example.com", || {
    Box::pin(async {
        // Your API call
        client.get("https://api.example.com/data").send().await
    })
}).await;
```

### Service Presets

Pre-configured settings for common service types:

```rust
// HTTP APIs - moderate protection
ServicePresets::http_api()
// Failure threshold: 5, Reset: 30s

// Database connections - quick recovery
ServicePresets::database()
// Failure threshold: 3, Reset: 60s

// Critical services - conservative
ServicePresets::critical_service()
// Failure threshold: 2, Reset: 5 minutes

// High-volume services - lenient
ServicePresets::high_volume()
// Failure threshold: 20 or 50%, Reset: 30s
```

### Monitoring and Alerts

```rust
// Set up alert handler
let config = CircuitBreakerConfig::new()
    .with_alert_handler(|message| {
        eprintln!("CIRCUIT BREAKER ALERT: {}", message);
        // Send to monitoring system
    });

// Get metrics
let metrics = breaker.metrics().await;
println!("Success rate: {:.2}%", metrics.success_rate());
println!("State: {:?}", metrics.current_state);
println!("Rejections: {}", metrics.total_rejected);

// Check circuit health
if metrics.is_critical() {
    // Circuit is open
} else if metrics.is_degraded() {
    // Circuit is half-open or showing failures
}
```

## Integration with Tools

### WebSearchTool Example

```rust
impl WebSearchTool {
    async fn search_with_circuit_breaker(&self, query: &str) -> Result<SearchResults> {
        let manager = &self.circuit_breaker_manager;
        
        manager.execute("duckduckgo", || {
            Box::pin(self.perform_search(query))
        }).await
    }
}
```

### Rate Limiting + Circuit Breaker

```rust
// Combined protection
let result = rate_limiter
    .execute_with_retry("api-service", || {
        Box::pin(async {
            circuit_breaker.execute(|| {
                Box::pin(make_api_call())
            }).await
        })
    })
    .await;
```

## Metrics and Monitoring

### Available Metrics

- **Total Requests**: allowed, rejected
- **Operations**: successes, failures
- **State Info**: current state, time in state, state changes
- **Performance**: response times (average, p95)
- **Health Status**: healthy, degraded, critical

### Alert Levels

```rust
pub enum AlertLevel {
    Healthy,   // Circuit closed, high success rate
    Warning,   // Circuit half-open or degraded performance
    Critical,  // Circuit open
}
```

## Best Practices

1. **Choose Appropriate Thresholds**
   - Consider service importance and traffic volume
   - Start conservative and adjust based on metrics

2. **Set Reasonable Timeouts**
   - Reset timeout should allow service to recover
   - Too short: Circuit may oscillate
   - Too long: Slow recovery

3. **Monitor Metrics**
   - Track state changes and success rates
   - Set up alerts for circuit opens
   - Review patterns to optimize settings

4. **Test Recovery**
   - Half-open state limits test traffic
   - Configure success threshold for confidence
   - Monitor recovery patterns

5. **Layer Protection**
   - Combine with rate limiting
   - Add retry logic with backoff
   - Implement timeouts

## Testing

Comprehensive test coverage includes:
- State transition testing
- Concurrent access scenarios  
- Alert handler verification
- Metrics accuracy
- Recovery testing

Run tests:
```bash
cargo test -p llmspell-utils circuit_breaker
```

## Performance Considerations

- **Minimal Overhead**: ~1μs for state check
- **Async Operations**: Non-blocking state management
- **Memory Efficient**: O(n) where n = number of services
- **Lock Contention**: RwLock for read-heavy workloads

## Future Enhancements

1. **Distributed Circuit Breakers**: Share state across instances
2. **Adaptive Thresholds**: ML-based threshold adjustment
3. **Circuit Breaker Chains**: Cascading protection
4. **Advanced Metrics**: Latency-based decisions
5. **Integration with Service Mesh**: Envoy/Istio compatibility

## Conclusion

The Circuit Breaker pattern provides essential fault tolerance for external service integrations. Combined with rate limiting, it creates a robust protection layer that prevents cascading failures while enabling automatic recovery.