//! Metadata indexing for efficient filtering

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::debug;

/// Metadata index for efficient filtering
///
/// This provides inverted indices for metadata fields to avoid
/// scanning all vectors during filtered queries.
#[derive(Debug, Clone)]
pub struct MetadataIndex {
    /// Inverted index: `field_name` -> `field_value` -> set of vector IDs
    field_indices: Arc<DashMap<String, DashMap<Value, HashSet<String>>>>,

    /// Track which fields are indexed
    indexed_fields: Arc<DashMap<String, IndexType>>,
}

/// Type of index for a metadata field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// Exact match index (for strings, numbers, booleans)
    Exact,
    /// Range index (for numbers)
    Range,
    /// Full-text index (for strings)
    FullText,
    /// Array contains index
    ArrayContains,
}

impl MetadataIndex {
    /// Create a new metadata index
    #[must_use]
    pub fn new() -> Self {
        Self {
            field_indices: Arc::new(DashMap::new()),
            indexed_fields: Arc::new(DashMap::new()),
        }
    }

    /// Configure which fields to index and how
    pub fn configure_field(&self, field: String, index_type: IndexType) {
        self.indexed_fields.insert(field.clone(), index_type);
        self.field_indices.entry(field).or_default();
    }

    /// Index metadata for a vector
    pub fn index_metadata(&self, vector_id: &str, metadata: &HashMap<String, Value>) {
        for (field, value) in metadata {
            // Only index configured fields
            if let Some(index_type) = self.indexed_fields.get(field) {
                match *index_type {
                    IndexType::Exact => {
                        self.index_exact_value(field, value, vector_id);
                    }
                    IndexType::Range => {
                        // For range queries, we'd need a B-tree or similar
                        // For now, just index as exact
                        self.index_exact_value(field, value, vector_id);
                    }
                    IndexType::FullText => {
                        // For full-text, we'd tokenize and index terms
                        // For now, just index as exact
                        self.index_exact_value(field, value, vector_id);
                    }
                    IndexType::ArrayContains => {
                        if let Value::Array(arr) = value {
                            for item in arr {
                                self.index_exact_value(field, item, vector_id);
                            }
                        } else {
                            self.index_exact_value(field, value, vector_id);
                        }
                    }
                }
            }
        }
    }

    /// Index an exact value
    fn index_exact_value(&self, field: &str, value: &Value, vector_id: &str) {
        if let Some(field_index) = self.field_indices.get(field) {
            field_index
                .entry(value.clone())
                .or_default()
                .insert(vector_id.to_string());
        }
    }

    /// Remove metadata for a vector
    pub fn remove_metadata(&self, vector_id: &str, metadata: &HashMap<String, Value>) {
        for (field, value) in metadata {
            if let Some(field_index) = self.field_indices.get(field) {
                if let Some(mut value_set) = field_index.get_mut(value) {
                    value_set.remove(vector_id);
                    if value_set.is_empty() {
                        field_index.remove(value);
                    }
                }
            }
        }
    }

    /// Find vectors matching metadata filters
    pub fn find_matching_vectors(&self, filters: &HashMap<String, Value>) -> HashSet<String> {
        let mut result_set: Option<HashSet<String>> = None;

        for (field, value) in filters {
            if let Some(field_index) = self.field_indices.get(field) {
                if let Some(matching_vectors) = field_index.get(value) {
                    let vectors = matching_vectors.clone();

                    if let Some(ref mut current_set) = result_set {
                        // Intersect with existing results
                        current_set.retain(|id| vectors.contains(id));
                    } else {
                        // First filter
                        result_set = Some(vectors);
                    }
                } else {
                    // No vectors match this filter
                    return HashSet::new();
                }
            } else {
                // Field not indexed, can't use index for this filter
                debug!("Field {} not indexed, falling back to scan", field);
            }
        }

        result_set.unwrap_or_default()
    }

    /// Check if a single vector matches filters (for unindexed fields)
    #[must_use]
    pub fn matches_filters(
        metadata: &HashMap<String, Value>,
        filters: &HashMap<String, Value>,
    ) -> bool {
        filters
            .iter()
            .all(|(key, value)| metadata.get(key) == Some(value))
    }
}

impl Default for MetadataIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Query optimizer for metadata filtering
#[derive(Debug)]
pub struct MetadataQueryOptimizer {
    index: MetadataIndex,
}

impl MetadataQueryOptimizer {
    /// Create a new query optimizer
    #[must_use]
    pub const fn new(index: MetadataIndex) -> Self {
        Self { index }
    }

    /// Optimize a query with metadata filters
    ///
    /// Returns a set of vector IDs that match the filters,
    /// or None if index can't be used for all filters
    #[must_use]
    pub fn optimize_query(&self, filters: &HashMap<String, Value>) -> Option<HashSet<String>> {
        // Check if all filters can use indices
        let all_indexed = filters
            .keys()
            .all(|field| self.index.indexed_fields.contains_key(field));

        if all_indexed {
            Some(self.index.find_matching_vectors(filters))
        } else {
            None
        }
    }

    /// Get query statistics
    #[must_use]
    pub fn query_stats(&self) -> QueryStats {
        let mut stats = QueryStats::default();

        for field_entry in self.index.field_indices.iter() {
            stats.indexed_fields += 1;
            stats.total_values += field_entry.value().len();

            for value_entry in field_entry.value() {
                stats.total_entries += value_entry.value().len();
            }
        }

        stats
    }
}

/// Statistics about the metadata index
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryStats {
    /// Number of indexed fields
    pub indexed_fields: usize,

    /// Total number of distinct values across all fields
    pub total_values: usize,

    /// Total number of entries (vector-field-value mappings)
    pub total_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_index_exact_match() {
        let index = MetadataIndex::new();
        index.configure_field("type".to_string(), IndexType::Exact);

        // Index some metadata
        let mut metadata1 = HashMap::new();
        metadata1.insert("type".to_string(), Value::String("document".to_string()));
        index.index_metadata("vec1", &metadata1);

        let mut metadata2 = HashMap::new();
        metadata2.insert("type".to_string(), Value::String("image".to_string()));
        index.index_metadata("vec2", &metadata2);

        // Query for documents
        let mut filters = HashMap::new();
        filters.insert("type".to_string(), Value::String("document".to_string()));

        let results = index.find_matching_vectors(&filters);
        assert_eq!(results.len(), 1);
        assert!(results.contains("vec1"));
    }

    #[test]
    fn test_metadata_index_array_contains() {
        let index = MetadataIndex::new();
        index.configure_field("tags".to_string(), IndexType::ArrayContains);

        // Index metadata with arrays
        let mut metadata = HashMap::new();
        metadata.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("rust".to_string()),
                Value::String("llm".to_string()),
            ]),
        );
        index.index_metadata("vec1", &metadata);

        // Query for a tag
        let mut filters = HashMap::new();
        filters.insert("tags".to_string(), Value::String("rust".to_string()));

        let results = index.find_matching_vectors(&filters);
        assert_eq!(results.len(), 1);
        assert!(results.contains("vec1"));
    }

    #[test]
    fn test_query_optimizer() {
        let index = MetadataIndex::new();
        index.configure_field("type".to_string(), IndexType::Exact);

        let optimizer = MetadataQueryOptimizer::new(index);

        // Indexed field should use optimization
        let mut filters = HashMap::new();
        filters.insert("type".to_string(), Value::String("document".to_string()));
        assert!(optimizer.optimize_query(&filters).is_some());

        // Unindexed field should return None
        let mut filters = HashMap::new();
        filters.insert("unknown".to_string(), Value::String("value".to_string()));
        assert!(optimizer.optimize_query(&filters).is_none());
    }
}
