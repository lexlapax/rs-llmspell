// ABOUTME: Schema registry for managing multiple schema versions and lookups
// ABOUTME: Provides centralized schema storage, version resolution, and schema discovery

use super::{EnhancedStateSchema, SchemaVersion, SemanticVersion};
use llmspell_state_traits::{StateError, StateResult};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaRegistryError {
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Schema version conflict: {version} already exists")]
    VersionConflict { version: SemanticVersion },

    #[error("Invalid schema registration: {reason}")]
    InvalidRegistration { reason: String },

    #[error("Schema validation failed: {details}")]
    ValidationFailed { details: String },
}

impl From<SchemaRegistryError> for StateError {
    fn from(err: SchemaRegistryError) -> Self {
        StateError::MigrationError(err.to_string())
    }
}

/// Registry for managing state schemas across versions
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    /// Schemas indexed by version for fast lookup
    schemas: Arc<RwLock<BTreeMap<SemanticVersion, Arc<EnhancedStateSchema>>>>,

    /// Schema metadata indexed by version
    metadata: Arc<RwLock<BTreeMap<SemanticVersion, SchemaVersion>>>,

    /// Name-based schema lookup for convenience
    named_schemas: Arc<RwLock<HashMap<String, SemanticVersion>>>,

    /// Current active schema version
    current_version: Arc<RwLock<Option<SemanticVersion>>>,
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(BTreeMap::new())),
            metadata: Arc::new(RwLock::new(BTreeMap::new())),
            named_schemas: Arc::new(RwLock::new(HashMap::new())),
            current_version: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a new schema version
    pub fn register_schema(
        &self,
        schema: EnhancedStateSchema,
        metadata: Option<SchemaVersion>,
    ) -> Result<(), SchemaRegistryError> {
        let version = schema.version.clone();

        // Check for version conflicts
        {
            let schemas = self.schemas.read();
            if schemas.contains_key(&version) {
                return Err(SchemaRegistryError::VersionConflict { version });
            }
        }

        // Validate schema
        self.validate_schema(&schema)?;

        // Register the schema
        {
            let mut schemas = self.schemas.write();
            let mut metadata_map = self.metadata.write();

            schemas.insert(version.clone(), Arc::new(schema));

            if let Some(meta) = metadata {
                metadata_map.insert(version.clone(), meta);
            } else {
                metadata_map.insert(version.clone(), SchemaVersion::new(version.clone()));
            }
        }

        // Set as current version if this is the first schema or newer than current
        {
            let mut current = self.current_version.write();
            match current.as_ref() {
                None => *current = Some(version),
                Some(current_ver) if version > *current_ver => *current = Some(version),
                _ => {}
            }
        }

        Ok(())
    }

    /// Register a named schema for easier lookup
    pub fn register_named_schema(&self, name: String, version: SemanticVersion) -> StateResult<()> {
        // Verify the version exists
        {
            let schemas = self.schemas.read();
            if !schemas.contains_key(&version) {
                return Err(StateError::MigrationError(format!(
                    "Cannot register named schema '{}': version {} not found",
                    name, version
                )));
            }
        }

        let mut named = self.named_schemas.write();
        named.insert(name, version);
        Ok(())
    }

    /// Get schema by version
    pub fn get_schema(&self, version: &SemanticVersion) -> Option<Arc<EnhancedStateSchema>> {
        let schemas = self.schemas.read();
        schemas.get(version).cloned()
    }

    /// Get schema by name
    pub fn get_schema_by_name(&self, name: &str) -> Option<Arc<EnhancedStateSchema>> {
        let named = self.named_schemas.read();
        let version = named.get(name)?;

        let schemas = self.schemas.read();
        schemas.get(version).cloned()
    }

    /// Get current (latest) schema
    pub fn get_current_schema(&self) -> Option<Arc<EnhancedStateSchema>> {
        let current = self.current_version.read();
        let version = current.as_ref()?;

        let schemas = self.schemas.read();
        schemas.get(version).cloned()
    }

    /// Get schema metadata
    pub fn get_schema_metadata(&self, version: &SemanticVersion) -> Option<SchemaVersion> {
        let metadata = self.metadata.read();
        metadata.get(version).cloned()
    }

    /// List all registered schema versions
    pub fn list_versions(&self) -> Vec<SemanticVersion> {
        let schemas = self.schemas.read();
        schemas.keys().cloned().collect()
    }

    /// List all named schemas
    pub fn list_named_schemas(&self) -> HashMap<String, SemanticVersion> {
        let named = self.named_schemas.read();
        named.clone()
    }

    /// Find schemas compatible with a given version
    pub fn find_compatible_schemas(&self, version: &SemanticVersion) -> Vec<SemanticVersion> {
        let schemas = self.schemas.read();
        schemas
            .keys()
            .filter(|v| v.is_compatible_with(version))
            .cloned()
            .collect()
    }

    /// Find schemas that require migration from a given version
    pub fn find_migration_candidates(
        &self,
        from_version: &SemanticVersion,
    ) -> Vec<SemanticVersion> {
        let schemas = self.schemas.read();
        schemas
            .keys()
            .filter(|v| *v > from_version)
            .cloned()
            .collect()
    }

    /// Get the latest version in a major version line
    pub fn get_latest_in_major(&self, major: u32) -> Option<SemanticVersion> {
        let schemas = self.schemas.read();
        schemas.keys().filter(|v| v.major == major).max().cloned()
    }

    /// Check if a version exists in the registry
    pub fn has_version(&self, version: &SemanticVersion) -> bool {
        let schemas = self.schemas.read();
        schemas.contains_key(version)
    }

    /// Remove a schema version (use with caution)
    pub fn remove_schema(&self, version: &SemanticVersion) -> Result<(), SchemaRegistryError> {
        {
            let mut schemas = self.schemas.write();
            let mut metadata = self.metadata.write();

            if !schemas.contains_key(version) {
                return Err(SchemaRegistryError::SchemaNotFound(version.to_string()));
            }

            schemas.remove(version);
            metadata.remove(version);
        }

        // Remove from named schemas if referenced
        {
            let mut named = self.named_schemas.write();
            named.retain(|_, v| v != version);
        }

        // Update current version if this was the current
        {
            let mut current = self.current_version.write();
            if current.as_ref() == Some(version) {
                let schemas = self.schemas.read();
                *current = schemas.keys().max().cloned();
            }
        }

        Ok(())
    }

    /// Set the current active schema version
    pub fn set_current_version(&self, version: SemanticVersion) -> Result<(), SchemaRegistryError> {
        {
            let schemas = self.schemas.read();
            if !schemas.contains_key(&version) {
                return Err(SchemaRegistryError::SchemaNotFound(version.to_string()));
            }
        }

        let mut current = self.current_version.write();
        *current = Some(version);
        Ok(())
    }

    /// Get statistics about the registry
    pub fn get_stats(&self) -> RegistryStats {
        let schemas = self.schemas.read();
        let named = self.named_schemas.read();
        let current = self.current_version.read();

        let total_schemas = schemas.len();
        let named_schemas = named.len();
        let current_version = current.clone();

        let major_versions: std::collections::HashSet<u32> =
            schemas.keys().map(|v| v.major).collect();
        let major_versions_count = major_versions.len();

        let latest_version = schemas.keys().max().cloned();
        let oldest_version = schemas.keys().min().cloned();

        RegistryStats {
            total_schemas,
            named_schemas,
            major_versions_count,
            current_version,
            latest_version,
            oldest_version,
        }
    }

    /// Validate a schema before registration
    fn validate_schema(&self, schema: &EnhancedStateSchema) -> Result<(), SchemaRegistryError> {
        // Basic validation checks
        if schema.fields.is_empty() && schema.version.major > 0 {
            return Err(SchemaRegistryError::ValidationFailed {
                details: "Schema version > 0.x.x cannot have empty fields".to_string(),
            });
        }

        // Check for circular dependencies
        if schema.dependencies.contains(&schema.version) {
            return Err(SchemaRegistryError::ValidationFailed {
                details: "Schema cannot depend on itself".to_string(),
            });
        }

        // Validate field schemas
        for (field_name, field_schema) in &schema.fields {
            if field_name.is_empty() {
                return Err(SchemaRegistryError::ValidationFailed {
                    details: "Field names cannot be empty".to_string(),
                });
            }

            if field_schema.field_type.is_empty() {
                return Err(SchemaRegistryError::ValidationFailed {
                    details: format!("Field '{}' must have a type", field_name),
                });
            }
        }

        Ok(())
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_schemas: usize,
    pub named_schemas: usize,
    pub major_versions_count: usize,
    pub current_version: Option<SemanticVersion>,
    pub latest_version: Option<SemanticVersion>,
    pub oldest_version: Option<SemanticVersion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FieldSchema;

    fn create_test_schema(version: SemanticVersion) -> EnhancedStateSchema {
        let mut schema = EnhancedStateSchema::new(version);
        schema.add_field(
            "test_field".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                required: true,
                default_value: None,
                validators: vec![],
            },
        );
        schema
    }

    #[test]
    fn test_registry_creation() {
        let registry = SchemaRegistry::new();
        let stats = registry.get_stats();

        assert_eq!(stats.total_schemas, 0);
        assert_eq!(stats.named_schemas, 0);
        assert_eq!(stats.current_version, None);
    }

    #[test]
    fn test_schema_registration() {
        let registry = SchemaRegistry::new();
        let schema = create_test_schema(SemanticVersion::new(1, 0, 0));
        let version = schema.version.clone();

        registry.register_schema(schema, None).unwrap();

        assert!(registry.has_version(&version));
        assert_eq!(registry.get_stats().total_schemas, 1);

        let retrieved = registry.get_schema(&version).unwrap();
        assert_eq!(retrieved.version, version);
    }

    #[test]
    fn test_version_conflict() {
        let registry = SchemaRegistry::new();
        let schema1 = create_test_schema(SemanticVersion::new(1, 0, 0));
        let schema2 = create_test_schema(SemanticVersion::new(1, 0, 0));

        registry.register_schema(schema1, None).unwrap();

        let result = registry.register_schema(schema2, None);
        assert!(matches!(
            result,
            Err(SchemaRegistryError::VersionConflict { .. })
        ));
    }

    #[test]
    fn test_named_schema_registration() {
        let registry = SchemaRegistry::new();
        let schema = create_test_schema(SemanticVersion::new(1, 0, 0));
        let version = schema.version.clone();

        registry.register_schema(schema, None).unwrap();
        registry
            .register_named_schema("test".to_string(), version.clone())
            .unwrap();

        let retrieved = registry.get_schema_by_name("test").unwrap();
        assert_eq!(retrieved.version, version);

        let named_schemas = registry.list_named_schemas();
        assert_eq!(named_schemas.len(), 1);
        assert_eq!(named_schemas.get("test"), Some(&version));
    }

    #[test]
    fn test_current_version_tracking() {
        let registry = SchemaRegistry::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        // Register in non-sequential order
        registry
            .register_schema(create_test_schema(v1_1_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v1_0_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v2_0_0.clone()), None)
            .unwrap();

        // Current should be the latest (v2.0.0)
        let current = registry.get_current_schema().unwrap();
        assert_eq!(current.version, v2_0_0);

        let stats = registry.get_stats();
        assert_eq!(stats.current_version, Some(v2_0_0.clone()));
        assert_eq!(stats.latest_version, Some(v2_0_0));
        assert_eq!(stats.oldest_version, Some(v1_0_0));
    }

    #[test]
    fn test_compatibility_search() {
        let registry = SchemaRegistry::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v1_2_0 = SemanticVersion::new(1, 2, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        registry
            .register_schema(create_test_schema(v1_0_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v1_1_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v1_2_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v2_0_0.clone()), None)
            .unwrap();

        let compatible = registry.find_compatible_schemas(&v1_0_0);
        assert_eq!(compatible.len(), 3); // v1.0.0, v1.1.0 and v1.2.0
        assert!(compatible.contains(&v1_0_0)); // Version is compatible with itself
        assert!(compatible.contains(&v1_1_0));
        assert!(compatible.contains(&v1_2_0));
        assert!(!compatible.contains(&v2_0_0)); // Different major version
    }

    #[test]
    fn test_migration_candidates() {
        let registry = SchemaRegistry::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        registry
            .register_schema(create_test_schema(v1_0_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v1_1_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v2_0_0.clone()), None)
            .unwrap();

        let candidates = registry.find_migration_candidates(&v1_0_0);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&v1_1_0));
        assert!(candidates.contains(&v2_0_0));
    }

    #[test]
    fn test_latest_in_major() {
        let registry = SchemaRegistry::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_2_5 = SemanticVersion::new(1, 2, 5);
        let v1_1_3 = SemanticVersion::new(1, 1, 3);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        registry
            .register_schema(create_test_schema(v1_0_0.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v1_2_5.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v1_1_3.clone()), None)
            .unwrap();
        registry
            .register_schema(create_test_schema(v2_0_0.clone()), None)
            .unwrap();

        let latest_v1 = registry.get_latest_in_major(1).unwrap();
        assert_eq!(latest_v1, v1_2_5);

        let latest_v2 = registry.get_latest_in_major(2).unwrap();
        assert_eq!(latest_v2, v2_0_0);

        let latest_v3 = registry.get_latest_in_major(3);
        assert!(latest_v3.is_none());
    }

    #[test]
    fn test_schema_removal() {
        let registry = SchemaRegistry::new();
        let version = SemanticVersion::new(1, 0, 0);
        let schema = create_test_schema(version.clone());

        registry.register_schema(schema, None).unwrap();
        registry
            .register_named_schema("test".to_string(), version.clone())
            .unwrap();

        assert!(registry.has_version(&version));
        assert!(registry.get_schema_by_name("test").is_some());

        registry.remove_schema(&version).unwrap();

        assert!(!registry.has_version(&version));
        assert!(registry.get_schema_by_name("test").is_none());
        assert_eq!(registry.get_stats().total_schemas, 0);
    }

    #[test]
    fn test_schema_validation() {
        let registry = SchemaRegistry::new();

        // Test empty schema validation
        let mut invalid_schema = EnhancedStateSchema::new(SemanticVersion::new(1, 0, 0));
        // Empty fields should be invalid for version > 0
        let result = registry.register_schema(invalid_schema.clone(), None);
        assert!(matches!(
            result,
            Err(SchemaRegistryError::ValidationFailed { .. })
        ));

        // Test self-dependency
        invalid_schema.add_dependency(invalid_schema.version.clone());
        let result = registry.register_schema(invalid_schema, None);
        assert!(matches!(
            result,
            Err(SchemaRegistryError::ValidationFailed { .. })
        ));
    }
}
