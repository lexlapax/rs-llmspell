//! Null session recorder implementation for testing
//!
//! Provides a no-op session recorder that implements the `SessionRecorder` trait
//! without any actual recording functionality. Safe for use in tests as it
//! performs no file I/O or memory allocation.

use crate::session_recorder::{
    SessionEvent, SessionRecorder, SessionRecorderConfig, SessionStats, TimestampedEvent,
};
use std::error::Error;

/// Null session recorder that does nothing (for testing)
pub struct NullSessionRecorder {
    /// Configuration (stored but not used)
    config: SessionRecorderConfig,
    /// Track if recording is active
    is_recording: bool,
    /// Event counter for minimal stats
    event_count: usize,
}

impl NullSessionRecorder {
    /// Create a new null session recorder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            config: SessionRecorderConfig {
                max_memory_mb: 0,
                compression_threshold_mb: 0,
                sampling_threshold_events_per_sec: 0.0,
                adaptive_sampling: false,
                storage_vs_cpu_preference: crate::session_recorder::TradeoffPreference::Balanced,
                environment_preset: crate::session_recorder::Environment::Testing,
                sampling_rate: 1.0,
            },
            is_recording: false,
            event_count: 0,
        }
    }
}

impl Default for NullSessionRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRecorder for NullSessionRecorder {
    fn start_recording(&mut self, config: SessionRecorderConfig) -> Result<(), Box<dyn Error>> {
        if self.is_recording {
            return Err("Recording is already active".into());
        }
        self.config = config;
        self.is_recording = true;
        self.event_count = 0;
        Ok(())
    }

    fn record_event(&mut self, _event: SessionEvent) -> Result<(), Box<dyn Error>> {
        if !self.is_recording {
            return Err("Recording is not active".into());
        }
        self.event_count += 1;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<SessionStats, Box<dyn Error>> {
        if !self.is_recording {
            return Err("Recording is not active".into());
        }

        self.is_recording = false;

        // Return minimal valid stats
        Ok(SessionStats {
            event_count: self.event_count,
            events_sampled: 0,
            duration: std::time::Duration::ZERO,
            memory_used: 0,
            compression_used: false,
            sampling_rate: self.config.sampling_rate,
        })
    }

    fn should_sample(&self, _event_frequency: f64) -> bool {
        // Always sample in null implementation (but doesn't actually record)
        true
    }

    fn adapt_compression(&mut self, _session_size: usize) {
        // No-op - no compression in null implementation
    }

    fn config(&self) -> &SessionRecorderConfig {
        &self.config
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn save(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // Return empty data
        Ok(Vec::new())
    }

    fn load(&mut self, _data: &[u8]) -> Result<(), Box<dyn Error>> {
        // No-op - accept any data
        Ok(())
    }

    fn get_events(&self) -> &[TimestampedEvent] {
        // Always return empty slice
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_context::SharedExecutionContext;

    #[test]
    fn test_null_session_recorder_lifecycle() {
        let mut recorder = NullSessionRecorder::new();

        // Initially not recording
        assert!(!recorder.is_recording());

        // Start recording
        let config = SessionRecorderConfig::default();
        assert!(recorder.start_recording(config).is_ok());
        assert!(recorder.is_recording());

        // Cannot start when already recording
        assert!(recorder
            .start_recording(SessionRecorderConfig::default())
            .is_err());

        // Record events (no-op)
        let event = SessionEvent::ScriptStart {
            script_path: "test.lua".to_string(),
            context: SharedExecutionContext::new(),
        };
        assert!(recorder.record_event(event).is_ok());

        // Stop recording
        let result = recorder.stop_recording();
        assert!(result.is_ok());
        assert!(!recorder.is_recording());

        let stats = result.unwrap();
        assert_eq!(stats.event_count, 1);
        assert_eq!(stats.memory_used, 0); // No actual memory used

        // Cannot stop when not recording
        assert!(recorder.stop_recording().is_err());
    }

    #[test]
    fn test_null_session_recorder_safe_for_tests() {
        // Verify it's safe to use in test scenarios without side effects
        let mut recorder = NullSessionRecorder::new();
        recorder
            .start_recording(SessionRecorderConfig::default())
            .unwrap();

        // These should all be safe no-ops
        for i in 0..1000 {
            let event = SessionEvent::ScriptStart {
                script_path: format!("test_{i}.lua"),
                context: SharedExecutionContext::new(),
            };
            recorder.record_event(event).unwrap();
        }

        // Should track count but use no memory
        let stats = recorder.stop_recording().unwrap();
        assert_eq!(stats.event_count, 1000);
        assert_eq!(stats.memory_used, 0);
        assert_eq!(stats.events_sampled, 0);

        // Save/load should be no-ops
        let data = recorder.save().unwrap();
        assert!(data.is_empty());

        assert!(recorder.load(&data).is_ok());
        assert!(recorder.get_events().is_empty());
    }

    #[test]
    fn test_null_session_recorder_always_samples() {
        let recorder = NullSessionRecorder::new();

        // Should always return true for sampling (but doesn't actually record)
        assert!(recorder.should_sample(0.0));
        assert!(recorder.should_sample(1000.0));
        assert!(recorder.should_sample(1_000_000.0));
    }

    #[test]
    fn test_null_session_recorder_compression_noop() {
        let mut recorder = NullSessionRecorder::new();

        // Adapt compression should be no-op
        recorder.adapt_compression(0);
        recorder.adapt_compression(1_000_000);
        recorder.adapt_compression(usize::MAX);

        // Should have no effect
        assert_eq!(recorder.config().compression_threshold_mb, 0);
    }
}
