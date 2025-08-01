// ABOUTME: Enhanced schema versioning system with semantic versioning and compatibility checks
// ABOUTME: Provides schema registry, migration planning, and compatibility validation

pub mod compatibility;
pub mod migration;
pub mod registry;
pub mod version;

pub use compatibility::{CompatibilityChecker, CompatibilityMatrix, CompatibilityResult};
pub use migration::{MigrationPlan, MigrationPlanner, MigrationPlannerError};
pub use registry::{SchemaRegistry, SchemaRegistryError};
pub use version::{SchemaVersion, SemanticVersion};

use crate::config::{CompatibilityLevel, MigrationStep, StateSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced state schema with semantic versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedStateSchema {
    pub version: SemanticVersion,
    pub hash: String,
    pub created_at: std::time::SystemTime,
    pub fields: HashMap<String, crate::config::FieldSchema>,
    pub compatibility: CompatibilityLevel,
    pub migration_path: Vec<MigrationStep>,
    pub dependencies: Vec<SemanticVersion>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EnhancedStateSchema {
    pub fn new(version: SemanticVersion) -> Self {
        Self {
            version,
            hash: Self::generate_hash(),
            created_at: std::time::SystemTime::now(),
            fields: HashMap::new(),
            compatibility: CompatibilityLevel::BackwardCompatible,
            migration_path: Vec::new(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn from_legacy(legacy: StateSchema) -> Self {
        Self {
            version: SemanticVersion::new(legacy.version, 0, 0),
            hash: legacy.hash,
            created_at: std::time::SystemTime::now(),
            fields: legacy.fields,
            compatibility: legacy.compatibility,
            migration_path: legacy.migration_path,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn to_legacy(&self) -> StateSchema {
        StateSchema {
            version: self.version.major,
            hash: self.hash.clone(),
            fields: self.fields.clone(),
            compatibility: self.compatibility.clone(),
            migration_path: self.migration_path.clone(),
        }
    }

    pub fn add_field(&mut self, name: String, field: crate::config::FieldSchema) {
        self.fields.insert(name, field);
        self.hash = Self::generate_hash();
    }

    pub fn remove_field(&mut self, name: &str) -> Option<crate::config::FieldSchema> {
        let result = self.fields.remove(name);
        if result.is_some() {
            self.hash = Self::generate_hash();
        }
        result
    }

    pub fn add_dependency(&mut self, version: SemanticVersion) {
        if !self.dependencies.contains(&version) {
            self.dependencies.push(version);
        }
    }

    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    fn generate_hash() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FieldSchema;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_enhanced_schema_creation() {
        let version = SemanticVersion::new(1, 2, 3);
        let schema = EnhancedStateSchema::new(version.clone());

        assert_eq!(schema.version, version);
        assert!(!schema.hash.is_empty());
        assert!(schema.fields.is_empty());
        assert_eq!(schema.compatibility, CompatibilityLevel::BackwardCompatible);
        assert!(schema.migration_path.is_empty());
        assert!(schema.dependencies.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_legacy_conversion() {
        let legacy = StateSchema::v1();
        let enhanced = EnhancedStateSchema::from_legacy(legacy.clone());
        let converted_back = enhanced.to_legacy();

        assert_eq!(converted_back.version, legacy.version);
        assert_eq!(converted_back.hash, legacy.hash);
        assert_eq!(converted_back.fields, legacy.fields);
        assert_eq!(converted_back.compatibility, legacy.compatibility);
        assert_eq!(converted_back.migration_path, legacy.migration_path);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_field_operations() {
        let mut schema = EnhancedStateSchema::new(SemanticVersion::new(1, 0, 0));
        let original_hash = schema.hash.clone();

        let field = FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec![],
        };

        schema.add_field("test_field".to_string(), field.clone());
        assert_ne!(schema.hash, original_hash);
        assert!(schema.fields.contains_key("test_field"));

        let removed = schema.remove_field("test_field");
        assert!(removed.is_some());
        assert!(!schema.fields.contains_key("test_field"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_dependency_management() {
        let mut schema = EnhancedStateSchema::new(SemanticVersion::new(2, 0, 0));
        let dep1 = SemanticVersion::new(1, 0, 0);
        let dep2 = SemanticVersion::new(1, 5, 0);

        schema.add_dependency(dep1.clone());
        schema.add_dependency(dep2.clone());
        schema.add_dependency(dep1.clone()); // Duplicate - should not be added

        assert_eq!(schema.dependencies.len(), 2);
        assert!(schema.dependencies.contains(&dep1));
        assert!(schema.dependencies.contains(&dep2));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_metadata_operations() {
        let mut schema = EnhancedStateSchema::new(SemanticVersion::new(1, 0, 0));

        schema.set_metadata("author".to_string(), serde_json::json!("Test Author"));
        schema.set_metadata("description".to_string(), serde_json::json!("Test schema"));

        assert_eq!(schema.metadata.len(), 2);
        assert_eq!(
            schema.metadata.get("author"),
            Some(&serde_json::json!("Test Author"))
        );
        assert_eq!(
            schema.metadata.get("description"),
            Some(&serde_json::json!("Test schema"))
        );
    }
}
