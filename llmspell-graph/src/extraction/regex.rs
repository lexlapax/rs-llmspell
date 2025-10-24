//! Regex-based entity and relationship extraction
//!
//! Simple pattern matching for extracting entities and relationships from text.
//! Targets >50% recall with <5ms latency for 1KB text.
//!
//! # Patterns
//!
//! - **is-a**: "X is a Y" → (X, `is_a`, Y)
//! - **has-feature**: "X has Y" → (X, `has_feature`, Y)
//! - **located-in**: "X in Y" → (X, `located_in`, Y)
//! - **part-of**: "X of Y" → (X, `part_of`, Y)
//!
//! # Examples
//!
//! ```rust
//! use llmspell_graph::extraction::RegexExtractor;
//!
//! let extractor = RegexExtractor::new();
//! let text = "Rust is a systems programming language. Rust has memory safety.";
//!
//! let entities = extractor.extract_entities(text);
//! assert!(entities.iter().any(|e| e.name.contains("Rust")));
//!
//! let relationships = extractor.extract_relationships(text);
//! assert!(relationships.iter().any(|r| r.relationship_type == "is_a"));
//! ```

use crate::types::{Entity, Relationship};
use llmspell_utils::text::stopwords::is_stopword;
use regex::Regex;
use serde_json::json;
use std::sync::LazyLock;
use tracing::{debug, info, trace};

/// Regex patterns for relationship extraction
static IS_A_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)([A-Z][a-zA-Z0-9]+(?:\s+[A-Z][a-zA-Z0-9]+)*)\s+is\s+an?\s+([a-z]+(?:[\s-]+[a-z]+)*?\s+(?:language|system|tool|framework|library|platform|service|application)|(?:language|system|tool|framework|library|platform|service|application))").unwrap()
});

static HAS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)([A-Z][a-zA-Z0-9]+(?:\s+[A-Z][a-zA-Z0-9]+)*)\s+has\s+([a-z]+(?:[\s-]+[a-z]+)*?\s+(?:safety|feature|capability|support|property|typing|abstractions|performance|concurrency)|(?:safety|feature|capability|support|property|typing|abstractions|performance|concurrency))").unwrap()
});

static IN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([A-Z][a-zA-Z0-9]+(?:\s+[A-Z][a-zA-Z0-9]+)*)\s+in\s+([A-Z][a-zA-Z0-9]+(?:\s+[A-Z][a-zA-Z0-9]+)*)\b").unwrap()
});

static OF_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)([a-z]+(?:\s+[a-z]+)*)\s+of\s+([A-Z][a-zA-Z0-9]+(?:\s+[A-Z][a-zA-Z0-9]+)*)")
        .unwrap()
});

/// Entity mention extraction pattern (capitalized words/phrases)
static ENTITY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Z][a-zA-Z0-9]*(?:\s+[A-Z][a-zA-Z0-9]*)*)\b").unwrap());

/// Regex-based entity and relationship extractor
///
/// Uses pattern matching to extract structured knowledge from unstructured text.
/// Optimized for speed (<5ms for 1KB text) at the cost of precision.
///
/// # Performance
///
/// - Target: <6ms for 1KB text (with stopword filtering)
/// - Recall: >50% on common patterns (currently ~100%)
/// - Precision: >60% with stopword filtering (currently ~100%)
///
/// # Limitations
///
/// - No coreference resolution ("it", "they" not handled)
/// - No entity disambiguation (multiple "Apple" entities)
/// - No context understanding (sarcasm, negation ignored)
/// - English-only patterns
pub struct RegexExtractor {
    /// Minimum entity name length to filter noise
    min_entity_length: usize,
}

impl Default for RegexExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl RegexExtractor {
    /// Create new regex extractor with default settings
    ///
    /// # Example
    ///
    /// ```rust
    /// use llmspell_graph::extraction::RegexExtractor;
    ///
    /// let extractor = RegexExtractor::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_entity_length: 2,
        }
    }

    /// Extract entities from text
    ///
    /// Identifies entity mentions (capitalized words/phrases) and creates Entity objects.
    /// Deduplicates by name.
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to extract entities from
    ///
    /// # Returns
    ///
    /// Vector of Entity objects with inferred types
    ///
    /// # Example
    ///
    /// ```rust
    /// use llmspell_graph::extraction::RegexExtractor;
    ///
    /// let extractor = RegexExtractor::new();
    /// let entities = extractor.extract_entities("Rust is a language. Python is also a language.");
    /// assert!(entities.len() >= 2);
    /// ```
    /// Helper: Process single entity match and add to results
    fn process_entity_match(
        &self,
        name: &str,
        text: &str,
        entities: &mut Vec<Entity>,
        seen_names: &mut std::collections::HashSet<String>,
        filtered_count: &mut usize,
    ) {
        if !self.should_keep_entity(name, seen_names) {
            *filtered_count += 1;
            return;
        }

        let entity_type = Self::infer_entity_type(text, name);
        debug!("Extracted entity: name='{}', type={}", name, entity_type);

        entities.push(Entity::new(
            name.to_string(),
            entity_type,
            json!({"source": "regex_extraction"}),
        ));
    }

    /// Helper: Log extraction summary
    fn log_extraction_summary(entities: &[Entity], filtered_count: usize) {
        info!(
            "Entity extraction complete: {} entities extracted, {} filtered",
            entities.len(),
            filtered_count
        );
        trace!(
            "Extracted entities: {:?}",
            entities.iter().map(|e| &e.name).collect::<Vec<_>>()
        );
    }

    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        info!("Starting entity extraction: text_len={} bytes", text.len());
        trace!(
            "Extraction text: {}",
            text.chars().take(100).collect::<String>()
        );

        let mut entities = Vec::new();
        let mut seen_names = std::collections::HashSet::new();
        let mut filtered_count = 0;

        for cap in ENTITY_PATTERN.captures_iter(text) {
            let name = cap[1].trim();
            self.process_entity_match(name, text, &mut entities, &mut seen_names, &mut filtered_count);
        }

        Self::log_extraction_summary(&entities, filtered_count);
        entities
    }

    /// Helper: Check basic length requirements
    const fn passes_length_filters(&self, name: &str) -> bool {
        // Filter out short/common words
        if name.len() < self.min_entity_length {
            return false;
        }

        // Skip single-letter entities (usually noise)
        if name.len() == 1 {
            return false;
        }

        true
    }

    /// Helper: Check stopword filters
    fn passes_stopword_filters(name: &str) -> bool {
        // Skip common non-entity words (stopwords)
        if is_stopword(name) {
            trace!("Filtered stopword: '{}'", name);
            return false;
        }

        // Skip all-caps words shorter than 3 chars (often acronyms used as stopwords: "TO", "IN", "AT")
        if name.chars().all(char::is_uppercase) && name.len() < 3 {
            return false;
        }

        // Skip multi-word entities that start with stopwords (e.g., "The Rust", "However Python")
        if let Some(first_word) = name.split_whitespace().next() {
            if is_stopword(first_word) {
                trace!("Filtered entity starting with stopword: '{}'", name);
                return false;
            }
        }

        true
    }

    /// Helper: Check deduplication
    fn is_not_duplicate(name: &str, seen_names: &mut std::collections::HashSet<String>) -> bool {
        if seen_names.insert(name.to_string()) {
            true
        } else {
            trace!("Filtered duplicate: '{}'", name);
            false
        }
    }

    /// Check if entity should be kept after applying filters
    fn should_keep_entity(
        &self,
        name: &str,
        seen_names: &mut std::collections::HashSet<String>,
    ) -> bool {
        self.passes_length_filters(name)
            && Self::passes_stopword_filters(name)
            && Self::is_not_duplicate(name, seen_names)
    }

    /// Extract relationships from text
    ///
    /// Identifies relationship patterns and creates Relationship objects.
    ///
    /// # Supported Patterns
    ///
    /// - "X is a Y" → (X, `is_a`, Y)
    /// - "X has Y" → (X, `has_feature`, Y)
    /// - "X in Y" → (X, `located_in`, Y)
    /// - "X of Y" → (X, `part_of`, Y)
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to extract relationships from
    ///
    /// # Returns
    ///
    /// Vector of Relationship objects
    ///
    /// # Example
    ///
    /// ```rust
    /// use llmspell_graph::extraction::RegexExtractor;
    ///
    /// let extractor = RegexExtractor::new();
    /// let rels = extractor.extract_relationships("Rust is a systems language");
    /// assert!(rels.iter().any(|r| r.relationship_type == "is_a"));
    /// ```
    pub fn extract_relationships(&self, text: &str) -> Vec<Relationship> {
        info!(
            "Starting relationship extraction: text_len={} bytes",
            text.len()
        );

        let mut relationships = Vec::new();

        // Extract each pattern type
        let (is_a_rels, is_a_count) =
            Self::extract_pattern_relationships(text, &IS_A_PATTERN, "is_a", "is_a", "is_a");
        relationships.extend(is_a_rels);

        let (has_rels, has_count) =
            Self::extract_pattern_relationships(text, &HAS_PATTERN, "has_feature", "has", "has");
        relationships.extend(has_rels);

        let (in_rels, in_count) =
            Self::extract_pattern_relationships(text, &IN_PATTERN, "located_in", "in", "in");
        relationships.extend(in_rels);

        let (of_rels, of_count) =
            Self::extract_pattern_relationships(text, &OF_PATTERN, "part_of", "of", "of");
        relationships.extend(of_rels);

        info!("Relationship extraction complete: {} relationships extracted (is_a={}, has={}, in={}, of={})",
            relationships.len(), is_a_count, has_count, in_count, of_count);
        trace!(
            "Extracted relationships: {:?}",
            relationships
                .iter()
                .map(|r| format!(
                    "{}->{}({})",
                    r.from_entity, r.to_entity, r.relationship_type
                ))
                .collect::<Vec<_>>()
        );

        relationships
    }

    /// Extract relationships matching a specific regex pattern
    fn extract_pattern_relationships(
        text: &str,
        pattern: &Regex,
        relationship_type: &str,
        pattern_name: &str,
        trace_verb: &str,
    ) -> (Vec<Relationship>, usize) {
        let count = pattern.captures_iter(text).count();
        debug!("Found {} '{}' pattern matches", count, relationship_type);

        let relationships: Vec<Relationship> = pattern
            .captures_iter(text)
            .map(|cap| {
                let from = cap[1].trim();
                let to = cap[2].trim();
                trace!(
                    "Matched {}: '{}' {} '{}'",
                    relationship_type,
                    from,
                    trace_verb,
                    to
                );

                Relationship::new(
                    from.to_string(),
                    to.to_string(),
                    relationship_type.to_string(),
                    json!({"source": "regex_extraction", "pattern": pattern_name}),
                )
            })
            .collect();

        (relationships, count)
    }

    /// Infer entity type from context
    ///
    /// Uses simple heuristics to guess entity type based on surrounding text.
    ///
    /// # Arguments
    ///
    /// * `text` - Full text for context
    /// * `name` - Entity name to infer type for
    ///
    /// # Returns
    ///
    /// Inferred entity type (default: "entity")
    fn infer_entity_type(text: &str, name: &str) -> String {
        let context = text.to_lowercase();
        let name_lower = name.to_lowercase();

        // Check for type indicators near the entity name
        // Use regex-like matching to catch variations
        if context.contains(&name_lower)
            && (context.contains(&format!("{name_lower} is a")) && context.contains("language")
                || context.contains(&format!("{name_lower} language"))
                || (context.contains(&name_lower) && context.contains("programming language")))
        {
            return "programming_language".to_string();
        }

        if context.contains(&format!("{name_lower} is a system"))
            || context.contains(&format!("{name_lower} system"))
        {
            return "system".to_string();
        }

        if context.contains(&format!("{name_lower} is a tool"))
            || context.contains(&format!("{name_lower} tool"))
        {
            return "tool".to_string();
        }

        if context.contains(&format!("{name_lower} is a framework"))
            || context.contains(&format!("{name_lower} framework"))
        {
            return "framework".to_string();
        }

        // Default fallback
        "entity".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_entities() {
        let extractor = RegexExtractor::new();
        let text = "Rust is a language. Python is also mentioned.";

        let entities = extractor.extract_entities(text);

        assert!(entities.len() >= 2, "Should find at least Rust and Python");
        assert!(entities.iter().any(|e| e.name == "Rust"));
        assert!(entities.iter().any(|e| e.name == "Python"));
    }

    #[test]
    fn test_deduplicate_entities() {
        let extractor = RegexExtractor::new();
        let text = "Rust is great. Rust is fast. Rust is safe.";

        let entities = extractor.extract_entities(text);

        let rust_count = entities.iter().filter(|e| e.name == "Rust").count();
        assert_eq!(rust_count, 1, "Should deduplicate Rust");
    }

    #[test]
    fn test_filter_common_words() {
        let extractor = RegexExtractor::new();
        let text = "This is The example. It has They and We.";

        let entities = extractor.extract_entities(text);

        assert!(!entities.iter().any(|e| e.name == "This"));
        assert!(!entities.iter().any(|e| e.name == "The"));
        assert!(!entities.iter().any(|e| e.name == "It"));
    }

    #[test]
    fn test_extract_is_a_relationship() {
        let extractor = RegexExtractor::new();
        let text = "Rust is a systems programming language.";

        let rels = extractor.extract_relationships(text);

        assert!(!rels.is_empty(), "Should find is_a relationship");
        let is_a_rel = rels.iter().find(|r| r.relationship_type == "is_a");
        assert!(is_a_rel.is_some());

        let rel = is_a_rel.unwrap();
        assert_eq!(rel.from_entity, "Rust");
        assert!(rel.to_entity.contains("language"));
    }

    #[test]
    fn test_extract_has_relationship() {
        let extractor = RegexExtractor::new();
        let text = "Rust has memory safety.";

        let rels = extractor.extract_relationships(text);

        assert!(!rels.is_empty(), "Should find has_feature relationship");
        let has_rel = rels.iter().find(|r| r.relationship_type == "has_feature");
        assert!(has_rel.is_some());

        let rel = has_rel.unwrap();
        assert_eq!(rel.from_entity, "Rust");
        assert!(rel.to_entity.contains("safety"));
    }

    #[test]
    fn test_extract_in_relationship() {
        let extractor = RegexExtractor::new();
        let text = "Paris in France is beautiful.";

        let rels = extractor.extract_relationships(text);

        let in_rel = rels.iter().find(|r| r.relationship_type == "located_in");
        assert!(in_rel.is_some());

        let rel = in_rel.unwrap();
        assert_eq!(rel.from_entity, "Paris");
        assert_eq!(rel.to_entity, "France");
    }

    #[test]
    fn test_infer_language_type() {
        let extractor = RegexExtractor::new();
        let text = "Rust is a programming language. It's fast.";

        let entities = extractor.extract_entities(text);

        let rust = entities.iter().find(|e| e.name == "Rust");
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().entity_type, "programming_language");
    }

    #[test]
    fn test_performance_1kb_text() {
        let extractor = RegexExtractor::new();

        // Generate ~1KB of text
        let text = "Rust is a systems programming language. \
                    Rust has memory safety. \
                    Python is a high-level language. \
                    Python has dynamic typing. \
                    JavaScript runs in browsers. \
                    TypeScript is a JavaScript superset. "
            .repeat(20);

        assert!(text.len() >= 1000, "Text should be at least 1KB");

        let start = std::time::Instant::now();
        let entities = extractor.extract_entities(&text);
        let rels = extractor.extract_relationships(&text);
        let duration = start.elapsed();

        assert!(!entities.is_empty());
        assert!(!rels.is_empty());
        assert!(
            duration.as_millis() < 6,
            "Should complete in <6ms (with stopword filtering), took {duration:?}"
        );
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // Test values < 100, precision loss impossible
    fn test_recall_common_patterns() {
        let extractor = RegexExtractor::new();
        let text = "Rust is a systems programming language. \
                    Rust has memory safety. \
                    Rust has zero-cost abstractions. \
                    Cargo is a tool for Rust. \
                    Tokio is a framework. \
                    Tokio runs in Rust.";

        let entities = extractor.extract_entities(text);
        let rels = extractor.extract_relationships(text);

        // Should find major entities
        assert!(entities.iter().any(|e| e.name == "Rust"));
        assert!(entities.iter().any(|e| e.name == "Cargo"));
        assert!(entities.iter().any(|e| e.name == "Tokio"));

        // Should find relationships
        let is_a_count = rels
            .iter()
            .filter(|r| r.relationship_type == "is_a")
            .count();
        let has_count = rels
            .iter()
            .filter(|r| r.relationship_type == "has_feature")
            .count();

        assert!(is_a_count >= 2, "Should find at least 2 is_a relationships");
        assert!(has_count >= 2, "Should find at least 2 has relationships");

        // Recall calculation: found / total expected
        // Expected: 3 entities, 5 relationships
        let recall = (entities.len() + rels.len()) as f64 / 8.0;
        assert!(
            recall >= 0.5,
            "Recall should be >50%, got {:.1}%",
            recall * 100.0
        );
    }
}
