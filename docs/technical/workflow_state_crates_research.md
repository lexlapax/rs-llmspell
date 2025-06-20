# Workflow and State Management Crates Research

## Overview
This document researches Rust crates for workflow orchestration, state management, and event systems that could be integrated into rs-llmspell's architecture.

## 1. Workflow Engine Crates

### 1.1 temporal-sdk-rust
**Repository**: https://github.com/temporalio/sdk-rust
**Status**: Production-ready (v0.1.x)
**Architecture**: Client-server, distributed workflow orchestration

#### Features
- **Durable Execution**: Workflows survive failures and restarts
- **Workflow as Code**: Define workflows in pure Rust
- **Activity Support**: Long-running tasks with retries
- **Signals and Queries**: External workflow interaction
- **Versioning**: Safe workflow evolution
- **Testing Framework**: Built-in test utilities

#### Architecture Analysis
```rust
#[workflow]
impl MyWorkflow {
    #[workflow::run]
    async fn run(&self, input: WorkflowInput) -> Result<WorkflowOutput> {
        // Workflows are deterministic
        let activity_result = workflow::execute_activity(
            MyActivity::process,
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(30)),
                ..Default::default()
            },
            input.clone(),
        ).await?;
        
        // Can wait for signals
        let signal = workflow::await_signal::<ApprovalSignal>().await;
        
        Ok(WorkflowOutput { result: activity_result })
    }
}
```

#### Pros
- **Reliability**: Battle-tested distributed workflow engine
- **Scalability**: Handles millions of workflows
- **Visibility**: Built-in UI for workflow monitoring
- **Language Support**: Multiple SDK languages

#### Cons
- **Complexity**: Requires Temporal server infrastructure
- **Overhead**: Heavy for simple workflows
- **Learning Curve**: Complex concepts (determinism, versioning)
- **External Dependency**: Not embeddable

#### Integration Feasibility
- **LIMITATION**: Too heavyweight for embedded scripting use case
- **LIMITATION**: Requires external server, not suitable for rs-llmspell
- **INSIGHT**: Good patterns for workflow design but not directly usable

### 1.2 flowrs
**Repository**: https://github.com/flowrs/flowrs
**Status**: Early development (0.1.x)
**Architecture**: Lightweight, embeddable workflow engine

#### Features
- **Simple API**: Builder pattern for workflow definition
- **Async Native**: Built on tokio
- **JSON Definition**: Workflows definable as JSON
- **Conditional Logic**: If/else branching
- **Parallel Execution**: Fork/join patterns

#### Architecture Analysis
```rust
let workflow = Workflow::builder()
    .step("fetch_data", |ctx| async move {
        let data = fetch_from_api().await?;
        ctx.set("data", data);
        Ok(())
    })
    .parallel(vec![
        Step::new("process_a", process_a),
        Step::new("process_b", process_b),
    ])
    .step("combine", combine_results)
    .build();

let result = workflow.execute(Context::new()).await?;
```

#### Pros
- **Lightweight**: Minimal dependencies
- **Embeddable**: Designed for embedding
- **Simple**: Easy to understand and use
- **Flexible**: JSON or code-based definitions

#### Cons
- **Immature**: Early development stage
- **Limited Features**: No persistence, signals, or queries
- **Documentation**: Sparse documentation
- **Community**: Small user base

#### Integration Feasibility
- **POTENTIAL**: Could be wrapped for rs-llmspell workflows
- **CONCERN**: May need significant enhancement for production use
- **BENEFIT**: Lightweight enough to embed in scripting context

### 1.3 State Machine Crates Comparison

#### sm (State Machine)
**Repository**: https://github.com/rusty-rockets/sm
**Features**:
- Compile-time state machine verification
- Type-safe transitions
- Zero runtime overhead
- Macro-based DSL

```rust
sm! {
    CircuitBreaker {
        InitialStates { Closed }

        Closed(Unsuccessful) => Open [SetOpenedAt],
        Open(TimerExpired) => HalfOpen,
        HalfOpen(Successful) => Closed,
        HalfOpen(Unsuccessful) => Open [SetOpenedAt],
    }
}
```

#### statig
**Repository**: https://github.com/mdeloof/statig
**Features**:
- Hierarchical state machines
- Event-driven transitions
- Async support
- No_std compatible

```rust
#[state_machine]
impl Led {
    #[state]
    async fn on(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::off()),
            Event::ButtonPressed => Super,
        }
    }
}
```

#### finny
**Repository**: https://github.com/finny-rs/finny
**Features**:
- Actor-based state machines
- Queue management
- Timer support
- Inspection and debugging

```rust
let fsm = FsmBuilder::new()
    .initial_state::<Closed>()
    .state::<Open>()
    .state::<HalfOpen>()
    .transition::<Closed, Open, Failed>()
    .transition::<Open, HalfOpen, TimerExpired>()
    .build();
```

#### Comparison Summary
| Feature | sm | statig | finny |
|---------|-----|---------|--------|
| Compile-time Safety | ✅ | ✅ | ✅ |
| Async Support | ❌ | ✅ | ✅ |
| Hierarchical | ❌ | ✅ | ❌ |
| Event Queue | ❌ | ❌ | ✅ |
| No_std | ✅ | ✅ | ❌ |
| Inspection | ❌ | ✅ | ✅ |

#### Integration Feasibility
- **USE CASE**: State machines for agent lifecycle management
- **RECOMMENDATION**: statig for async hierarchical states
- **ALTERNATIVE**: finny for actor-based agent coordination

## 2. State Management Solutions

### 2.1 sled
**Repository**: https://github.com/spacejam/sled
**Type**: Embedded database
**Status**: Beta (0.34.x)

#### Features
- **Performance**: Lock-free, log-structured design
- **ACID Transactions**: Full ACID compliance
- **API**: Simple key-value interface
- **Async Support**: Works with tokio
- **Compression**: Built-in compression support

#### Architecture Analysis
```rust
let db = sled::open("my_db")?;
let tree = db.open_tree("agent_state")?;

// Atomic operations
tree.insert("agent_1", serde_json::to_vec(&agent_state)?)?;

// Transactions
tree.transaction(|tree| {
    let current = tree.get("counter")?.unwrap_or_default();
    let new_val = increment(&current);
    tree.insert("counter", new_val)?;
    Ok(())
})?;

// Subscribe to changes
let subscriber = tree.watch_prefix("");
while let Some(event) = subscriber.next() {
    match event {
        sled::Event::Insert { key, value } => handle_insert(key, value),
        sled::Event::Remove { key } => handle_remove(key),
    }
}
```

#### Pros
- **Performance**: Very fast, especially for reads
- **Embedded**: No external dependencies
- **Features**: Transactions, subscriptions, merge operators
- **Rust Native**: Designed for Rust from ground up

#### Cons
- **Beta Status**: Not yet 1.0
- **Memory Usage**: Can be memory intensive
- **Crash Recovery**: Recovery can be slow for large datasets
- **API Stability**: May change before 1.0

### 2.2 rocksdb
**Repository**: https://github.com/rust-rocksdb/rust-rocksdb
**Type**: Embedded database (binding to RocksDB)
**Status**: Stable (0.21.x)

#### Features
- **Production Proven**: Used by many large-scale systems
- **Performance**: Optimized for SSDs
- **Column Families**: Multiple logical databases
- **Compaction**: Background optimization
- **Snapshots**: Consistent read views

#### Architecture Analysis
```rust
let mut opts = Options::default();
opts.create_if_missing(true);
opts.set_max_open_files(10000);

let db = DB::open(&opts, "agent_state")?;

// Column families for different state types
let agent_cf = db.cf_handle("agents").unwrap();
let tool_cf = db.cf_handle("tools").unwrap();

// Write batch for atomic updates
let mut batch = WriteBatch::default();
batch.put_cf(&agent_cf, b"agent_1", b"state_data");
batch.put_cf(&tool_cf, b"tool_1", b"tool_data");
db.write(batch)?;

// Iterators
let iter = db.iterator_cf(&agent_cf, IteratorMode::Start);
for (key, value) in iter {
    process_agent_state(&key, &value);
}
```

#### Pros
- **Battle Tested**: Proven in production
- **Performance**: Excellent for large datasets
- **Features**: Rich feature set
- **Stability**: Stable API

#### Cons
- **C++ Dependency**: Requires RocksDB library
- **Complexity**: Many tuning parameters
- **Binary Size**: Large binary size
- **Compilation**: Slow compilation times

### 2.3 async-std storage patterns
**Note**: async-std doesn't provide storage directly but patterns for async storage

#### Common Patterns
```rust
// Actor-based state management
struct StateActor {
    state: HashMap<String, Value>,
    subscribers: Vec<Sender<StateChange>>,
}

impl StateActor {
    async fn handle(&mut self, msg: Message) -> Result<Response> {
        match msg {
            Message::Get(key) => Ok(Response::Value(self.state.get(&key).cloned())),
            Message::Set(key, value) => {
                self.state.insert(key.clone(), value.clone());
                self.notify_subscribers(StateChange::Updated(key, value)).await;
                Ok(Response::Ok)
            }
        }
    }
}

// Channel-based coordination
let (tx, rx) = channel::<StateCommand>(100);
async_std::task::spawn(async move {
    let mut state = State::new();
    while let Some(cmd) = rx.recv().await {
        state.process(cmd).await;
    }
});
```

### 2.4 Other Notable State Management Options

#### redb
- **Type**: Pure Rust embedded database
- **Features**: ACID, zero-copy reads, multiversion concurrency
- **Status**: Stable (1.x)
- **Pros**: No unsafe code, good performance
- **Cons**: Newer, smaller community

#### heed
- **Type**: LMDB wrapper for Rust
- **Features**: Memory-mapped, zero-copy, ACID
- **Status**: Stable
- **Pros**: Excellent read performance
- **Cons**: Complex write transactions

#### fjall
- **Type**: LSM-based storage engine
- **Features**: Similar to RocksDB but pure Rust
- **Status**: Early development
- **Pros**: No C dependencies
- **Cons**: Not production ready

## 3. Event System Crates

### 3.1 tokio-stream
**Part of**: Tokio ecosystem
**Type**: Async stream utilities

#### Features
- **Stream Trait**: Async iterator pattern
- **Combinators**: Map, filter, merge, etc.
- **Broadcast**: Multi-consumer streams
- **Utilities**: Timeout, throttle, buffer

#### Architecture Analysis
```rust
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tokio::sync::broadcast;

// Event bus using broadcast channel
let (tx, _rx) = broadcast::channel::<Event>(100);

// Publisher
async fn publish_events(tx: broadcast::Sender<Event>) {
    tx.send(Event::AgentStarted { id: "agent_1" }).unwrap();
}

// Subscriber with stream processing
async fn process_events(rx: broadcast::Receiver<Event>) {
    let stream = BroadcastStream::new(rx);
    
    stream
        .filter(|event| matches!(event, Ok(Event::AgentStarted { .. })))
        .for_each(|event| async move {
            handle_agent_started(event.unwrap()).await;
        })
        .await;
}
```

#### Pros
- **Integration**: Perfect tokio integration
- **Performance**: Zero-cost abstractions
- **Combinators**: Rich stream processing
- **Ecosystem**: Part of tokio

#### Cons
- **Tokio Lock-in**: Requires tokio runtime
- **Learning Curve**: Stream combinators complexity
- **Memory**: Buffering can consume memory

### 3.2 crossbeam-channel
**Part of**: Crossbeam project
**Type**: Multi-producer multi-consumer channels

#### Features
- **Channel Types**: Bounded, unbounded, rendezvous
- **Select**: Multi-channel selection
- **Performance**: Lock-free implementation
- **Compatibility**: Works with any runtime

#### Architecture Analysis
```rust
use crossbeam_channel::{bounded, select, Receiver, Sender};

// Event bus with multiple channel types
struct EventBus {
    high_priority: (Sender<Event>, Receiver<Event>),
    normal: (Sender<Event>, Receiver<Event>),
    low_priority: (Sender<Event>, Receiver<Event>),
}

impl EventBus {
    fn new() -> Self {
        Self {
            high_priority: bounded(10),
            normal: bounded(100),
            low_priority: bounded(1000),
        }
    }
    
    fn process_events(&self) {
        loop {
            select! {
                recv(self.high_priority.1) -> event => {
                    handle_high_priority(event?);
                }
                recv(self.normal.1) -> event => {
                    handle_normal(event?);
                }
                recv(self.low_priority.1) -> event => {
                    handle_low_priority(event?);
                }
            }
        }
    }
}
```

#### Pros
- **Performance**: Excellent performance
- **Flexibility**: Multiple channel types
- **Runtime Agnostic**: Works anywhere
- **Select**: Powerful selection primitive

#### Cons
- **Sync Only**: Not async native
- **Integration**: Needs adapter for async
- **API**: Lower level than alternatives

### 3.3 event-emitter-rs
**Repository**: https://github.com/Electron100/event-emitter-rs
**Type**: Node.js style event emitter

#### Features
- **Simple API**: on, emit, remove_listener
- **Async Support**: Async event handlers
- **Type Safe**: Generic over event types
- **Lightweight**: Minimal dependencies

#### Architecture Analysis
```rust
use event_emitter_rs::EventEmitter;

#[derive(Clone)]
enum AgentEvent {
    Started { id: String },
    Completed { id: String, result: String },
    Failed { id: String, error: String },
}

let mut emitter = EventEmitter::new();

// Register handlers
emitter.on("agent_event", |event: AgentEvent| {
    match event {
        AgentEvent::Started { id } => println!("Agent {} started", id),
        AgentEvent::Completed { id, result } => println!("Agent {} completed: {}", id, result),
        AgentEvent::Failed { id, error } => println!("Agent {} failed: {}", id, error),
    }
});

// Emit events
emitter.emit("agent_event", AgentEvent::Started { id: "agent_1".into() });
```

#### Pros
- **Simple**: Easy to understand API
- **Familiar**: Node.js style
- **Type Safe**: Rust type safety
- **Lightweight**: Small footprint

#### Cons
- **Features**: Limited compared to alternatives
- **Performance**: Not optimized for high throughput
- **Patterns**: May encourage coupling

### 3.4 Other Notable Event Systems

#### async-broadcast
- **Type**: Async broadcast channel
- **Features**: Multiple producers/consumers, lag handling
- **Use Case**: When tokio broadcast isn't enough

#### bus
- **Type**: Broadcast channel with typed topics
- **Features**: Type-safe topics, sync/async support
- **Use Case**: Simple pub/sub needs

#### postage
- **Type**: Async channel library
- **Features**: Multiple channel types, combinators
- **Use Case**: Alternative to tokio channels

## 4. Integration Recommendations

### 4.1 Workflow Engine Choice
**Recommendation**: Build custom lightweight workflow engine inspired by flowrs
**Rationale**:
- Temporal is too heavyweight for embedded use
- flowrs provides good patterns but needs enhancement
- Custom implementation can integrate tightly with agent hierarchy

**Key Features Needed**:
- Deterministic execution for replay
- State persistence between steps
- Parallel and sequential composition
- Error handling and retries
- Integration with script engines

### 4.2 State Management Choice
**Primary**: sled for development, rocksdb for production
**Rationale**:
- sled: Fast development, good Rust integration
- rocksdb: Production proven, better for large scale
- Both support subscriptions for state changes

**Abstraction Layer**:
```rust
trait StateStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn watch(&self, prefix: &str) -> Result<StateWatcher>;
}
```

### 4.3 Event System Choice
**Recommendation**: Hybrid approach
- tokio-stream for internal async events
- crossbeam-channel for cross-thread coordination
- Custom EventBus abstraction on top

**Architecture**:
```rust
struct EventBus {
    // Async events within tokio runtime
    async_bus: broadcast::Sender<Event>,
    
    // Sync events for script engines
    sync_bus: crossbeam_channel::Sender<Event>,
    
    // Stream processors
    processors: Vec<Box<dyn EventProcessor>>,
}
```

## 5. Implementation Strategy

### 5.1 Phased Approach
1. **Phase 1**: StateStore trait with sled implementation
2. **Phase 2**: EventBus with tokio-stream
3. **Phase 3**: Basic workflow engine
4. **Phase 4**: State machine integration for agents
5. **Phase 5**: Full orchestration capabilities

### 5.2 Testing Strategy
- Unit tests for each component
- Integration tests with script engines
- Performance benchmarks
- Chaos testing for state recovery

### 5.3 Performance Considerations
- Lazy state loading
- Event batching
- Async I/O throughout
- Configurable persistence strategies

## 6. Risks and Mitigations

### 6.1 Risks
- **Complexity**: Multiple moving parts
- **Performance**: State persistence overhead
- **Reliability**: Ensuring consistency
- **Memory**: Event buffering and state caching

### 6.2 Mitigations
- **Abstraction**: Clean interfaces between components
- **Configuration**: Tunable performance parameters
- **Monitoring**: Built-in metrics and tracing
- **Testing**: Comprehensive test coverage

## 7. Conclusion

### Selected Stack
- **Workflow**: Custom engine inspired by flowrs patterns
- **State Management**: sled (dev) / rocksdb (prod) behind trait
- **State Machines**: statig for hierarchical async states
- **Events**: tokio-stream + crossbeam hybrid
- **Persistence**: Pluggable storage backends

### Next Steps
1. Design StateStore trait and implementations
2. Implement EventBus abstraction
3. Create workflow engine prototype
4. Integrate with agent hierarchy
5. Add script engine bindings

This stack provides the foundation for reliable, scalable workflow orchestration while remaining lightweight enough for embedded scripting scenarios.