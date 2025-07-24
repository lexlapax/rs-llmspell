// ABOUTME: Event serialization for cross-language compatibility
// ABOUTME: Handles JSON serialization and language-specific format conversion

use crate::universal_event::{Language, UniversalEvent};
use anyhow::Result;
use serde_json::Value;

/// Event serializer for cross-language compatibility
pub struct EventSerializer;

impl EventSerializer {
    /// Serialize event to JSON
    pub fn to_json(event: &UniversalEvent) -> Result<String> {
        serde_json::to_string(event).map_err(Into::into)
    }

    /// Deserialize event from JSON
    pub fn from_json(json: &str) -> Result<UniversalEvent> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Convert event to language-specific format
    pub fn to_language_format(event: &UniversalEvent, target_language: Language) -> Result<Value> {
        match target_language {
            Language::Rust => Ok(serde_json::to_value(event)?),
            Language::Lua => {
                // Convert to Lua-friendly format
                Ok(serde_json::json!({
                    "id": event.id,
                    "event_type": event.event_type,
                    "data": event.data,
                    "language": event.language.as_str(),
                    "timestamp": event.timestamp.to_rfc3339(),
                    "sequence": event.sequence,
                }))
            }
            Language::JavaScript => {
                // Convert to JavaScript-friendly format
                Ok(serde_json::json!({
                    "id": event.id,
                    "eventType": event.event_type,
                    "data": event.data,
                    "language": event.language.as_str(),
                    "timestamp": event.timestamp.to_rfc3339(),
                    "sequence": event.sequence,
                }))
            }
            Language::Python => {
                // Convert to Python-friendly format
                Ok(serde_json::json!({
                    "id": event.id,
                    "event_type": event.event_type,
                    "data": event.data,
                    "language": event.language.as_str(),
                    "timestamp": event.timestamp.to_rfc3339(),
                    "sequence": event.sequence,
                }))
            }
            Language::Unknown => Ok(serde_json::to_value(event)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_json_serialization() {
        let event = UniversalEvent::new("test", Value::Null, Language::Rust);

        let json = EventSerializer::to_json(&event).unwrap();
        let deserialized = EventSerializer::from_json(&json).unwrap();

        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.language, deserialized.language);
    }

    #[test]
    fn test_language_formats() {
        let event = UniversalEvent::new(
            "test.event",
            serde_json::json!({"key": "value"}),
            Language::Rust,
        );

        let lua_format = EventSerializer::to_language_format(&event, Language::Lua).unwrap();
        assert_eq!(lua_format["event_type"], "test.event");

        let js_format = EventSerializer::to_language_format(&event, Language::JavaScript).unwrap();
        assert_eq!(js_format["eventType"], "test.event");
    }
}
