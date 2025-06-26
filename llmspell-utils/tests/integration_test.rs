// ABOUTME: Integration tests for llmspell-utils crate
// ABOUTME: Verifies that all modules are properly exported and accessible

//! Integration tests for the llmspell-utils crate

use llmspell_utils::*;
use std::path::Path;

#[test]
fn test_string_utils_exports() {
    // Test string manipulation functions
    assert_eq!(truncate("Hello, world!", 5), "He...");
    assert_eq!(sanitize("  Hello\x00World  "), "HelloWorld");

    let wrapped = word_wrap("This is a long line", 10);
    assert_eq!(wrapped.len(), 2);

    assert_eq!(to_snake_case("HelloWorld"), "hello_world");
    assert_eq!(to_camel_case("hello_world"), "helloWorld");
    assert_eq!(to_pascal_case("hello_world"), "HelloWorld");

    let indented = indent("Line 1\nLine 2", 2);
    assert!(indented.starts_with("  Line 1"));

    assert!(is_valid_identifier("valid_name"));
    assert!(!is_valid_identifier("123invalid"));

    assert_eq!(normalize_whitespace("Hello    world"), "Hello world");
}

#[test]
fn test_system_info_exports() {
    // Test system info functions
    let info = get_system_info().unwrap();
    assert!(!info.os.is_empty());
    assert!(info.cpu_cores > 0);

    let cpu_count = get_cpu_count();
    assert!(cpu_count > 0);

    let os = OperatingSystem::current();
    let _ = format!("{}", os); // Test Display trait

    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1_048_576), "1.0 MB");
}

#[test]
fn test_error_builders_exports() {
    use llmspell_utils::templates;

    // Test error builder
    let error = ErrorBuilder::new("Test error")
        .with_context("key", "value")
        .build();
    assert_eq!(error.message(), "Test error");
    assert_eq!(error.get_context("key"), Some("value"));

    // Test error templates
    let io_err = templates::io_error("Failed to read", "/tmp/test.txt").build();
    assert_eq!(io_err.get_context("error_type"), Some("io"));

    let val_err = templates::validation_error("Invalid", "test@", "missing domain").build();
    assert_eq!(val_err.get_context("error_type"), Some("validation"));
}

#[test]
fn test_id_generator_exports() {
    // Test ID generation
    let id = generate_component_id("test");
    assert!(id.starts_with("test_"));
    assert!(validate_component_id(&id, Some("test")));

    let short_id = generate_short_id("short");
    assert!(short_id.starts_with("short_"));
    assert!(short_id.len() < 20);

    let det_id1 = generate_deterministic_id(NAMESPACE_AGENT, "my-agent");
    let det_id2 = generate_deterministic_id(NAMESPACE_AGENT, "my-agent");
    assert_eq!(det_id1, det_id2);

    let builder_id = ComponentIdBuilder::new()
        .with_prefix("custom")
        .short()
        .build();
    assert!(builder_id.starts_with("custom_"));
}

#[test]
fn test_serialization_exports() {
    use std::collections::HashMap;

    // Test JSON serialization
    let mut map = HashMap::new();
    map.insert("key", "value");

    let json_str = to_json(&map).unwrap();
    let pretty_json = to_json_pretty(&map).unwrap();
    assert!(json_str.contains("\"key\":\"value\""));
    assert!(pretty_json.contains("{\n"));

    let deserialized: HashMap<String, String> = from_json(&json_str).unwrap();
    assert_eq!(deserialized["key"], "value");

    // Test format conversion
    let yaml = convert_format(&json_str, Format::Json, Format::Yaml).unwrap();
    assert!(yaml.contains("key: value"));

    // Test merge functionality
    let base = json!({"a": 1, "b": 2});
    let other = json!({"b": 3, "c": 4});
    let merged = merge_json(&base, &other);
    assert_eq!(merged["a"], 1);
    assert_eq!(merged["b"], 3);
    assert_eq!(merged["c"], 4);
}

#[test]
fn test_file_utils_exports() {
    // Test file_utils functions
    assert!(is_absolute_path(Path::new("/home")));
    assert!(!is_absolute_path(Path::new("relative")));

    let normalized = normalize_path(Path::new("/home/../home"));
    assert_eq!(normalized, Path::new("/home"));

    let joined = join_paths(&[Path::new("/home"), Path::new("user")]);
    assert_eq!(joined, Path::new("/home/user"));

    let parent = parent_dir(Path::new("/home/user"));
    assert_eq!(parent, Some(Path::new("/home").to_path_buf()));

    // Test expand_path with HOME
    if let Ok(home) = std::env::var("HOME") {
        let expanded = expand_path("~/test").unwrap();
        assert!(expanded.starts_with(&home));
    }
}

#[tokio::test]
async fn test_async_utils_exports() {
    use std::time::Duration;

    // Test timeout
    let result = timeout(Duration::from_millis(100), async { 42 }).await;
    assert_eq!(result.unwrap(), 42);

    // Test timeout with default
    let result = timeout_with_default(
        Duration::from_millis(10),
        async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        },
        0,
    )
    .await;
    assert_eq!(result, 0);

    // Test retry
    let mut count = 0;
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        backoff_factor: 1.0,
        max_delay: Duration::from_millis(10),
        jitter: false,
    };

    let result = retry_async(config, || {
        count += 1;
        async move {
            if count < 2 {
                Err("Not ready")
            } else {
                Ok("Success")
            }
        }
    })
    .await;

    assert_eq!(result.unwrap(), "Success");

    // Test concurrent map
    let numbers = vec![1, 2, 3];
    let results = concurrent_map(numbers.into_iter(), 2, |n| async move { n * 2 }).await;
    assert_eq!(results, vec![2, 4, 6]);
}
