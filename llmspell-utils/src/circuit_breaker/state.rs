//! ABOUTME: Circuit breaker states and transitions
//! ABOUTME: Defines the three states: Closed (normal), Open (rejecting), and `HalfOpen` (testing)

use std::fmt;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CircuitState {
    /// Circuit is closed - normal operation, requests are allowed
    #[default]
    Closed,
    /// Circuit is open - requests are rejected due to failures
    Open,
    /// Circuit is half-open - limited requests allowed for testing
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

impl CircuitState {
    /// Check if the circuit allows requests
    #[must_use]
    pub fn allows_requests(&self) -> bool {
        matches!(self, CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Check if the circuit is protecting (rejecting most requests)
    #[must_use]
    pub fn is_protecting(&self) -> bool {
        matches!(self, CircuitState::Open)
    }
}

/// State transition event for tracking
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Previous state
    pub from: CircuitState,
    /// New state
    pub to: CircuitState,
    /// Reason for transition
    pub reason: String,
    /// Timestamp of transition
    pub timestamp: std::time::Instant,
}

impl StateTransition {
    /// Create a new state transition
    #[must_use]
    pub fn new(from: CircuitState, to: CircuitState, reason: String) -> Self {
        Self {
            from,
            to,
            reason,
            timestamp: std::time::Instant::now(),
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_state_allows_requests() {
        assert!(CircuitState::Closed.allows_requests());
        assert!(!CircuitState::Open.allows_requests());
        assert!(CircuitState::HalfOpen.allows_requests());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_state_is_protecting() {
        assert!(!CircuitState::Closed.is_protecting());
        assert!(CircuitState::Open.is_protecting());
        assert!(!CircuitState::HalfOpen.is_protecting());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_state_display() {
        assert_eq!(CircuitState::Closed.to_string(), "Closed");
        assert_eq!(CircuitState::Open.to_string(), "Open");
        assert_eq!(CircuitState::HalfOpen.to_string(), "HalfOpen");
    }
}
