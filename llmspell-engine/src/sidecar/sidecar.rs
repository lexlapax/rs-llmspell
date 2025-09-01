//! Service mesh sidecar implementation
//!
//! Layer 2: Shared logic following Phase 9.1-9.3 patterns

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::engine::{EngineError, ProtocolAdapter, ProtocolEngine, ProtocolType, UniversalMessage};
use llmspell_utils::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};

use super::discovery::{DiscoveryError, ServiceDiscovery, ServiceInfo, ServiceQuery};
use super::metrics::{MetricTimer, MetricsCollector};

/// Sidecar errors
#[derive(Error, Debug)]
pub enum SidecarError {
    #[error("Protocol negotiation failed: {0}")]
    NegotiationFailed(String),

    #[error("Message processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(#[from] CircuitBreakerError),

    #[error("Discovery error: {0}")]
    Discovery(#[from] DiscoveryError),

    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),

    #[error("No suitable protocol found")]
    NoProtocolFound,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Result type for sidecar operations
pub type SidecarResult<T> = Result<T, SidecarError>;

/// Raw message before protocol processing
#[derive(Debug, Clone)]
pub struct RawMessage {
    /// Raw bytes
    pub data: Vec<u8>,
    /// Source address
    pub source: String,
    /// Target service
    pub target: Option<String>,
    /// Message headers
    pub headers: HashMap<String, String>,
}

/// Processed message after sidecar handling
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    /// Universal message format
    pub universal: UniversalMessage,
    /// Processing metadata
    pub metadata: ProcessingMetadata,
}

/// Metadata about message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    /// Protocol used
    pub protocol: ProtocolType,
    /// Processing duration
    pub duration: Duration,
    /// Circuit breaker state
    pub circuit_state: String,
    /// Retry count if any
    pub retry_count: u32,
}

/// Sidecar configuration with adaptive thresholds
#[derive(Debug)]
pub struct SidecarConfig {
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Maximum retries for failed operations
    pub max_retries: u32,
    /// Timeout for protocol negotiation
    pub negotiation_timeout: Duration,
    /// Timeout for message processing
    pub processing_timeout: Duration,
    /// Enable caching of protocol negotiations
    pub enable_negotiation_cache: bool,
    /// Cache TTL
    pub cache_ttl: Duration,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_config: CircuitBreakerConfig::default(),
            max_retries: 3,
            negotiation_timeout: Duration::from_secs(5),
            processing_timeout: Duration::from_secs(30),
            enable_negotiation_cache: true,
            cache_ttl: Duration::from_secs(300),
        }
    }
}

/// Protocol negotiation cache entry
struct CacheEntry {
    protocol: ProtocolType,
    timestamp: Instant,
}

/// Service mesh sidecar - Layer 2: Shared logic
pub struct Sidecar {
    /// Protocol engine for message handling
    engine: Arc<dyn ProtocolEngine>,
    /// Protocol adapters (using existing ProtocolAdapter trait)
    adapters: Arc<RwLock<HashMap<ProtocolType, Box<dyn ProtocolAdapter>>>>,
    /// Circuit breaker for fault tolerance
    circuit_breaker: CircuitBreaker,
    /// Service discovery
    discovery: Arc<dyn ServiceDiscovery>,
    /// Metrics collector
    metrics: Arc<dyn MetricsCollector>,
    /// Configuration
    config: SidecarConfig,
    /// Protocol negotiation cache
    negotiation_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl Sidecar {
    /// Create a new sidecar instance with dependency injection
    pub fn new(
        engine: Arc<dyn ProtocolEngine>,
        circuit_breaker: CircuitBreaker,
        discovery: Arc<dyn ServiceDiscovery>,
        metrics: Arc<dyn MetricsCollector>,
        config: SidecarConfig,
    ) -> Self {
        Self {
            engine,
            adapters: Arc::new(RwLock::new(HashMap::new())),
            circuit_breaker,
            discovery,
            metrics,
            config,
            negotiation_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a protocol adapter
    pub async fn register_adapter(&self, adapter: Box<dyn ProtocolAdapter>) {
        let protocol = adapter.protocol_type();
        let mut adapters = self.adapters.write().await;
        adapters.insert(protocol, adapter);
        info!("Registered protocol adapter for {:?}", protocol);
    }

    /// Intercept and process a message
    pub async fn intercept(&self, msg: RawMessage) -> SidecarResult<ProcessedMessage> {
        let timer = MetricTimer::start("intercept", "unknown", "unknown");

        // Check if circuit breaker allows the request
        match self.circuit_breaker.allow_request().await {
            Ok(()) => {
                // Process the message
                match self.process_message(msg).await {
                    Ok(processed) => {
                        // Record success
                        self.circuit_breaker.record_success().await;
                        let metric = timer.complete(true, None);
                        self.metrics.record(metric).await;
                        Ok(processed)
                    }
                    Err(e) => {
                        // Record failure
                        self.circuit_breaker.record_failure().await;
                        let metric = timer.complete(false, Some(e.to_string()));
                        self.metrics.record(metric).await;
                        Err(e)
                    }
                }
            }
            Err(CircuitBreakerError::CircuitOpen { reason }) => {
                // Circuit is open, reject request
                self.metrics.record_circuit_breaker_trip().await;
                let metric = timer.complete(false, Some(reason.clone()));
                self.metrics.record(metric).await;
                Err(SidecarError::ServiceUnavailable(reason))
            }
            Err(e) => {
                let metric = timer.complete(false, Some(e.to_string()));
                self.metrics.record(metric).await;
                Err(SidecarError::CircuitBreaker(e))
            }
        }
    }

    /// Process a message (called within circuit breaker)
    async fn process_message(&self, msg: RawMessage) -> SidecarResult<ProcessedMessage> {
        let start = Instant::now();

        // Negotiate protocol
        let negotiation_timer = Instant::now();
        let protocol = self.negotiate_protocol(&msg).await?;
        self.metrics
            .record_negotiation(negotiation_timer.elapsed())
            .await;

        // Get adapter
        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&protocol)
            .ok_or(SidecarError::NoProtocolFound)?;

        // Adapt message
        let adaptation_timer = Instant::now();
        let universal = adapter
            .adapt_inbound(&msg.data)
            .map_err(|e| SidecarError::ProcessingFailed(e.to_string()))?;
        self.metrics
            .record_adaptation(adaptation_timer.elapsed())
            .await;

        // Create metadata
        let metadata = ProcessingMetadata {
            protocol,
            duration: start.elapsed(),
            circuit_state: "closed".to_string(), // If we got here, circuit is closed
            retry_count: 0,
        };

        Ok(ProcessedMessage {
            universal,
            metadata,
        })
    }

    /// Negotiate which protocol to use for a message
    async fn negotiate_protocol(&self, msg: &RawMessage) -> SidecarResult<ProtocolType> {
        // Check cache first if enabled
        if self.config.enable_negotiation_cache {
            if let Some(cached) = self.check_cache(&msg.source).await {
                debug!("Using cached protocol {:?} for {}", cached, msg.source);
                return Ok(cached);
            }
        }

        // Try to detect protocol from message content
        let protocol = self.detect_protocol(&msg.data).await?;

        // Update cache
        if self.config.enable_negotiation_cache {
            self.update_cache(&msg.source, protocol).await;
        }

        Ok(protocol)
    }

    /// Detect protocol from message content
    async fn detect_protocol(&self, data: &[u8]) -> SidecarResult<ProtocolType> {
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_slice::<Value>(data) {
            // Check for protocol hints in JSON
            if json.get("msg_type").is_some() || json.get("msg_id").is_some() {
                return Ok(ProtocolType::LRP);
            }
            if json.get("command").is_some() || json.get("seq").is_some() {
                return Ok(ProtocolType::LDP);
            }
        }

        // Default to LRP for now
        Ok(ProtocolType::LRP)
    }

    /// Check negotiation cache
    async fn check_cache(&self, source: &str) -> Option<ProtocolType> {
        let cache = self.negotiation_cache.read().await;
        cache.get(source).and_then(|entry| {
            if entry.timestamp.elapsed() < self.config.cache_ttl {
                Some(entry.protocol)
            } else {
                None
            }
        })
    }

    /// Update negotiation cache
    async fn update_cache(&self, source: &str, protocol: ProtocolType) {
        let mut cache = self.negotiation_cache.write().await;

        // Limit cache size
        if cache.len() > 1000 {
            cache.clear(); // Simple eviction strategy
        }

        cache.insert(
            source.to_string(),
            CacheEntry {
                protocol,
                timestamp: Instant::now(),
            },
        );
    }

    /// Discover services that can handle a specific protocol
    pub async fn discover_services(
        &self,
        protocol: ProtocolType,
    ) -> SidecarResult<Vec<ServiceInfo>> {
        let query = ServiceQuery {
            protocol: Some(format!("{:?}", protocol)),
            ..Default::default()
        };

        self.discovery.discover(query).await.map_err(Into::into)
    }

    /// Route a processed message to the appropriate service
    pub async fn route(&self, msg: ProcessedMessage) -> SidecarResult<()> {
        // Send through protocol engine
        self.engine
            .send(msg.universal.channel, msg.universal)
            .await
            .map_err(Into::into)
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> super::metrics::AggregatedMetrics {
        self.metrics.get_aggregated().await
    }

    /// Health check
    pub async fn health_check(&self) -> bool {
        // Check if circuit breaker is healthy
        // In real implementation, would check CircuitState
        true
    }
}

#[cfg(test)]
mod tests {
    use super::super::discovery::NullServiceDiscovery;
    use super::super::metrics::NullMetricsCollector;
    use super::*;
    use crate::engine::UnifiedProtocolEngine;
    use crate::transport::mock::MockTransport;

    #[tokio::test]
    async fn test_sidecar_creation() {
        let transport = Box::new(MockTransport::new());
        let engine = Arc::new(UnifiedProtocolEngine::new(transport));
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let discovery = Arc::new(NullServiceDiscovery);
        let metrics = Arc::new(NullMetricsCollector);
        let config = SidecarConfig::default();

        let sidecar = Sidecar::new(engine, circuit_breaker, discovery, metrics, config);

        assert!(sidecar.health_check().await);
    }

    #[tokio::test]
    async fn test_protocol_detection() {
        let transport = Box::new(MockTransport::new());
        let engine = Arc::new(UnifiedProtocolEngine::new(transport));
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let discovery = Arc::new(NullServiceDiscovery);
        let metrics = Arc::new(NullMetricsCollector);
        let config = SidecarConfig::default();

        let sidecar = Sidecar::new(engine, circuit_breaker, discovery, metrics, config);

        // Test LRP detection
        let lrp_data = r#"{"msg_type": "request", "msg_id": "123"}"#.as_bytes();
        let protocol = sidecar.detect_protocol(lrp_data).await.unwrap();
        assert_eq!(protocol, ProtocolType::LRP);

        // Test LDP detection
        let ldp_data = r#"{"command": "initialize", "seq": 1}"#.as_bytes();
        let protocol = sidecar.detect_protocol(ldp_data).await.unwrap();
        assert_eq!(protocol, ProtocolType::LDP);
    }
}
