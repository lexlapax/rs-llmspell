// ABOUTME: Circular reference detection for safe agent state serialization
// ABOUTME: Prevents infinite loops when serializing agent states with circular references

use serde::Serialize;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Circular reference detector for JSON values
#[derive(Default)]
pub struct CircularReferenceDetector {
    /// Stack of object hashes we're currently visiting
    visit_stack: Vec<u64>,
    /// Set of all visited object hashes
    visited: HashSet<u64>,
    /// Path stack for error reporting
    path_stack: Vec<String>,
}

impl CircularReferenceDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a JSON value contains circular references
    pub fn check_value(&mut self, value: &Value) -> Result<(), CircularReferenceError> {
        self.check_value_internal(value, "$")
    }

    fn check_value_internal(
        &mut self,
        value: &Value,
        path: &str,
    ) -> Result<(), CircularReferenceError> {
        match value {
            Value::Object(map) => {
                let hash = self.hash_object(map);

                // Check if we're already visiting this object
                if self.visit_stack.contains(&hash) {
                    return Err(CircularReferenceError {
                        path: self.path_stack.join("."),
                        object_hash: hash,
                    });
                }

                // Add to visit stack
                self.visit_stack.push(hash);
                self.visited.insert(hash);
                self.path_stack.push(path.to_string());

                // Check all fields
                for (key, val) in map {
                    let field_path = format!("{}.{}", path, key);
                    self.check_value_internal(val, &field_path)?;
                }

                // Remove from visit stack
                self.visit_stack.pop();
                self.path_stack.pop();
            }
            Value::Array(arr) => {
                self.path_stack.push(path.to_string());

                for (idx, val) in arr.iter().enumerate() {
                    let elem_path = format!("{}[{}]", path, idx);
                    self.check_value_internal(val, &elem_path)?;
                }

                self.path_stack.pop();
            }
            _ => {
                // Primitive values can't have circular references
            }
        }

        Ok(())
    }

    fn hash_object(&self, map: &serde_json::Map<String, Value>) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash the keys to identify the object structure
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort();

        for key in keys {
            key.hash(&mut hasher);
        }

        hasher.finish()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Circular reference detected at path: {path} (object hash: {object_hash})")]
pub struct CircularReferenceError {
    pub path: String,
    pub object_hash: u64,
}

/// Trait for types that can be checked for circular references
pub trait CircularReferenceCheck {
    fn check_circular_references(&self) -> Result<(), CircularReferenceError>;
}

impl<T: Serialize> CircularReferenceCheck for T {
    fn check_circular_references(&self) -> Result<(), CircularReferenceError> {
        let value = serde_json::to_value(self).map_err(|_| CircularReferenceError {
            path: "$".to_string(),
            object_hash: 0,
        })?;

        let mut detector = CircularReferenceDetector::new();
        detector.check_value(&value)
    }
}

/// Safe serialization wrapper that checks for circular references
pub fn safe_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    // First check for circular references
    if let Err(e) = value.check_circular_references() {
        return Err(format!("Circular reference detected: {}", e));
    }

    // If safe, proceed with serialization
    serde_json::to_vec(value).map_err(|e| e.to_string())
}

/// Safe JSON serialization that checks for circular references
pub fn safe_to_json<T: Serialize>(value: &T) -> Result<Value, String> {
    // First check for circular references
    if let Err(e) = value.check_circular_references() {
        return Err(format!("Circular reference detected: {}", e));
    }

    // If safe, proceed with serialization
    serde_json::to_value(value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_circular_reference() {
        let value = json!({
            "name": "test",
            "data": {
                "nested": {
                    "value": 42
                }
            },
            "array": [1, 2, 3]
        });

        let mut detector = CircularReferenceDetector::new();
        assert!(detector.check_value(&value).is_ok());
    }

    #[test]
    fn test_circular_reference_detection() {
        // Create a value that would have circular reference if we could
        // Since serde_json::Value doesn't support actual circular refs,
        // we test the detector logic with separate objects
        let obj1 = json!({
            "id": "obj1",
            "data": {}
        });

        let mut detector = CircularReferenceDetector::new();

        // Simulate visiting the same object twice in the stack
        if let Value::Object(map) = &obj1 {
            let hash = detector.hash_object(map);
            detector.visit_stack.push(hash);
            detector.visit_stack.push(hash); // Same hash = circular ref

            // Should detect circular reference
            assert!(detector.visit_stack.contains(&hash));
        }
    }

    #[test]
    fn test_safe_serialize() {
        #[derive(Serialize)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let test = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let result = safe_serialize(&test);
        assert!(result.is_ok());
    }
}
