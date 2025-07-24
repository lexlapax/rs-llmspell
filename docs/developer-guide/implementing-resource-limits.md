# Implementing Resource Limits in Tools

⚠️ **EVOLVING CODEBASE**: This guide contains API examples that may not match the current implementation. Always verify against the actual codebase in `llmspell-utils/src/resource_limits.rs` before implementing.

**Phase 3.3 Status**: Resource limits are implemented but with simplified APIs compared to examples shown here.

This guide explains how to implement resource limits in new tools for rs-llmspell.

## Implementation Status

### ✅ **Currently Available (Phase 3.3)**
- **ResourceLimits**: Memory, CPU, file size, operation count, timeout limits
- **ResourceTracker**: Operation tracking with automatic cleanup
- **MemoryGuard**: RAII memory tracking with automatic release
- **Tool Integration**: ResourceLimited trait and ResourceLimitExt extension
- **Helper Functions**: File operation validation and data processing tracking

### 📋 **Planned Enhancements (Phase 4+)**
- **Advanced Monitoring**: Real-time resource usage dashboards
- **Dynamic Limits**: Runtime limit adjustment based on system load
- **Resource Pooling**: Shared resource pools across tool instances
- **Advanced Metrics**: Detailed performance and usage analytics

## Basic Resource Limiting

```rust
use llmspell_utils::resource_limits::{ResourceLimits, ResourceTracker};

// ✅ CORRECT: Create resource limits (this API is accurate)
let limits = ResourceLimits {
    max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
    max_cpu_time_ms: Some(5_000),             // 5 seconds
    max_operations: Some(10_000),             // 10K operations
    max_file_size_bytes: Some(50 * 1024 * 1024), // 50MB max file
    max_concurrent_ops: Some(100),            // 100 concurrent operations
    operation_timeout_ms: Some(30_000),       // 30 second timeout
};

// ✅ CORRECT: Create tracker
let tracker = ResourceTracker::new(limits);

// ✅ CORRECT: Track operations and CPU time together
track_operation!(tracker)?; // This macro calls both track_operation() and check_cpu_time()

// ✅ CORRECT: Track memory with auto-cleanup
let _memory_guard = MemoryGuard::new(&tracker, 1_000_000)?; // 1MB
// Memory is automatically released when guard drops

// ✅ CORRECT: Execute with timeout
let result = tracker.with_timeout(async {
    // Your async operation
    some_async_operation().await
}).await?;
```

## Using Memory Guards

```rust
use llmspell_utils::resource_limits::{ResourceLimits, ResourceTracker, MemoryGuard};

let tracker = ResourceTracker::new(ResourceLimits::default());

{
    // ✅ CORRECT: Allocate memory with automatic cleanup
    let _guard = MemoryGuard::new(&tracker, 1_000_000)?;
    
    // Memory is tracked while guard is in scope
    process_data();
    
} // Memory automatically released when guard drops
```

## Tool Integration

```rust
use llmspell_tools::resource_limited::{ResourceLimited, ResourceLimitExt};
use llmspell_utils::resource_limits::ResourceLimits;

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

## Resource Monitoring

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

## Implementation Patterns

### Pattern 1: File Processing with Limits

```rust
impl MyFileTool {
    async fn process_file(&self, path: &Path) -> Result<String> {
        // Check file size before reading
        let metadata = fs::metadata(path).await?;
        self.tracker.track_memory(metadata.len() as usize)?;
        
        // Use memory guard for file content
        let _guard = MemoryGuard::new(&self.tracker, metadata.len() as usize)?;
        
        // Read with operation tracking
        self.tracker.track_operation()?;
        let content = fs::read_to_string(path).await?;
        
        // Process with CPU time checks
        self.tracker.check_cpu_time()?;
        let result = self.process_content(&content)?;
        
        Ok(result)
    }
}
```

### Pattern 2: Network Operations

```rust
impl MyNetworkTool {
    async fn fetch_data(&self, url: &str) -> Result<String> {
        // Use timeout from resource limits
        let timeout = self.tracker.limits().operation_timeout
            .unwrap_or(Duration::from_secs(30));
        
        // Execute with resource tracking
        self.tracker.with_timeout(async {
            self.tracker.track_operation()?;
            
            let response = reqwest::get(url)
                .timeout(timeout)
                .send()
                .await?;
            
            // Track response size
            if let Some(size) = response.content_length() {
                self.tracker.track_memory(size as usize)?;
            }
            
            Ok(response.text().await?)
        }).await
    }
}
```

### Pattern 3: Concurrent Operations

```rust
impl MyConcurrentTool {
    async fn parallel_process(&self, items: Vec<Item>) -> Result<Vec<Output>> {
        use futures::stream::{self, StreamExt};
        
        // Get concurrency limit
        let limit = self.tracker.limits().max_concurrent_operations
            .unwrap_or(10);
        
        // Process with controlled concurrency
        let results = stream::iter(items)
            .map(|item| async {
                // Use concurrent guard
                let _guard = ConcurrentGuard::new(&self.tracker)?;
                self.process_item(item).await
            })
            .buffer_unordered(limit)
            .collect::<Vec<_>>()
            .await;
        
        Ok(results)
    }
}
```

## Performance Considerations

- Resource tracking adds minimal overhead (<1% in most cases)
- Use `track_operation!` macro in tight loops for combined tracking
- Consider batching operations to reduce tracking overhead
- Memory guards have zero-cost abstractions when limits aren't exceeded

## Testing Resource Limits

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_limit() {
        let tool = MyTool::new().with_resource_limits(ResourceLimits {
            max_memory_bytes: Some(1024), // 1KB limit
            ..Default::default()
        });
        
        // Should fail with large input
        let result = tool.process(vec![0u8; 2048]).await;
        assert!(matches!(result, Err(LLMSpellError::ResourceLimit { .. })));
    }
    
    #[tokio::test]
    async fn test_cpu_time_limit() {
        let tool = MyTool::new().with_resource_limits(ResourceLimits {
            max_cpu_time_ms: Some(100), // 100ms limit
            ..Default::default()
        });
        
        // Should timeout on expensive operation
        let result = tool.expensive_operation().await;
        assert!(matches!(result, Err(LLMSpellError::ResourceLimit { .. })));
    }
}
```