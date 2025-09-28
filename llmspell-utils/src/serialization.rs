// ABOUTME: Common serialization and deserialization utilities for JSON, YAML, and TOML
// ABOUTME: Provides unified serialization interface used across LLMSpell components

//! Serialization and deserialization helpers
//!
//! This module provides common serialization utilities for JSON, YAML, and TOML,
//! with consistent error handling and type conversions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;
use toml::Value as TomlValue;

/// Serialize a value to JSON string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::to_json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     port: u16,
/// }
///
/// let config = Config {
///     name: "server".to_string(),
///     port: 8080,
/// };
///
/// let json = to_json(&config).unwrap();
/// assert!(json.contains("\"name\":\"server\""));
/// ```
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn to_json<T>(value: &T) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    serde_json::to_string(value)
}

/// Serialize a value to pretty-printed JSON
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::to_json_pretty;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key", "value");
///
/// let json = to_json_pretty(&map).unwrap();
/// assert!(json.contains("{\n"));
/// ```
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn to_json_pretty<T>(value: &T) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value)
}

/// Serialize a value to JSON with custom indentation
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::to_json_with_indent;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("nested", HashMap::from([("value", 42)]));
///
/// let json = to_json_with_indent(&map, 2).unwrap();
/// assert!(json.contains("  \"nested\""));
/// ```
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn to_json_with_indent<T>(value: &T, indent: usize) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    let indent_bytes = vec![b' '; indent];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes.as_slice());
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser)?;
    String::from_utf8(buf).map_err(|e| serde::ser::Error::custom(e.to_string()))
}

/// Deserialize a value from JSON string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::from_json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Config {
///     name: String,
///     port: u16,
/// }
///
/// let json = r#"{"name": "server", "port": 8080}"#;
/// let config: Config = from_json(json).unwrap();
/// assert_eq!(config.name, "server");
/// ```
///
/// # Errors
///
/// Returns an error if deserialization fails
pub fn from_json<'a, T>(s: &'a str) -> Result<T, serde_json::Error>
where
    T: Deserialize<'a>,
{
    serde_json::from_str(s)
}

/// Serialize a value to TOML string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::to_toml;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     database: Database,
/// }
///
/// #[derive(Serialize)]
/// struct Database {
///     url: String,
///     pool_size: u32,
/// }
///
/// let config = Config {
///     database: Database {
///         url: "postgres://localhost".to_string(),
///         pool_size: 10,
///     },
/// };
///
/// let toml = to_toml(&config).unwrap();
/// assert!(toml.contains("[database]"));
/// ```
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn to_toml<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    toml::to_string(value).context("Failed to serialize to TOML")
}

/// Serialize a value to pretty-printed TOML
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn to_toml_pretty<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    toml::to_string_pretty(value).context("Failed to serialize to pretty TOML")
}

/// Deserialize a value from TOML string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::from_toml;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     name: String,
///     debug: bool,
/// }
///
/// let toml = r#"
/// name = "myapp"
/// debug = true
/// "#;
///
/// let config: Config = from_toml(toml).unwrap();
/// assert_eq!(config.name, "myapp");
/// ```
///
/// # Errors
///
/// Returns an error if deserialization fails
pub fn from_toml<T>(s: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    toml::from_str(s).context("Failed to deserialize from TOML")
}

/// Merge two JSON values
///
/// When merging:
/// - Objects are merged recursively
/// - Arrays are replaced (not concatenated)
/// - Other values are replaced by the new value
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::{merge_json, from_json};
/// use serde_json::Value;
///
/// let base: Value = from_json(r#"{"a": 1, "b": {"c": 2}}"#).unwrap();
/// let override_value: Value = from_json(r#"{"b": {"d": 3}, "e": 4}"#).unwrap();
///
/// let merged = merge_json(&base, &override_value);
/// let result = merged.as_object().unwrap();
/// assert_eq!(result["a"], 1);
/// assert_eq!(result["e"], 4);
/// ```
#[must_use]
pub fn merge_json(base: &JsonValue, other: &JsonValue) -> JsonValue {
    match (base, other) {
        (JsonValue::Object(base_obj), JsonValue::Object(other_obj)) => {
            let mut merged = base_obj.clone();
            for (key, value) in other_obj {
                match merged.get(key) {
                    Some(base_value) => {
                        merged.insert(key.clone(), merge_json(base_value, value));
                    }
                    None => {
                        merged.insert(key.clone(), value.clone());
                    }
                }
            }
            JsonValue::Object(merged)
        }
        _ => other.clone(),
    }
}

/// Merge two TOML values
///
/// Similar to `merge_json` but for TOML values
#[must_use]
pub fn merge_toml(base: &TomlValue, other: &TomlValue) -> TomlValue {
    match (base, other) {
        (TomlValue::Table(base_table), TomlValue::Table(other_table)) => {
            let mut merged = base_table.clone();
            for (key, value) in other_table {
                match merged.get(key) {
                    Some(base_value) => {
                        merged.insert(key.clone(), merge_toml(base_value, value));
                    }
                    None => {
                        merged.insert(key.clone(), value.clone());
                    }
                }
            }
            TomlValue::Table(merged)
        }
        _ => other.clone(),
    }
}

/// Convert between serialization formats
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::{convert_format, Format};
///
/// let json = r#"{"name": "test", "value": 42}"#;
/// let toml = convert_format(json, Format::Json, Format::Toml).unwrap();
/// assert!(toml.contains("name = \"test\""));
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Input is not valid for the source format
/// - Conversion to the target format fails
pub fn convert_format(input: &str, from: Format, to: Format) -> Result<String> {
    // First, deserialize to a generic Value type
    let value: serde_json::Value = match from {
        Format::Json => from_json(input)?,
        Format::Toml => from_toml(input)?,
    };

    // Then serialize to the target format
    match to {
        Format::Json => to_json_pretty(&value).context("Failed to convert to JSON"),
        Format::Toml => to_toml_pretty(&value),
    }
}

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// JSON format
    Json,
    /// TOML format
    Toml,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Toml => write!(f, "TOML"),
        }
    }
}

impl Format {
    /// Get the file extension for this format
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
        }
    }

    /// Detect format from file extension
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

/// Safe deserialization with default values
///
/// If deserialization fails, returns the default value for the type
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::from_json_or_default;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Default, PartialEq, Debug)]
/// struct Config {
///     timeout: u64,
/// }
///
/// let valid = r#"{"timeout": 30}"#;
/// let invalid = r#"{"invalid": "json"}"#;
///
/// let config1: Config = from_json_or_default(valid);
/// assert_eq!(config1.timeout, 30);
///
/// let config2: Config = from_json_or_default(invalid);
/// assert_eq!(config2.timeout, 0); // Default value
/// ```
#[must_use]
pub fn from_json_or_default<'a, T>(s: &'a str) -> T
where
    T: Deserialize<'a> + Default,
{
    from_json(s).unwrap_or_default()
}

/// Validate JSON against a simple schema
///
/// This is a basic validator that checks required fields exist
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::serialization::validate_json_fields;
///
/// let json = r#"{"name": "test", "port": 8080}"#;
/// assert!(validate_json_fields(json, &["name", "port"]).is_ok());
/// assert!(validate_json_fields(json, &["name", "missing"]).is_err());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - JSON parsing fails
/// - The JSON is not an object
/// - Any required field is missing
pub fn validate_json_fields(json: &str, required_fields: &[&str]) -> Result<()> {
    let value: JsonValue = from_json(json)?;

    if let Some(obj) = value.as_object() {
        for field in required_fields {
            if !obj.contains_key(*field) {
                anyhow::bail!("Missing required field: {}", field);
            }
        }
        Ok(())
    } else {
        anyhow::bail!("JSON value is not an object")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_json_serialization() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let original = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        // Test basic serialization
        let json = to_json(&original).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"value\":42"));

        // Test pretty printing
        let pretty = to_json_pretty(&original).unwrap();
        assert!(pretty.contains("{\n"));
        assert!(pretty.contains("  \"name\""));

        // Test custom indentation
        let custom_indent = to_json_with_indent(&original, 4).unwrap();
        assert!(custom_indent.contains("    \"name\""));

        // Test deserialization
        let deserialized: TestStruct = from_json(&json).unwrap();
        assert_eq!(deserialized, original);
    }
    #[test]
    fn test_toml_serialization() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Config {
            server: ServerConfig,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct ServerConfig {
            host: String,
            port: u16,
        }

        let config = Config {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
            },
        };

        let toml = to_toml(&config).unwrap();
        assert!(toml.contains("[server]"));
        assert!(toml.contains("host = \"localhost\""));

        let deserialized: Config = from_toml(&toml).unwrap();
        assert_eq!(deserialized, config);
    }
    #[test]
    fn test_merge_json() {
        let base: JsonValue = from_json(
            r#"{
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            },
            "e": [1, 2, 3]
        }"#,
        )
        .unwrap();

        let override_value: JsonValue = from_json(
            r#"{
            "b": {
                "d": 4,
                "f": 5
            },
            "e": [4, 5],
            "g": 6
        }"#,
        )
        .unwrap();

        let merged = merge_json(&base, &override_value);
        let obj = merged.as_object().unwrap();

        assert_eq!(obj["a"], 1);
        assert_eq!(obj["g"], 6);
        assert_eq!(obj["e"], json!([4, 5])); // Arrays are replaced

        let b_obj = obj["b"].as_object().unwrap();
        assert_eq!(b_obj["c"], 2); // Preserved from base
        assert_eq!(b_obj["d"], 4); // Overridden
        assert_eq!(b_obj["f"], 5); // Added from override
    }
    #[test]
    fn test_format_conversion() {
        let json = r#"{"name": "test", "values": [1, 2, 3]}"#;

        // JSON to TOML
        let toml = convert_format(json, Format::Json, Format::Toml).unwrap();
        assert!(toml.contains("name = \"test\""));

        // TOML back to JSON
        let json2 = convert_format(&toml, Format::Toml, Format::Json).unwrap();
        let parsed1: JsonValue = from_json(json).unwrap();
        let parsed2: JsonValue = from_json(&json2).unwrap();
        assert_eq!(parsed1, parsed2);
    }
    #[test]
    fn test_format_detection() {
        assert_eq!(Format::from_extension("json"), Some(Format::Json));
        assert_eq!(Format::from_extension("toml"), Some(Format::Toml));
        assert_eq!(Format::from_extension("yaml"), None);
        assert_eq!(Format::from_extension("yml"), None);
        assert_eq!(Format::from_extension("txt"), None);

        assert_eq!(Format::Json.extension(), "json");
        assert_eq!(Format::Toml.extension(), "toml");
    }
    #[test]
    fn test_from_json_or_default() {
        #[derive(Deserialize, Default, PartialEq, Debug)]
        struct Settings {
            timeout: u64,
            retries: u32,
        }

        let valid = r#"{"timeout": 30, "retries": 3}"#;
        let settings: Settings = from_json_or_default(valid);
        assert_eq!(settings.timeout, 30);
        assert_eq!(settings.retries, 3);

        let invalid = r#"{"invalid": true}"#;
        let settings: Settings = from_json_or_default(invalid);
        assert_eq!(settings.timeout, 0);
        assert_eq!(settings.retries, 0);
    }
    #[test]
    fn test_validate_json_fields() {
        let json = r#"{
            "name": "myapp",
            "version": "1.0.0",
            "features": ["logging", "metrics"]
        }"#;

        assert!(validate_json_fields(json, &["name", "version"]).is_ok());
        assert!(validate_json_fields(json, &["name", "missing"]).is_err());

        let not_object = r"[]";
        assert!(validate_json_fields(not_object, &["any"]).is_err());
    }
    #[test]
    fn test_error_handling() {
        // Invalid JSON
        let result: Result<HashMap<String, String>, _> = from_json("invalid json");
        assert!(result.is_err());

        // Invalid TOML
        let result: Result<HashMap<String, String>> = from_toml("[[invalid toml");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_json_roundtrip(s: String, n: i32, b: bool) {
            #[derive(Serialize, Deserialize, PartialEq, Debug)]
            struct TestData {
                string: String,
                number: i32,
                boolean: bool,
            }

            let data = TestData {
                string: s,
                number: n,
                boolean: b,
            };

            let json = to_json(&data).unwrap();
            let recovered: TestData = from_json(&json).unwrap();
            assert_eq!(data, recovered);
        }
        #[test]
        fn test_merge_preserves_structure(
            keys in prop::collection::vec("[a-z]+", 1..5),
            values in prop::collection::vec(0i32..100, 1..5)
        ) {
            if keys.len() != values.len() {
                return Ok(());
            }

            let mut base_map = serde_json::Map::new();
            let mut override_map = serde_json::Map::new();

            for (i, (k, v)) in keys.iter().zip(values.iter()).enumerate() {
                if i % 2 == 0 {
                    base_map.insert(k.clone(), json!(v));
                } else {
                    override_map.insert(k.clone(), json!(v * 2));
                }
            }

            let base = JsonValue::Object(base_map);
            let override_val = JsonValue::Object(override_map);
            let merged = merge_json(&base, &override_val);

            // Verify all keys are present
            if let Some(obj) = merged.as_object() {
                for k in &keys {
                    assert!(obj.contains_key(k));
                }
            }
        }
    }
}

// Re-export commonly used json macro
pub use serde_json::json;
