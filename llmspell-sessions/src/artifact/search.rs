//! ABOUTME: Artifact search and query functionality for efficient artifact discovery
//! ABOUTME: Provides filtering, sorting, and pagination for artifact collections

use super::metadata::MetadataIndex;
use super::types::{ArtifactId, ArtifactMetadata, ArtifactType};
use crate::SessionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Sort order for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by creation date (newest first)
    DateDesc,
    /// Sort by creation date (oldest first)
    DateAsc,
    /// Sort by name (A-Z)
    NameAsc,
    /// Sort by name (Z-A)
    NameDesc,
    /// Sort by size (largest first)
    SizeDesc,
    /// Sort by size (smallest first)
    SizeAsc,
    /// Sort by type then name
    TypeThenName,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::DateDesc
    }
}

/// Extended artifact query with sorting and pagination
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactSearchQuery {
    /// Filter by specific session ID
    pub session_id: Option<SessionId>,
    /// Filter by artifact type
    pub artifact_type: Option<ArtifactType>,
    /// Filter by name pattern (substring match, case-insensitive)
    pub name_pattern: Option<String>,
    /// Filter by tags (artifacts must have all specified tags)
    pub tags: Vec<String>,
    /// Filter by creation date (after)
    pub created_after: Option<DateTime<Utc>>,
    /// Filter by creation date (before)
    pub created_before: Option<DateTime<Utc>>,
    /// Minimum artifact size in bytes
    pub min_size: Option<usize>,
    /// Maximum artifact size in bytes
    pub max_size: Option<usize>,
    /// Sort order for results
    pub sort_order: SortOrder,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// Search result with pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching artifacts
    pub artifacts: Vec<ArtifactMetadata>,
    /// Total number of matches (before pagination)
    pub total_count: usize,
    /// Offset used in the query
    pub offset: usize,
    /// Whether there are more results
    pub has_more: bool,
}

/// Artifact search engine
pub struct ArtifactSearch {
    /// The metadata index to search
    index: MetadataIndex,
}

impl ArtifactSearch {
    /// Create a new search engine with the given index
    pub fn new(index: MetadataIndex) -> Self {
        Self { index }
    }

    /// Search artifacts with the given query
    pub fn search(&self, query: &ArtifactSearchQuery) -> SearchResult {
        // First, collect all matching metadata with their IDs
        let mut matches: Vec<(&ArtifactId, &ArtifactMetadata)> = self
            .index
            .metadata_cache
            .iter()
            .filter(|(id, metadata)| self.matches_query(id, metadata, query))
            .collect();

        let total_count = matches.len();

        // Apply sorting
        self.sort_results(&mut matches, query.sort_order);

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100); // Default limit

        let paginated_matches: Vec<ArtifactMetadata> = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(_, metadata)| metadata.clone())
            .collect();

        let has_more = offset + paginated_matches.len() < total_count;

        SearchResult {
            artifacts: paginated_matches,
            total_count,
            offset,
            has_more,
        }
    }

    /// Get artifacts by session ID
    pub fn get_by_session(&self, session_id: &SessionId) -> Vec<ArtifactMetadata> {
        self.index
            .by_session
            .get(session_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.index.metadata_cache.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get artifacts by type
    pub fn get_by_type(&self, artifact_type: &ArtifactType) -> Vec<ArtifactMetadata> {
        self.index
            .by_type
            .get(artifact_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.index.metadata_cache.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get artifacts by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<ArtifactMetadata> {
        self.index
            .by_tag
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.index.metadata_cache.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Count artifacts by session
    pub fn count_by_session(&self, session_id: &SessionId) -> usize {
        self.index
            .by_session
            .get(session_id)
            .map_or(0, std::collections::HashSet::len)
    }

    /// Count artifacts by type
    pub fn count_by_type(&self, artifact_type: &ArtifactType) -> usize {
        self.index
            .by_type
            .get(artifact_type)
            .map_or(0, std::collections::HashSet::len)
    }

    /// Get all unique tags
    pub fn get_all_tags(&self) -> Vec<String> {
        self.index.by_tag.keys().cloned().collect()
    }

    /// Check if a metadata entry matches the query
    fn matches_query(
        &self,
        artifact_id: &ArtifactId,
        metadata: &ArtifactMetadata,
        query: &ArtifactSearchQuery,
    ) -> bool {
        // Check session filter
        if let Some(ref session_id) = query.session_id {
            if artifact_id.session_id != *session_id {
                return false;
            }
        }

        // Check artifact type
        if let Some(ref artifact_type) = query.artifact_type {
            if metadata.artifact_type != *artifact_type {
                return false;
            }
        }

        // Check name pattern (case-insensitive substring match)
        if let Some(ref pattern) = query.name_pattern {
            let pattern_lower = pattern.to_lowercase();
            if !metadata.name.to_lowercase().contains(&pattern_lower) {
                return false;
            }
        }

        // Check tags (must have all specified tags)
        if !query.tags.is_empty() {
            for required_tag in &query.tags {
                if !metadata.tags.contains(required_tag) {
                    return false;
                }
            }
        }

        // Check creation date range
        if let Some(created_after) = query.created_after {
            if metadata.created_at < created_after {
                return false;
            }
        }

        if let Some(created_before) = query.created_before {
            if metadata.created_at > created_before {
                return false;
            }
        }

        // Check size range
        if let Some(min_size) = query.min_size {
            let actual_size = metadata.original_size.unwrap_or(metadata.size);
            if actual_size < min_size {
                return false;
            }
        }

        if let Some(max_size) = query.max_size {
            let actual_size = metadata.original_size.unwrap_or(metadata.size);
            if actual_size > max_size {
                return false;
            }
        }

        true
    }

    /// Sort results according to the specified order
    fn sort_results(&self, results: &mut [(&ArtifactId, &ArtifactMetadata)], order: SortOrder) {
        match order {
            SortOrder::DateDesc => {
                results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
            }
            SortOrder::DateAsc => {
                results.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));
            }
            SortOrder::NameAsc => {
                results.sort_by(|a, b| a.1.name.cmp(&b.1.name));
            }
            SortOrder::NameDesc => {
                results.sort_by(|a, b| b.1.name.cmp(&a.1.name));
            }
            SortOrder::SizeDesc => {
                results.sort_by(|a, b| {
                    let size_a = a.1.original_size.unwrap_or(a.1.size);
                    let size_b = b.1.original_size.unwrap_or(b.1.size);
                    size_b.cmp(&size_a)
                });
            }
            SortOrder::SizeAsc => {
                results.sort_by(|a, b| {
                    let size_a = a.1.original_size.unwrap_or(a.1.size);
                    let size_b = b.1.original_size.unwrap_or(b.1.size);
                    size_a.cmp(&size_b)
                });
            }
            SortOrder::TypeThenName => {
                results.sort_by(|a, b| match a.1.artifact_type.cmp(&b.1.artifact_type) {
                    Ordering::Equal => a.1.name.cmp(&b.1.name),
                    other => other,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::types::ArtifactId;

    fn create_test_metadata(
        name: &str,
        artifact_type: ArtifactType,
        size: usize,
        tags: Vec<String>,
    ) -> (ArtifactId, ArtifactMetadata) {
        let session_id = SessionId::new();
        let artifact_id = ArtifactId::new(format!("hash_{}", name), session_id, 1);
        let mut metadata = ArtifactMetadata::new(name.to_string(), artifact_type, size);
        metadata.tags = tags;
        (artifact_id, metadata)
    }
    #[test]
    fn test_search_by_name_pattern() {
        let mut index = MetadataIndex::new();

        // Add test data
        let (id1, meta1) =
            create_test_metadata("test_file.txt", ArtifactType::UserInput, 1024, vec![]);
        let (id2, meta2) =
            create_test_metadata("config.json", ArtifactType::SystemGenerated, 512, vec![]);
        let (id3, meta3) =
            create_test_metadata("test_output.log", ArtifactType::ToolResult, 2048, vec![]);

        index.add_metadata(id1, meta1);
        index.add_metadata(id2, meta2);
        index.add_metadata(id3, meta3);

        let search = ArtifactSearch::new(index);

        // Search for files with "test" in the name
        let query = ArtifactSearchQuery {
            name_pattern: Some("test".to_string()),
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.total_count, 2);
        assert!(result.artifacts.iter().all(|a| a.name.contains("test")));
    }
    #[test]
    fn test_search_by_type() {
        let mut index = MetadataIndex::new();

        // Add test data
        let (id1, meta1) =
            create_test_metadata("input1.txt", ArtifactType::UserInput, 1024, vec![]);
        let (id2, meta2) = create_test_metadata("input2.txt", ArtifactType::UserInput, 512, vec![]);
        let (id3, meta3) =
            create_test_metadata("output.log", ArtifactType::ToolResult, 2048, vec![]);

        index.add_metadata(id1, meta1);
        index.add_metadata(id2, meta2);
        index.add_metadata(id3, meta3);

        let search = ArtifactSearch::new(index);

        // Search for UserInput artifacts
        let result = search.get_by_type(&ArtifactType::UserInput);
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .all(|a| a.artifact_type == ArtifactType::UserInput));
    }
    #[test]
    fn test_search_with_tags() {
        let mut index = MetadataIndex::new();

        // Add test data
        let (id1, meta1) = create_test_metadata(
            "important.txt",
            ArtifactType::UserInput,
            1024,
            vec!["important".to_string(), "reviewed".to_string()],
        );
        let (id2, meta2) = create_test_metadata(
            "draft.txt",
            ArtifactType::UserInput,
            512,
            vec!["draft".to_string()],
        );
        let (id3, meta3) = create_test_metadata(
            "final.txt",
            ArtifactType::UserInput,
            2048,
            vec!["important".to_string(), "final".to_string()],
        );

        index.add_metadata(id1, meta1);
        index.add_metadata(id2, meta2);
        index.add_metadata(id3, meta3);

        let search = ArtifactSearch::new(index);

        // Search for artifacts with "important" tag
        let query = ArtifactSearchQuery {
            tags: vec!["important".to_string()],
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.total_count, 2);
        assert!(result
            .artifacts
            .iter()
            .all(|a| a.tags.contains(&"important".to_string())));
    }
    #[test]
    fn test_search_with_size_filters() {
        let mut index = MetadataIndex::new();

        // Add test data
        let (id1, meta1) = create_test_metadata("small.txt", ArtifactType::UserInput, 512, vec![]);
        let (id2, meta2) =
            create_test_metadata("medium.txt", ArtifactType::UserInput, 1024, vec![]);
        let (id3, meta3) = create_test_metadata("large.txt", ArtifactType::UserInput, 4096, vec![]);

        index.add_metadata(id1, meta1);
        index.add_metadata(id2, meta2);
        index.add_metadata(id3, meta3);

        let search = ArtifactSearch::new(index);

        // Search for artifacts between 600 and 2000 bytes
        let query = ArtifactSearchQuery {
            min_size: Some(600),
            max_size: Some(2000),
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.total_count, 1);
        assert_eq!(result.artifacts[0].name, "medium.txt");
    }
    #[test]
    fn test_search_sorting() {
        let mut index = MetadataIndex::new();

        // Add test data
        let (id1, meta1) = create_test_metadata("zebra.txt", ArtifactType::UserInput, 512, vec![]);
        let (id2, meta2) = create_test_metadata("alpha.txt", ArtifactType::UserInput, 1024, vec![]);
        let (id3, meta3) = create_test_metadata("beta.txt", ArtifactType::UserInput, 256, vec![]);

        index.add_metadata(id1, meta1);
        index.add_metadata(id2, meta2);
        index.add_metadata(id3, meta3);

        let search = ArtifactSearch::new(index);

        // Search with name ascending sort
        let query = ArtifactSearchQuery {
            sort_order: SortOrder::NameAsc,
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.artifacts[0].name, "alpha.txt");
        assert_eq!(result.artifacts[1].name, "beta.txt");
        assert_eq!(result.artifacts[2].name, "zebra.txt");

        // Search with size descending sort
        let query = ArtifactSearchQuery {
            sort_order: SortOrder::SizeDesc,
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.artifacts[0].size, 1024); // alpha.txt
        assert_eq!(result.artifacts[1].size, 512); // zebra.txt
        assert_eq!(result.artifacts[2].size, 256); // beta.txt
    }
    #[test]
    fn test_search_pagination() {
        let mut index = MetadataIndex::new();

        // Add test data
        for i in 0..10 {
            let (id, meta) = create_test_metadata(
                &format!("file_{}.txt", i),
                ArtifactType::UserInput,
                1024 * (i + 1),
                vec![],
            );
            index.add_metadata(id, meta);
        }

        let search = ArtifactSearch::new(index);

        // First page
        let query = ArtifactSearchQuery {
            offset: Some(0),
            limit: Some(3),
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.artifacts.len(), 3);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);

        // Second page
        let query = ArtifactSearchQuery {
            offset: Some(3),
            limit: Some(3),
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.artifacts.len(), 3);
        assert!(result.has_more);

        // Last page
        let query = ArtifactSearchQuery {
            offset: Some(9),
            limit: Some(3),
            ..Default::default()
        };

        let result = search.search(&query);
        assert_eq!(result.artifacts.len(), 1);
        assert!(!result.has_more);
    }
}
