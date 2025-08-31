//! Comprehensive validation system tests
//!
//! Tests the comprehensive script validation system integration with `DiagnosticsBridge`

use llmspell_bridge::condition_evaluator::{CompiledCondition, ConditionEvaluator, DebugContext};
use llmspell_bridge::diagnostics_bridge::{DiagnosticsBridge, ValidationReport};
use llmspell_bridge::execution_context::SharedExecutionContext;
use llmspell_bridge::variable_inspector::{ContextBatcher, VariableInspector};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

// Mock ConditionEvaluator for testing
struct MockConditionEvaluator {
    should_fail: bool,
}

impl ConditionEvaluator for MockConditionEvaluator {
    fn compile_condition(&self, expression: &str) -> Result<CompiledCondition, Box<dyn Error>> {
        if self.should_fail || expression.contains("syntax_error") {
            Err("Compilation failed".into())
        } else {
            Ok(CompiledCondition {
                expression: expression.to_string(),
                compiled_data: Some(b"test".to_vec()),
                metadata: HashMap::new(),
            })
        }
    }

    fn evaluate_condition(
        &self,
        _expression: &str,
        _compiled: Option<&CompiledCondition>,
        _context: &dyn DebugContext,
    ) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }
}

// Mock VariableInspector for testing
struct MockVariableInspector {
    api_issues: Vec<String>,
}

impl VariableInspector for MockVariableInspector {
    fn inspect_variables(
        &self,
        _variable_names: &[String],
        _batcher: &mut ContextBatcher,
    ) -> HashMap<String, JsonValue> {
        HashMap::new()
    }

    fn watch_variable(&self, _name: String, _batcher: &mut ContextBatcher) {}

    fn unwatch_variable(&self, _name: &str, _batcher: &mut ContextBatcher) {}

    fn get_all_cached_variables(&self) -> Vec<llmspell_bridge::debug_state_cache::CachedVariable> {
        Vec::new()
    }

    fn invalidate_cache(&self) {}

    fn process_context_updates(
        &self,
        _updates: Vec<llmspell_bridge::variable_inspector::ContextUpdate>,
    ) {
    }

    fn validate_api_usage(
        &self,
        _script: &str,
        _context: &SharedExecutionContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.api_issues.clone())
    }
}

#[tokio::test]
async fn test_comprehensive_validation_success() {
    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: false }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues: vec![] }));

    let mut context = SharedExecutionContext::new();
    let script = "print('Hello, world!')";

    let result = bridge.validate_script_comprehensive(script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.is_valid);
    assert_eq!(report.errors.len(), 0);
    assert!(report.validation_duration_us > 0);
}

#[tokio::test]
async fn test_comprehensive_validation_syntax_error() {
    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: true }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues: vec![] }));

    let mut context = SharedExecutionContext::new();
    let script = "print('Hello, syntax_error')";

    let result = bridge.validate_script_comprehensive(script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(!report.is_valid);
    assert!(!report.errors.is_empty());
    assert!(report.errors[0].message.contains("Compilation failed"));
}

#[tokio::test]
async fn test_comprehensive_validation_api_warnings() {
    let api_issues = vec![
        "Dangerous function 'os.execute' detected".to_string(),
        "Global variable assignment detected".to_string(),
    ];

    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: false }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues }));

    let mut context = SharedExecutionContext::new();
    let script = "os.execute('ls')";

    let result = bridge.validate_script_comprehensive(script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.is_valid); // No syntax errors
    assert_eq!(report.warnings.len(), 2);
    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("os.execute")));
}

#[tokio::test]
async fn test_comprehensive_validation_performance_warnings() {
    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: false }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues: vec![] }));

    let mut context = SharedExecutionContext::new();

    // Set high execution count to trigger performance warning
    context.performance_metrics.execution_count = 15000;
    context.performance_metrics.memory_allocated = 150_000_000; // 150MB

    let script = "print('test')";

    let result = bridge.validate_script_comprehensive(script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.is_valid);
    assert!(report.warnings.len() >= 2); // execution count + memory warnings
    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("execution count")));
    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("memory allocation")));
}

#[tokio::test]
async fn test_comprehensive_validation_security_patterns() {
    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: false }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues: vec![] }));

    let mut context = SharedExecutionContext::new();

    // Script with multiple security issues
    let script = r#"
        eval('malicious code')
        password="secret123"
        SELECT * FROM users WHERE id = $id
        <script>alert('xss')</script>
        ../../../etc/passwd
    "#;

    let result = bridge.validate_script_comprehensive(script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.warnings.len() >= 4); // Multiple security warnings

    // Check for specific security warnings
    let warning_messages: Vec<&str> = report.warnings.iter().map(|w| w.message.as_str()).collect();
    assert!(warning_messages.iter().any(|msg| msg.contains("eval")));
    assert!(warning_messages
        .iter()
        .any(|msg| msg.contains("credential")));
    assert!(warning_messages.iter().any(|msg| msg.contains("XSS")));
    assert!(warning_messages.iter().any(|msg| msg.contains("traversal")));
}

#[tokio::test]
async fn test_comprehensive_validation_suggestions() {
    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: false }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues: vec![] }));

    let mut context = SharedExecutionContext::new();

    // Large script to trigger suggestion
    let large_script = (0..1500)
        .map(|i| format!("print('line {i}')"))
        .collect::<Vec<_>>()
        .join("\n");

    let result = bridge.validate_script_comprehensive(&large_script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(!report.suggestions.is_empty());
    assert!(report.suggestions[0]
        .message
        .contains("Large script detected"));
    assert!(report.suggestions[0].suggestion.is_some());
}

#[tokio::test]
async fn test_comprehensive_validation_empty_script() {
    let bridge = DiagnosticsBridge::new()
        .with_condition_evaluator(Arc::new(MockConditionEvaluator { should_fail: false }))
        .with_variable_inspector(Arc::new(MockVariableInspector { api_issues: vec![] }));

    let mut context = SharedExecutionContext::new();
    let script = "";

    let result = bridge.validate_script_comprehensive(script, &mut context);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(!report.is_valid);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].message, "Script content is empty");
}

#[tokio::test]
async fn test_validation_report_methods() {
    let mut report = ValidationReport::new();

    assert!(report.is_valid);
    assert_eq!(report.total_issues(), 0);

    report.add_error("Test error".to_string(), None);
    assert!(!report.is_valid);
    assert_eq!(report.total_issues(), 1);

    report.add_warning("security".to_string(), "Test warning".to_string(), None);
    assert_eq!(report.total_issues(), 2);

    report.add_suggestion(
        "performance".to_string(),
        "Test suggestion".to_string(),
        "Fix it".to_string(),
    );
    assert_eq!(report.total_issues(), 3);

    report.set_duration(1000);
    assert_eq!(report.validation_duration_us, 1000);
}

#[tokio::test]
async fn test_legacy_validate_script_compatibility() {
    let bridge = DiagnosticsBridge::new();

    // Test empty script
    let result = bridge.validate_script("", "lua");
    assert!(!result.valid);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].message, "Script content is empty");

    // Test valid script
    let result = bridge.validate_script("print('hello')", "lua");
    assert!(result.valid);
    assert_eq!(result.errors.len(), 0);

    // Test oversized script
    let large_script = "a".repeat(2_000_000);
    let result = bridge.validate_script(&large_script, "lua");
    assert!(!result.valid);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].message, "Script content too large (>1MB)");
}
