// ABOUTME: Language-agnostic event serialization for cross-language event propagation
// ABOUTME: Handles UniversalEvent conversion to/from language-specific formats with JSON fallback

use anyhow::{Context, Result};
use llmspell_events::{EventMetadata, Language, UniversalEvent};
use serde_json::Value as JsonValue;

/// Event serialization utilities for cross-language support
pub struct EventSerialization;

impl EventSerialization {
    /// Convert `UniversalEvent` to JSON representation suitable for cross-language transfer
    pub fn universal_event_to_json(event: &UniversalEvent) -> Result<JsonValue> {
        let json = serde_json::json!({
            "id": event.id,
            "event_type": event.event_type,
            "data": event.data,
            "timestamp": event.timestamp.to_rfc3339(),
            "sequence": event.sequence,
            "correlation_id": event.metadata.correlation_id,
            "language": format!("{:?}", event.language),
            "ttl": event.metadata.ttl,
            "metadata": event.metadata
        });

        Ok(json)
    }

    /// Convert JSON representation back to `UniversalEvent`
    pub fn json_to_universal_event(json: &JsonValue) -> Result<UniversalEvent> {
        let event_type = json["event_type"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing event_type field"))?;

        let data = json["data"].clone();

        let language_str = json["language"].as_str().unwrap_or("Rust");
        let language = match language_str {
            "Lua" => Language::Lua,
            "JavaScript" | "Javascript" => Language::JavaScript,
            "Python" => Language::Python,
            "Rust" => Language::Rust,
            _ => Language::Unknown,
        };

        let mut event = UniversalEvent::new(event_type, data, language);

        // Set optional fields if present
        if let Some(id_str) = json["id"].as_str() {
            if let Ok(id) = uuid::Uuid::parse_str(id_str) {
                event.id = id;
            }
        }

        if let Some(correlation_id_str) = json["correlation_id"].as_str() {
            if let Ok(correlation_id) = uuid::Uuid::parse_str(correlation_id_str) {
                event.metadata.correlation_id = correlation_id;
            }
        }

        if let Some(sequence) = json["sequence"].as_u64() {
            event.sequence = sequence;
        }

        if let Some(ttl_secs) = json["ttl"].as_u64() {
            event.metadata.ttl = Some(ttl_secs);
        }

        // Handle full metadata if present
        if let Some(metadata) = json["metadata"].as_object() {
            if let Ok(parsed_metadata) =
                serde_json::from_value::<EventMetadata>(serde_json::Value::Object(metadata.clone()))
            {
                event.metadata = parsed_metadata;
            }
        }

        // Parse timestamp if present
        if let Some(timestamp_str) = json["timestamp"].as_str() {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                event.timestamp = timestamp.into();
            }
        }

        Ok(event)
    }

    /// Serialize event data for a specific language context
    pub fn serialize_for_language(
        event: &UniversalEvent,
        target_language: Language,
    ) -> Result<JsonValue> {
        let mut serialized = Self::universal_event_to_json(event)?;

        // Add language-specific hints
        serialized["target_language"] = JsonValue::String(format!("{target_language:?}"));
        serialized["serialization_version"] = JsonValue::String("1.0".to_string());

        // Language-specific data transformations
        match target_language {
            Language::Lua => {
                // Ensure arrays start at index 1 for Lua compatibility
                if let Some(data) = serialized["data"].as_array_mut() {
                    let mut lua_array = serde_json::Map::new();
                    for (i, item) in data.iter().enumerate() {
                        lua_array.insert((i + 1).to_string(), item.clone());
                    }
                    serialized["data"] = JsonValue::Object(lua_array);
                }
            }
            Language::JavaScript => {
                // Ensure compatibility with JavaScript JSON parsing
                Self::ensure_js_compatibility(&mut serialized["data"])?;
            }
            Language::Python | Language::Rust | Language::Unknown => {
                // No transformations needed for Python/Rust/Unknown
            }
        }

        Ok(serialized)
    }

    /// Deserialize event data from a specific language context
    pub fn deserialize_from_language(
        data: &JsonValue,
        source_language: Language,
    ) -> Result<UniversalEvent> {
        let mut working_data = data.clone();

        // Language-specific data transformations
        match source_language {
            Language::Lua => {
                // Convert Lua 1-indexed arrays back to 0-indexed
                if let Some(data_obj) = working_data["data"].as_object_mut() {
                    if Self::is_lua_array(data_obj) {
                        let mut array_items = Vec::new();
                        let mut indices: Vec<usize> =
                            data_obj.keys().filter_map(|k| k.parse().ok()).collect();
                        indices.sort_unstable();

                        for index in indices {
                            if let Some(value) = data_obj.get(&index.to_string()) {
                                array_items.push(value.clone());
                            }
                        }

                        working_data["data"] = JsonValue::Array(array_items);
                    }
                }
            }
            Language::JavaScript => {
                // Handle JavaScript-specific deserialization
                Self::normalize_js_data(&mut working_data["data"])?;
            }
            Language::Python | Language::Rust | Language::Unknown => {
                // No transformations needed for Python/Rust/Unknown
            }
        }

        Self::json_to_universal_event(&working_data)
    }

    /// Check if a JSON object represents a Lua-style array (numeric string keys)
    fn is_lua_array(obj: &serde_json::Map<String, JsonValue>) -> bool {
        if obj.is_empty() {
            return false;
        }

        // Check if all keys are numeric strings starting from 1
        let mut indices: Vec<usize> = obj.keys().filter_map(|k| k.parse().ok()).collect();

        if indices.is_empty() {
            return false;
        }

        indices.sort_unstable();
        indices[0] == 1 && indices == (1..=indices.len()).collect::<Vec<_>>()
    }

    /// Ensure JSON data is compatible with JavaScript
    fn ensure_js_compatibility(data: &mut JsonValue) -> Result<()> {
        match data {
            JsonValue::Object(obj) => {
                for (_, value) in obj.iter_mut() {
                    Self::ensure_js_compatibility(value)?;
                }
            }
            JsonValue::Array(arr) => {
                for value in arr.iter_mut() {
                    Self::ensure_js_compatibility(value)?;
                }
            }
            JsonValue::Number(num) => {
                // Ensure numbers are within JavaScript safe integer range
                if let Some(int_val) = num.as_i64() {
                    if int_val.abs() > 9_007_199_254_740_991 {
                        // Number.MAX_SAFE_INTEGER
                        *data = JsonValue::String(int_val.to_string());
                    }
                }
            }
            _ => {} // String, Bool, Null are already compatible
        }

        Ok(())
    }

    /// Normalize JavaScript data after deserialization
    fn normalize_js_data(data: &mut JsonValue) -> Result<()> {
        match data {
            JsonValue::Object(obj) => {
                for (_, value) in obj.iter_mut() {
                    Self::normalize_js_data(value)?;
                }
            }
            JsonValue::Array(arr) => {
                for value in arr.iter_mut() {
                    Self::normalize_js_data(value)?;
                }
            }
            JsonValue::String(s) => {
                // Try to convert stringified numbers back to numbers
                if let Ok(int_val) = s.parse::<i64>() {
                    if int_val.abs() <= 9_007_199_254_740_991 {
                        *data = JsonValue::Number(int_val.into());
                    }
                } else if let Ok(float_val) = s.parse::<f64>() {
                    if float_val.is_finite() {
                        if let Some(num) = serde_json::Number::from_f64(float_val) {
                            *data = JsonValue::Number(num);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Validate that an event can be safely serialized/deserialized for a target language
    pub fn validate_event_compatibility(
        event: &UniversalEvent,
        target_language: Language,
    ) -> Result<()> {
        // Try round-trip serialization
        let serialized = Self::serialize_for_language(event, target_language)
            .with_context(|| format!("Failed to serialize event for {target_language:?}"))?;

        let _deserialized = Self::deserialize_from_language(&serialized, target_language)
            .with_context(|| format!("Failed to deserialize event from {target_language:?}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_universal_event_json_roundtrip() {
        let original = UniversalEvent::new(
            "test.event",
            json!({"key": "value", "number": 42}),
            Language::Rust,
        );

        let json = EventSerialization::universal_event_to_json(&original).unwrap();
        let restored = EventSerialization::json_to_universal_event(&json).unwrap();

        assert_eq!(original.event_type, restored.event_type);
        assert_eq!(original.data, restored.data);
        assert_eq!(original.language, restored.language);
    }
    #[test]
    fn test_lua_array_serialization() {
        let event = UniversalEvent::new(
            "test.array",
            json!(["item1", "item2", "item3"]),
            Language::Rust,
        );

        let lua_serialized =
            EventSerialization::serialize_for_language(&event, Language::Lua).unwrap();

        // Should convert array to Lua-style 1-indexed object
        let data = &lua_serialized["data"];
        assert!(data.is_object());
        assert_eq!(data["1"], "item1");
        assert_eq!(data["2"], "item2");
        assert_eq!(data["3"], "item3");

        // Test round-trip
        let restored =
            EventSerialization::deserialize_from_language(&lua_serialized, Language::Lua).unwrap();
        assert_eq!(restored.data, json!(["item1", "item2", "item3"]));
    }
    #[test]
    fn test_javascript_large_number_handling() {
        let large_number = 9_007_199_254_740_992i64; // Exceeds JS safe integer
        let event = UniversalEvent::new(
            "test.bignumber",
            json!({"big": large_number}),
            Language::Rust,
        );

        let js_serialized =
            EventSerialization::serialize_for_language(&event, Language::JavaScript).unwrap();

        // Large number should be converted to string
        assert_eq!(js_serialized["data"]["big"], large_number.to_string());

        // Test round-trip
        let restored =
            EventSerialization::deserialize_from_language(&js_serialized, Language::JavaScript)
                .unwrap();
        assert_eq!(restored.data["big"], large_number.to_string());
    }
    #[test]
    fn test_language_detection() {
        let test_cases = vec![
            ("Lua", Language::Lua),
            ("JavaScript", Language::JavaScript),
            ("Javascript", Language::JavaScript),
            ("Python", Language::Python),
            ("Rust", Language::Rust),
            ("Unknown", Language::Unknown),
            ("UnknownLang", Language::Unknown), // Default fallback
        ];

        for (lang_str, expected) in test_cases {
            let json = json!({
                "event_type": "test",
                "data": {},
                "language": lang_str
            });

            let event = EventSerialization::json_to_universal_event(&json).unwrap();
            assert_eq!(event.language, expected);
        }
    }
    #[test]
    fn test_event_compatibility_validation() {
        let event = UniversalEvent::new(
            "test.validation",
            json!({"array": [1, 2, 3], "object": {"nested": true}}),
            Language::Unknown,
        );

        // Should validate successfully for all languages
        assert!(EventSerialization::validate_event_compatibility(&event, Language::Lua).is_ok());
        assert!(
            EventSerialization::validate_event_compatibility(&event, Language::JavaScript).is_ok()
        );
        assert!(EventSerialization::validate_event_compatibility(&event, Language::Python).is_ok());
        assert!(EventSerialization::validate_event_compatibility(&event, Language::Rust).is_ok());
    }
    #[test]
    fn test_lua_array_detection() {
        let lua_array = serde_json::Map::from_iter([
            ("1".to_string(), json!("first")),
            ("2".to_string(), json!("second")),
            ("3".to_string(), json!("third")),
        ]);

        assert!(EventSerialization::is_lua_array(&lua_array));

        let not_array = serde_json::Map::from_iter([
            ("key1".to_string(), json!("value1")),
            ("key2".to_string(), json!("value2")),
        ]);

        assert!(!EventSerialization::is_lua_array(&not_array));

        let sparse_array = serde_json::Map::from_iter([
            ("1".to_string(), json!("first")),
            ("3".to_string(), json!("third")), // Missing "2"
        ]);

        assert!(!EventSerialization::is_lua_array(&sparse_array));
    }
}
