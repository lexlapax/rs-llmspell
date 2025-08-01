//! ABOUTME: Artifact state correlation for tracking relationships between state and artifacts
//! ABOUTME: Provides correlation IDs and metadata for artifact-state relationships

use crate::types::ComponentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Unique identifier for artifacts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub String);

impl ArtifactId {
    /// Create a new artifact ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new random artifact ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Unique identifier for the artifact
    pub id: ArtifactId,

    /// Type of artifact (e.g., "code", "document", "image", "output")
    pub artifact_type: String,

    /// Human-readable name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Last modification timestamp
    pub modified_at: SystemTime,

    /// Size in bytes (if applicable)
    pub size_bytes: Option<u64>,

    /// MIME type (if applicable)
    pub mime_type: Option<String>,

    /// Agent/component that created this artifact
    pub created_by: ComponentId,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl ArtifactMetadata {
    /// Create new artifact metadata
    pub fn new(
        id: ArtifactId,
        artifact_type: String,
        name: String,
        created_by: ComponentId,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            artifact_type,
            name,
            description: None,
            created_at: now,
            modified_at: now,
            size_bytes: None,
            mime_type: None,
            created_by,
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Add a tag to the artifact
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Add custom metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }
}

/// Correlation between state operations and artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCorrelation {
    /// Correlation ID linking state and artifact
    pub correlation_id: String,

    /// ID of the artifact
    pub artifact_id: ArtifactId,

    /// Component that owns the state
    pub component_id: ComponentId,

    /// State operation that created/modified the artifact
    pub operation: StateOperation,

    /// Timestamp of the correlation
    pub timestamp: SystemTime,

    /// Optional parent artifact (for derived artifacts)
    pub parent_artifact: Option<ArtifactId>,

    /// Relationship type to parent
    pub relationship: Option<ArtifactRelationship>,
}

/// Types of state operations that can affect artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateOperation {
    /// Artifact was created
    Created,

    /// Artifact was modified
    Modified,

    /// Artifact was deleted
    Deleted,

    /// Artifact was referenced in state
    Referenced,

    /// Artifact was derived from another
    Derived,

    /// Custom operation
    Custom(String),
}

/// Relationship types between artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactRelationship {
    /// This artifact was derived from parent
    DerivedFrom,

    /// This artifact is a version of parent
    VersionOf,

    /// This artifact is part of parent
    PartOf,

    /// This artifact references parent
    References,

    /// Custom relationship
    Custom(String),
}

/// Manages artifact correlations
#[derive(Debug, Clone)]
pub struct ArtifactCorrelationManager {
    /// Storage for correlations
    correlations: Arc<RwLock<HashMap<String, ArtifactCorrelation>>>,

    /// Index by artifact ID
    artifact_index: Arc<RwLock<HashMap<ArtifactId, Vec<String>>>>,

    /// Index by component ID
    component_index: Arc<RwLock<HashMap<ComponentId, Vec<String>>>>,
}

impl ArtifactCorrelationManager {
    /// Create a new correlation manager
    pub fn new() -> Self {
        Self {
            correlations: Arc::new(RwLock::new(HashMap::new())),
            artifact_index: Arc::new(RwLock::new(HashMap::new())),
            component_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new correlation
    pub async fn add_correlation(&self, correlation: ArtifactCorrelation) {
        let correlation_id = correlation.correlation_id.clone();
        let artifact_id = correlation.artifact_id.clone();
        let component_id = correlation.component_id;

        // Store correlation
        self.correlations
            .write()
            .await
            .insert(correlation_id.clone(), correlation);

        // Update artifact index
        self.artifact_index
            .write()
            .await
            .entry(artifact_id)
            .or_insert_with(Vec::new)
            .push(correlation_id.clone());

        // Update component index
        self.component_index
            .write()
            .await
            .entry(component_id)
            .or_insert_with(Vec::new)
            .push(correlation_id);
    }

    /// Create and add a correlation for artifact creation
    pub async fn correlate_creation(
        &self,
        artifact_id: ArtifactId,
        component_id: ComponentId,
        parent_artifact: Option<ArtifactId>,
    ) -> String {
        let correlation_id = uuid::Uuid::new_v4().to_string();
        let correlation = ArtifactCorrelation {
            correlation_id: correlation_id.clone(),
            artifact_id,
            component_id,
            operation: StateOperation::Created,
            timestamp: SystemTime::now(),
            parent_artifact: parent_artifact.clone(),
            relationship: parent_artifact
                .as_ref()
                .map(|_| ArtifactRelationship::DerivedFrom),
        };

        self.add_correlation(correlation).await;
        correlation_id
    }

    /// Get correlations for an artifact
    pub async fn get_by_artifact(&self, artifact_id: &ArtifactId) -> Vec<ArtifactCorrelation> {
        let artifact_index = self.artifact_index.read().await;
        let correlations = self.correlations.read().await;

        artifact_index
            .get(artifact_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| correlations.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get correlations for a component
    pub async fn get_by_component(&self, component_id: &ComponentId) -> Vec<ArtifactCorrelation> {
        let component_index = self.component_index.read().await;
        let correlations = self.correlations.read().await;

        component_index
            .get(component_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| correlations.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get artifacts created by a component
    pub async fn get_artifacts_by_component(&self, component_id: &ComponentId) -> Vec<ArtifactId> {
        self.get_by_component(component_id)
            .await
            .into_iter()
            .filter(|c| matches!(c.operation, StateOperation::Created))
            .map(|c| c.artifact_id)
            .collect()
    }

    /// Find artifact lineage (parent chain)
    pub async fn get_lineage(&self, artifact_id: &ArtifactId) -> Vec<ArtifactId> {
        let mut lineage = Vec::new();
        let mut current = Some(artifact_id.clone());
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = current {
            if !visited.insert(id.clone()) {
                break; // Circular reference protection
            }

            lineage.push(id.clone());

            // Find parent
            let correlations = self.get_by_artifact(&id).await;
            current = correlations
                .iter()
                .find(|c| {
                    matches!(
                        c.operation,
                        StateOperation::Created | StateOperation::Derived
                    )
                })
                .and_then(|c| c.parent_artifact.clone());
        }

        lineage
    }

    /// Clear all correlations
    pub async fn clear(&self) {
        self.correlations.write().await.clear();
        self.artifact_index.write().await.clear();
        self.component_index.write().await.clear();
    }
}

impl Default for ArtifactCorrelationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "core")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_artifact_id() {
        let id1 = ArtifactId::new("test-artifact");
        assert_eq!(id1.to_string(), "test-artifact");

        let id2 = ArtifactId::generate();
        assert!(!id2.0.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_artifact_metadata() {
        let mut metadata = ArtifactMetadata::new(
            ArtifactId::new("test"),
            "code".to_string(),
            "test.rs".to_string(),
            ComponentId::new(),
        );

        metadata.add_tag("rust");
        metadata.add_tag("test");
        metadata.add_tag("rust"); // Duplicate should not be added

        assert_eq!(metadata.tags.len(), 2);

        metadata.add_metadata("language", serde_json::json!("rust"));
        assert_eq!(metadata.metadata.get("language").unwrap(), "rust");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_correlation_manager() {
        let manager = ArtifactCorrelationManager::new();
        let component_id = ComponentId::new();
        let artifact_id = ArtifactId::new("test-artifact");

        // Create correlation
        let correlation_id = manager
            .correlate_creation(artifact_id.clone(), component_id.clone(), None)
            .await;

        // Verify correlation exists
        let by_artifact = manager.get_by_artifact(&artifact_id).await;
        assert_eq!(by_artifact.len(), 1);
        assert_eq!(by_artifact[0].correlation_id, correlation_id);

        let by_component = manager.get_by_component(&component_id).await;
        assert_eq!(by_component.len(), 1);

        let artifacts = manager.get_artifacts_by_component(&component_id).await;
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0], artifact_id);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_artifact_lineage() {
        let manager = ArtifactCorrelationManager::new();
        let component_id = ComponentId::new();

        // Create parent artifact
        let parent_id = ArtifactId::new("parent");
        manager
            .correlate_creation(parent_id.clone(), component_id.clone(), None)
            .await;

        // Create child artifact
        let child_id = ArtifactId::new("child");
        manager
            .correlate_creation(
                child_id.clone(),
                component_id.clone(),
                Some(parent_id.clone()),
            )
            .await;

        // Create grandchild artifact
        let grandchild_id = ArtifactId::new("grandchild");
        manager
            .correlate_creation(grandchild_id.clone(), component_id, Some(child_id.clone()))
            .await;

        // Check lineage
        let lineage = manager.get_lineage(&grandchild_id).await;
        assert_eq!(lineage.len(), 3);
        assert_eq!(lineage[0], grandchild_id);
        assert_eq!(lineage[1], child_id);
        assert_eq!(lineage[2], parent_id);
    }
}
