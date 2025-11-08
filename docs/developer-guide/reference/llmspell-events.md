# llmspell-events

**Event system with pub/sub and correlation**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-events) | [Source](../../../../llmspell-events)

---

## Overview

`llmspell-events` provides a high-performance event bus with pub/sub patterns, event correlation, persistence, and replay capabilities.

**Key Features:**
- 📡 Pub/sub event bus
- 🔗 Event correlation
- 💾 Event persistence
- 🔄 Event replay
- 📊 Event metrics
- 🎯 Pattern matching subscriptions
- ⚡ 90K+ events/sec throughput
- 🔐 Event filtering

## Event System

```rust
use llmspell_events::{EventBus, Event, Subscription};

let event_bus = EventBus::new();

// Subscribe to events
let subscription = event_bus.subscribe("user.*", |event: Event| {
    println!("User event: {:?}", event);
}).await?;

// Publish events
event_bus.publish(Event {
    id: Uuid::new_v4(),
    event_type: "user.login".to_string(),
    data: json!({"user_id": "123"}),
    timestamp: SystemTime::now(),
    correlation_id: None,
}).await?;
```

## Event Correlation

```rust
let correlation_id = Uuid::new_v4();

// Correlated events
event_bus.publish_correlated(
    "workflow.start",
    json!({"workflow": "analysis"}),
    correlation_id,
).await?;

event_bus.publish_correlated(
    "workflow.step",
    json!({"step": "data_fetch"}),
    correlation_id,
).await?;

// Query correlated events
let events = event_bus.get_correlated(correlation_id).await?;
```

## Pattern Subscriptions

```rust
// Wildcard patterns
event_bus.subscribe("agent.*", handle_agent_events).await?;
event_bus.subscribe("*.error", handle_errors).await?;
event_bus.subscribe("workflow.*.complete", handle_completion).await?;

// Exact match
event_bus.subscribe("system.shutdown", handle_shutdown).await?;
```

## Event Persistence

```rust
use llmspell_events::PersistentEventBus;

let event_bus = PersistentEventBus::new(EventConfig {
    persistence_path: Some("./events.db".into()),
    retention_days: 30,
    ..Default::default()
})?;

// Events are automatically persisted
event_bus.publish(event).await?;

// Query historical events
let events = event_bus.query(EventQuery {
    event_types: Some(vec!["agent.execution".to_string()]),
    time_range: Some((yesterday, now)),
    limit: Some(100),
}).await?;
```

## Performance

- **Throughput**: 90,000+ events/second
- **Latency**: <1ms p99
- **Memory**: O(subscribers) + O(buffer_size)

## Related Documentation

- [llmspell-hooks](llmspell-hooks.md) - Hook system integration
- [llmspell-sessions](llmspell-sessions.md) - Session event tracking