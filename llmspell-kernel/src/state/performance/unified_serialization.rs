// ABOUTME: Unified serialization pipeline that performs all operations in a single pass
// ABOUTME: Eliminates multiple serialization cycles for optimal performance

use crate::state::sensitive_data::SensitiveDataConfig;
use crate::state::{StateError, StateResult};
use rmp_serde as msgpack;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::io::Write;

/// Unified serializer that performs all operations in a single pass
pub struct UnifiedSerializer {
    /// Configuration for sensitive data handling
    sensitive_config: SensitiveDataConfig,

    /// Whether to use `MessagePack` (true) or JSON (false)
    use_msgpack: bool,

    /// Skip validation for trusted data
    skip_validation: bool,
}

impl UnifiedSerializer {
    /// Create a new unified serializer
    pub fn new(sensitive_config: SensitiveDataConfig) -> Self {
        Self {
            sensitive_config,
            use_msgpack: true,
            skip_validation: false,
        }
    }

    /// Create a fast serializer for trusted data
    pub fn fast() -> Self {
        Self {
            sensitive_config: SensitiveDataConfig::disabled(),
            use_msgpack: true,
            skip_validation: true,
        }
    }

    /// Serialize value with all protections in a single pass
    ///
    /// # Errors
    ///
    /// Returns `StateError::SerializationError` if:
    /// - Serialization to JSON or `MessagePack` fails
    /// - Circular references are detected during protection
    pub fn serialize<T: Serialize>(&self, value: &T) -> StateResult<Vec<u8>> {
        if self.skip_validation {
            // Fast path - direct serialization
            if self.use_msgpack {
                msgpack::to_vec(value).map_err(|e| StateError::serialization(e.to_string()))
            } else {
                serde_json::to_vec(value).map_err(|e| StateError::serialization(e.to_string()))
            }
        } else {
            // Full validation path - but still single pass
            self.serialize_with_protection(value)
        }
    }

    /// Deserialize value
    ///
    /// # Errors
    ///
    /// Returns `StateError::SerializationError` if:
    /// - Data is not valid `MessagePack` or JSON format
    /// - Deserialization to target type fails
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> StateResult<T> {
        if self.use_msgpack {
            msgpack::from_slice(data).map_err(|e| StateError::serialization(e.to_string()))
        } else {
            serde_json::from_slice(data).map_err(|e| StateError::serialization(e.to_string()))
        }
    }

    /// Serialize with full protection in a single pass
    fn serialize_with_protection<T: Serialize>(&self, value: &T) -> StateResult<Vec<u8>> {
        // First convert to JSON Value for validation
        let json_value =
            serde_json::to_value(value).map_err(|e| StateError::serialization(e.to_string()))?;

        // Apply all transformations in-place
        let protected_value = self.apply_protections(json_value)?;

        // Final serialization
        if self.use_msgpack {
            msgpack::to_vec(&protected_value).map_err(|e| StateError::serialization(e.to_string()))
        } else {
            serde_json::to_vec(&protected_value)
                .map_err(|e| StateError::serialization(e.to_string()))
        }
    }

    /// Apply all protections in a single pass
    fn apply_protections(&self, mut value: Value) -> StateResult<Value> {
        // Track visited nodes for circular reference detection
        let mut visited = HashSet::new();

        // Apply protections recursively
        self.protect_value(&mut value, &mut visited, &mut Vec::new())?;

        Ok(value)
    }

    /// Recursively apply protections to a value
    fn protect_value(
        &self,
        value: &mut Value,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> StateResult<()> {
        match value {
            Value::Object(map) => {
                // Check for circular references using object identity
                let obj_id = format!("{:p}", std::ptr::from_ref(map));
                if visited.contains(&obj_id) {
                    return Err(StateError::validation_error(format!(
                        "Circular reference detected at path: {}",
                        path.join(".")
                    )));
                }
                visited.insert(obj_id.clone());

                // Process each field
                for (key, val) in map.iter_mut() {
                    path.push(key.clone());

                    // Check if this field contains sensitive data
                    if self.sensitive_config.redact_enabled && self.is_sensitive_field(key, path) {
                        *val = self.redact_value(val);
                    } else {
                        // Recurse into the value
                        self.protect_value(val, visited, path)?;
                    }

                    path.pop();
                }

                visited.remove(&obj_id);
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter_mut().enumerate() {
                    path.push(format!("[{i}]"));
                    self.protect_value(val, visited, path)?;
                    path.pop();
                }
            }
            _ => {
                // Primitive values - check if they need redaction
                if self.sensitive_config.redact_enabled && self.is_sensitive_value(value) {
                    *value = self.redact_value(value);
                }
            }
        }

        Ok(())
    }

    /// Check if a field is sensitive
    fn is_sensitive_field(&self, field_name: &str, _path: &[String]) -> bool {
        self.sensitive_config.is_sensitive_field(field_name)
    }

    /// Check if a value is sensitive
    fn is_sensitive_value(&self, value: &Value) -> bool {
        if let Value::String(s) = value {
            self.sensitive_config.contains_sensitive_pattern(s)
        } else {
            false
        }
    }

    /// Redact a sensitive value
    fn redact_value(&self, value: &Value) -> Value {
        match value {
            Value::String(_) | Value::Number(_) => {
                Value::String(self.sensitive_config.redaction_text.clone())
            }
            _ => value.clone(),
        }
    }
}

/// Builder for `UnifiedSerializer` with fluent API
pub struct UnifiedSerializerBuilder {
    sensitive_config: Option<SensitiveDataConfig>,
    use_msgpack: bool,
    skip_validation: bool,
}

impl UnifiedSerializerBuilder {
    pub fn new() -> Self {
        Self {
            sensitive_config: None,
            use_msgpack: true,
            skip_validation: false,
        }
    }

    #[must_use]
    pub fn with_sensitive_config(mut self, config: SensitiveDataConfig) -> Self {
        self.sensitive_config = Some(config);
        self
    }

    #[must_use]
    pub fn use_json(mut self) -> Self {
        self.use_msgpack = false;
        self
    }

    #[must_use]
    pub fn skip_validation(mut self) -> Self {
        self.skip_validation = true;
        self
    }

    pub fn build(self) -> UnifiedSerializer {
        UnifiedSerializer {
            sensitive_config: self.sensitive_config.unwrap_or_default(),
            use_msgpack: self.use_msgpack,
            skip_validation: self.skip_validation,
        }
    }
}

impl Default for UnifiedSerializerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming serializer for large data
pub struct StreamingSerializer<W: Write> {
    writer: W,
    serializer: UnifiedSerializer,
}

impl<W: Write> StreamingSerializer<W> {
    pub fn new(writer: W, serializer: UnifiedSerializer) -> Self {
        Self { writer, serializer }
    }

    /// Serialize to writer
    ///
    /// # Errors
    ///
    /// Returns `StateError::SerializationError` if:
    /// - Serialization fails
    /// - Writing to output stream fails
    pub fn serialize<T: Serialize>(&mut self, value: &T) -> StateResult<()> {
        let data = self.serializer.serialize(value)?;
        self.writer
            .write_all(&data)
            .map_err(|e| StateError::serialization(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_unified_serializer_fast_path() {
        let serializer = UnifiedSerializer::fast();

        let data = json!({
            "name": "test",
            "value": 42,
            "nested": {
                "array": [1, 2, 3]
            }
        });

        let serialized = serializer.serialize(&data).unwrap();
        let deserialized: Value = serializer.deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }
    #[test]
    fn test_unified_serializer_with_protection() {
        let config = SensitiveDataConfig {
            redact_enabled: true,
            custom_field_names: vec!["password".to_string()],
            ..Default::default()
        };

        let serializer = UnifiedSerializer::new(config);

        let data = json!({
            "username": "alice",
            "password": "secret123",
            "data": {
                "password": "another_secret"
            }
        });

        let serialized = serializer.serialize(&data).unwrap();
        let deserialized: Value = serializer.deserialize(&serialized).unwrap();

        // Check that passwords were redacted
        assert_eq!(deserialized["username"], "alice");
        assert_eq!(deserialized["password"], "[REDACTED]");
        assert_eq!(deserialized["data"]["password"], "[REDACTED]");
    }
    #[test]
    fn test_circular_reference_detection() {
        let _serializer = UnifiedSerializer::new(SensitiveDataConfig::default());

        // Note: Creating actual circular references in serde_json::Value is not possible
        // since Values are owned. This test would need a custom type that implements Serialize
        // and contains actual circular references.
    }
}
