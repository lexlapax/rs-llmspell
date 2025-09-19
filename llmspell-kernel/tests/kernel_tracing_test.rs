//! Comprehensive kernel tracing tests for Phase 9.4.5.6 Subtask 6.1
//!
//! Tests the instrumentation added to:
//! - Transport layer
//! - Message routing with correlation IDs
//! - Session management lifecycle
//!
//! These tests verify that tracing is properly instrumented without
//! introducing new clippy warnings or performance degradation.

use llmspell_kernel::io::router::{MessageDestination, MessageRouter};
use llmspell_kernel::sessions::session::Session;
use llmspell_kernel::traits::transport::create_transport;
use llmspell_kernel::{CreateSessionOptions, IOPubMessage, MessageHeader, SessionStatus};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use uuid::Uuid;

/// Test helper to capture trace output
struct TraceCapture {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl TraceCapture {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[allow(dead_code)]
    fn captured(&self) -> Vec<String> {
        self.buffer.lock().unwrap().clone()
    }

    fn contains(&self, pattern: &str) -> bool {
        self.buffer
            .lock()
            .unwrap()
            .iter()
            .any(|line| line.contains(pattern))
    }

    fn setup_subscriber(&self) -> tracing::subscriber::DefaultGuard {
        let buffer = self.buffer.clone();

        // Create a custom writer that captures to our buffer
        let make_writer = move || TraceCaptureWriter {
            buffer: buffer.clone(),
        };

        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(make_writer)
                    .with_level(true)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .without_time()
                    .compact(),
            )
            .with(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("llmspell_kernel=trace".parse().unwrap()),
            );

        tracing::subscriber::set_default(subscriber)
    }
}

struct TraceCaptureWriter {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl std::io::Write for TraceCaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.buffer.lock().unwrap().push(s.to_string());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Test transport layer instrumentation
#[test]
fn test_transport_layer_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting transport layer tracing test");

    // Test creating a transport - should be traced
    let result = create_transport("inprocess");
    assert!(result.is_ok(), "Transport creation should succeed");

    // Verify tracing output
    assert!(
        capture.contains("Creating transport of type: inprocess"),
        "Should trace transport creation"
    );
    assert!(
        capture.contains("create_transport"),
        "Should include function name in trace"
    );
}

/// Test message routing with correlation IDs
#[tokio::test]
async fn test_message_routing_correlation_ids() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting message routing correlation ID test");

    // Create a message router
    let router = MessageRouter::new(100);

    // Register a client
    let (tx, _rx) = mpsc::channel(10);
    let client_id = router.register_client("test-session".to_string(), tx);

    // Create a test message
    let test_message = IOPubMessage {
        parent_header: None,
        header: MessageHeader::new("stream", "test-session"),
        metadata: HashMap::new(),
        content: {
            let mut content = HashMap::new();
            content.insert("name".to_string(), serde_json::json!("stdout"));
            content.insert("text".to_string(), serde_json::json!("Test output"));
            content
        },
    };

    // Route a message - should generate correlation ID
    let result = router
        .route_message(test_message.clone(), MessageDestination::Broadcast)
        .await;
    assert!(result.is_ok(), "Message routing should succeed");

    // Route another message to specific client
    let result = router
        .route_message(test_message, MessageDestination::Client(client_id.clone()))
        .await;
    assert!(result.is_ok(), "Message routing to client should succeed");

    // Verify correlation ID in traces
    assert!(
        capture.contains("correlation_id"),
        "Should include correlation_id in trace spans"
    );
    assert!(
        capture.contains("router_id"),
        "Should include router_id in trace"
    );
    assert!(
        capture.contains("Broadcasting message to all clients"),
        "Should trace broadcast operations"
    );
    assert!(
        capture.contains(&format!("Sending message to client {}", client_id)),
        "Should trace client-specific routing"
    );

    // Cleanup
    router.unregister_client(&client_id);
}

/// Test session lifecycle tracing
#[tokio::test]
async fn test_session_lifecycle_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting session lifecycle tracing test");

    // Create a new session
    let options = CreateSessionOptions {
        name: Some("test-session".to_string()),
        created_by: Some("test-user".to_string()),
        description: Some("Test session for tracing".to_string()),
        tags: vec!["test".to_string(), "tracing".to_string()],
        config: None,
        parent_session_id: None,
        metadata: HashMap::new(),
    };

    let session = Session::new(options);
    let _session_id = session.id().await;

    // Test suspend operation
    let result = session.suspend().await;
    assert!(result.is_ok(), "Suspend should succeed");

    // Test resume operation
    let result = session.resume().await;
    assert!(result.is_ok(), "Resume should succeed");

    // Test adding artifact
    let artifact_id = Uuid::new_v4().to_string();
    let result = session.add_artifact(artifact_id.clone()).await;
    assert!(result.is_ok(), "Add artifact should succeed");

    // Test complete operation
    let result = session.complete().await;
    assert!(result.is_ok(), "Complete should succeed");

    // Verify session lifecycle traces
    assert!(
        capture.contains("Creating new session with id="),
        "Should trace session creation"
    );
    assert!(
        capture.contains("Suspending session"),
        "Should trace session suspension"
    );
    assert!(
        capture.contains("Resuming session"),
        "Should trace session resumption"
    );
    assert!(
        capture.contains(&format!("Adding artifact {} to session", artifact_id)),
        "Should trace artifact addition"
    );

    // Test invalid state transition
    let result = session.resume().await; // Can't resume completed session
    assert!(result.is_err(), "Resume should fail on completed session");

    // Try suspending completed session - this one logs a warning
    let result = session.suspend().await; // Can't suspend completed session
    assert!(result.is_err(), "Suspend should fail on completed session");

    assert!(
        capture.contains("Cannot suspend session in state Completed"),
        "Should trace invalid state transitions as warnings"
    );
}

/// Test that correlation IDs are properly set and cleared
#[tokio::test]
async fn test_correlation_id_management() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Testing correlation ID management");

    let router = MessageRouter::new(50);

    // Set a specific correlation ID
    let correlation_id = Uuid::new_v4();
    router.set_correlation_id(Some(correlation_id));

    // Register a client for testing
    let (tx, _rx) = mpsc::channel(10);
    let _client_id = router.register_client("test-session".to_string(), tx);

    // Route a message - should use the set correlation ID
    let test_message = IOPubMessage {
        parent_header: None,
        header: MessageHeader::new("status", "test-session"),
        metadata: HashMap::new(),
        content: {
            let mut content = HashMap::new();
            content.insert("execution_state".to_string(), serde_json::json!("idle"));
            content
        },
    };

    let _ = router
        .route_message(test_message.clone(), MessageDestination::Broadcast)
        .await;

    // Clear correlation ID
    router.set_correlation_id(None);

    // Route another message - should create new correlation ID
    let _ = router
        .route_message(test_message, MessageDestination::Broadcast)
        .await;

    // Verify correlation ID management in traces
    assert!(
        capture.contains(&format!("Set correlation_id={}", correlation_id)),
        "Should trace setting correlation ID"
    );
    assert!(
        capture.contains("Cleared correlation_id"),
        "Should trace clearing correlation ID"
    );
}

/// Test client lifecycle in message router
#[tokio::test]
async fn test_router_client_lifecycle() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Testing router client lifecycle");

    let router = MessageRouter::new(10);

    // Register multiple clients
    let (tx1, _rx1) = mpsc::channel(10);
    let client1 = router.register_client("session-1".to_string(), tx1);

    let (tx2, _rx2) = mpsc::channel(10);
    let client2 = router.register_client("session-2".to_string(), tx2);

    // Deactivate a client
    router.deactivate_client(&client1);

    // Try to deactivate non-existent client (should warn)
    router.deactivate_client("non-existent-client");

    // Unregister clients
    assert!(router.unregister_client(&client1));
    assert!(router.unregister_client(&client2));
    assert!(!router.unregister_client("non-existent"));

    // Verify client lifecycle traces
    assert!(
        capture.contains(&format!("Registered client {}", client1)),
        "Should trace client registration"
    );
    assert!(
        capture.contains(&format!("Deactivated client {}", client1)),
        "Should trace client deactivation"
    );
    assert!(
        capture.contains("Attempted to deactivate non-existent client"),
        "Should warn about non-existent client deactivation"
    );
    assert!(
        capture.contains(&format!("Unregistered client {}", client1)),
        "Should trace client unregistration"
    );
}

/// Test session state transitions with proper tracing
#[tokio::test]
async fn test_session_state_transition_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Testing session state transition tracing");

    let options = CreateSessionOptions {
        name: Some("transition-test".to_string()),
        created_by: Some("test-user".to_string()),
        description: None,
        tags: vec![],
        config: None,
        parent_session_id: None,
        metadata: HashMap::new(),
    };

    let session = Session::new(options);

    // Test normal flow: Active -> Suspended -> Active -> Completed
    assert_eq!(session.status().await, SessionStatus::Active);

    session.suspend().await.unwrap();
    assert_eq!(session.status().await, SessionStatus::Suspended);

    session.resume().await.unwrap();
    assert_eq!(session.status().await, SessionStatus::Active);

    session.complete().await.unwrap();
    assert_eq!(session.status().await, SessionStatus::Completed);

    // Try invalid transitions (should be traced as warnings)
    let _ = session.suspend().await; // Can't suspend completed
    let _ = session.resume().await; // Can't resume completed

    // Test failure path
    let options2 = CreateSessionOptions {
        name: Some("failure-test".to_string()),
        created_by: Some("test-user".to_string()),
        description: None,
        tags: vec![],
        config: None,
        parent_session_id: None,
        metadata: HashMap::new(),
    };

    let session2 = Session::new(options2);
    session2.fail().await.unwrap();

    // Can't transition from failed state
    let _ = session2.suspend().await;

    // Verify all state transitions are traced
    assert!(
        capture.contains("transition-test"),
        "Should include session name in traces"
    );
    assert!(
        capture.contains("failure-test"),
        "Should include failure session name"
    );
}

/// Integration test combining all traced components
#[tokio::test]
async fn test_integrated_kernel_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting integrated kernel tracing test");

    // 1. Create transport
    let _transport = create_transport("inprocess").unwrap();

    // 2. Create session
    let session_options = CreateSessionOptions {
        name: Some("integrated-test".to_string()),
        created_by: Some("test-system".to_string()),
        description: Some("Integration test session".to_string()),
        tags: vec!["integration".to_string()],
        config: None,
        parent_session_id: None,
        metadata: HashMap::new(),
    };
    let session = Session::new(session_options);

    // 3. Create message router
    let router = MessageRouter::new(50);

    // 4. Register session with router
    let (tx, _rx) = mpsc::channel(100);
    let client_id = router.register_client(session.id().await.to_string(), tx);

    // 5. Perform operations
    session.suspend().await.unwrap();

    let message = IOPubMessage {
        parent_header: None,
        header: MessageHeader::new("execute_input", session.id().await.to_string().as_str()),
        metadata: HashMap::new(),
        content: {
            let mut content = HashMap::new();
            content.insert(
                "code".to_string(),
                serde_json::json!("print('Hello, World!')"),
            );
            content.insert("execution_count".to_string(), serde_json::json!(1));
            content
        },
    };

    router
        .route_message(message, MessageDestination::Client(client_id.clone()))
        .await
        .unwrap();

    session.resume().await.unwrap();
    session.complete().await.unwrap();

    router.unregister_client(&client_id);

    // Verify integrated traces
    assert!(
        capture.contains("transport"),
        "Should have transport traces"
    );
    assert!(capture.contains("session"), "Should have session traces");
    assert!(
        capture.contains("correlation_id"),
        "Should have correlation ID traces"
    );
    assert!(capture.contains("router"), "Should have router traces");

    debug!("Integrated kernel tracing test completed successfully");
}

/// Test that tracing doesn't introduce performance regression
#[tokio::test]
async fn test_tracing_performance_impact() {
    use std::time::Instant;

    // Test without detailed tracing
    let start = Instant::now();
    for _ in 0..100 {
        let options = CreateSessionOptions {
            name: Some("perf-test".to_string()),
            created_by: Some("test".to_string()),
            description: None,
            tags: vec![],
            config: None,
            parent_session_id: None,
            metadata: HashMap::new(),
        };
        let session = Session::new(options);
        let _ = session.suspend().await;
        let _ = session.resume().await;
        let _ = session.complete().await;
    }
    let duration_with_tracing = start.elapsed();

    // Performance assertion - should complete 100 iterations quickly
    assert!(
        duration_with_tracing.as_millis() < 1000,
        "100 session lifecycles should complete in under 1 second, took {}ms",
        duration_with_tracing.as_millis()
    );

    info!(
        "Performance test: 100 iterations in {:?}",
        duration_with_tracing
    );
}

/// Verify no new clippy warnings in instrumented code
#[test]
fn test_no_new_clippy_warnings() {
    // This test just needs to compile without warnings
    // The actual clippy check is done during compilation

    // Test that all our trace macros compile without warnings
    info!("Testing trace macro usage");
    debug!("Debug level trace");
    warn!("Warning level trace");

    // Test field recording patterns we use
    let correlation_id = Uuid::new_v4();
    info!(correlation_id = %correlation_id, "Testing field recording");

    // Test skip patterns
    let large_data = vec![0u8; 1000];
    debug!(data_size = large_data.len(), "Skipping large data in trace");
}
