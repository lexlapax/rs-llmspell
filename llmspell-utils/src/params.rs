// ABOUTME: Parameter extraction and validation helpers for consistent tool implementation
// ABOUTME: Provides utilities to extract parameters from AgentInput and validate them

//! Parameter extraction and validation utilities
//!
//! This module provides consistent parameter handling across all LLMSpell tools,
//! reducing boilerplate and ensuring uniform error messages.

use llmspell_core::types::AgentInput;
use llmspell_core::{LLMSpellError, Result as LLMResult};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// Extract the parameters object from an `AgentInput`
///
/// # Examples
/// ```rust,ignore
/// let params = extract_parameters(&input)?;
/// let operation = extract_required_string(params, "operation")?;
/// ```
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameters object is missing
pub fn extract_parameters(input: &AgentInput) -> LLMResult<&Value> {
    input
        .parameters
        .get("parameters")
        .ok_or_else(|| LLMSpellError::Validation {
            message: "Missing parameters object".to_string(),
            field: Some("parameters".to_string()),
        })
}

/// Extract a required string parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not a string
pub fn extract_required_string<'a>(params: &'a Value, key: &str) -> LLMResult<&'a str> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional string parameter
#[must_use]
pub fn extract_optional_string<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(Value::as_str)
}

/// Extract a required string parameter with a default value
#[must_use]
pub fn extract_string_with_default<'a>(params: &'a Value, key: &str, default: &'a str) -> &'a str {
    extract_optional_string(params, key).unwrap_or(default)
}

/// Extract a required boolean parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not a boolean
pub fn extract_required_bool(params: &Value, key: &str) -> LLMResult<bool> {
    params
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required boolean parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional boolean parameter
#[must_use]
pub fn extract_optional_bool(params: &Value, key: &str) -> Option<bool> {
    params.get(key).and_then(Value::as_bool)
}

/// Extract a boolean parameter with a default value
#[must_use]
pub fn extract_bool_with_default(params: &Value, key: &str, default: bool) -> bool {
    extract_optional_bool(params, key).unwrap_or(default)
}

/// Extract a required integer parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not an integer
pub fn extract_required_i64(params: &Value, key: &str) -> LLMResult<i64> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required integer parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional integer parameter
#[must_use]
pub fn extract_optional_i64(params: &Value, key: &str) -> Option<i64> {
    params.get(key).and_then(Value::as_i64)
}

/// Extract a required unsigned integer parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not an unsigned integer
pub fn extract_required_u64(params: &Value, key: &str) -> LLMResult<u64> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required unsigned integer parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional unsigned integer parameter
#[must_use]
pub fn extract_optional_u64(params: &Value, key: &str) -> Option<u64> {
    params.get(key).and_then(Value::as_u64)
}

/// Extract a required float parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not a float
pub fn extract_required_f64(params: &Value, key: &str) -> LLMResult<f64> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required float parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional float parameter
#[must_use]
pub fn extract_optional_f64(params: &Value, key: &str) -> Option<f64> {
    params.get(key).and_then(Value::as_f64)
}

/// Extract a required array parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not an array
pub fn extract_required_array<'a>(params: &'a Value, key: &str) -> LLMResult<&'a Vec<Value>> {
    params
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required array parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional array parameter
#[must_use]
pub fn extract_optional_array<'a>(params: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    params.get(key).and_then(Value::as_array)
}

/// Extract a required object parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or not an object
pub fn extract_required_object<'a>(
    params: &'a Value,
    key: &str,
) -> LLMResult<&'a serde_json::Map<String, Value>> {
    params
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| LLMSpellError::Validation {
            message: format!("Missing required object parameter '{key}'"),
            field: Some(key.to_string()),
        })
}

/// Extract an optional object parameter
#[must_use]
pub fn extract_optional_object<'a>(
    params: &'a Value,
    key: &str,
) -> Option<&'a serde_json::Map<String, Value>> {
    params.get(key).and_then(Value::as_object)
}

/// Extract and deserialize a required typed parameter
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the parameter is missing or cannot be deserialized
pub fn extract_required_typed<T: DeserializeOwned>(params: &Value, key: &str) -> LLMResult<T> {
    let value = params.get(key).ok_or_else(|| LLMSpellError::Validation {
        message: format!("Missing required parameter '{key}'"),
        field: Some(key.to_string()),
    })?;

    serde_json::from_value(value.clone()).map_err(|e| LLMSpellError::Validation {
        message: format!("Invalid parameter '{key}': {e}"),
        field: Some(key.to_string()),
    })
}

/// Extract and deserialize an optional typed parameter
#[must_use]
pub fn extract_optional_typed<T: DeserializeOwned>(params: &Value, key: &str) -> Option<T> {
    params
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Validate that at least one of the specified parameters exists
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if none of the specified parameters exist
pub fn require_one_of(params: &Value, keys: &[&str]) -> LLMResult<()> {
    for key in keys {
        if params.get(key).is_some() {
            return Ok(());
        }
    }

    Err(LLMSpellError::Validation {
        message: format!(
            "At least one of these parameters is required: {}",
            keys.join(", ")
        ),
        field: None,
    })
}

/// Validate that all of the specified parameters exist
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if any of the specified parameters are missing
pub fn require_all_of(params: &Value, keys: &[&str]) -> LLMResult<()> {
    let missing: Vec<&str> = keys
        .iter()
        .filter(|&&key| params.get(key).is_none())
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(LLMSpellError::Validation {
            message: format!("Missing required parameters: {}", missing.join(", ")),
            field: None,
        })
    }
}

/// Extract parameters directly from `AgentInput` (without nested "parameters" object)
#[must_use]
pub fn extract_direct_parameters(input: &AgentInput) -> &HashMap<String, Value> {
    &input.parameters
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_input(params: Value) -> AgentInput {
        AgentInput {
            text: "test".to_string(),
            media: vec![],
            context: None,
            parameters: serde_json::from_value(json!({ "parameters": params })).unwrap(),
            output_modalities: vec![],
        }
    }

    #[test]
    fn test_extract_parameters() {
        let input = create_test_input(json!({ "key": "value" }));
        let params = extract_parameters(&input).unwrap();
        assert_eq!(params, &json!({ "key": "value" }));
    }

    #[test]
    fn test_extract_required_string() {
        let params = json!({ "name": "test" });
        assert_eq!(extract_required_string(&params, "name").unwrap(), "test");
        assert!(extract_required_string(&params, "missing").is_err());
    }

    #[test]
    fn test_extract_optional_string() {
        let params = json!({ "name": "test" });
        assert_eq!(extract_optional_string(&params, "name"), Some("test"));
        assert_eq!(extract_optional_string(&params, "missing"), None);
    }

    #[test]
    fn test_extract_string_with_default() {
        let params = json!({ "name": "test" });
        assert_eq!(
            extract_string_with_default(&params, "name", "default"),
            "test"
        );
        assert_eq!(
            extract_string_with_default(&params, "missing", "default"),
            "default"
        );
    }

    #[test]
    fn test_extract_required_bool() {
        let params = json!({ "enabled": true });
        assert!(extract_required_bool(&params, "enabled").unwrap());
        assert!(extract_required_bool(&params, "missing").is_err());
    }

    #[test]
    fn test_extract_numbers() {
        let params = json!({
            "int": 42,
            "uint": 42,
            "float": 3.14159
        });

        assert_eq!(extract_required_i64(&params, "int").unwrap(), 42);
        assert_eq!(extract_required_u64(&params, "uint").unwrap(), 42);
        assert!((extract_required_f64(&params, "float").unwrap() - 3.14159).abs() < 0.00001);
    }

    #[test]
    fn test_extract_arrays() {
        let params = json!({ "list": [1, 2, 3] });
        let array = extract_required_array(&params, "list").unwrap();
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_extract_objects() {
        let params = json!({ "config": { "key": "value" } });
        let obj = extract_required_object(&params, "config").unwrap();
        assert_eq!(obj.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_require_one_of() {
        let params = json!({ "a": 1, "c": 3 });
        assert!(require_one_of(&params, &["a", "b"]).is_ok());
        assert!(require_one_of(&params, &["b", "c"]).is_ok());
        assert!(require_one_of(&params, &["x", "y"]).is_err());
    }

    #[test]
    fn test_require_all_of() {
        let params = json!({ "a": 1, "b": 2, "c": 3 });
        assert!(require_all_of(&params, &["a", "b"]).is_ok());
        assert!(require_all_of(&params, &["a", "b", "c"]).is_ok());
        assert!(require_all_of(&params, &["a", "x"]).is_err());
    }

    #[test]
    fn test_extract_direct_parameters() {
        let mut parameters = HashMap::new();
        parameters.insert("key".to_string(), json!("value"));

        let input = AgentInput {
            text: "test".to_string(),
            media: vec![],
            context: None,
            parameters,
            output_modalities: vec![],
        };

        let params = extract_direct_parameters(&input);
        assert_eq!(params.get("key").and_then(|v| v.as_str()), Some("value"));
    }
}
