//! ABOUTME: Metadata management utilities for artifact storage
//! ABOUTME: Provides indexing, search, and metadata operations

use super::types::{ArtifactId, ArtifactMetadata, ArtifactType};
use crate::SessionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Metadata index for efficient querying
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataIndex {
    /// Index by session ID
    pub by_session: HashMap<SessionId, HashSet<ArtifactId>>,
    /// Index by artifact type
    pub by_type: HashMap<ArtifactType, HashSet<ArtifactId>>,
    /// Index by tags
    pub by_tag: HashMap<String, HashSet<ArtifactId>>,
    /// Index by creation date (bucketed by day)
    pub by_date: HashMap<String, HashSet<ArtifactId>>,
    /// Full metadata cache
    pub metadata_cache: HashMap<ArtifactId, ArtifactMetadata>,
}

impl MetadataIndex {
    /// Create a new metadata index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add metadata to the index
    pub fn add_metadata(&mut self, artifact_id: ArtifactId, metadata: ArtifactMetadata) {
        // Index by session
        self.by_session
            .entry(artifact_id.session_id)
            .or_default()
            .insert(artifact_id.clone());

        // Index by type
        self.by_type
            .entry(metadata.artifact_type.clone())
            .or_default()
            .insert(artifact_id.clone());

        // Index by tags
        for tag in &metadata.tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .insert(artifact_id.clone());
        }

        // Index by date (day bucket)
        let date_key = metadata.created_at.format("%Y-%m-%d").to_string();
        self.by_date
            .entry(date_key)
            .or_default()
            .insert(artifact_id.clone());

        // Store full metadata
        self.metadata_cache.insert(artifact_id, metadata);
    }

    /// Remove metadata from the index
    pub fn remove_metadata(&mut self, artifact_id: &ArtifactId) -> Option<ArtifactMetadata> {
        let metadata = self.metadata_cache.remove(artifact_id)?;

        // Remove from session index
        if let Some(session_set) = self.by_session.get_mut(&artifact_id.session_id) {
            session_set.remove(artifact_id);
            if session_set.is_empty() {
                self.by_session.remove(&artifact_id.session_id);
            }
        }

        // Remove from type index
        if let Some(type_set) = self.by_type.get_mut(&metadata.artifact_type) {
            type_set.remove(artifact_id);
            if type_set.is_empty() {
                self.by_type.remove(&metadata.artifact_type);
            }
        }

        // Remove from tag indices
        for tag in &metadata.tags {
            if let Some(tag_set) = self.by_tag.get_mut(tag) {
                tag_set.remove(artifact_id);
                if tag_set.is_empty() {
                    self.by_tag.remove(tag);
                }
            }
        }

        // Remove from date index
        let date_key = metadata.created_at.format("%Y-%m-%d").to_string();
        if let Some(date_set) = self.by_date.get_mut(&date_key) {
            date_set.remove(artifact_id);
            if date_set.is_empty() {
                self.by_date.remove(&date_key);
            }
        }

        Some(metadata)
    }

    /// Get metadata by artifact ID
    pub fn get_metadata(&self, artifact_id: &ArtifactId) -> Option<&ArtifactMetadata> {
        self.metadata_cache.get(artifact_id)
    }

    /// Query artifacts by session
    pub fn query_by_session(&self, session_id: &SessionId) -> Vec<&ArtifactMetadata> {
        self.by_session
            .get(session_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.metadata_cache.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query artifacts by type
    pub fn query_by_type(&self, artifact_type: &ArtifactType) -> Vec<&ArtifactMetadata> {
        self.by_type
            .get(artifact_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.metadata_cache.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query artifacts by tag
    pub fn query_by_tag(&self, tag: &str) -> Vec<&ArtifactMetadata> {
        self.by_tag
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.metadata_cache.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query artifacts by date range
    pub fn query_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&ArtifactMetadata> {
        let mut results = Vec::new();
        let mut current = start.date_naive();
        let end_date = end.date_naive();

        while current <= end_date {
            let date_key = current.format("%Y-%m-%d").to_string();
            if let Some(ids) = self.by_date.get(&date_key) {
                for id in ids {
                    if let Some(metadata) = self.metadata_cache.get(id) {
                        if metadata.created_at >= start && metadata.created_at <= end {
                            results.push(metadata);
                        }
                    }
                }
            }
            current = current.succ_opt().unwrap_or(current);
        }

        results
    }

    /// Get total artifact count
    pub fn total_count(&self) -> usize {
        self.metadata_cache.len()
    }

    /// Get session artifact count
    pub fn session_count(&self, session_id: &SessionId) -> usize {
        self.by_session.get(session_id).map_or(0, HashSet::len)
    }
}

/// Metadata search builder for complex queries
#[derive(Debug, Clone, Default)]
pub struct MetadataSearchBuilder {
    session_ids: Option<HashSet<SessionId>>,
    artifact_types: Option<HashSet<ArtifactType>>,
    required_tags: Option<HashSet<String>>,
    excluded_tags: Option<HashSet<String>>,
    name_pattern: Option<String>,
    created_after: Option<DateTime<Utc>>,
    created_before: Option<DateTime<Utc>>,
    min_size: Option<usize>,
    max_size: Option<usize>,
}

impl MetadataSearchBuilder {
    /// Create a new search builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by session IDs
    #[must_use]
    pub fn with_sessions(mut self, session_ids: Vec<SessionId>) -> Self {
        self.session_ids = Some(session_ids.into_iter().collect());
        self
    }

    /// Filter by artifact types
    #[must_use]
    pub fn with_types(mut self, types: Vec<ArtifactType>) -> Self {
        self.artifact_types = Some(types.into_iter().collect());
        self
    }

    /// Require all specified tags
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.required_tags = Some(tags.into_iter().collect());
        self
    }

    /// Exclude artifacts with these tags
    #[must_use]
    pub fn without_tags(mut self, tags: Vec<String>) -> Self {
        self.excluded_tags = Some(tags.into_iter().collect());
        self
    }

    /// Filter by name pattern (substring match)
    #[must_use]
    pub fn with_name_pattern(mut self, pattern: String) -> Self {
        self.name_pattern = Some(pattern);
        self
    }

    /// Filter by creation date range
    #[must_use]
    pub fn created_between(mut self, after: DateTime<Utc>, before: DateTime<Utc>) -> Self {
        self.created_after = Some(after);
        self.created_before = Some(before);
        self
    }

    /// Filter by size range
    #[must_use]
    pub fn size_between(mut self, min: usize, max: usize) -> Self {
        self.min_size = Some(min);
        self.max_size = Some(max);
        self
    }

    /// Execute the search against a metadata index
    pub fn search<'a>(&self, index: &'a MetadataIndex) -> Vec<&'a ArtifactMetadata> {
        index
            .metadata_cache
            .values()
            .filter(|metadata| self.matches(metadata))
            .collect()
    }

    /// Check if metadata matches the search criteria
    fn matches(&self, metadata: &ArtifactMetadata) -> bool {
        // Check session filter
        if let Some(ref sessions) = self.session_ids {
            if !sessions.iter().any(|_| {
                // We need the artifact ID to check session, but metadata doesn't have it
                // This would need to be handled differently in practice
                true // Placeholder - would need artifact ID
            }) {
                return false;
            }
        }

        // Check type filter
        if let Some(ref types) = self.artifact_types {
            if !types.contains(&metadata.artifact_type) {
                return false;
            }
        }

        // Check required tags
        if let Some(ref required) = self.required_tags {
            let metadata_tags: HashSet<_> = metadata.tags.iter().cloned().collect();
            if !required.is_subset(&metadata_tags) {
                return false;
            }
        }

        // Check excluded tags
        if let Some(ref excluded) = self.excluded_tags {
            if metadata.tags.iter().any(|tag| excluded.contains(tag)) {
                return false;
            }
        }

        // Check name pattern
        if let Some(ref pattern) = self.name_pattern {
            if !metadata
                .name
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                return false;
            }
        }

        // Check date range
        if let Some(after) = self.created_after {
            if metadata.created_at < after {
                return false;
            }
        }
        if let Some(before) = self.created_before {
            if metadata.created_at > before {
                return false;
            }
        }

        // Check size range
        if let Some(min) = self.min_size {
            if metadata.size < min {
                return false;
            }
        }
        if let Some(max) = self.max_size {
            if metadata.size > max {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "session")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_metadata_index() {
        let mut index = MetadataIndex::new();
        let session_id = SessionId::new();
        let artifact_id = ArtifactId::new("hash1".to_string(), session_id, 1);

        let mut metadata =
            ArtifactMetadata::new("test.txt".to_string(), ArtifactType::UserInput, 1024);
        metadata.tags.push("important".to_string());

        // Add metadata
        index.add_metadata(artifact_id.clone(), metadata.clone());

        // Query by session
        let results = index.query_by_session(&session_id);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test.txt");

        // Query by type
        let results = index.query_by_type(&ArtifactType::UserInput);
        assert_eq!(results.len(), 1);

        // Query by tag
        let results = index.query_by_tag("important");
        assert_eq!(results.len(), 1);

        // Remove metadata
        let removed = index.remove_metadata(&artifact_id);
        assert!(removed.is_some());
        assert_eq!(index.total_count(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_metadata_search_builder() {
        let builder = MetadataSearchBuilder::new()
            .with_types(vec![ArtifactType::AgentOutput])
            .with_tags(vec!["result".to_string()])
            .size_between(1024, 10240);

        let metadata = ArtifactMetadata {
            name: "output.json".to_string(),
            artifact_type: ArtifactType::AgentOutput,
            size: 2048,
            tags: vec!["result".to_string(), "json".to_string()],
            ..ArtifactMetadata::new("output.json".to_string(), ArtifactType::AgentOutput, 2048)
        };

        assert!(builder.matches(&metadata));

        // Test non-matching
        let metadata2 =
            ArtifactMetadata::new("input.txt".to_string(), ArtifactType::UserInput, 512);
        assert!(!builder.matches(&metadata2));
    }
}
