//! Session recording and replay trait abstraction for debugging and analysis
//!
//! Provides trait-based abstraction for recording debug sessions with adaptive
//! performance configuration based on session size and operation complexity.

use crate::execution_bridge::{DebugState, StackFrame, Variable};
use crate::execution_context::{SharedExecutionContext, SourceLocation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Session event types that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// Script execution started
    ScriptStart {
        script_path: String,
        #[serde(skip)]
        context: SharedExecutionContext,
    },
    /// Variable value changed
    VariableChange {
        variable: Variable,
        location: SourceLocation,
    },
    /// Function was called
    FunctionCall {
        stack_frame: StackFrame,
        arguments: Vec<Variable>,
    },
    /// Tool was invoked
    ToolInvocation {
        tool_name: String,
        arguments: serde_json::Value,
        #[serde(skip)]
        context: SharedExecutionContext,
    },
    /// Breakpoint was hit
    BreakpointHit {
        location: SourceLocation,
        stack: Vec<StackFrame>,
        locals: Vec<Variable>,
    },
    /// Debug state changed
    DebugStateChange {
        old_state: DebugState,
        new_state: DebugState,
    },
}

/// Timestamped session event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The event itself
    pub event: SessionEvent,
}

/// Storage vs CPU tradeoff preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeoffPreference {
    /// Prefer using more storage to save CPU (less compression)
    PreferStorage,
    /// Balanced approach
    Balanced,
    /// Prefer using more CPU to save storage (more compression)
    PreferCPU,
}

impl Default for TradeoffPreference {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Environment preset for session recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment - verbose recording
    Development,
    /// Testing environment - moderate recording
    Testing,
    /// Production environment - minimal recording
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Configuration for session recording with adaptive thresholds
#[derive(Debug, Clone)]
pub struct SessionRecorderConfig {
    /// Maximum memory usage in megabytes
    pub max_memory_mb: usize,
    /// Memory threshold to start compression (MB)
    pub compression_threshold_mb: usize,
    /// Event rate threshold for sampling (events/sec)
    pub sampling_threshold_events_per_sec: f64,
    /// Enable adaptive sampling based on event rate
    pub adaptive_sampling: bool,
    /// Storage vs CPU preference for compression
    pub storage_vs_cpu_preference: TradeoffPreference,
    /// Environment preset
    pub environment_preset: Environment,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
}

impl Default for SessionRecorderConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 100,
            compression_threshold_mb: 50,
            sampling_threshold_events_per_sec: 100.0,
            adaptive_sampling: true,
            storage_vs_cpu_preference: TradeoffPreference::Balanced,
            environment_preset: Environment::Development,
            sampling_rate: 1.0,
        }
    }
}

impl SessionRecorderConfig {
    /// Create config for production environment (minimal overhead)
    #[must_use]
    pub const fn production() -> Self {
        Self {
            max_memory_mb: 50,
            compression_threshold_mb: 25,
            sampling_threshold_events_per_sec: 50.0,
            adaptive_sampling: true,
            storage_vs_cpu_preference: TradeoffPreference::PreferCPU,
            environment_preset: Environment::Production,
            sampling_rate: 0.1, // Sample only 10% in production
        }
    }

    /// Create config for testing environment
    #[must_use]
    pub const fn testing() -> Self {
        Self {
            max_memory_mb: 200,
            compression_threshold_mb: 100,
            sampling_threshold_events_per_sec: 200.0,
            adaptive_sampling: true,
            storage_vs_cpu_preference: TradeoffPreference::Balanced,
            environment_preset: Environment::Testing,
            sampling_rate: 0.5, // Sample 50% in testing
        }
    }

    /// Check if sampling should occur based on current rate
    #[must_use]
    pub fn should_sample(&self, current_rate: f64) -> bool {
        if !self.adaptive_sampling {
            return rand::random::<f64>() <= self.sampling_rate;
        }

        // Adaptive sampling - reduce rate if above threshold
        if current_rate > self.sampling_threshold_events_per_sec {
            let reduction_factor = self.sampling_threshold_events_per_sec / current_rate;
            rand::random::<f64>() <= (self.sampling_rate * reduction_factor).max(0.01)
        } else {
            rand::random::<f64>() <= self.sampling_rate
        }
    }
}

/// Session recording statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of events recorded
    pub event_count: usize,
    /// Number of events sampled (not recorded due to sampling)
    pub events_sampled: usize,
    /// Session duration
    pub duration: std::time::Duration,
    /// Memory used (bytes)
    pub memory_used: usize,
    /// Whether compression was used
    pub compression_used: bool,
    /// Current sampling rate
    pub sampling_rate: f64,
}

/// Trait for session recording implementations
pub trait SessionRecorder: Send + Sync {
    /// Start recording a session
    ///
    /// # Errors
    /// Returns error if recording cannot be started or is already active
    fn start_recording(&mut self, config: SessionRecorderConfig) -> Result<(), Box<dyn Error>>;

    /// Record a session event
    ///
    /// # Errors
    /// Returns error if the event cannot be recorded
    fn record_event(&mut self, event: SessionEvent) -> Result<(), Box<dyn Error>>;

    /// Stop recording and get statistics
    ///
    /// # Errors
    /// Returns error if recording is not active
    fn stop_recording(&mut self) -> Result<SessionStats, Box<dyn Error>>;

    /// Check if an event should be sampled based on current frequency
    fn should_sample(&self, event_frequency: f64) -> bool;

    /// Adapt compression based on session size
    fn adapt_compression(&mut self, session_size: usize);

    /// Get current configuration
    fn config(&self) -> &SessionRecorderConfig;

    /// Check if recording is active
    fn is_recording(&self) -> bool;

    /// Save recorded session to bytes
    ///
    /// # Errors
    /// Returns error if session cannot be serialized
    fn save(&self) -> Result<Vec<u8>, Box<dyn Error>>;

    /// Load session from bytes for replay
    ///
    /// # Errors
    /// Returns error if session cannot be deserialized
    fn load(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;

    /// Get recorded events for replay
    fn get_events(&self) -> &[TimestampedEvent];
}

/// JSON file-based session recorder
pub struct JsonFileRecorder {
    config: SessionRecorderConfig,
    events: Vec<TimestampedEvent>,
    start_time: Option<DateTime<Utc>>,
    is_recording: bool,
    events_sampled: usize,
    compression_enabled: bool,
}

impl JsonFileRecorder {
    /// Create a new JSON file recorder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SessionRecorderConfig::default(),
            events: Vec::new(),
            start_time: None,
            is_recording: false,
            events_sampled: 0,
            compression_enabled: false,
        }
    }

    /// Estimate current memory usage
    const fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: 1KB per event on average
        self.events.len() * 1024
    }
}

impl Default for JsonFileRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRecorder for JsonFileRecorder {
    fn start_recording(&mut self, config: SessionRecorderConfig) -> Result<(), Box<dyn Error>> {
        if self.is_recording {
            return Err("Recording is already active".into());
        }

        self.config = config;
        self.events.clear();
        self.start_time = Some(Utc::now());
        self.is_recording = true;
        self.events_sampled = 0;
        self.compression_enabled = false;

        Ok(())
    }

    fn record_event(&mut self, event: SessionEvent) -> Result<(), Box<dyn Error>> {
        if !self.is_recording {
            return Err("Recording is not active".into());
        }

        // Check memory usage and adapt compression
        let memory_mb = self.estimate_memory_usage() / (1024 * 1024);
        if memory_mb >= self.config.compression_threshold_mb {
            self.adapt_compression(self.estimate_memory_usage());
        }

        // Check if we should sample this event
        #[allow(clippy::cast_precision_loss)]
        let event_rate = self.events.len() as f64
            / self
                .start_time
                .map_or(1.0, |t| (Utc::now() - t).num_seconds() as f64)
                .max(1.0);

        if !self.should_sample(event_rate) {
            self.events_sampled += 1;
            return Ok(());
        }

        self.events.push(TimestampedEvent {
            timestamp: Utc::now(),
            event,
        });

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<SessionStats, Box<dyn Error>> {
        if !self.is_recording {
            return Err("Recording is not active".into());
        }

        let start_time = self.start_time.ok_or("No start time recorded")?;
        let duration = (Utc::now() - start_time)
            .to_std()
            .unwrap_or(std::time::Duration::ZERO);

        let stats = SessionStats {
            event_count: self.events.len(),
            events_sampled: self.events_sampled,
            duration,
            memory_used: self.estimate_memory_usage(),
            compression_used: self.compression_enabled,
            sampling_rate: self.config.sampling_rate,
        };

        self.is_recording = false;
        self.start_time = None;

        Ok(stats)
    }

    fn should_sample(&self, event_frequency: f64) -> bool {
        self.config.should_sample(event_frequency)
    }

    fn adapt_compression(&mut self, session_size: usize) {
        let size_mb = session_size / (1024 * 1024);
        if size_mb >= self.config.compression_threshold_mb {
            self.compression_enabled = true;
            // In a real implementation, we would compress older events here
        }
    }

    fn config(&self) -> &SessionRecorderConfig {
        &self.config
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn save(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // Note: In a real implementation, we would use compression when enabled
        serde_json::to_vec(&self.events).map_err(Into::into)
    }

    fn load(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.events = serde_json::from_slice(data)?;
        self.is_recording = false;
        Ok(())
    }

    fn get_events(&self) -> &[TimestampedEvent] {
        &self.events
    }
}

/// In-memory session recorder for testing
pub struct InMemoryRecorder {
    config: SessionRecorderConfig,
    events: Vec<TimestampedEvent>,
    start_time: Option<DateTime<Utc>>,
    is_recording: bool,
    events_sampled: usize,
}

impl InMemoryRecorder {
    /// Create a new in-memory recorder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SessionRecorderConfig::default(),
            events: Vec::new(),
            start_time: None,
            is_recording: false,
            events_sampled: 0,
        }
    }
}

impl Default for InMemoryRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRecorder for InMemoryRecorder {
    fn start_recording(&mut self, config: SessionRecorderConfig) -> Result<(), Box<dyn Error>> {
        if self.is_recording {
            return Err("Recording is already active".into());
        }

        self.config = config;
        self.events.clear();
        self.start_time = Some(Utc::now());
        self.is_recording = true;
        self.events_sampled = 0;

        Ok(())
    }

    fn record_event(&mut self, event: SessionEvent) -> Result<(), Box<dyn Error>> {
        if !self.is_recording {
            return Err("Recording is not active".into());
        }

        self.events.push(TimestampedEvent {
            timestamp: Utc::now(),
            event,
        });

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<SessionStats, Box<dyn Error>> {
        if !self.is_recording {
            return Err("Recording is not active".into());
        }

        let start_time = self.start_time.ok_or("No start time recorded")?;
        let duration = (Utc::now() - start_time)
            .to_std()
            .unwrap_or(std::time::Duration::ZERO);

        let stats = SessionStats {
            event_count: self.events.len(),
            events_sampled: self.events_sampled,
            duration,
            memory_used: self.events.len() * std::mem::size_of::<TimestampedEvent>(),
            compression_used: false,
            sampling_rate: self.config.sampling_rate,
        };

        self.is_recording = false;

        Ok(stats)
    }

    fn should_sample(&self, event_frequency: f64) -> bool {
        self.config.should_sample(event_frequency)
    }

    fn adapt_compression(&mut self, _session_size: usize) {
        // No compression for in-memory recorder
    }

    fn config(&self) -> &SessionRecorderConfig {
        &self.config
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn save(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        serde_json::to_vec(&self.events).map_err(Into::into)
    }

    fn load(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.events = serde_json::from_slice(data)?;
        Ok(())
    }

    fn get_events(&self) -> &[TimestampedEvent] {
        &self.events
    }
}

// Simple random implementation to avoid external dependencies
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[allow(clippy::cast_precision_loss)]
    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        let hash = hasher.finish();
        T::from((hash as f64) / (u64::MAX as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_recorder_config_defaults() {
        let config = SessionRecorderConfig::default();
        assert_eq!(config.max_memory_mb, 100);
        assert_eq!(config.environment_preset, Environment::Development);
        assert!(config.adaptive_sampling);
    }

    #[test]
    fn test_session_recorder_config_presets() {
        let prod_config = SessionRecorderConfig::production();
        assert_eq!(prod_config.max_memory_mb, 50);
        assert_eq!(prod_config.sampling_rate, 0.1);

        let test_config = SessionRecorderConfig::testing();
        assert_eq!(test_config.max_memory_mb, 200);
        assert_eq!(test_config.sampling_rate, 0.5);
    }

    #[test]
    fn test_json_file_recorder_lifecycle() {
        let mut recorder = JsonFileRecorder::new();
        
        assert!(!recorder.is_recording());
        
        // Start recording
        let config = SessionRecorderConfig::default();
        assert!(recorder.start_recording(config).is_ok());
        assert!(recorder.is_recording());
        
        // Cannot start when already recording
        assert!(recorder
            .start_recording(SessionRecorderConfig::default())
            .is_err());
        
        // Record some events
        let event = SessionEvent::ScriptStart {
            script_path: "test.lua".to_string(),
            context: SharedExecutionContext::new(),
        };
        assert!(recorder.record_event(event).is_ok());
        
        // Stop recording
        let stats = recorder.stop_recording();
        assert!(stats.is_ok());
        assert!(!recorder.is_recording());
        
        let stats = stats.unwrap();
        assert_eq!(stats.event_count, 1);
    }

    #[test]
    fn test_in_memory_recorder() {
        let mut recorder = InMemoryRecorder::new();
        
        recorder
            .start_recording(SessionRecorderConfig::default())
            .unwrap();
        
        let event1 = SessionEvent::ScriptStart {
            script_path: "test.lua".to_string(),
            context: SharedExecutionContext::new(),
        };
        
        let event2 = SessionEvent::DebugStateChange {
            old_state: DebugState::Running,
            new_state: DebugState::Paused {
                reason: crate::execution_bridge::PauseReason::Breakpoint,
                location: crate::execution_bridge::ExecutionLocation {
                    source: "test.lua".to_string(),
                    line: 10,
                    column: None,
                },
            },
        };
        
        recorder.record_event(event1).unwrap();
        recorder.record_event(event2).unwrap();
        
        assert_eq!(recorder.get_events().len(), 2);
        
        let stats = recorder.stop_recording().unwrap();
        assert_eq!(stats.event_count, 2);
    }

    #[test]
    fn test_save_and_load() {
        let mut recorder = JsonFileRecorder::new();
        
        recorder
            .start_recording(SessionRecorderConfig::default())
            .unwrap();
        
        let event = SessionEvent::ScriptStart {
            script_path: "test.lua".to_string(),
            context: SharedExecutionContext::new(),
        };
        
        recorder.record_event(event).unwrap();
        recorder.stop_recording().unwrap();
        
        // Save
        let data = recorder.save().unwrap();
        assert!(!data.is_empty());
        
        // Load into new recorder
        let mut new_recorder = JsonFileRecorder::new();
        assert!(new_recorder.load(&data).is_ok());
        assert_eq!(new_recorder.get_events().len(), 1);
    }
}