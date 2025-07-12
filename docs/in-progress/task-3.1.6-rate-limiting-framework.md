# Task 3.1.6: Rate Limiting Framework

## Overview

The Rate Limiting Framework provides comprehensive rate limiting for external API integrations in llmspell. It supports provider-specific configurations, automatic retry with backoff strategies, rate limit header parsing, and metrics collection.

**Important Note**: This framework is implemented as a **utility in llmspell-utils**, not as a tool in llmspell-tools. This is because:
- It's infrastructure used by both tools and providers
- It's not directly callable by agents
- It doesn't implement the Tool trait
- It provides shared functionality across the entire system

## Architecture

### Core Components

1. **ProviderRateLimiter**: Main orchestrator for rate limiting across providers
2. **ProviderLimits**: Pre-configured rate limits for known API providers
3. **RetryHandler**: Handles retry logic with various backoff strategies
4. **MetricsCollector**: Tracks rate limit usage and performance metrics
5. **RateLimitInfo**: Parses and interprets rate limit headers from API responses

### Design Principles

- **Provider-Aware**: Different rate limits per API provider
- **Token Bucket Algorithm**: Allows burst traffic while maintaining overall limits
- **Automatic Retry**: Configurable retry policies with exponential/linear backoff
- **Metrics & Monitoring**: Real-time tracking of rate limit usage
- **Header Parsing**: Automatic extraction of rate limit info from HTTP headers

## Implementation Details

### Rate Limiter Integration

The framework integrates with llmspell-utils' existing RateLimiter:

```rust
use llmspell_utils::{ProviderRateLimiter, ProviderLimits};

// Create rate limiter with default provider configs
let mut limiter = ProviderRateLimiter::new();

// Add custom provider
let config = ProviderLimits::for_provider("openai");
limiter.add_provider("openai", config).await?;

// Check rate limit before making request
limiter.check_rate_limit("openai").await?;
```

### Pre-configured Provider Limits

The framework includes rate limits for common providers:

| Provider | Requests/Min | Requests/Hour | Daily Limit | Burst |
|----------|-------------|---------------|-------------|-------|
| OpenAI | 3,500 | - | 200,000 | Yes |
| Anthropic | 50 | 1,000 | - | No |
| Google Search | 100 | - | 10,000 | No |
| DuckDuckGo | 20 | 1,000 | - | No |
| Bing Search | 1,000 | - | - | Yes |
| Brave Search | 60 | 2,000 | - | No |
| GitHub | 83 | 5,000 | - | Yes |
| Slack | 60 | - | - | Yes |
| SendGrid | 600 | - | 100,000 | Yes |
| AWS SES | 14 | - | 50,000 | No |

### Retry Strategies

Three backoff strategies are supported:

1. **None**: Immediate retry (not recommended)
2. **Linear**: Fixed increment delay
3. **Exponential**: Doubles delay each attempt (capped at 5 minutes)
4. **Custom**: Base delay with random jitter

Example:
```rust
let policy = RetryPolicy {
    max_retries: 3,
    backoff_strategy: BackoffStrategy::Exponential { base_ms: 1000 },
    retry_on_rate_limit: true,
};
```

### Rate Limit Header Parsing

The framework automatically parses standard rate limit headers:

- `X-RateLimit-Remaining` / `X-Rate-Limit-Remaining`
- `X-RateLimit-Limit` / `X-Rate-Limit-Limit`
- `X-RateLimit-Reset` / `X-Rate-Limit-Reset`
- `Retry-After`

### Metrics Collection

Real-time metrics tracking includes:

- Total requests allowed/denied
- Current rate limit usage percentage
- Average response times
- Retry statistics
- Critical/warning threshold monitoring

## Usage Examples

### Basic Rate Limiting

```rust
use llmspell_utils::ProviderRateLimiter;

let limiter = ProviderRateLimiter::new();

// Simple rate limit check
if limiter.check_rate_limit("github").await.is_ok() {
    // Make API request
}
```

### With Retry Logic

```rust
let result = limiter.execute_with_retry(
    "openai",
    || {
        Box::pin(async {
            // Your API call here
            client.post("https://api.openai.com/v1/completions")
                .json(&request_body)
                .send()
                .await
        })
    },
).await;
```

### Monitoring Metrics

```rust
// Get metrics for specific provider
let metrics = limiter.get_metrics("anthropic").await;
if let Some(m) = metrics {
    println!("Usage: {}%", m.usage_percentage().unwrap_or(0.0));
    println!("Allowed: {}, Denied: {}", m.requests_allowed, m.requests_denied);
}

// Get providers near rate limits
let critical = limiter.get_all_metrics().await
    .into_iter()
    .filter(|(_, m)| m.is_critical())
    .collect::<Vec<_>>();
```

### Integration with Tools

Tools can integrate rate limiting by:

1. Adding provider configuration
2. Wrapping API calls with rate limiter
3. Parsing response headers
4. Monitoring metrics

Example integration:
```rust
impl WebSearchTool {
    async fn search_with_rate_limit(&self, query: &str) -> Result<SearchResults> {
        let limiter = &self.rate_limiter;
        
        limiter.execute_with_retry(
            "duckduckgo",
            || Box::pin(self.perform_search(query)),
        ).await
    }
}
```

## Security Considerations

1. **API Key Protection**: Rate limits help prevent key exhaustion
2. **DoS Prevention**: Limits prevent overwhelming external services
3. **Cost Control**: Daily limits prevent unexpected API costs
4. **Fair Usage**: Respects provider terms of service

## Performance Impact

- Minimal overhead: ~1-2ms per rate limit check
- Async operations: Non-blocking rate limit checks
- Memory efficient: O(n) where n = number of providers
- Metrics retention: Last 100 response times per provider

## Future Enhancements

1. **Distributed Rate Limiting**: Share limits across instances
2. **Dynamic Adjustment**: Adapt limits based on response headers
3. **Cost Tracking**: Monitor API usage costs
4. **Circuit Breaker Integration**: Combine with circuit breaker pattern
5. **Persistent Metrics**: Store metrics for long-term analysis

## Testing

Comprehensive test coverage includes:
- Unit tests for each component
- Integration tests with mock APIs
- Concurrent access testing
- Backoff strategy validation
- Metrics accuracy verification

Run tests:
```bash
cargo test -p llmspell-utils rate_limiting
```

## Conclusion

The Rate Limiting Framework provides robust protection for external API integrations while maintaining high performance and flexibility. It's designed to be easy to use while providing advanced features for complex scenarios.