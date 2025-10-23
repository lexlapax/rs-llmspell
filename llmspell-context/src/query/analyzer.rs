//! Query understanding with intent classification and entity extraction
//!
//! Provides fast, regex-based query analysis for intent classification (<1ms hot path).
//! Designed for real-time query understanding, NOT domain-specific entity extraction.
//!
//! # Performance
//!
//! - <1ms P99 latency (hot path requirement)
//! - Early-exit on first intent match
//! - Simple patterns only (not domain-specific)
//!
//! # Comparison with `RegexExtractor`
//!
//! - **`RegexQueryAnalyzer`**: Simple intent classification (^how do i) for queries
//! - **`RegexExtractor`**: Complex domain extraction (fn|struct|impl) for consolidation
//! - Zero pattern overlap, independent evolution paths

use crate::error::Result;
use crate::traits::QueryAnalyzer;
use crate::types::{QueryIntent, QueryUnderstanding};
use async_trait::async_trait;
use llmspell_utils::text::stopwords::is_stopword;
use regex::Regex;
use std::sync::LazyLock;

/// Compiled intent patterns for fast matching
static INTENT_PATTERNS: LazyLock<Vec<(Regex, QueryIntent)>> = LazyLock::new(|| {
    vec![
        // HowTo: "How do I...", "How can I...", "How to..."
        (
            Regex::new(r"^(?i)how\s+(?:do|can|to|should)\s+(?:i|we)?\s*").unwrap(),
            QueryIntent::HowTo,
        ),
        // WhatIs: "What is...", "What are...", "What does..."
        (
            Regex::new(r"^(?i)what\s+(?:is|are|does|do)\s+").unwrap(),
            QueryIntent::WhatIs,
        ),
        // WhyDoes: "Why does...", "Why is...", "Why are..."
        (
            Regex::new(r"^(?i)why\s+(?:does|is|are|do)\s+").unwrap(),
            QueryIntent::WhyDoes,
        ),
        // Debug: error, bug, broken, fail, crash, exception
        // Matches variations: crash/crashed/crashes, fail/fails/failed, etc.
        (
            Regex::new(r"(?i)\b(?:error|bug|broken|fail|crash|exception|panic)(?:s|ed|ing)?\b")
                .unwrap(),
            QueryIntent::Debug,
        ),
        // Explain: "Explain...", "Describe...", "Tell me about..."
        (
            Regex::new(r"^(?i)(?:explain|describe|tell\s+me\s+about)\s+").unwrap(),
            QueryIntent::Explain,
        ),
    ]
});

/// Entity patterns for simple extraction
static ENTITY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // CamelCase identifiers (e.g., HashMap, VecDeque)
        Regex::new(r"\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b").unwrap(),
        // snake_case identifiers (e.g., hash_map, vec_deque)
        Regex::new(r"\b([a-z_][a-z0-9_]{2,})\b").unwrap(),
        // SCREAMING_SNAKE_CASE constants
        Regex::new(r"\b([A-Z_][A-Z0-9_]{2,})\b").unwrap(),
    ]
});

/// Regex-based query analyzer for fast intent classification
///
/// Optimized for <1ms P99 latency on hot path. Uses simple patterns
/// for intent classification, NOT complex domain-specific patterns.
#[derive(Debug, Clone)]
pub struct RegexQueryAnalyzer;

impl RegexQueryAnalyzer {
    /// Create a new regex-based query analyzer
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extract entities from query using simple patterns
    ///
    /// Extracts CamelCase, `snake_case`, and `SCREAMING_SNAKE_CASE` identifiers.
    /// Does NOT extract domain-specific entities (that's `RegexExtractor`'s job).
    fn extract_entities(query: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for pattern in ENTITY_PATTERNS.iter() {
            for cap in pattern.captures_iter(query) {
                if let Some(entity) = cap.get(1) {
                    let entity_str = entity.as_str();
                    // Deduplicate and filter out single-letter entities
                    if entity_str.len() > 1 && seen.insert(entity_str.to_string()) {
                        entities.push(entity_str.to_string());
                    }
                }
            }
        }

        entities
    }

    /// Extract keywords from query by filtering stopwords
    ///
    /// Uses llmspell-utils stopwords for O(1) filtering.
    fn extract_keywords(query: &str) -> Vec<String> {
        query
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter_map(|token| {
                let trimmed = token.trim();
                if trimmed.is_empty() || trimmed.len() < 2 {
                    return None;
                }

                // Capitalize for stopword check
                let capitalized = capitalize(trimmed);
                if is_stopword(&capitalized) {
                    None
                } else {
                    Some(trimmed.to_lowercase())
                }
            })
            .collect()
    }

    /// Classify query intent using early-exit pattern matching
    ///
    /// Returns first matching intent for optimal performance.
    fn classify_intent(query: &str) -> QueryIntent {
        INTENT_PATTERNS
            .iter()
            .find(|(pattern, _)| pattern.is_match(query))
            .map_or(QueryIntent::Unknown, |(_, intent)| intent.clone())
    }
}

impl Default for RegexQueryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueryAnalyzer for RegexQueryAnalyzer {
    async fn understand(&self, query: &str) -> Result<QueryUnderstanding> {
        // Fast path: early-exit intent classification
        let intent = Self::classify_intent(query);

        // Extract entities and keywords
        let entities = Self::extract_entities(query);
        let keywords = Self::extract_keywords(query);

        Ok(QueryUnderstanding {
            intent,
            entities,
            keywords,
        })
    }
}

/// Capitalize first character for stopword checking
#[inline]
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intent_how_to() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer.understand("How do I use HashMap?").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::HowTo);
    }

    #[tokio::test]
    async fn test_intent_what_is() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer.understand("What is a closure?").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::WhatIs);
    }

    #[tokio::test]
    async fn test_intent_why_does() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("Why does Rust use move semantics?")
            .await
            .unwrap();
        assert_eq!(understanding.intent, QueryIntent::WhyDoes);
    }

    #[tokio::test]
    async fn test_intent_debug() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer.understand("My code has a bug").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::Debug);
    }

    #[tokio::test]
    async fn test_intent_explain() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("Explain lifetimes in Rust")
            .await
            .unwrap();
        assert_eq!(understanding.intent, QueryIntent::Explain);
    }

    #[tokio::test]
    async fn test_intent_unknown() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer.understand("Just some random text").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::Unknown);
    }

    #[tokio::test]
    async fn test_entity_extraction_camelcase() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("How do I use HashMap and VecDeque?")
            .await
            .unwrap();
        assert!(understanding.entities.contains(&"HashMap".to_string()));
        assert!(understanding.entities.contains(&"VecDeque".to_string()));
    }

    #[tokio::test]
    async fn test_entity_extraction_snake_case() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("What is hash_map in Rust?")
            .await
            .unwrap();
        assert!(understanding.entities.contains(&"hash_map".to_string()));
    }

    #[tokio::test]
    async fn test_keyword_extraction() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("How do I implement concurrency in Rust?")
            .await
            .unwrap();

        // Should extract meaningful keywords
        assert!(understanding.keywords.contains(&"implement".to_string()));
        assert!(understanding.keywords.contains(&"concurrency".to_string()));
        assert!(understanding.keywords.contains(&"rust".to_string()));

        // Should filter stopwords
        assert!(!understanding.keywords.contains(&"do".to_string()));
        assert!(!understanding.keywords.contains(&"i".to_string()));
        assert!(!understanding.keywords.contains(&"in".to_string()));
    }

    #[tokio::test]
    async fn test_deduplication() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("HashMap HashMap HashMap")
            .await
            .unwrap();

        // Should deduplicate entities
        assert_eq!(
            understanding
                .entities
                .iter()
                .filter(|e| *e == "HashMap")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn test_case_insensitive_intent() {
        let analyzer = RegexQueryAnalyzer::new();

        // Lowercase
        let u1 = analyzer.understand("how do i use rust").await.unwrap();
        assert_eq!(u1.intent, QueryIntent::HowTo);

        // Uppercase
        let u2 = analyzer.understand("HOW DO I USE RUST").await.unwrap();
        assert_eq!(u2.intent, QueryIntent::HowTo);

        // Mixed case
        let u3 = analyzer.understand("HoW dO I uSe RuSt").await.unwrap();
        assert_eq!(u3.intent, QueryIntent::HowTo);
    }

    #[tokio::test]
    async fn test_debug_intent_keywords() {
        let analyzer = RegexQueryAnalyzer::new();

        let test_cases = vec![
            "I have an error in my code",
            "This code is broken",
            "My program crashed",
            "Exception thrown here",
            "Panic at runtime",
            "Getting a bug when I run this",
        ];

        for query in test_cases {
            let understanding = analyzer.understand(query).await.unwrap();
            assert_eq!(
                understanding.intent,
                QueryIntent::Debug,
                "Failed for query: {}",
                query
            );
        }
    }

    #[tokio::test]
    async fn test_intent_precedence() {
        let analyzer = RegexQueryAnalyzer::new();

        // "Why does X fail" should be WhyDoes, not Debug (intent at start takes precedence)
        let understanding = analyzer.understand("Why does this fail?").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::WhyDoes);

        // "X fails" without leading intent should be Debug
        let understanding = analyzer.understand("This fails").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::Debug);
    }

    #[tokio::test]
    async fn test_empty_query() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer.understand("").await.unwrap();
        assert_eq!(understanding.intent, QueryIntent::Unknown);
        assert!(understanding.entities.is_empty());
        assert!(understanding.keywords.is_empty());
    }

    #[tokio::test]
    async fn test_complex_query() {
        let analyzer = RegexQueryAnalyzer::new();
        let understanding = analyzer
            .understand("How do I use HashMap with VecDeque for concurrent programming?")
            .await
            .unwrap();

        assert_eq!(understanding.intent, QueryIntent::HowTo);
        assert!(understanding.entities.contains(&"HashMap".to_string()));
        assert!(understanding.entities.contains(&"VecDeque".to_string()));
        assert!(understanding.keywords.contains(&"concurrent".to_string()));
        assert!(understanding.keywords.contains(&"programming".to_string()));
    }
}
