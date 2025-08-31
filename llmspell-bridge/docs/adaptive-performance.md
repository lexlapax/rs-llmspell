# Adaptive Performance Configuration

## Overview

The llmspell-bridge crate implements adaptive performance configuration throughout its components, ensuring that performance characteristics automatically adjust based on workload, environment, and system resources.

## Core Concepts

### Workload Classification

Operations are classified into four categories based on expected duration:

```rust
pub enum WorkloadClassifier {
    Micro,   // <1ms expected
    Light,   // <10ms expected  
    Medium,  // <100ms expected
    Heavy,   // <1s expected
}
```

### Environment Presets

Components support different environment configurations:

```rust
pub enum Environment {
    Production,  // Minimal overhead, aggressive optimization
    Development, // Balanced performance and debugging
    Testing,     // Comprehensive data collection
    Benchmark,   // Performance measurement focus
}
```

## Adaptive Components

### 1. HookProfiler

The `HookProfiler` adapts its sampling rate based on observed overhead:

```rust
pub struct HookProfilingConfig {
    pub max_overhead_percent: f64,      // Target maximum overhead
    pub sampling_rate: f64,              // Initial sampling rate
    pub adaptive_sampling: bool,         // Enable adaptive adjustment
    pub workload_thresholds: WorkloadThresholds,
}
```

**Adaptive Behavior:**
- Monitors actual overhead vs target
- Reduces sampling rate when overhead exceeds threshold
- Increases sampling for light workloads
- Maintains statistical significance

**Example Usage:**
```rust
let config = HookProfilingConfig {
    max_overhead_percent: 1.0,  // Target <1% overhead
    sampling_rate: 0.1,          // Start with 10% sampling
    adaptive_sampling: true,
    workload_thresholds: WorkloadThresholds::default(),
};
```

### 2. CircuitBreaker

The `CircuitBreaker` uses workload-aware error thresholds:

```rust
impl CircuitBreaker {
    fn get_threshold(&self, workload: &WorkloadClassifier) -> f64 {
        match workload {
            WorkloadClassifier::Micro => 0.1,   // 10% for micro ops
            WorkloadClassifier::Light => 0.2,   // 20% for light ops
            WorkloadClassifier::Medium => 0.4,  // 40% for medium ops
            WorkloadClassifier::Heavy => 0.6,   // 60% for heavy ops
        }
    }
}
```

**Adaptive Behavior:**
- Higher tolerance for heavy operations
- Quick circuit opening for micro operations
- Exponential backoff for recovery attempts
- Workload-specific half-open testing

### 3. SessionRecorder

The `SessionRecorder` adapts compression and sampling based on session size:

```rust
pub struct SessionRecorderConfig {
    pub max_memory_mb: usize,
    pub compression_threshold_mb: usize,
    pub sampling_threshold_events_per_sec: f64,
    pub adaptive_sampling: bool,
    pub storage_vs_cpu_preference: TradeoffPreference,
}
```

**Adaptive Behavior:**
- Enables compression when memory threshold exceeded
- Reduces sampling rate for high-frequency events
- Adjusts based on storage vs CPU preference
- Environment-specific defaults

### 4. ProfilingConfig

Profiling configuration adapts based on operational context:

```rust
impl ProfilingConfig {
    pub fn production() -> Self {
        Self {
            enabled: false,           // Disabled by default
            sample_rate: 100,         // Low frequency when enabled
            max_overhead_percent: 0.5, // Very low overhead target
            adaptive_thresholds: true,
        }
    }

    pub fn development() -> Self {
        Self {
            enabled: true,
            sample_rate: 1000,        // Higher sampling
            max_overhead_percent: 5.0, // Accept more overhead
            adaptive_thresholds: true,
        }
    }
}
```

## Benchmark Philosophy

Performance tests follow these principles:

1. **Measure, Don't Assert**: Tests report metrics rather than failing on thresholds
2. **Adaptive Baselines**: Performance expectations adjust based on workload classification
3. **Environment Awareness**: Different thresholds for different environments

Example test pattern:
```rust
#[test]
fn test_performance() {
    let config = ProfilingConfig::benchmark();
    let start = Instant::now();
    
    // Perform operation
    let result = perform_operation();
    
    let duration = start.elapsed();
    let workload = classify_workload(&operation);
    
    // Report metrics (don't assert fixed thresholds)
    println!("Operation completed in {:?} (workload: {:?})", 
             duration, workload);
    
    // Only fail if adaptation itself is broken
    assert!(config.should_adapt(duration, workload));
}
```

## Configuration Guidelines

### Production Environment
- Minimize overhead (<1% target)
- Disable non-essential features
- Use aggressive sampling
- Prefer CPU over storage

### Development Environment
- Balance debugging and performance
- Enable most features
- Moderate sampling rates
- Balanced resource usage

### Testing Environment
- Comprehensive data collection
- High sampling rates
- Storage over CPU preference
- All features enabled

### Benchmark Environment
- Consistent measurement conditions
- Disable adaptive features during measurement
- Report raw metrics
- No artificial limits

## Benefits

1. **Automatic Optimization**: Components self-tune based on actual workload
2. **Resource Awareness**: Adapts to available system resources
3. **Environment Appropriate**: Different behavior for prod vs dev
4. **No Magic Numbers**: All thresholds are configurable or adaptive
5. **Graceful Degradation**: Performance degrades smoothly under load

## Implementation Checklist

When implementing adaptive performance:

- [ ] Define workload classifications for your operations
- [ ] Implement environment-specific configurations
- [ ] Add adaptive sampling/throttling mechanisms
- [ ] Use percentage-based rather than absolute thresholds
- [ ] Provide configuration overrides
- [ ] Test adaptation behavior under various loads
- [ ] Document performance characteristics
- [ ] Avoid hardcoded performance thresholds