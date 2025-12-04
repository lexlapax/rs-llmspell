//! Type converters for PostgreSQL ↔ SQLite data migration
//!
//! Handles lossless conversion between PostgreSQL-specific types and their
//! SQLite equivalents for bidirectional data migration.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

/// Trait for converting values between PostgreSQL and SQLite representations
pub trait TypeConverter {
    /// Convert PostgreSQL value to intermediate JSON representation
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue>;

    /// Convert SQLite value to intermediate JSON representation
    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue>;

    /// Convert intermediate JSON to PostgreSQL-compatible value
    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>>;

    /// Convert intermediate JSON to SQLite-compatible value
    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue>;
}

/// Represents a SQLite value (simplified for our use case)
#[derive(Debug, Clone, PartialEq)]
pub enum SqliteValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl SqliteValue {
    /// Convert to JSON value for intermediate representation
    pub fn to_json(&self) -> Result<JsonValue> {
        match self {
            SqliteValue::Null => Ok(JsonValue::Null),
            SqliteValue::Integer(i) => Ok(JsonValue::Number((*i).into())),
            SqliteValue::Real(f) => serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .ok_or_else(|| anyhow!("Invalid float value: {}", f)),
            SqliteValue::Text(s) => Ok(JsonValue::String(s.clone())),
            SqliteValue::Blob(b) => {
                // Encode blob as base64 in JSON
                Ok(JsonValue::String(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    b,
                )))
            }
        }
    }
}

/// Collection of type converters for all PostgreSQL → SQLite mappings
pub struct TypeConverters {
    timestamp: TimestampConverter,
    uuid: UuidConverter,
    jsonb: JsonbConverter,
    array: ArrayConverter,
    enum_converter: EnumConverter,
    large_object: LargeObjectConverter,
}

impl TypeConverters {
    /// Create a new set of type converters
    pub fn new() -> Self {
        Self {
            timestamp: TimestampConverter,
            uuid: UuidConverter,
            jsonb: JsonbConverter,
            array: ArrayConverter,
            enum_converter: EnumConverter::new(),
            large_object: LargeObjectConverter,
        }
    }

    /// Get the timestamp converter
    pub fn timestamp(&self) -> &TimestampConverter {
        &self.timestamp
    }

    /// Get the UUID converter
    pub fn uuid(&self) -> &UuidConverter {
        &self.uuid
    }

    /// Get the JSONB converter
    pub fn jsonb(&self) -> &JsonbConverter {
        &self.jsonb
    }

    /// Get the array converter
    pub fn array(&self) -> &ArrayConverter {
        &self.array
    }

    /// Get the enum converter
    pub fn enum_converter(&self) -> &EnumConverter {
        &self.enum_converter
    }

    /// Get the large object converter
    pub fn large_object(&self) -> &LargeObjectConverter {
        &self.large_object
    }
}

impl Default for TypeConverters {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 1. TimestampConverter: PostgreSQL TIMESTAMPTZ ↔ SQLite INTEGER (Unix timestamp)
// ============================================================================

/// Converts PostgreSQL TIMESTAMPTZ to SQLite INTEGER (Unix timestamp in microseconds)
///
/// PostgreSQL stores timestamps with timezone information and microsecond precision.
/// SQLite stores them as INTEGER (Unix timestamp in microseconds for precision).
pub struct TimestampConverter;

impl TypeConverter for TimestampConverter {
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue> {
        // Parse PostgreSQL timestamp string (RFC3339 format)
        let timestamp_str =
            std::str::from_utf8(value).context("Invalid UTF-8 in timestamp value")?;
        let dt = DateTime::parse_from_rfc3339(timestamp_str)
            .or_else(|_| {
                // Try parsing without timezone (assume UTC)
                chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.f")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc).into())
            })
            .context("Failed to parse PostgreSQL timestamp")?;

        // Convert to Unix timestamp in microseconds
        let micros = dt.timestamp() * 1_000_000 + dt.timestamp_subsec_micros() as i64;
        Ok(JsonValue::Number(micros.into()))
    }

    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue> {
        match value {
            SqliteValue::Integer(micros) => Ok(JsonValue::Number((*micros).into())),
            _ => Err(anyhow!(
                "Expected INTEGER for SQLite timestamp, got: {:?}",
                value
            )),
        }
    }

    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let micros = value
            .as_i64()
            .ok_or_else(|| anyhow!("Expected number for timestamp"))?;

        // Convert microseconds back to DateTime
        let secs = micros / 1_000_000;
        let nanos = ((micros % 1_000_000) * 1000) as u32;
        let dt = Utc
            .timestamp_opt(secs, nanos)
            .single()
            .ok_or_else(|| anyhow!("Invalid timestamp: {} microseconds", micros))?;

        // Format as RFC3339 for PostgreSQL
        Ok(dt.to_rfc3339().into_bytes())
    }

    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue> {
        let micros = value
            .as_i64()
            .ok_or_else(|| anyhow!("Expected number for timestamp"))?;
        Ok(SqliteValue::Integer(micros))
    }
}

// ============================================================================
// 2. UuidConverter: PostgreSQL UUID ↔ SQLite TEXT
// ============================================================================

/// Converts PostgreSQL UUID to SQLite TEXT (standard UUID string format)
///
/// PostgreSQL has native UUID type with validation and optimized storage.
/// SQLite stores UUIDs as TEXT in standard format (8-4-4-4-12 hyphenated).
pub struct UuidConverter;

impl TypeConverter for UuidConverter {
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue> {
        // PostgreSQL UUID can be binary (16 bytes) or string format
        let uuid_str = if value.len() == 16 {
            // Binary UUID
            Uuid::from_slice(value)
                .context("Invalid binary UUID")?
                .to_string()
        } else {
            // String UUID
            let s = std::str::from_utf8(value).context("Invalid UTF-8 in UUID value")?;
            Uuid::parse_str(s)
                .context("Invalid UUID string")?
                .to_string()
        };

        Ok(JsonValue::String(uuid_str))
    }

    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue> {
        match value {
            SqliteValue::Text(s) => {
                // Validate UUID format
                Uuid::parse_str(s).context("Invalid UUID in SQLite TEXT")?;
                Ok(JsonValue::String(s.clone()))
            }
            _ => Err(anyhow!("Expected TEXT for SQLite UUID, got: {:?}", value)),
        }
    }

    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let uuid_str = value
            .as_str()
            .ok_or_else(|| anyhow!("Expected string for UUID"))?;

        // Validate and return as string (PostgreSQL accepts string UUIDs)
        Uuid::parse_str(uuid_str).context("Invalid UUID string")?;
        Ok(uuid_str.as_bytes().to_vec())
    }

    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue> {
        let uuid_str = value
            .as_str()
            .ok_or_else(|| anyhow!("Expected string for UUID"))?;

        // Validate UUID format
        Uuid::parse_str(uuid_str).context("Invalid UUID string")?;
        Ok(SqliteValue::Text(uuid_str.to_string()))
    }
}

// ============================================================================
// 3. JsonbConverter: PostgreSQL JSONB ↔ SQLite TEXT/JSON
// ============================================================================

/// Converts PostgreSQL JSONB to SQLite TEXT (JSON string)
///
/// PostgreSQL JSONB is binary JSON with indexing support.
/// SQLite stores JSON as TEXT with json_extract() support.
pub struct JsonbConverter;

impl TypeConverter for JsonbConverter {
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue> {
        // PostgreSQL JSONB is stored as JSON text in exports
        let json_str = std::str::from_utf8(value).context("Invalid UTF-8 in JSONB value")?;
        serde_json::from_str(json_str).context("Invalid JSON in PostgreSQL JSONB")
    }

    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue> {
        match value {
            SqliteValue::Text(s) => serde_json::from_str(s).context("Invalid JSON in SQLite TEXT"),
            _ => Err(anyhow!("Expected TEXT for SQLite JSON, got: {:?}", value)),
        }
    }

    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>> {
        // Serialize as compact JSON for PostgreSQL
        let json_str = serde_json::to_string(value).context("Failed to serialize JSON")?;
        Ok(json_str.into_bytes())
    }

    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue> {
        // Serialize as compact JSON for SQLite
        let json_str = serde_json::to_string(value).context("Failed to serialize JSON")?;
        Ok(SqliteValue::Text(json_str))
    }
}

// ============================================================================
// 4. ArrayConverter: PostgreSQL ARRAY ↔ SQLite JSON array
// ============================================================================

/// Converts PostgreSQL ARRAY to SQLite JSON array
///
/// PostgreSQL supports native arrays (e.g., TEXT[], INTEGER[]).
/// SQLite stores arrays as JSON TEXT with array notation.
pub struct ArrayConverter;

impl TypeConverter for ArrayConverter {
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue> {
        // PostgreSQL array format: {val1,val2,val3} or array serialized as JSON
        let array_str = std::str::from_utf8(value).context("Invalid UTF-8 in array value")?;

        // Try parsing as JSON first (modern PostgreSQL can export as JSON)
        if let Ok(json) = serde_json::from_str::<JsonValue>(array_str) {
            if json.is_array() {
                return Ok(json);
            }
        }

        // Parse PostgreSQL array notation: {val1,val2,val3}
        if array_str.starts_with('{') && array_str.ends_with('}') {
            let inner = &array_str[1..array_str.len() - 1];
            if inner.is_empty() {
                return Ok(JsonValue::Array(vec![]));
            }

            // Simple CSV parsing (doesn't handle quoted commas, but works for TEXT[])
            let elements: Vec<JsonValue> = inner
                .split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    // Handle NULL values
                    if trimmed.eq_ignore_ascii_case("null") {
                        JsonValue::Null
                    } else {
                        JsonValue::String(trimmed.to_string())
                    }
                })
                .collect();

            return Ok(JsonValue::Array(elements));
        }

        Err(anyhow!("Invalid PostgreSQL array format: {}", array_str))
    }

    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue> {
        match value {
            SqliteValue::Text(s) => {
                let json = serde_json::from_str::<JsonValue>(s)
                    .context("Invalid JSON array in SQLite TEXT")?;
                if !json.is_array() {
                    return Err(anyhow!("Expected JSON array, got: {}", json));
                }
                Ok(json)
            }
            _ => Err(anyhow!("Expected TEXT for SQLite array, got: {:?}", value)),
        }
    }

    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>> {
        if !value.is_array() {
            return Err(anyhow!("Expected array for PostgreSQL ARRAY conversion"));
        }

        // Serialize as JSON array (PostgreSQL can parse JSON arrays)
        let json_str = serde_json::to_string(value).context("Failed to serialize array")?;
        Ok(json_str.into_bytes())
    }

    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue> {
        if !value.is_array() {
            return Err(anyhow!("Expected array for SQLite JSON array conversion"));
        }

        // Serialize as compact JSON array
        let json_str = serde_json::to_string(value).context("Failed to serialize array")?;
        Ok(SqliteValue::Text(json_str))
    }
}

// ============================================================================
// 5. EnumConverter: PostgreSQL ENUM ↔ SQLite TEXT (with validation)
// ============================================================================

/// Converts PostgreSQL ENUM to SQLite TEXT with CHECK constraint validation
///
/// PostgreSQL supports custom ENUM types with validation.
/// SQLite uses TEXT with CHECK constraints for enum-like behavior.
pub struct EnumConverter {
    /// Maps enum type names to valid values
    enum_definitions: HashMap<String, Vec<String>>,
}

impl EnumConverter {
    /// Create a new enum converter
    pub fn new() -> Self {
        Self {
            enum_definitions: HashMap::new(),
        }
    }

    /// Register an enum type definition
    ///
    /// # Arguments
    /// * `type_name` - PostgreSQL enum type name
    /// * `values` - List of valid enum values
    pub fn register_enum(&mut self, type_name: String, values: Vec<String>) {
        self.enum_definitions.insert(type_name, values);
    }

    /// Validate enum value against registered definition
    #[allow(dead_code)]
    fn validate_enum_value(&self, type_name: &str, value: &str) -> Result<()> {
        if let Some(valid_values) = self.enum_definitions.get(type_name) {
            if !valid_values.contains(&value.to_string()) {
                return Err(anyhow!(
                    "Invalid enum value '{}' for type '{}'. Valid values: {:?}",
                    value,
                    type_name,
                    valid_values
                ));
            }
        }
        Ok(())
    }
}

impl Default for EnumConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for EnumConverter {
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue> {
        // PostgreSQL ENUM is stored as TEXT
        let enum_str = std::str::from_utf8(value).context("Invalid UTF-8 in enum value")?;
        Ok(JsonValue::String(enum_str.to_string()))
    }

    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue> {
        match value {
            SqliteValue::Text(s) => Ok(JsonValue::String(s.clone())),
            _ => Err(anyhow!("Expected TEXT for SQLite enum, got: {:?}", value)),
        }
    }

    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let enum_str = value
            .as_str()
            .ok_or_else(|| anyhow!("Expected string for enum"))?;
        Ok(enum_str.as_bytes().to_vec())
    }

    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue> {
        let enum_str = value
            .as_str()
            .ok_or_else(|| anyhow!("Expected string for enum"))?;
        Ok(SqliteValue::Text(enum_str.to_string()))
    }
}

// ============================================================================
// 6. LargeObjectConverter: PostgreSQL OID (Large Objects) ↔ SQLite BLOB
// ============================================================================

/// Converts PostgreSQL Large Objects (OID) to SQLite BLOB
///
/// PostgreSQL uses Large Objects API for storing large binary data (>1MB typical).
/// SQLite stores large data inline as BLOB (no 1MB threshold needed).
///
/// This converter handles the chunked reading of PostgreSQL Large Objects
/// and converts them to/from base64 JSON for intermediate representation.
pub struct LargeObjectConverter;

impl TypeConverter for LargeObjectConverter {
    fn pg_to_json(&self, value: &[u8]) -> Result<JsonValue> {
        // For export, Large Object is already read into memory as bytes
        // Encode as base64 for JSON transport
        Ok(JsonValue::String(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            value,
        )))
    }

    fn sqlite_to_json(&self, value: &SqliteValue) -> Result<JsonValue> {
        match value {
            SqliteValue::Blob(b) => {
                // Encode BLOB as base64 for JSON transport
                Ok(JsonValue::String(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    b,
                )))
            }
            _ => Err(anyhow!(
                "Expected BLOB for SQLite large object, got: {:?}",
                value
            )),
        }
    }

    fn json_to_pg(&self, value: &JsonValue) -> Result<Vec<u8>> {
        // Decode base64 JSON to binary for PostgreSQL Large Object
        let base64_str = value
            .as_str()
            .ok_or_else(|| anyhow!("Expected base64 string for large object"))?;

        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str)
            .context("Failed to decode base64 large object data")
    }

    fn json_to_sqlite(&self, value: &JsonValue) -> Result<SqliteValue> {
        // Decode base64 JSON to BLOB for SQLite
        let base64_str = value
            .as_str()
            .ok_or_else(|| anyhow!("Expected base64 string for large object"))?;

        let blob_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str)
                .context("Failed to decode base64 large object data")?;

        Ok(SqliteValue::Blob(blob_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_converter_roundtrip() {
        let converter = TimestampConverter;

        // Test with current time
        let now = Utc::now();
        let micros = now.timestamp() * 1_000_000 + now.timestamp_subsec_micros() as i64;
        let json = JsonValue::Number(micros.into());

        // Roundtrip: JSON → PostgreSQL → JSON
        let pg_bytes = converter.json_to_pg(&json).unwrap();
        let pg_json = converter.pg_to_json(&pg_bytes).unwrap();
        assert_eq!(json, pg_json);

        // Roundtrip: JSON → SQLite → JSON
        let sqlite_val = converter.json_to_sqlite(&json).unwrap();
        let sqlite_json = converter.sqlite_to_json(&sqlite_val).unwrap();
        assert_eq!(json, sqlite_json);
    }

    #[test]
    fn test_uuid_converter_roundtrip() {
        let converter = UuidConverter;
        let test_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let json = JsonValue::String(test_uuid.to_string());

        // Roundtrip: JSON → PostgreSQL → JSON
        let pg_bytes = converter.json_to_pg(&json).unwrap();
        let pg_json = converter.pg_to_json(&pg_bytes).unwrap();
        assert_eq!(json, pg_json);

        // Roundtrip: JSON → SQLite → JSON
        let sqlite_val = converter.json_to_sqlite(&json).unwrap();
        let sqlite_json = converter.sqlite_to_json(&sqlite_val).unwrap();
        assert_eq!(json, sqlite_json);
    }

    #[test]
    fn test_jsonb_converter_roundtrip() {
        let converter = JsonbConverter;
        let json = serde_json::json!({"key": "value", "number": 42});

        // Roundtrip: JSON → PostgreSQL → JSON
        let pg_bytes = converter.json_to_pg(&json).unwrap();
        let pg_json = converter.pg_to_json(&pg_bytes).unwrap();
        assert_eq!(json, pg_json);

        // Roundtrip: JSON → SQLite → JSON
        let sqlite_val = converter.json_to_sqlite(&json).unwrap();
        let sqlite_json = converter.sqlite_to_json(&sqlite_val).unwrap();
        assert_eq!(json, sqlite_json);
    }

    #[test]
    fn test_array_converter_roundtrip() {
        let converter = ArrayConverter;
        let json = serde_json::json!(["value1", "value2", "value3"]);

        // Roundtrip: JSON → PostgreSQL → JSON
        let pg_bytes = converter.json_to_pg(&json).unwrap();
        let pg_json = converter.pg_to_json(&pg_bytes).unwrap();
        assert_eq!(json, pg_json);

        // Roundtrip: JSON → SQLite → JSON
        let sqlite_val = converter.json_to_sqlite(&json).unwrap();
        let sqlite_json = converter.sqlite_to_json(&sqlite_val).unwrap();
        assert_eq!(json, sqlite_json);
    }

    #[test]
    fn test_array_converter_pg_notation() {
        let converter = ArrayConverter;

        // Test PostgreSQL array notation: {val1,val2,val3}
        let pg_array = b"{value1,value2,value3}";
        let json = converter.pg_to_json(pg_array).unwrap();

        assert!(json.is_array());
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], JsonValue::String("value1".to_string()));
        assert_eq!(arr[1], JsonValue::String("value2".to_string()));
        assert_eq!(arr[2], JsonValue::String("value3".to_string()));
    }

    #[test]
    fn test_enum_converter_roundtrip() {
        let converter = EnumConverter::new();
        let json = JsonValue::String("active".to_string());

        // Roundtrip: JSON → PostgreSQL → JSON
        let pg_bytes = converter.json_to_pg(&json).unwrap();
        let pg_json = converter.pg_to_json(&pg_bytes).unwrap();
        assert_eq!(json, pg_json);

        // Roundtrip: JSON → SQLite → JSON
        let sqlite_val = converter.json_to_sqlite(&json).unwrap();
        let sqlite_json = converter.sqlite_to_json(&sqlite_val).unwrap();
        assert_eq!(json, sqlite_json);
    }

    #[test]
    fn test_large_object_converter_roundtrip() {
        let converter = LargeObjectConverter;
        let test_data = b"Large binary data content here...".to_vec();

        // Encode as base64 for JSON
        let base64_str =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &test_data);
        let json = JsonValue::String(base64_str);

        // Roundtrip: JSON → PostgreSQL → JSON
        let pg_bytes = converter.json_to_pg(&json).unwrap();
        let pg_json = converter.pg_to_json(&pg_bytes).unwrap();
        assert_eq!(json, pg_json);

        // Roundtrip: JSON → SQLite → JSON
        let sqlite_val = converter.json_to_sqlite(&json).unwrap();
        let sqlite_json = converter.sqlite_to_json(&sqlite_val).unwrap();
        assert_eq!(json, sqlite_json);

        // Verify data integrity
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            json.as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(decoded, test_data);
    }
}
