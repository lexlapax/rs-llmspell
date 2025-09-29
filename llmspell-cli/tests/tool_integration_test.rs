//! Integration tests for tool CLI commands

use llmspell_cli::cli::{OutputFormat, ToolCommands};
use llmspell_cli::tool_source::{CapabilityMatcher, LocalToolResolver, ToolResolver, ToolSource};

#[test]
fn test_tool_source_parsing() {
    // Test local source
    let source = ToolSource::parse("local").unwrap();
    assert!(matches!(source, ToolSource::Local));

    // Test empty defaults to local
    let source = ToolSource::parse("").unwrap();
    assert!(matches!(source, ToolSource::Local));

    // Test invalid source
    let result = ToolSource::parse("invalid:source");
    assert!(result.is_err());
}

#[test]
fn test_capability_matcher() {
    use llmspell_cli::tool_source::ToolInfo;

    let tool = ToolInfo {
        name: "calculator".to_string(),
        description: "Mathematical calculations and equations".to_string(),
        category: "utility".to_string(),
        security_level: "safe".to_string(),
        source: ToolSource::Local,
        capabilities: vec!["math".to_string(), "compute".to_string()],
    };

    // Test search term matching
    let matcher = CapabilityMatcher::new().with_search_terms(vec!["calc".to_string()]);
    assert!(matcher.matches(&tool));

    let matcher = CapabilityMatcher::new().with_search_terms(vec!["equation".to_string()]);
    assert!(matcher.matches(&tool));

    let matcher = CapabilityMatcher::new().with_search_terms(vec!["nonexistent".to_string()]);
    assert!(!matcher.matches(&tool));

    // Test category matching
    let matcher = CapabilityMatcher::new().with_categories(vec!["utility".to_string()]);
    assert!(matcher.matches(&tool));

    let matcher = CapabilityMatcher::new().with_categories(vec!["filesystem".to_string()]);
    assert!(!matcher.matches(&tool));

    // Test combined matching
    let matcher = CapabilityMatcher::new()
        .with_search_terms(vec!["math".to_string()])
        .with_categories(vec!["utility".to_string()]);
    assert!(matcher.matches(&tool));
}

#[test]
fn test_local_tool_resolver() {
    let resolver = LocalToolResolver::new();

    // Test listing tools
    let tools = resolver.list().unwrap();
    assert!(!tools.is_empty());
    assert!(tools.contains(&"calculator".to_string()));

    // Test getting tool info
    let info = resolver.info("calculator").unwrap();
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.name, "calculator");

    // Test searching tools
    let matcher = CapabilityMatcher::new().with_search_terms(vec!["calc".to_string()]);
    let results = resolver.search(matcher).unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|t| t.name == "calculator"));
}

#[test]
fn test_tool_commands_enum() {
    // Test list command parsing
    let list_cmd = ToolCommands::List {
        category: Some("utility".to_string()),
        format: Some(OutputFormat::Json),
    };

    match list_cmd {
        ToolCommands::List { category, format } => {
            assert_eq!(category, Some("utility".to_string()));
            assert_eq!(format, Some(OutputFormat::Json));
        }
        _ => panic!("Expected List command"),
    }

    // Test invoke command
    let invoke_cmd = ToolCommands::Invoke {
        name: "calculator".to_string(),
        params: serde_json::json!({"expression": "2+2"}),
        stream: false,
    };

    match invoke_cmd {
        ToolCommands::Invoke {
            name,
            params,
            stream,
        } => {
            assert_eq!(name, "calculator");
            assert_eq!(params["expression"], "2+2");
            assert!(!stream);
        }
        _ => panic!("Expected Invoke command"),
    }
}

#[tokio::test]
async fn test_tool_output_formatting() {
    use llmspell_cli::output::OutputFormatter;

    let tools = vec![
        "calculator".to_string(),
        "file_operations".to_string(),
        "web_scraper".to_string(),
    ];

    // Test text format
    let formatter = OutputFormatter::new(OutputFormat::Text);
    assert!(formatter.print_tool_list(&tools).is_ok());

    // Test JSON format
    let formatter = OutputFormatter::new(OutputFormat::Json);
    assert!(formatter.print_tool_list(&tools).is_ok());

    // Test pretty format
    let formatter = OutputFormatter::new(OutputFormat::Pretty);
    assert!(formatter.print_tool_list(&tools).is_ok());
}

#[test]
fn test_tool_registry() {
    use llmspell_cli::tool_source::ToolResolverRegistry;

    let registry = ToolResolverRegistry::new();

    // Test local resolver access
    let local = registry.local();
    let tools = local.list().unwrap();
    assert!(!tools.is_empty());
}

// Test helper for validating JSON output
#[test]
fn test_tool_json_serialization() {
    use llmspell_cli::tool_source::ToolInfo;

    let tool_info = ToolInfo {
        name: "test_tool".to_string(),
        description: "Test description".to_string(),
        category: "test".to_string(),
        security_level: "safe".to_string(),
        source: ToolSource::Local,
        capabilities: vec!["test1".to_string(), "test2".to_string()],
    };

    // Serialize to JSON
    let json = serde_json::to_string(&tool_info).unwrap();
    assert!(json.contains("test_tool"));
    assert!(json.contains("Test description"));

    // Deserialize back
    let deserialized: ToolInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, tool_info.name);
    assert_eq!(deserialized.description, tool_info.description);
    assert_eq!(deserialized.capabilities.len(), 2);
}
