# Debug Infrastructure Architecture

Internal architecture and design decisions for LLMSpell's debug infrastructure.

## Overview

The debug infrastructure is designed with the following principles:

1. **Zero-cost abstraction** when debugging is disabled
2. **Language agnostic** core with script-specific bindings
3. **Thread-safe** for concurrent script execution
4. **Extensible** for future language support
5. **Performance-first** with minimal overhead

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                 Script Languages                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │     Lua     │  │ JavaScript  │  │   Python    │     │
│  │   Globals   │  │   Globals   │  │   Globals   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                 Bridge Layer                            │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              DebugBridge                            │ │
│  │  (Script-safe interface with interior mutability)  │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                 Core Rust Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │DebugManager │  │ Profiler    │  │ModuleFilter │     │
│  │(Global State)│  │(Performance)│  │(Filtering)  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Global Debug Manager (`llmspell-utils/src/debug/mod.rs`)

The central coordinator for all debug operations.

```rust
pub struct DebugManager {
    level: AtomicU8,                                    // Current debug level
    enabled: AtomicBool,                                // Master enable/disable
    output_handler: Arc<RwLock<Box<dyn DebugOutput>>>, // Pluggable output
    profiler: Arc<Profiler>,                           // Performance tracking
    performance_trackers: DashMap<String, Arc<PerformanceTracker>>, // Active timers
    module_filters: Arc<RwLock<EnhancedModuleFilter>>, // Module filtering
    capture_buffer: Arc<BufferOutput>,                 // Captured entries
}
```

**Key Design Decisions:**

- **Static singleton** via `LazyLock` for global access
- **Atomic operations** for level/enabled for performance
- **Interior mutability** using `RwLock` and `DashMap` for thread safety
- **Trait-based output** for extensibility

### 2. Debug Bridge (`llmspell-bridge/src/debug_bridge.rs`)

Script-safe interface layer that wraps the core functionality.

```rust
#[derive(Clone)]
pub struct DebugBridge {
    manager: Arc<DebugManager>,                          // Reference to global manager
    trackers: Arc<Mutex<HashMap<String, Arc<PerformanceTracker>>>>, // Script-local trackers
}
```

**Key Design Decisions:**

- **Cloneable** for easy sharing between script contexts
- **UUID-based timer IDs** for global uniqueness
- **Interior mutability** for script-safe mutation
- **Error conversion** from Rust errors to script-safe types

### 3. Performance Profiler (`llmspell-utils/src/debug/performance.rs`)

Comprehensive performance tracking with statistical analysis.

```rust
pub struct Profiler {
    trackers: Arc<RwLock<ProfilerState>>,
}

struct ProfilerState {
    trackers: Vec<Arc<PerformanceTracker>>,
    start_time: Instant,
}

pub struct PerformanceTracker {
    name: String,
    start_time: Instant,
    laps: Arc<RwLock<Vec<(String, Duration)>>>,
    children: Arc<RwLock<Vec<Arc<PerformanceTracker>>>>,
    // Memory tracking placeholders
    memory_start: Option<u64>,
    memory_end: Option<u64>,
}
```

**Statistical Analysis:**
- Median, 95th percentile, 99th percentile
- Mean and standard deviation
- Memory delta tracking
- Custom event recording

### 4. Module Filtering (`llmspell-utils/src/debug/module_filter.rs`)

Advanced pattern matching for targeted debugging.

```rust
pub struct EnhancedModuleFilter {
    exact_matches: HashMap<String, bool>,               // Fast exact lookups
    pattern_cache: HashMap<String, (Regex, bool)>,     // Compiled regex patterns
    hierarchical_rules: Vec<(String, bool)>,           // Hierarchical matching
    default_enabled: bool,                              // Fallback behavior
}
```

**Pattern Types:**
1. **Exact**: Direct string matching (fastest)
2. **Hierarchical**: Module tree matching (`parent.child.*`)
3. **Wildcard**: Glob patterns (`*.test`, `work?low`)
4. **Regex**: Full regular expression support

**Matching Priority:**
1. Exact matches (highest priority)
2. Hierarchical rules (by specificity - longer first)
3. Compiled regex patterns
4. Default behavior (lowest priority)

### 5. Stack Trace Collection (`llmspell-bridge/src/lua/stacktrace.rs`)

Lua-specific stack trace capture using debug library.

```rust
pub struct StackTrace {
    frames: Vec<StackFrame>,
    max_depth: usize,
    truncated: bool,
    error: Option<String>,
}

pub struct StackFrame {
    name: Option<String>,
    source: Option<String>,
    line: Option<i32>,
    line_defined: Option<i32>,
    what: String,
    num_upvalues: u8,
    num_params: u8,
    locals: HashMap<String, String>,
    upvalues: HashMap<String, String>,
}
```

**Implementation Notes:**
- Uses Lua's `debug.getinfo()` for frame information
- `debug.getlocal()` for local variable capture
- `debug.getupvalue()` for closure variable capture
- Graceful fallback when debug library unavailable

### 6. Object Dumping (`llmspell-bridge/src/lua/object_dump.rs`)

Advanced Lua value introspection and formatting.

```rust
pub struct DumpOptions {
    pub max_depth: usize,
    pub indent_size: usize,
    pub max_string_length: usize,
    pub max_array_elements: usize,
    pub max_table_pairs: usize,
    pub show_types: bool,
    pub show_addresses: bool,
    pub compact_mode: bool,
}
```

**Features:**
- Circular reference detection
- Array vs hash table detection
- Type annotation
- Memory address display
- Configurable depth limiting
- Multiple output formats (default, compact, verbose)

## Thread Safety Design

### Atomic Operations

```rust
// Fast path for level checking (no locks)
level.load(Ordering::Relaxed)

// Master enable/disable (no locks)
enabled.load(Ordering::Relaxed)
```

### Reader-Writer Locks

```rust
// Rare writes (configuration), frequent reads (filtering)
module_filters: Arc<RwLock<EnhancedModuleFilter>>

// Rare writes (setup), frequent reads (output)
output_handler: Arc<RwLock<Box<dyn DebugOutput>>>
```

### Lock-Free Collections

```rust
// Concurrent access to performance trackers
performance_trackers: DashMap<String, Arc<PerformanceTracker>>
```

### Interior Mutability Pattern

```rust
// Allow mutation through shared references for script safety
trackers: Arc<Mutex<HashMap<String, Arc<PerformanceTracker>>>>
```

## Performance Optimizations

### 1. Early Bailout

```rust
pub fn log(&self, level: DebugLevel, message: impl Into<String>, module: Option<String>) {
    // Quick exit if disabled or level filtered
    if !self.is_enabled() || !self.should_log(level, module.as_deref()) {
        return;
    }
    // ... expensive work only if needed
}
```

### 2. Lazy String Conversion

```rust
// Accept `impl Into<String>` to avoid allocation until needed
pub fn log(&self, level: DebugLevel, message: impl Into<String>, module: Option<String>)
```

### 3. Compiled Regex Caching

```rust
// Compile patterns once, reuse many times
pattern_cache: HashMap<String, (Regex, bool)>
```

### 4. Statistical Sample Optimization

```rust
// Calculate percentiles efficiently with sorted sampling
let mut sorted_times: Vec<Duration> = lap_times.clone();
sorted_times.sort_unstable();
```

## Error Handling Strategy

### Non-Failing Design

The debug infrastructure is designed to never fail user code:

```rust
// All public APIs return safe defaults on error
pub fn start_timer(&self, name: &str) -> String {
    let tracker = self.manager.start_timer(name);
    let id = format!("timer_{}", uuid::Uuid::new_v4());
    self.trackers.lock().insert(id.clone(), tracker);
    id // Always returns valid ID
}

pub fn stop_timer(&self, id: &str) -> Option<f64> {
    self.trackers
        .lock()
        .remove(id)
        .map(|tracker| tracker.stop().as_secs_f64() * 1000.0)
    // Returns None instead of panicking
}
```

### Error Isolation

```rust
// Errors in debug system don't propagate to user code
match self.write_debug_entry(&entry) {
    Ok(()) => {},
    Err(e) => {
        // Log internally, don't fail user operation
        eprintln!("Debug system error: {}", e);
    }
}
```

## Memory Management

### Circular Buffer for Captured Entries

```rust
pub struct BufferOutput {
    entries: Arc<Mutex<VecDeque<DebugEntry>>>,
    max_entries: usize,
}

impl BufferOutput {
    pub fn write(&self, entry: &DebugEntry) {
        let mut entries = self.entries.lock();
        if entries.len() >= self.max_entries {
            entries.pop_front(); // Remove oldest
        }
        entries.push_back(entry.clone());
    }
}
```

### Automatic Cleanup

```rust
// Timers automatically clean up when dropped
impl Drop for PerformanceTracker {
    fn drop(&mut self) {
        // Remove from global tracking if needed
    }
}
```

### Memory Tracking Integration

```rust
// Placeholder for future memory profiling
pub struct MemoryStats {
    pub used_bytes: u64,
    pub allocated_bytes: u64,
    pub resident_bytes: u64,
    pub collections: u32,
}
```

## Extensibility Points

### 1. Output Handlers

```rust
pub trait DebugOutput: Send + Sync {
    fn write(&self, entry: &DebugEntry);
    fn flush(&self);
}

// Easy to add new outputs
pub struct SyslogOutput { ... }
pub struct FileOutput { ... }
pub struct NetworkOutput { ... }
```

### 2. Language Bindings

```rust
// Pattern for adding new script languages
pub trait ScriptDebugGlobal {
    fn inject_globals(&self, context: &mut ScriptContext) -> Result<()>;
}

// Implementations for each language
impl ScriptDebugGlobal for LuaDebugGlobal { ... }
impl ScriptDebugGlobal for JavaScriptDebugGlobal { ... }
impl ScriptDebugGlobal for PythonDebugGlobal { ... }
```

### 3. Custom Event Types

```rust
// Extensible event system
pub struct TimingEvent {
    pub timestamp: Duration,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
}
```

## Configuration Integration

### Environment Variables

```rust
// Automatic environment integration
impl DebugManager {
    pub fn from_env() -> Self {
        let level = env::var("LLMSPELL_DEBUG_LEVEL")
            .unwrap_or_else(|_| "info".to_string())
            .parse()
            .unwrap_or(DebugLevel::Info);
            
        let enabled = env::var("LLMSPELL_DEBUG_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
            
        // ... setup with environment values
    }
}
```

### Configuration File Integration

```toml
[debug]
level = "debug"
enabled = true
capture_limit = 50000

[[debug.filters]]
pattern = "workflow.*"
enabled = true
type = "hierarchical"
```

## Testing Infrastructure

### Unit Tests

- Each component has comprehensive unit tests
- Mock implementations for external dependencies
- Property-based testing for filter patterns

### Integration Tests

- End-to-end script execution tests
- Multi-threaded safety tests
- Performance regression tests

### Benchmarks

```rust
#[bench]
fn bench_debug_logging_disabled(b: &mut Bencher) {
    let manager = DebugManager::new();
    manager.set_enabled(false);
    
    b.iter(|| {
        manager.log(DebugLevel::Info, "test message", None);
    });
}
```

## Future Extensions

### 1. Distributed Tracing

```rust
// Integration with OpenTelemetry
pub struct TracingContext {
    span_id: SpanId,
    trace_id: TraceId,
    parent_span: Option<SpanId>,
}
```

### 2. Real-time Monitoring

```rust
// WebSocket-based real-time debug streaming
pub struct RealtimeOutput {
    websocket: Arc<WebSocket>,
    filters: Vec<String>,
}
```

### 3. Debug Session Recording

```rust
// Record entire debug sessions for replay
pub struct SessionRecorder {
    events: Vec<DebugEvent>,
    start_time: Instant,
}
```

## Implementation Notes

### Why Interior Mutability?

Script languages require `&self` methods for binding safety, but we need mutation for state tracking. Interior mutability (`Arc<Mutex<T>>`, `Arc<RwLock<T>>`) allows safe mutation through shared references.

### Why Global State?

A global debug manager ensures:
- Consistent configuration across all scripts
- Shared performance tracking
- Centralized output handling
- Memory efficiency (single buffer, not per-script)

### Why DashMap for Trackers?

`DashMap` provides:
- Lock-free reads for hot paths
- Concurrent writes for timer operations
- Better performance than `RwLock<HashMap>` for mixed workloads

### Why Trait Objects for Output?

Trait objects (`Box<dyn DebugOutput>`) allow:
- Runtime configuration of output handlers
- Easy testing with mock outputs
- Plugin-style extensibility
- Zero-cost when not changing outputs

This architecture balances performance, safety, and extensibility while maintaining the zero-cost abstraction principle when debugging is disabled.