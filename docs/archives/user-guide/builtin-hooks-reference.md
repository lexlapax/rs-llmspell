# Built-in Hooks Reference

## Introduction

rs-llmspell provides a comprehensive set of production-ready hooks that you can use immediately without writing custom hook logic. These built-in hooks cover common use cases like monitoring, security, caching, and error handling.

## Using Built-in Hooks

### Registration

```lua
-- Register a built-in hook by name
Hook.enable_builtin("RateLimiter", {
    max_requests_per_minute = 60,
    by_component = true
})

-- Register multiple built-in hooks
Hook.enable_builtin({
    {name = "SecurityValidator", config = {strict_mode = true}},
    {name = "CostTracker", config = {alert_threshold = 10.0}},
    {name = "PerformanceMonitor"}
})
```

## Security Hooks

### 1. SecurityValidator

**Purpose**: Validates inputs for security threats
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`
**Priority**: `highest`

```lua
Hook.enable_builtin("SecurityValidator", {
    strict_mode = true,              -- Enable all checks
    check_sql_injection = true,      -- SQL injection detection
    check_command_injection = true,  -- Command injection detection
    check_path_traversal = true,     -- Path traversal detection
    check_xss = true,               -- XSS detection
    max_input_length = 50000,       -- Maximum input size
    blocked_patterns = {            -- Custom patterns to block
        "password.*=.*",
        "api[_-]?key.*=.*"
    }
})
```

**Example Detection**:
```lua
-- This input would be blocked:
"'; DROP TABLE users; --"
"../../etc/passwd"
"<script>alert('xss')</script>"
```

### 2. InputSanitizer

**Purpose**: Cleans and normalizes inputs
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`
**Priority**: `high`

```lua
Hook.enable_builtin("InputSanitizer", {
    trim_whitespace = true,         -- Remove leading/trailing spaces
    normalize_unicode = true,       -- Normalize Unicode characters
    remove_control_chars = true,    -- Remove non-printable characters
    escape_html = false,           -- HTML entity encoding
    max_length = 10000,            -- Truncate long inputs
    redact_patterns = {            -- Patterns to redact
        "ssn:\\s*\\d{3}-\\d{2}-\\d{4}",
        "credit.*card.*\\d{16}"
    },
    redact_replacement = "[REDACTED]"
})
```

### 3. AuthorizationChecker

**Purpose**: Enforces access control
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`, `BeforeStateWrite`
**Priority**: `highest`

```lua
Hook.enable_builtin("AuthorizationChecker", {
    require_auth = true,
    auth_header = "Authorization",
    allowed_roles = {
        ["admin"] = {"*"},           -- Admin can access everything
        ["user"] = {"agent.*", "tool.read.*"},
        ["guest"] = {"agent.chat"}
    },
    deny_by_default = true
})
```

## Performance Hooks

### 4. PerformanceMonitor

**Purpose**: Tracks execution times and performance metrics
**Hook Points**: All lifecycle hooks
**Priority**: `lowest`

```lua
Hook.enable_builtin("PerformanceMonitor", {
    enable_profiling = true,        -- Detailed profiling
    track_memory = true,            -- Memory usage tracking
    track_cpu = true,              -- CPU usage tracking
    slow_threshold_ms = 1000,      -- Warn on slow operations
    export_interval = 60,          -- Export metrics every 60s
    export_format = "prometheus"    -- prometheus, json, statsd
})
```

**Metrics Exported**:
- `llmspell_hook_duration_ms` - Hook execution time
- `llmspell_agent_duration_ms` - Agent execution time
- `llmspell_tool_duration_ms` - Tool execution time
- `llmspell_memory_usage_bytes` - Memory usage
- `llmspell_cpu_usage_percent` - CPU usage

### 5. CachingOptimizer

**Purpose**: Intelligent caching of results
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`
**Priority**: `normal`

```lua
Hook.enable_builtin("CachingOptimizer", {
    cache_agents = true,            -- Cache agent responses
    cache_tools = true,             -- Cache tool results
    default_ttl = 300,              -- 5 minutes default TTL
    max_cache_size_mb = 100,        -- Maximum cache size
    cache_key_fields = {            -- Fields to include in cache key
        "input.text",
        "model",
        "temperature"
    },
    ttl_by_component = {
        ["expensive-agent"] = 3600,  -- 1 hour for expensive ops
        ["volatile-tool"] = 60       -- 1 minute for volatile data
    }
})
```

### 6. ResourceLimiter

**Purpose**: Enforces resource usage limits
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`, `BeforeWorkflowStage`
**Priority**: `high`

```lua
Hook.enable_builtin("ResourceLimiter", {
    max_memory_mb = 512,            -- Maximum memory per operation
    max_cpu_seconds = 30,           -- Maximum CPU time
    max_concurrent = 10,            -- Maximum concurrent operations
    max_queue_size = 100,           -- Maximum queued operations
    timeout_ms = 60000,             -- Operation timeout
    action_on_limit = "cancel"      -- cancel, queue, or throttle
})
```

## Cost Management Hooks

### 7. CostTracker

**Purpose**: Tracks LLM token usage and costs
**Hook Points**: `AfterAgentExecution`
**Priority**: `low`

```lua
Hook.enable_builtin("CostTracker", {
    track_by_agent = true,          -- Track per agent
    track_by_user = true,           -- Track per user
    alert_threshold = 10.0,         -- Alert at $10
    daily_limit = 50.0,             -- Daily spending limit
    model_costs = {                 -- Cost per 1K tokens
        ["gpt-4"] = {input = 0.03, output = 0.06},
        ["gpt-3.5-turbo"] = {input = 0.001, output = 0.002},
        ["claude-3-opus"] = {input = 0.015, output = 0.075}
    },
    export_format = "csv",          -- csv, json, or webhook
    export_path = "/metrics/costs"
})
```

**Events Published**:
- `cost.threshold.warning` - Near threshold
- `cost.threshold.exceeded` - Threshold exceeded
- `cost.daily_limit.exceeded` - Daily limit hit

### 8. TokenOptimizer

**Purpose**: Reduces token usage intelligently
**Hook Points**: `BeforeAgentExecution`
**Priority**: `normal`

```lua
Hook.enable_builtin("TokenOptimizer", {
    compression_level = "balanced", -- minimal, balanced, aggressive
    remove_redundancy = true,       -- Remove redundant content
    summarize_context = true,       -- Summarize long contexts
    max_context_tokens = 4000,      -- Maximum context size
    preserve_recent = 1000,         -- Keep recent tokens intact
    techniques = {
        whitespace_reduction = true,
        abbreviation = true,
        semantic_compression = true
    }
})
```

## Reliability Hooks

### 9. RetryHandler

**Purpose**: Automatic retry with exponential backoff
**Hook Points**: `AgentError`, `ToolError`, `WorkflowError`
**Priority**: `high`

```lua
Hook.enable_builtin("RetryHandler", {
    max_retries = 3,               -- Maximum retry attempts
    initial_delay_ms = 1000,       -- Initial retry delay
    max_delay_ms = 30000,          -- Maximum retry delay
    exponential_base = 2,          -- Exponential backoff base
    jitter = true,                 -- Add random jitter
    retryable_errors = {           -- Patterns for retryable errors
        "timeout",
        "rate.*limit",
        "temporary.*failure",
        "connection.*refused"
    },
    retry_status_codes = {429, 502, 503, 504}
})
```

### 10. CircuitBreaker

**Purpose**: Prevents cascading failures
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`
**Priority**: `high`

```lua
Hook.enable_builtin("CircuitBreaker", {
    failure_threshold = 5,          -- Failures before opening
    success_threshold = 2,          -- Successes to close
    timeout_seconds = 30,           -- Time before half-open
    track_by_component = true,      -- Separate breaker per component
    failure_patterns = {            -- What counts as failure
        "error",
        "timeout",
        "failed"
    },
    fallback_action = "cache"       -- cache, default, or cancel
})
```

### 11. HealthChecker

**Purpose**: Monitors component health
**Hook Points**: All lifecycle hooks
**Priority**: `low`

```lua
Hook.enable_builtin("HealthChecker", {
    check_interval = 60,            -- Health check interval
    unhealthy_threshold = 3,        -- Failures before unhealthy
    healthy_threshold = 2,          -- Successes to recover
    checks = {
        response_time = {max_ms = 5000},
        error_rate = {max_percent = 5},
        throughput = {min_per_minute = 10}
    },
    on_unhealthy = "alert"          -- alert, disable, or fallback
})
```

## Monitoring Hooks

### 12. MetricsCollector

**Purpose**: Comprehensive metrics collection
**Hook Points**: All hooks
**Priority**: `lowest`

```lua
Hook.enable_builtin("MetricsCollector", {
    metrics = {                     -- Metrics to collect
        "execution_time",
        "token_usage",
        "error_rate",
        "throughput",
        "queue_depth",
        "cache_hit_rate"
    },
    aggregation_window = 60,        -- Aggregate every 60s
    percentiles = {50, 90, 95, 99}, -- Percentiles to track
    export_format = "prometheus",
    export_endpoint = "http://metrics:9090/metrics",
    labels = {                      -- Additional labels
        environment = "production",
        region = "us-east-1"
    }
})
```

### 13. AuditLogger

**Purpose**: Compliance and audit logging
**Hook Points**: All modification hooks
**Priority**: `low`

```lua
Hook.enable_builtin("AuditLogger", {
    log_level = "info",            -- Minimum log level
    include_input = true,          -- Log inputs
    include_output = false,        -- Log outputs (PII concern)
    include_metadata = true,       -- Log metadata
    redact_sensitive = true,       -- Redact sensitive data
    destinations = {               -- Where to send logs
        {type = "file", path = "/logs/audit.log"},
        {type = "syslog", host = "syslog.internal"},
        {type = "webhook", url = "https://audit.api/logs"}
    },
    retention_days = 90            -- Log retention
})
```

### 14. TracingIntegrator

**Purpose**: Distributed tracing integration
**Hook Points**: All hooks
**Priority**: `normal`

```lua
Hook.enable_builtin("TracingIntegrator", {
    provider = "opentelemetry",     -- opentelemetry, jaeger, zipkin
    endpoint = "http://otel:4317",
    service_name = "llmspell",
    sample_rate = 0.1,             -- Sample 10% of requests
    propagate_context = true,      -- Propagate trace context
    span_attributes = {            -- Additional attributes
        "deployment.environment",
        "agent.model",
        "tool.name"
    }
})
```

## Workflow Hooks

### 15. WorkflowOrchestrator

**Purpose**: Advanced workflow control
**Hook Points**: All workflow hooks
**Priority**: `normal`

```lua
Hook.enable_builtin("WorkflowOrchestrator", {
    enable_checkpoints = true,      -- Automatic checkpointing
    checkpoint_interval = 5,        -- Every 5 steps
    enable_parallelism = true,      -- Parallel step execution
    max_parallel = 4,              -- Maximum parallel steps
    enable_caching = true,         -- Cache step results
    enable_retry = true,           -- Retry failed steps
    timeout_per_step = 300,        -- 5 minute step timeout
    on_failure = "rollback"        -- rollback, continue, or abort
})
```

### 16. DependencyResolver

**Purpose**: Manages step dependencies
**Hook Points**: `BeforeWorkflowStage`
**Priority**: `high`

```lua
Hook.enable_builtin("DependencyResolver", {
    validate_dependencies = true,   -- Validate before execution
    wait_for_dependencies = true,   -- Wait for deps to complete
    timeout_waiting = 3600,        -- 1 hour max wait
    retry_failed_deps = true,      -- Retry failed dependencies
    circular_detection = true,     -- Detect circular deps
    on_missing_dep = "fail"       -- fail, skip, or mock
})
```

## Development Hooks

### 17. DebugLogger

**Purpose**: Detailed debugging output
**Hook Points**: All hooks
**Priority**: `lowest`

```lua
Hook.enable_builtin("DebugLogger", {
    log_all_hooks = true,          -- Log every hook call
    log_context = true,            -- Log full context
    log_timing = true,             -- Log execution times
    log_memory = true,             -- Log memory usage
    pretty_print = true,           -- Format output nicely
    max_depth = 5,                 -- Max object depth
    include_stack_trace = false,   -- Include call stacks
    output = "console"             -- console, file, or both
})
```

### 18. MockProvider

**Purpose**: Mock responses for testing
**Hook Points**: `BeforeAgentExecution`, `BeforeToolExecution`
**Priority**: `highest`

```lua
Hook.enable_builtin("MockProvider", {
    enabled = true,                -- Enable mocking
    mock_file = "mocks.json",      -- Mock definitions file
    fallback_to_real = true,       -- Use real if no mock
    record_mode = false,           -- Record real responses
    match_on = {                   -- Fields to match
        "component_name",
        "input.text"
    }
})
```

## Combining Built-in Hooks

### Production Setup Example

```lua
-- Production-ready hook configuration
local production_hooks = {
    -- Security first
    {name = "SecurityValidator", config = {strict_mode = true}},
    {name = "AuthorizationChecker", config = {require_auth = true}},
    
    -- Reliability
    {name = "RetryHandler", config = {max_retries = 3}},
    {name = "CircuitBreaker", config = {failure_threshold = 5}},
    
    -- Performance
    {name = "CachingOptimizer", config = {default_ttl = 300}},
    {name = "ResourceLimiter", config = {max_memory_mb = 512}},
    
    -- Cost management
    {name = "CostTracker", config = {alert_threshold = 50.0}},
    {name = "TokenOptimizer", config = {compression_level = "balanced"}},
    
    -- Monitoring
    {name = "MetricsCollector", config = {export_format = "prometheus"}},
    {name = "AuditLogger", config = {redact_sensitive = true}},
    {name = "TracingIntegrator", config = {sample_rate = 0.1}}
}

Hook.enable_builtin(production_hooks)
```

### Development Setup Example

```lua
-- Development-friendly hook configuration
local dev_hooks = {
    -- Debugging
    {name = "DebugLogger", config = {log_all_hooks = true}},
    {name = "MockProvider", config = {record_mode = true}},
    
    -- Light monitoring
    {name = "PerformanceMonitor", config = {enable_profiling = true}},
    {name = "MetricsCollector", config = {aggregation_window = 10}},
    
    -- No strict limits
    {name = "ResourceLimiter", config = {
        max_memory_mb = 2048,
        timeout_ms = 300000  -- 5 minutes
    }}
}

Hook.enable_builtin(dev_hooks)
```

## Disabling Built-in Hooks

```lua
-- Disable a single hook
Hook.disable_builtin("DebugLogger")

-- Disable multiple hooks
Hook.disable_builtin({"DebugLogger", "MockProvider"})

-- Disable all built-in hooks
Hook.disable_all_builtin()

-- Check if enabled
if Hook.is_builtin_enabled("CostTracker") then
    print("Cost tracking is active")
end
```

## Configuration Management

### Loading from File

```lua
-- hooks-config.json
{
    "production": {
        "SecurityValidator": {"strict_mode": true},
        "CostTracker": {"alert_threshold": 100.0}
    },
    "development": {
        "DebugLogger": {"log_all_hooks": true}
    }
}

-- Load configuration
local config = json.decode(File.read("hooks-config.json"))
local env = os.getenv("ENVIRONMENT") or "development"

for hook_name, hook_config in pairs(config[env]) do
    Hook.enable_builtin(hook_name, hook_config)
end
```

### Environment-based Configuration

```lua
-- Configure based on environment variables
if os.getenv("ENABLE_SECURITY") == "true" then
    Hook.enable_builtin("SecurityValidator", {
        strict_mode = os.getenv("SECURITY_MODE") == "strict"
    })
end

if os.getenv("ENABLE_MONITORING") == "true" then
    Hook.enable_builtin("MetricsCollector", {
        export_endpoint = os.getenv("METRICS_ENDPOINT")
    })
end
```

## Performance Impact

| Hook | Overhead | Memory Usage | Notes |
|------|----------|--------------|-------|
| SecurityValidator | 1-5ms | Low | Pattern matching cost |
| InputSanitizer | <1ms | Low | String operations |
| AuthorizationChecker | <1ms | Low | Lookup operations |
| PerformanceMonitor | <1ms | Medium | Metrics storage |
| CachingOptimizer | <1ms | High | Cache storage |
| ResourceLimiter | <1ms | Low | Counter tracking |
| CostTracker | <1ms | Low | Simple accumulation |
| TokenOptimizer | 5-50ms | Medium | Compression cost |
| RetryHandler | 0ms | Low | Only on errors |
| MetricsCollector | 1-2ms | Medium | Aggregation cost |

## Best Practices

1. **Start with Security**: Always enable security hooks in production
2. **Monitor Performance**: Use performance hooks to identify bottlenecks
3. **Set Appropriate Limits**: Configure resource limits based on your needs
4. **Enable Gradually**: Start with few hooks, add more as needed
5. **Test Configuration**: Test hook configurations in development first
6. **Use Environments**: Different configurations for dev/staging/prod
7. **Monitor Overhead**: Watch for cumulative hook overhead

## Next Steps

- **[Hooks Guide](./hooks-guide.md)**: Understanding hook system
- **[Hook Patterns](./hook-patterns.md)**: Common hook patterns
- **[Hook Development Guide](../developer-guide/hook-development-guide.md)**: Create custom hooks
- **[Examples](./examples/builtin-hooks-examples.md)**: Real-world usage examples

## Summary

- 18+ production-ready built-in hooks available
- Cover security, performance, reliability, and monitoring
- Minimal configuration required
- Designed for <5ms overhead per hook
- Environment-aware configuration support
- Easy to enable, disable, and configure
