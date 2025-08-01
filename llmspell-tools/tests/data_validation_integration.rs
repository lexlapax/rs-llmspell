//! ABOUTME: Integration tests for DataValidationTool
//! ABOUTME: Tests comprehensive validation scenarios with multiple rule types

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_tools::{
    util::{DataValidationConfig, ValidationResult},
    DataValidationTool,
};
use serde_json::{json, Value};

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_simple_required_validation() {
    let tool = DataValidationTool::new();

    // Test with null value
    let params = json!({
        "input": null,
        "rules": {
            "rules": [
                {"type": "required"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
    assert_eq!(validation_result.errors.len(), 1);
    assert!(validation_result.errors[0].message.contains("required"));

    // Test with non-null value
    let params = json!({
        "input": "hello",
        "rules": {
            "rules": [
                {"type": "required"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);
    assert_eq!(validation_result.errors.len(), 0);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_multiple_validation_rules() {
    let tool = DataValidationTool::new();

    let params = json!({
        "input": "ab",
        "rules": {
            "rules": [
                {"type": "required"},
                {"type": "type", "expected": "string"},
                {"type": "length", "min": 3, "max": 10},
                {"type": "pattern", "regex": "^[a-z]+$"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
    assert_eq!(validation_result.errors.len(), 1); // Only length validation should fail
    assert!(validation_result.errors[0].message.contains("at least 3"));
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_numeric_range_validation() {
    let tool = DataValidationTool::new();

    // Test value within range
    let params = json!({
        "input": 25,
        "rules": {
            "rules": [
                {"type": "type", "expected": "number"},
                {"type": "range", "min": 18, "max": 65}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test value outside range
    let params = json!({
        "input": 70,
        "rules": {
            "rules": [
                {"type": "range", "min": 18, "max": 65}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
    assert!(validation_result.errors[0].message.contains("at most 65"));
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_enum_validation() {
    let tool = DataValidationTool::new();

    let params = json!({
        "input": "blue",
        "rules": {
            "rules": [
                {"type": "enum", "values": ["red", "green", "blue"]}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test with invalid value
    let params = json!({
        "input": "yellow",
        "rules": {
            "rules": [
                {"type": "enum", "values": ["red", "green", "blue"]}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
    assert!(validation_result.errors[0]
        .message
        .contains("must be one of"));
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_email_and_url_validation() {
    let tool = DataValidationTool::new();

    // Test valid email
    let params = json!({
        "input": "user@example.com",
        "rules": {
            "rules": [
                {"type": "email"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test valid URL
    let params = json!({
        "input": "https://example.com/path?query=value",
        "rules": {
            "rules": [
                {"type": "url"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_date_validation() {
    let tool = DataValidationTool::new();

    let params = json!({
        "input": "2023-12-25 14:30:00",
        "rules": {
            "rules": [
                {"type": "date", "format": "%Y-%m-%d %H:%M:%S"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test invalid date format
    let params = json!({
        "input": "25/12/2023",
        "rules": {
            "rules": [
                {"type": "date", "format": "%Y-%m-%d"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_array_validation() {
    let tool = DataValidationTool::new();

    let params = json!({
        "input": ["apple", "banana", "cherry"],
        "rules": {
            "rules": [
                {
                    "type": "array",
                    "min_items": 2,
                    "max_items": 5,
                    "unique": true,
                    "item_rules": {
                        "rules": [
                            {"type": "type", "expected": "string"},
                            {"type": "length", "min": 3, "max": 10}
                        ]
                    }
                }
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test with duplicate items
    let params = json!({
        "input": ["apple", "banana", "apple"],
        "rules": {
            "rules": [
                {
                    "type": "array",
                    "unique": true
                }
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
    assert!(validation_result.errors[0].message.contains("unique"));
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_nested_object_validation() {
    let tool = DataValidationTool::new();

    let params = json!({
        "input": {
            "user": {
                "name": "John Doe",
                "email": "john@example.com",
                "age": 30,
                "address": {
                    "street": "123 Main St",
                    "city": "New York",
                    "zipcode": "10001"
                }
            }
        },
        "rules": {
            "rules": [
                {
                    "type": "object",
                    "properties": {
                        "user": {
                            "rules": [
                                {
                                    "type": "object",
                                    "properties": {
                                        "name": {
                                            "rules": [
                                                {"type": "required"},
                                                {"type": "type", "expected": "string"}
                                            ]
                                        },
                                        "email": {
                                            "rules": [
                                                {"type": "required"},
                                                {"type": "email"}
                                            ]
                                        },
                                        "age": {
                                            "rules": [
                                                {"type": "type", "expected": "number"},
                                                {"type": "range", "min": 0, "max": 150}
                                            ]
                                        },
                                        "address": {
                                            "rules": [
                                                {
                                                    "type": "object",
                                                    "properties": {
                                                        "street": {
                                                            "rules": [{"type": "type", "expected": "string"}]
                                                        },
                                                        "city": {
                                                            "rules": [{"type": "type", "expected": "string"}]
                                                        },
                                                        "zipcode": {
                                                            "rules": [
                                                                {"type": "pattern", "regex": "^\\d{5}$"}
                                                            ]
                                                        }
                                                    },
                                                    "required": ["street", "city"],
                                                    "additional_properties": false
                                                }
                                            ]
                                        }
                                    },
                                    "required": ["name", "email"],
                                    "additional_properties": false
                                }
                            ]
                        }
                    },
                    "required": ["user"],
                    "additional_properties": false
                }
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_custom_validators() {
    let tool = DataValidationTool::new();

    // Test phone validator
    let params = json!({
        "input": "+1234567890",
        "rules": {
            "rules": [
                {"type": "custom", "name": "phone"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test UUID validator
    let params = json!({
        "input": "550e8400-e29b-41d4-a716-446655440000",
        "rules": {
            "rules": [
                {"type": "custom", "name": "uuid"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);

    // Test credit card validator
    let params = json!({
        "input": "4532015112830366", // Valid test credit card number
        "rules": {
            "rules": [
                {"type": "custom", "name": "credit_card"}
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(validation_result.valid);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_fail_fast_configuration() {
    let config = DataValidationConfig {
        fail_fast: true,
        ..Default::default()
    };
    let tool = DataValidationTool::with_config(config);

    let params = json!({
        "input": "a", // Too short, wrong pattern
        "rules": {
            "rules": [
                {"type": "length", "min": 3},
                {"type": "pattern", "regex": "^[0-9]+$"},
                {"type": "email"} // Should not be reached due to fail_fast
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);
    assert_eq!(validation_result.errors.len(), 1); // Only first error due to fail_fast
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_tool_metadata() {
    let tool = DataValidationTool::new();
    let schema = tool.schema();

    assert_eq!(schema.name, "data_validation");
    assert_eq!(schema.parameters.len(), 2);

    let data_param = &schema.parameters[0];
    assert_eq!(data_param.name, "input");
    assert!(data_param.required);

    let rules_param = &schema.parameters[1];
    assert_eq!(rules_param.name, "rules");
    assert!(rules_param.required);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_validation_error_details() {
    let tool = DataValidationTool::new();

    let params = json!({
        "input": {
            "email": "invalid-email",
            "age": 200
        },
        "rules": {
            "rules": [
                {
                    "type": "object",
                    "properties": {
                        "name": {
                            "rules": [
                                {"type": "required"},
                                {"type": "type", "expected": "string"},
                                {"type": "length", "min": 1}
                            ]
                        },
                        "email": {
                            "rules": [
                                {"type": "email"}
                            ]
                        },
                        "age": {
                            "rules": [
                                {"type": "range", "max": 150}
                            ]
                        }
                    },
                    "required": ["name"],  // Make name required so we get an error
                    "additional_properties": true
                }
            ]
        }
    });

    let input = AgentInput::text("").with_parameter("parameters", params);
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    let validation_result: ValidationResult =
        serde_json::from_value(output["result"].clone()).unwrap();
    assert!(!validation_result.valid);

    // We should have at least 1 error (missing name field stops further validation)
    assert!(!validation_result.errors.is_empty());
    assert!(validation_result.errors[0]
        .message
        .contains("Missing required field"));

    // Check metadata in the response
    assert_eq!(
        output["metadata"]["error_count"].as_u64().unwrap(),
        validation_result.errors.len() as u64
    );
}
