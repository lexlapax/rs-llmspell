//! ABOUTME: Integration tests for TemplateEngineTool
//! ABOUTME: Tests template rendering with both Tera and Handlebars engines

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::{AgentInput, ExecutionContext},
};
use llmspell_tools::TemplateEngineTool;
use serde_json::json;

#[tokio::test]
async fn test_tera_simple_variable_substitution() {
    let tool = TemplateEngineTool::new();

    let params = json!({
        "template": "Hello {{ name }}, welcome to {{ city }}!",
        "context": {
            "name": "Alice",
            "city": "Wonderland"
        },
        "engine": "tera"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert_eq!(result.text, "Hello Alice, welcome to Wonderland!");
    assert_eq!(
        result
            .metadata
            .extra
            .get("engine")
            .unwrap()
            .as_str()
            .unwrap(),
        "tera"
    );
}

#[tokio::test]
async fn test_tera_loops_and_conditions() {
    let tool = TemplateEngineTool::new();

    let params = json!({
        "template": r#"
{% if users %}
Users:
{% for user in users %}
- {{ user.name }} ({{ user.age }})
{% endfor %}
{% else %}
No users found.
{% endif %}
"#,
        "context": {
            "users": [
                {"name": "Alice", "age": 25},
                {"name": "Bob", "age": 30}
            ]
        },
        "engine": "tera"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(result.text.contains("Alice (25)"));
    assert!(result.text.contains("Bob (30)"));
}

#[tokio::test]
async fn test_handlebars_block_helpers() {
    let tool = TemplateEngineTool::new();

    let params = json!({
        "template": r#"
{{#if showGreeting}}
Hello {{name}}!
{{#each items}}
- {{this}}
{{/each}}
{{else}}
Goodbye!
{{/if}}
"#,
        "context": {
            "showGreeting": true,
            "name": "World",
            "items": ["apple", "banana", "cherry"]
        },
        "engine": "handlebars"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(result.text.contains("Hello World!"));
    assert!(result.text.contains("- apple"));
    assert!(result.text.contains("- banana"));
    assert!(result.text.contains("- cherry"));
}

#[tokio::test]
async fn test_handlebars_custom_helpers() {
    let tool = TemplateEngineTool::new();

    let params = json!({
        "template": "{{uppercase name}} - {{lowercase city}}",
        "context": {
            "name": "alice",
            "city": "WONDERLAND"
        },
        "engine": "handlebars"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert_eq!(result.text, "ALICE - wonderland");
}

#[tokio::test]
async fn test_auto_detection() {
    let tool = TemplateEngineTool::new();

    // Test Handlebars detection
    let hbs_params = json!({
        "template": "{{#if condition}}Yes{{/if}}",
        "context": {"condition": true},
        "auto_detect": true
    });

    let input = AgentInput::text("").with_parameter("parameters", hbs_params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    assert_eq!(result.text, "Yes");
    assert_eq!(
        result
            .metadata
            .extra
            .get("engine")
            .unwrap()
            .as_str()
            .unwrap(),
        "handlebars"
    );

    // Test Tera detection
    let tera_params = json!({
        "template": "{% if condition %}Yes{% endif %}",
        "context": {"condition": true},
        "auto_detect": true
    });

    let input = AgentInput::text("").with_parameter("parameters", tera_params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    assert_eq!(result.text, "Yes");
    assert_eq!(
        result
            .metadata
            .extra
            .get("engine")
            .unwrap()
            .as_str()
            .unwrap(),
        "tera"
    );
}

#[tokio::test]
async fn test_html_escaping() {
    let tool = TemplateEngineTool::new();

    // Test with auto-escape enabled (default)
    let params = json!({
        "template": "<div>{{ content }}</div>",
        "context": {
            "content": "<script>alert('XSS')</script>"
        },
        "engine": "tera"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should escape HTML entities
    assert!(result.text.contains("&lt;script&gt;"));
    assert!(!result.text.contains("<script>"));
}

#[tokio::test]
async fn test_complex_data_structures() {
    let tool = TemplateEngineTool::new();

    let params = json!({
        "template": r#"
Company: {{ company.name }}
Employees:
{% for dept, employees in departments %}
  {{ dept }}:
  {% for emp in employees %}
    - {{ emp.name }}: {{ emp.role }}
  {% endfor %}
{% endfor %}
"#,
        "context": {
            "company": {
                "name": "TechCorp"
            },
            "departments": {
                "Engineering": [
                    {"name": "Alice", "role": "Senior Engineer"},
                    {"name": "Bob", "role": "Junior Engineer"}
                ],
                "Sales": [
                    {"name": "Charlie", "role": "Sales Manager"}
                ]
            }
        },
        "engine": "tera"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(result.text.contains("Company: TechCorp"));
    assert!(result.text.contains("Engineering:"));
    assert!(result.text.contains("Alice: Senior Engineer"));
    assert!(result.text.contains("Sales:"));
    assert!(result.text.contains("Charlie: Sales Manager"));
}

#[tokio::test]
async fn test_error_handling() {
    let tool = TemplateEngineTool::new();

    // Test invalid template syntax
    let params = json!({
        "template": "{{ name",  // Unclosed variable
        "context": {"name": "Test"},
        "engine": "tera"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool.execute(input, ExecutionContext::default()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid Tera template"));
}

#[tokio::test]
async fn test_missing_parameters() {
    let tool = TemplateEngineTool::new();

    // Test missing template parameter
    let params = json!({
        "context": {"name": "Test"}
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool.execute(input, ExecutionContext::default()).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing 'template' parameter"));
}

#[tokio::test]
async fn test_tool_schema() {
    let tool = TemplateEngineTool::new();
    let schema = tool.schema();

    assert_eq!(schema.name, "template_engine");
    assert_eq!(schema.parameters.len(), 4);

    // Check required parameters
    let template_param = &schema.parameters[0];
    assert_eq!(template_param.name, "template");
    assert!(template_param.required);

    // Check optional parameters
    let context_param = &schema.parameters[1];
    assert_eq!(context_param.name, "context");
    assert!(!context_param.required);

    let engine_param = &schema.parameters[2];
    assert_eq!(engine_param.name, "engine");
    assert!(!engine_param.required);
    assert_eq!(
        engine_param.default.as_ref().unwrap().as_str().unwrap(),
        "tera"
    );
}

#[tokio::test]
async fn test_metadata_in_output() {
    let tool = TemplateEngineTool::new();

    let template = "Hello {{ name }}!";
    let params = json!({
        "template": template,
        "context": {"name": "World"},
        "engine": "tera"
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check metadata
    assert_eq!(
        result
            .metadata
            .extra
            .get("engine")
            .unwrap()
            .as_str()
            .unwrap(),
        "tera"
    );
    assert_eq!(
        result
            .metadata
            .extra
            .get("template_length")
            .unwrap()
            .as_u64()
            .unwrap(),
        template.len() as u64
    );
}
