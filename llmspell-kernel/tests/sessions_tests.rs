//! Integration tests for sessions module

// Include all the session test modules
mod sessions {
    mod access_control_test;
    mod event_correlation_test;
    mod middleware_test;
    mod performance_test;
    mod policy_performance_test;
    mod policy_test;
    mod security_validation_test;

    // Include common module if needed by tests
    mod common;
}