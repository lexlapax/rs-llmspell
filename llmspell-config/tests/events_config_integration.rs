//! Integration tests for event configuration

use llmspell_config::LLMSpellConfig;
use std::env;

// Helper function to clean up all event environment variables (no longer needed with registry approach)
#[allow(dead_code)]
fn cleanup_event_env_vars() {
    let env_vars = [
        "LLMSPELL_EVENTS_ENABLED",
        "LLMSPELL_EVENTS_BUFFER_SIZE",
        "LLMSPELL_EVENTS_EMIT_TIMING_EVENTS",
        "LLMSPELL_EVENTS_EMIT_STATE_EVENTS",
        "LLMSPELL_EVENTS_EMIT_DEBUG_EVENTS",
        "LLMSPELL_EVENTS_MAX_EVENTS_PER_SECOND",
        "LLMSPELL_EVENTS_FILTERING_INCLUDE_TYPES",
        "LLMSPELL_EVENTS_FILTERING_EXCLUDE_TYPES",
        "LLMSPELL_EVENTS_FILTERING_INCLUDE_COMPONENTS",
        "LLMSPELL_EVENTS_FILTERING_EXCLUDE_COMPONENTS",
        "LLMSPELL_EVENTS_EXPORT_STDOUT",
        "LLMSPELL_EVENTS_EXPORT_FILE",
        "LLMSPELL_EVENTS_EXPORT_WEBHOOK",
        "LLMSPELL_EVENTS_EXPORT_PRETTY_JSON",
    ];

    for var in &env_vars {
        env::remove_var(var);
    }
}

#[test]
fn test_events_config_defaults() {
    let config = LLMSpellConfig::default();
    let events = &config.events;

    assert!(events.enabled);
    assert_eq!(events.buffer_size, 10000);
    assert!(events.emit_timing_events);
    assert!(!events.emit_state_events);
    assert!(!events.emit_debug_events);
    assert_eq!(events.max_events_per_second, None);

    // Test default filtering
    assert_eq!(events.filtering.include_types, vec!["*"]);
    assert!(events.filtering.exclude_types.is_empty());
    assert_eq!(events.filtering.include_components, vec!["*"]);
    assert!(events.filtering.exclude_components.is_empty());

    // Test default export
    assert!(!events.export.stdout);
    assert_eq!(events.export.file, None);
    assert_eq!(events.export.webhook, None);
    assert!(!events.export.pretty_json);
}

#[test]
fn test_events_config_toml_parsing() {
    let toml_content = r#"
[events]
enabled = false
buffer_size = 5000
emit_timing_events = false
emit_state_events = true
emit_debug_events = true
max_events_per_second = 500

[events.filtering]
include_types = ["workflow.*", "agent.*"]
exclude_types = ["debug.*"]
include_components = ["agent-*"]
exclude_components = ["test-*"]

[events.export]
stdout = true
file = "/tmp/events.log"
webhook = "https://example.com/events"
pretty_json = true
"#;

    let config: LLMSpellConfig = toml::from_str(toml_content).expect("Failed to parse TOML");

    let events = &config.events;
    assert!(!events.enabled);
    assert_eq!(events.buffer_size, 5000);
    assert!(!events.emit_timing_events);
    assert!(events.emit_state_events);
    assert!(events.emit_debug_events);
    assert_eq!(events.max_events_per_second, Some(500));

    // Test filtering
    assert_eq!(
        events.filtering.include_types,
        vec!["workflow.*", "agent.*"]
    );
    assert_eq!(events.filtering.exclude_types, vec!["debug.*"]);
    assert_eq!(events.filtering.include_components, vec!["agent-*"]);
    assert_eq!(events.filtering.exclude_components, vec!["test-*"]);

    // Test export
    assert!(events.export.stdout);
    assert_eq!(events.export.file, Some("/tmp/events.log".to_string()));
    assert_eq!(
        events.export.webhook,
        Some("https://example.com/events".to_string())
    );
    assert!(events.export.pretty_json);
}

#[test]
fn test_events_config_minimal_toml() {
    let toml_content = r#"
[events]
enabled = false
"#;

    let config: LLMSpellConfig =
        toml::from_str(toml_content).expect("Failed to parse minimal TOML");

    let events = &config.events;
    assert!(!events.enabled);
    // All other fields should have defaults
    assert_eq!(events.buffer_size, 10000);
    assert!(events.emit_timing_events);
}

#[test]
fn test_events_config_empty_toml() {
    let toml_content = "";

    let config: LLMSpellConfig = toml::from_str(toml_content).expect("Failed to parse empty TOML");

    // Should use all defaults
    let events = &config.events;
    assert!(events.enabled);
    assert_eq!(events.buffer_size, 10000);
}

#[test]
fn test_events_config_env_overrides() {
    use llmspell_config::{env_registry::register_standard_vars, EnvRegistry};
    use std::collections::HashMap;

    // Create isolated registry with test-specific overrides (no real env vars)
    let registry = EnvRegistry::new();
    register_standard_vars(&registry).expect("Failed to register standard vars");

    // Set up test overrides without touching global environment
    let mut overrides = HashMap::new();
    overrides.insert("LLMSPELL_EVENTS_ENABLED".to_string(), "false".to_string());
    registry
        .with_overrides(overrides)
        .expect("Failed to set overrides");

    // Build config directly from registry (bypasses load_from_env entirely)
    let env_config = registry
        .build_config()
        .expect("Failed to build config from registry");

    // Apply to default config
    let mut config = LLMSpellConfig::default();
    config
        .merge_from_json(&env_config)
        .expect("Failed to merge env config");

    // Verify basic override works
    let events = &config.events;
    assert!(
        !events.enabled,
        "LLMSPELL_EVENTS_ENABLED should override to false"
    );
}

#[test]
fn test_events_config_toml_with_env_override() {
    use llmspell_config::{env_registry::register_standard_vars, EnvRegistry};
    use std::collections::HashMap;

    let toml_content = r#"
[events]
enabled = true
buffer_size = 5000
emit_timing_events = true

[events.export]
stdout = false
"#;

    // Parse TOML first without environment overrides
    let mut config: LLMSpellConfig = toml::from_str(toml_content).expect("Failed to parse TOML");

    // Debug output
    println!("After TOML parsing:");
    println!("  enabled: {}", config.events.enabled);
    println!("  buffer_size: {}", config.events.buffer_size);
    println!("  emit_timing_events: {}", config.events.emit_timing_events);
    println!("  export.stdout: {}", config.events.export.stdout);

    // Verify TOML values were parsed correctly - start with basic check
    assert!(config.events.enabled); // From TOML
    println!(
        "Buffer size assertion: {} == 5000",
        config.events.buffer_size
    );

    // Create isolated registry with test-specific overrides (no real env vars)
    let registry = EnvRegistry::new();
    register_standard_vars(&registry).expect("Failed to register standard vars");

    // Set up test overrides for enabled field only
    let mut overrides = HashMap::new();
    overrides.insert("LLMSPELL_EVENTS_ENABLED".to_string(), "false".to_string());
    registry
        .with_overrides(overrides)
        .expect("Failed to set overrides");

    // Build config from registry and merge
    let env_config = registry
        .build_config()
        .expect("Failed to build config from registry");
    config
        .merge_from_json(&env_config)
        .expect("Failed to merge env config");

    // Environment should override TOML for enabled field, others unchanged
    assert!(!config.events.enabled); // Overridden by env
    assert_eq!(config.events.buffer_size, 5000); // From TOML (unchanged)
    assert!(config.events.emit_timing_events); // From TOML (unchanged)
    assert!(!config.events.export.stdout); // From TOML (unchanged)
}

#[test]
fn test_events_config_validation_integration() {
    // Test valid configuration passes validation
    let valid_toml = r#"
[events]
enabled = true
buffer_size = 10000
max_events_per_second = 1000

[events.export]
stdout = true
"#;

    let config = LLMSpellConfig::from_toml(valid_toml).expect("Failed to parse valid TOML");
    assert!(config.validate().is_ok());

    // Test invalid configuration fails validation
    let invalid_toml = r#"
[events]
enabled = true
buffer_size = 0

[events.export]
webhook = "invalid-url"
"#;

    let result = LLMSpellConfig::from_toml(invalid_toml);
    assert!(result.is_err());
}

#[test]
fn test_events_config_builder_pattern() {
    // Test that we can use the builder pattern to create a config
    let config = LLMSpellConfig::builder().default_engine("lua").build();

    // Events should have default values
    let events = &config.events;
    assert!(events.enabled);
    assert_eq!(events.buffer_size, 10000);
    assert!(events.emit_timing_events);
    assert!(!events.emit_state_events);
    assert!(!events.emit_debug_events);
    assert_eq!(events.max_events_per_second, None);

    // Test validation passes
    assert!(config.validate().is_ok());
}
