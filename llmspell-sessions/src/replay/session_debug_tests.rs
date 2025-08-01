//! ABOUTME: Tests for session debugging functionality
//! ABOUTME: Validates state inspection, timeline navigation, comparison, and error analysis

// Tests are included directly in the parent module
use super::{ErrorAnalysis, SessionDebugData, SessionDebugger, SessionState, StateComparison};
use crate::SessionId;
use llmspell_hooks::persistence::{
    CapturedState, ReplayError, ReplayErrorType, ReplaySession, TimelineEntry,
};
use llmspell_hooks::result::HookResult;
use llmspell_state_persistence::manager::SerializedHookExecution;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

fn create_test_captured_state(hook_id: &str, execution_id: Uuid) -> CapturedState {
    CapturedState {
        timestamp: SystemTime::now(),
        execution_id,
        hook_id: hook_id.to_string(),
        context_snapshot: json!({
            "test": "data",
            "hook": hook_id,
            "nested": {
                "field": "value"
            }
        }),
        result: HookResult::Continue,
        metadata: HashMap::from([("key".to_string(), "value".to_string())]),
    }
}

fn create_test_replay_session(_session_id: SessionId) -> ReplaySession {
    let mut captured_states = VecDeque::new();
    for i in 0..5 {
        captured_states.push_back(create_test_captured_state(
            &format!("hook_{}", i),
            Uuid::new_v4(),
        ));
    }

    let mut errors = Vec::new();
    errors.push(ReplayError {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "hook_1".to_string(),
        error_message: "Test error 1".to_string(),
        error_type: ReplayErrorType::ExecutionError,
    });
    errors.push(ReplayError {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "hook_2".to_string(),
        error_message: "Test error 2".to_string(),
        error_type: ReplayErrorType::ValidationError,
    });

    ReplaySession {
        config: llmspell_hooks::persistence::ReplaySessionConfig::default(),
        executions_replayed: 5,
        breakpoints: Vec::new(),
        captured_states,
        errors_encountered: errors,
        start_time: SystemTime::now(),
    }
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_session_debugger_creation() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    // Initially no data
    assert!(debugger.get_all_states(&session_id).is_err());
    assert!(debugger.get_timeline(&session_id).is_none());
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_import_replay_session() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();
    let replay_session = create_test_replay_session(session_id);

    // Import the replay session
    debugger.import_replay_session(session_id, &replay_session);

    // Verify states were imported
    let states = debugger.get_all_states(&session_id).unwrap();
    assert_eq!(states.len(), 5);

    // Verify errors were imported
    let error_analysis = debugger.analyze_errors(&session_id);
    assert_eq!(error_analysis.total_errors, 2);
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_inspect_state_at_timestamp() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    // Create states with specific timestamps
    let base_time = SystemTime::now() - Duration::from_secs(100);
    let mut states = VecDeque::new();

    for i in 0..5 {
        let mut state = create_test_captured_state(&format!("hook_{}", i), Uuid::new_v4());
        state.timestamp = base_time + Duration::from_secs(i * 10);
        states.push_back(state);
    }

    // Import states
    {
        let mut captured_states = debugger.captured_states.write().unwrap();
        captured_states.insert(session_id, states);
    }

    // Test inspection at various points
    // Before first state
    let result = debugger
        .inspect_state_at(&session_id, base_time - Duration::from_secs(10))
        .unwrap();
    assert!(result.is_none());

    // Between states (should get the previous state)
    let result = debugger
        .inspect_state_at(&session_id, base_time + Duration::from_secs(15))
        .unwrap();
    assert!(result.is_some());
    let state = result.unwrap();
    assert_eq!(state.hook_id, "hook_1");

    // After last state
    let result = debugger
        .inspect_state_at(&session_id, base_time + Duration::from_secs(100))
        .unwrap();
    assert!(result.is_some());
    let state = result.unwrap();
    assert_eq!(state.hook_id, "hook_4");
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_state_comparison() {
    let state1 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test_hook".to_string(),
        context: json!({
            "field1": "value1",
            "field2": 42,
            "field3": {
                "nested": "data"
            }
        }),
        result: "Continue".to_string(),
        metadata: HashMap::from([("meta1".to_string(), "value1".to_string())]),
    };

    let state2 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test_hook".to_string(),
        context: json!({
            "field1": "value1",
            "field2": 43,  // Changed
            "field3": {
                "nested": "modified"  // Changed
            },
            "field4": "new"  // Added
        }),
        result: "Modified".to_string(), // Changed
        metadata: HashMap::from([
            ("meta1".to_string(), "value1".to_string()),
            ("meta2".to_string(), "value2".to_string()), // Added
        ]),
    };

    let comparison = StateComparison::compare(&state1, &state2);

    // Check context differences
    assert_eq!(comparison.context_diffs.len(), 3); // field2 changed, field3.nested changed, field4 added

    // Check result differences
    assert_eq!(comparison.result_diffs.len(), 1);

    // Check metadata differences
    assert_eq!(comparison.metadata_diffs.len(), 1); // meta2 added

    // Verify summary
    assert!(comparison.summary.contains("3 context differences"));
    assert!(comparison.summary.contains("1 result differences"));
    assert!(comparison.summary.contains("1 metadata differences"));
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_compare_states_at_different_times() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    // Create two states with different data
    let time1 = SystemTime::now() - Duration::from_secs(60);
    let time2 = SystemTime::now() - Duration::from_secs(30);

    let mut states = VecDeque::new();

    let mut state1 = create_test_captured_state("hook_1", Uuid::new_v4());
    state1.timestamp = time1;
    state1.context_snapshot = json!({"value": 1});
    states.push_back(state1);

    let mut state2 = create_test_captured_state("hook_2", Uuid::new_v4());
    state2.timestamp = time2;
    state2.context_snapshot = json!({"value": 2});
    states.push_back(state2);

    // Import states
    {
        let mut captured_states = debugger.captured_states.write().unwrap();
        captured_states.insert(session_id, states);
    }

    // Compare states
    let comparison = debugger.compare_states(&session_id, time1, time2).unwrap();
    assert!(!comparison.context_diffs.is_empty());
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_timeline_navigation() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    // Create timeline entries
    let mut executions = Vec::new();
    for i in 0..5 {
        executions.push(SerializedHookExecution {
            hook_id: format!("hook_{}", i),
            execution_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![],
            result: "Continue".to_string(),
            timestamp: SystemTime::now() + Duration::from_secs(i * 10),
            duration: Duration::from_millis(100),
            metadata: HashMap::new(),
        });
    }

    // Import timeline
    debugger.update_timeline(session_id, executions);

    // Verify timeline
    let timeline = debugger.get_timeline(&session_id).unwrap();
    assert_eq!(timeline.len(), 5);

    // Create corresponding states
    let mut states = VecDeque::new();
    for entry in &timeline {
        let mut state = create_test_captured_state(&entry.hook_id, entry.execution_id);
        state.timestamp = entry.timestamp;
        states.push_back(state);
    }

    {
        let mut captured_states = debugger.captured_states.write().unwrap();
        captured_states.insert(session_id, states);
    }

    // Navigate to specific point
    let state = debugger.navigate_to_timeline_point(&session_id, 2).unwrap();
    assert_eq!(state.hook_id, "hook_2");

    // Invalid index
    assert!(debugger
        .navigate_to_timeline_point(&session_id, 10)
        .is_err());
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_error_analysis() {
    let errors = vec![
        ReplayError {
            timestamp: SystemTime::now(),
            execution_id: Uuid::new_v4(),
            hook_id: "hook_1".to_string(),
            error_message: "Execution failed".to_string(),
            error_type: ReplayErrorType::ExecutionError,
        },
        ReplayError {
            timestamp: SystemTime::now(),
            execution_id: Uuid::new_v4(),
            hook_id: "hook_1".to_string(),
            error_message: "Another execution failure".to_string(),
            error_type: ReplayErrorType::ExecutionError,
        },
        ReplayError {
            timestamp: SystemTime::now(),
            execution_id: Uuid::new_v4(),
            hook_id: "hook_2".to_string(),
            error_message: "Validation failed".to_string(),
            error_type: ReplayErrorType::ValidationError,
        },
        ReplayError {
            timestamp: SystemTime::now(),
            execution_id: Uuid::new_v4(),
            hook_id: "hook_3".to_string(),
            error_message: "Timeout".to_string(),
            error_type: ReplayErrorType::TimeoutError,
        },
    ];

    let analysis = ErrorAnalysis::from_errors(&errors);

    assert_eq!(analysis.total_errors, 4);
    assert_eq!(analysis.errors_by_type.get("ExecutionError").unwrap(), &2);
    assert_eq!(analysis.errors_by_type.get("ValidationError").unwrap(), &1);
    assert_eq!(analysis.errors_by_type.get("TimeoutError").unwrap(), &1);
    assert_eq!(analysis.errors_by_hook.get("hook_1").unwrap(), &2);
    assert_eq!(analysis.errors_by_hook.get("hook_2").unwrap(), &1);
    assert_eq!(
        analysis.most_common_error,
        Some(("ExecutionError".to_string(), 2))
    );
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_error_rate_calculation() {
    let base_time = SystemTime::now() - Duration::from_secs(3600); // 1 hour ago

    let errors = vec![
        ReplayError {
            timestamp: base_time,
            execution_id: Uuid::new_v4(),
            hook_id: "hook_1".to_string(),
            error_message: "Error 1".to_string(),
            error_type: ReplayErrorType::ExecutionError,
        },
        ReplayError {
            timestamp: base_time + Duration::from_secs(1800), // 30 minutes later
            execution_id: Uuid::new_v4(),
            hook_id: "hook_2".to_string(),
            error_message: "Error 2".to_string(),
            error_type: ReplayErrorType::ExecutionError,
        },
        ReplayError {
            timestamp: base_time + Duration::from_secs(3600), // 1 hour later
            execution_id: Uuid::new_v4(),
            hook_id: "hook_3".to_string(),
            error_message: "Error 3".to_string(),
            error_type: ReplayErrorType::ExecutionError,
        },
    ];

    let analysis = ErrorAnalysis::from_errors(&errors);

    // 3 errors over 1 hour = 3 errors/hour
    assert!(analysis.error_rate.is_some());
    let rate = analysis.error_rate.unwrap();
    assert!((rate - 3.0).abs() < 0.1); // Allow small floating point difference
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_export_debug_data() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();
    let replay_session = create_test_replay_session(session_id);

    // Import session data
    debugger.import_replay_session(session_id, &replay_session);

    // Add timeline
    let executions = vec![SerializedHookExecution {
        hook_id: "hook_1".to_string(),
        execution_id: Uuid::new_v4(),
        correlation_id: Uuid::new_v4(),
        hook_context: vec![],
        result: "Continue".to_string(),
        timestamp: SystemTime::now(),
        duration: Duration::from_millis(100),
        metadata: HashMap::new(),
    }];
    debugger.update_timeline(session_id, executions);

    // Export debug data
    let debug_data = debugger.export_debug_data(&session_id).unwrap();

    assert_eq!(debug_data.session_id, session_id);
    assert_eq!(debug_data.captured_states.len(), 5);
    assert!(debug_data.timeline.is_some());
    assert_eq!(debug_data.timeline.unwrap().len(), 1);
    assert_eq!(debug_data.error_analysis.total_errors, 2);
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_clear_session_data() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();
    let replay_session = create_test_replay_session(session_id);

    // Import data
    debugger.import_replay_session(session_id, &replay_session);

    // Verify data exists
    assert!(debugger.get_all_states(&session_id).is_ok());
    let error_analysis = debugger.analyze_errors(&session_id);
    assert_eq!(error_analysis.total_errors, 2);

    // Clear data
    debugger.clear_session_data(&session_id);

    // Verify data is gone
    assert!(debugger.get_all_states(&session_id).is_err());
    let error_analysis = debugger.analyze_errors(&session_id);
    assert_eq!(error_analysis.total_errors, 0);
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_add_error() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    // Initially no errors
    let analysis = debugger.analyze_errors(&session_id);
    assert_eq!(analysis.total_errors, 0);

    // Add an error
    let error = ReplayError {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test_hook".to_string(),
        error_message: "Test error".to_string(),
        error_type: ReplayErrorType::ExecutionError,
    };
    debugger.add_error(session_id, error);

    // Verify error was added
    let analysis = debugger.analyze_errors(&session_id);
    assert_eq!(analysis.total_errors, 1);
    assert_eq!(analysis.errors_by_hook.get("test_hook").unwrap(), &1);
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_state_comparison_with_arrays() {
    let state1 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test_hook".to_string(),
        context: json!({
            "array": [1, 2, 3],
            "nested": {
                "items": ["a", "b", "c"]
            }
        }),
        result: "Continue".to_string(),
        metadata: HashMap::new(),
    };

    let state2 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test_hook".to_string(),
        context: json!({
            "array": [1, 2, 3, 4],  // Added element
            "nested": {
                "items": ["a", "b", "d"]  // Changed element
            }
        }),
        result: "Continue".to_string(),
        metadata: HashMap::new(),
    };

    let comparison = StateComparison::compare(&state1, &state2);
    assert!(!comparison.context_diffs.is_empty());
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_hook_result_comparison() {
    let debugger = SessionDebugger::new();

    // Test identical results
    let result1 = HookResult::Continue;
    let result2 = HookResult::Continue;
    let comparison = debugger.compare_hook_results(&result1, &result2);
    assert!(comparison.identical);

    // Test different results
    let result1 = HookResult::Continue;
    let result2 = HookResult::Modified(json!({"changed": true}));
    let comparison = debugger.compare_hook_results(&result1, &result2);
    assert!(!comparison.identical);
    assert!(comparison.difference_type.is_some());
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_session_state_from_captured() {
    let captured = CapturedState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test_hook".to_string(),
        context_snapshot: json!({
            "test": "data"
        }),
        result: HookResult::Modified(json!({"modified": true})),
        metadata: HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]),
    };

    let session_state = SessionState::from_captured(&captured);

    assert_eq!(session_state.timestamp, captured.timestamp);
    assert_eq!(session_state.execution_id, captured.execution_id);
    assert_eq!(session_state.hook_id, captured.hook_id);
    assert_eq!(session_state.context, captured.context_snapshot);
    assert!(session_state.result.contains("Modified"));
    assert_eq!(session_state.metadata, captured.metadata);
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_timeline_entry_conversion() {
    let executions = vec![
        SerializedHookExecution {
            hook_id: "hook_1".to_string(),
            execution_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![],
            result: "Continue".to_string(),
            timestamp: SystemTime::now(),
            duration: Duration::from_millis(100),
            metadata: HashMap::from([("meta".to_string(), json!("value"))]),
        },
        SerializedHookExecution {
            hook_id: "hook_2".to_string(),
            execution_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![],
            result: "Modified".to_string(),
            timestamp: SystemTime::now() + Duration::from_secs(1),
            duration: Duration::from_millis(200),
            metadata: HashMap::new(),
        },
    ];

    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    debugger.update_timeline(session_id, executions);

    let timeline = debugger.get_timeline(&session_id).unwrap();
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].hook_id, "hook_1");
    assert_eq!(timeline[0].duration, Duration::from_millis(100));
    assert_eq!(timeline[1].hook_id, "hook_2");
    assert_eq!(timeline[1].duration, Duration::from_millis(200));
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_state_inspection_empty_session() {
    let debugger = SessionDebugger::new();
    let session_id = SessionId::new();

    // No states exist
    let result = debugger.inspect_state_at(&session_id, SystemTime::now());
    assert!(result.is_err());
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_metadata_comparison() {
    let state1 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test".to_string(),
        context: json!({}),
        result: "Continue".to_string(),
        metadata: HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]),
    };

    let state2 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test".to_string(),
        context: json!({}),
        result: "Continue".to_string(),
        metadata: HashMap::from([
            ("key1".to_string(), "value1_modified".to_string()), // Changed
            ("key3".to_string(), "value3".to_string()),          // Added
                                                                 // key2 removed
        ]),
    };

    let comparison = StateComparison::compare(&state1, &state2);
    assert_eq!(comparison.metadata_diffs.len(), 3); // 1 changed, 1 added, 1 removed
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_debug_data_serialization() {
    let debug_data = SessionDebugData {
        session_id: SessionId::new(),
        captured_states: vec![SessionState {
            timestamp: SystemTime::now(),
            execution_id: Uuid::new_v4(),
            hook_id: "test".to_string(),
            context: json!({"test": "data"}),
            result: "Continue".to_string(),
            metadata: HashMap::new(),
        }],
        timeline: Some(vec![TimelineEntry {
            timestamp: SystemTime::now(),
            relative_time: Duration::from_secs(0),
            execution_id: Uuid::new_v4(),
            hook_id: "test".to_string(),
            hook_point: llmspell_hooks::HookPoint::BeforeAgentExecution,
            component_id: "session".to_string(),
            duration: Duration::from_millis(100),
            result_type: "Continue".to_string(),
        }]),
        error_analysis: ErrorAnalysis {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            errors_by_hook: HashMap::new(),
            error_timeline: vec![],
            most_common_error: None,
            error_rate: None,
        },
        export_time: SystemTime::now(),
    };

    // Verify it can be serialized
    let serialized = serde_json::to_string(&debug_data).unwrap();
    assert!(serialized.contains("session_id"));
    assert!(serialized.contains("captured_states"));
    assert!(serialized.contains("timeline"));
    assert!(serialized.contains("error_analysis"));
}

#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "session")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_comparison_with_nested_objects() {
    let state1 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test".to_string(),
        context: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep"
                    }
                }
            }
        }),
        result: "Continue".to_string(),
        metadata: HashMap::new(),
    };

    let state2 = SessionState {
        timestamp: SystemTime::now(),
        execution_id: Uuid::new_v4(),
        hook_id: "test".to_string(),
        context: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": "modified"  // Changed deep value
                    }
                }
            }
        }),
        result: "Continue".to_string(),
        metadata: HashMap::new(),
    };

    let comparison = StateComparison::compare(&state1, &state2);
    assert_eq!(comparison.context_diffs.len(), 1);
    assert_eq!(
        comparison.context_diffs[0].path,
        "level1.level2.level3.value"
    );
}
