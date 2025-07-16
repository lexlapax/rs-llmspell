# Resource Limit Framework

## Overview

The Resource Limit Framework provides comprehensive resource management and monitoring for LLMSpell tools. It enforces limits on memory usage, CPU time, file sizes, and operation counts to prevent resource exhaustion and ensure system stability.

## Features

### 1. Resource Limits
- **Memory Limits**: Track and limit memory allocation
- **CPU Time Limits**: Monitor and restrict CPU usage
- **File Size Limits**: Control maximum file sizes for I/O operations
- **Operation Count Limits**: Limit the number of operations performed
- **Concurrent Operation Limits**: Control parallel execution
- **Operation Timeouts**: Prevent long-running operations

### 2. Resource Tracking
- Real-time tracking of resource usage
- Automatic cleanup with RAII guards
- Thread-safe operation counting
- Memory allocation/deallocation tracking

### 3. Resource Monitoring
- Component-level monitoring
- Event-based alerting system
- Aggregated statistics
- Historical event tracking

## Usage

### Basic Resource Limiting

```rust
use llmspell_utils::{ResourceLimits, ResourceTracker};

// Create resource limits
let limits = ResourceLimits {
    max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
    max_cpu_time_ms: Some(5_000),             // 5 seconds
    max_operations: Some(10_000),             // 10K operations
    ..Default::default()
};

// Create tracker
let tracker = ResourceTracker::new(limits);

// Track operations
tracker.track_operation()?;

// Track memory
tracker.track_memory(1_000_000)?; // 1MB

// Check CPU time
tracker.check_cpu_time()?;

// Execute with timeout
let result = tracker.with_timeout(async {
    // Your async operation
    do_something().await
}).await?;
```

### Using Memory Guards

```rust
use llmspell_utils::{ResourceTracker, MemoryGuard};

let tracker = ResourceTracker::new(ResourceLimits::default());

{
    // Allocate memory with automatic cleanup
    let _guard = MemoryGuard::new(&tracker, 1_000_000)?;
    
    // Memory is tracked while guard is in scope
    process_data();
    
} // Memory automatically released when guard drops
```

### Tool Integration

```rust
use llmspell_tools::{ResourceLimited, ResourceLimitExt};

// Method 1: Implement ResourceLimited trait
impl ResourceLimited for MyTool {
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits {
            max_memory_bytes: Some(50 * 1024 * 1024), // 50MB
            max_cpu_time_ms: Some(10_000),            // 10 seconds
            ..Default::default()
        }
    }
}

// Method 2: Wrap existing tool
let tool = MyTool::new();
let limited_tool = tool.with_resource_limits(ResourceLimits::strict());

// Method 3: Use convenience methods
let limited_tool = tool.with_strict_limits();
```

### Resource Monitoring

```rust
use llmspell_utils::{ResourceMonitor, MonitoringConfig};
use std::sync::Arc;

// Create monitor
let config = MonitoringConfig {
    warning_threshold: 0.8, // Warn at 80% usage
    check_interval: Duration::from_secs(1),
    ..Default::default()
};
let monitor = ResourceMonitor::new(config);

// Register components
let tracker = Arc::new(ResourceTracker::new(limits));
monitor.register_component("my_tool".to_string(), tracker);

// Start monitoring
monitor.start_monitoring().await;

// Get statistics
let stats = monitor.get_statistics();
println!("Total memory: {} bytes", stats.total_memory_bytes);
println!("Total operations: {}", stats.total_operations);
```

## Predefined Configurations

### Default Limits
```rust
let limits = ResourceLimits::default();
// Memory: 100MB, CPU: 30s, File: 50MB, Ops: 1M
```

### Strict Limits (Untrusted Operations)
```rust
let limits = ResourceLimits::strict();
// Memory: 10MB, CPU: 5s, File: 5MB, Ops: 10K
```

### Relaxed Limits (Trusted Operations)
```rust
let limits = ResourceLimits::relaxed();
// Memory: 1GB, CPU: 5min, File: 500MB, Ops: 100M
```

### Unlimited (Use with Caution)
```rust
let limits = ResourceLimits::unlimited();
// No limits enforced
```

## Error Handling

Resource limit violations result in `LLMSpellError::ResourceLimit`:

```rust
match tracker.track_memory(huge_amount) {
    Ok(()) => process_data(),
    Err(LLMSpellError::ResourceLimit { resource, limit, used }) => {
        eprintln!("Resource {} limit {} exceeded (used: {})", 
                  resource, limit, used);
    }
    Err(e) => return Err(e),
}
```

## Best Practices

1. **Choose Appropriate Limits**: Use strict limits for untrusted input, relaxed for trusted operations
2. **Monitor Critical Paths**: Add monitoring to resource-intensive operations
3. **Use Guards**: Prefer RAII guards (MemoryGuard, ConcurrentGuard) for automatic cleanup
4. **Handle Failures Gracefully**: Provide meaningful error messages when limits are exceeded
5. **Test Limits**: Verify your tools behave correctly when limits are reached

## Performance Considerations

- Resource tracking adds minimal overhead (<1% in most cases)
- Use `track_operation!` macro in tight loops for combined tracking
- Consider batching operations to reduce tracking overhead
- Memory guards have zero-cost abstractions when limits aren't exceeded

## Security Benefits

1. **DoS Prevention**: Limits prevent resource exhaustion attacks
2. **Memory Safety**: Prevents out-of-memory conditions
3. **CPU Protection**: Stops infinite loops and expensive computations
4. **File Size Control**: Prevents disk space exhaustion
5. **Concurrency Control**: Limits parallel execution to prevent thread exhaustion