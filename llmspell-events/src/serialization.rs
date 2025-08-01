// ABOUTME: Basic event serialization for JSON format
// ABOUTME: Handles core JSON serialization for UniversalEvent

use crate::universal_event::UniversalEvent;
use anyhow::Result;

/// Event serializer for JSON format
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

    /// Pretty-print event to JSON
    pub fn to_json_pretty(event: &UniversalEvent) -> Result<String> {
        serde_json::to_string_pretty(event).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_event::Language;
    use serde_json::Value;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_json_serialization() {
        let event = UniversalEvent::new("test", Value::Null, Language::Rust);

        let json = EventSerializer::to_json(&event).unwrap();
        let deserialized = EventSerializer::from_json(&json).unwrap();

        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.language, deserialized.language);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_pretty_json() {
        let event = UniversalEvent::new(
            "test.event",
            serde_json::json!({"key": "value"}),
            Language::Rust,
        );

        let pretty_json = EventSerializer::to_json_pretty(&event).unwrap();
        assert!(pretty_json.contains("\"test.event\""));
        assert!(pretty_json.contains("\n")); // Should have formatting
    }
}
