//! Common test utilities for llmspell-bridge tests
//!
//! This module provides shared test helpers and utilities that are used across
//! multiple test files to ensure consistent test setup and avoid code duplication.

#![allow(dead_code)]

use llmspell_bridge::{
    circuit_breaker::CircuitBreaker, diagnostics_bridge::DiagnosticsBridge,
    hook_profiler::HookProfiler, null_circuit_breaker::NullCircuitBreaker,
    null_hook_profiler::NullHookProfiler, null_profiler::NullProfiler,
    null_session_recorder::NullSessionRecorder, profiler::Profiler,
    session_recorder::SessionRecorder,
};

/// Create a test-safe `DiagnosticsBridge` with null implementations
///
/// This helper creates a `DiagnosticsBridge` with null implementations for all
/// injectable components to avoid side effects during testing (no file I/O,
/// no signal handlers, no real profiling).
///
/// # Example
/// ```rust
/// use common::create_test_bridge;
///
/// let bridge = create_test_bridge();
/// // Use bridge in tests without side effects
/// ```
#[must_use]
pub fn create_test_bridge() -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .hook_profiler(Box::new(NullHookProfiler::new()))
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(Box::new(NullSessionRecorder::new()))
        .build()
}

/// Create a test bridge with custom profiler
#[must_use]
pub fn create_test_bridge_with_profiler(profiler: Box<dyn Profiler>) -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(profiler)
        .hook_profiler(Box::new(NullHookProfiler::new()))
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(Box::new(NullSessionRecorder::new()))
        .build()
}

/// Create a test bridge with custom hook profiler
#[must_use]
pub fn create_test_bridge_with_hook_profiler(
    hook_profiler: Box<dyn HookProfiler>,
) -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .hook_profiler(hook_profiler)
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(Box::new(NullSessionRecorder::new()))
        .build()
}

/// Create a test bridge with custom circuit breaker
#[must_use]
pub fn create_test_bridge_with_circuit_breaker(
    circuit_breaker: Box<dyn CircuitBreaker>,
) -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .hook_profiler(Box::new(NullHookProfiler::new()))
        .circuit_breaker(circuit_breaker)
        .session_recorder(Box::new(NullSessionRecorder::new()))
        .build()
}

/// Create a test bridge with custom session recorder
#[must_use]
pub fn create_test_bridge_with_session_recorder(
    session_recorder: Box<dyn SessionRecorder>,
) -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .hook_profiler(Box::new(NullHookProfiler::new()))
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(session_recorder)
        .build()
}
