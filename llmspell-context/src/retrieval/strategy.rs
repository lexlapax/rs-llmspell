//! Retrieval strategy selection based on query understanding
//!
//! Routes queries to appropriate retrieval strategies (episodic, semantic, BM25, hybrid)
//! based on intent classification and query characteristics.

use crate::types::{QueryIntent, QueryUnderstanding, RetrievalStrategy};
use tracing::{debug, info, trace};

/// Strategy selector for choosing optimal retrieval approach
///
/// Uses query understanding (intent, entities, keywords) to select
/// the most appropriate retrieval strategy or combination of strategies.
#[derive(Debug, Clone)]
pub struct StrategySelector {
    /// Enable hybrid retrieval for complex queries
    enable_hybrid: bool,
    /// Minimum entity count to trigger semantic retrieval
    semantic_entity_threshold: usize,
}

impl StrategySelector {
    /// Create a new strategy selector with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enable_hybrid: true,
            semantic_entity_threshold: 2,
        }
    }

    /// Create a strategy selector with custom settings
    #[must_use]
    pub const fn with_config(enable_hybrid: bool, semantic_entity_threshold: usize) -> Self {
        Self {
            enable_hybrid,
            semantic_entity_threshold,
        }
    }

    /// Helper: Check Rule 1 - HowTo intent
    fn check_howto_rule(&self, understanding: &QueryUnderstanding) -> Option<RetrievalStrategy> {
        if understanding.intent == QueryIntent::HowTo {
            debug!("Selected Episodic strategy (Rule 1: HowTo intent)");
            Some(RetrievalStrategy::Episodic)
        } else {
            None
        }
    }

    /// Helper: Check Rule 2 - WhatIs/Explain with entities
    fn check_whatis_explain_rule(&self, understanding: &QueryUnderstanding) -> Option<RetrievalStrategy> {
        if (understanding.intent == QueryIntent::WhatIs || understanding.intent == QueryIntent::Explain)
            && understanding.entities.len() >= self.semantic_entity_threshold
        {
            debug!(
                "Selected Semantic strategy (Rule 2: WhatIs/Explain with {} entities)",
                understanding.entities.len()
            );
            Some(RetrievalStrategy::Semantic)
        } else {
            None
        }
    }

    /// Helper: Check Rule 3 - Debug intent
    fn check_debug_rule(&self, understanding: &QueryUnderstanding) -> Option<RetrievalStrategy> {
        if understanding.intent == QueryIntent::Debug && self.enable_hybrid {
            debug!("Selected Hybrid strategy (Rule 3: Debug intent)");
            Some(RetrievalStrategy::Hybrid)
        } else {
            None
        }
    }

    /// Helper: Check Rule 4 - WhyDoes with entities
    fn check_whydoes_rule(&self, understanding: &QueryUnderstanding) -> Option<RetrievalStrategy> {
        if understanding.intent == QueryIntent::WhyDoes
            && !understanding.entities.is_empty()
            && self.enable_hybrid
        {
            debug!("Selected Hybrid strategy (Rule 4: WhyDoes with entities)");
            Some(RetrievalStrategy::Hybrid)
        } else {
            None
        }
    }

    /// Helper: Check Rule 5 - Complex queries (many entities)
    fn check_complex_query_rule(&self, understanding: &QueryUnderstanding) -> Option<RetrievalStrategy> {
        if understanding.entities.len() >= 3 && self.enable_hybrid {
            debug!(
                "Selected Hybrid strategy (Rule 5: Complex query with {} entities)",
                understanding.entities.len()
            );
            Some(RetrievalStrategy::Hybrid)
        } else {
            None
        }
    }

    /// Helper: Check Rule 6 - Simple queries (few keywords)
    fn check_simple_query_rule(&self, understanding: &QueryUnderstanding) -> Option<RetrievalStrategy> {
        if understanding.keywords.len() < 2 && understanding.intent == QueryIntent::Unknown {
            debug!(
                "Selected Episodic strategy (Rule 6: Simple query with {} keywords)",
                understanding.keywords.len()
            );
            Some(RetrievalStrategy::Episodic)
        } else {
            None
        }
    }

    /// Select retrieval strategy based on query understanding
    ///
    /// # Strategy Selection Rules
    ///
    /// 1. **`HowTo` intent** → Episodic (recent examples)
    /// 2. **WhatIs/Explain intent + entities** → Semantic (knowledge graph)
    /// 3. **Debug intent** → Hybrid (recent errors + known solutions)
    /// 4. **`WhyDoes` intent + entities** → Hybrid (concepts + history)
    /// 5. **Complex queries (3+ entities)** → Hybrid
    /// 6. **Simple queries (<2 keywords)** → Episodic
    /// 7. **Default** → BM25 (keyword fallback)
    #[must_use]
    pub fn select(&self, understanding: &QueryUnderstanding) -> RetrievalStrategy {
        info!(
            "Selecting retrieval strategy: intent={:?}, entities={}, keywords={}",
            understanding.intent,
            understanding.entities.len(),
            understanding.keywords.len()
        );
        trace!("Query understanding: {:?}", understanding);

        // Apply rules in order, return first match
        if let Some(strategy) = self.check_howto_rule(understanding) {
            return strategy;
        }
        if let Some(strategy) = self.check_whatis_explain_rule(understanding) {
            return strategy;
        }
        if let Some(strategy) = self.check_debug_rule(understanding) {
            return strategy;
        }
        if let Some(strategy) = self.check_whydoes_rule(understanding) {
            return strategy;
        }
        if let Some(strategy) = self.check_complex_query_rule(understanding) {
            return strategy;
        }
        if let Some(strategy) = self.check_simple_query_rule(understanding) {
            return strategy;
        }

        // Rule 7: Default fallback
        debug!("Selected BM25 strategy (Rule 7: Default fallback)");
        RetrievalStrategy::BM25
    }

    /// Select retrieval strategy with fallback chain
    ///
    /// Returns primary strategy and fallback options for robustness.
    ///
    /// # Returns
    ///
    /// Vector of strategies in preference order:
    /// - `[primary, fallback1, fallback2]`
    #[must_use]
    pub fn select_with_fallback(
        &self,
        understanding: &QueryUnderstanding,
    ) -> Vec<RetrievalStrategy> {
        let primary = self.select(understanding);

        let fallback_chain = match primary {
            RetrievalStrategy::Episodic => {
                vec![RetrievalStrategy::Episodic, RetrievalStrategy::BM25]
            }
            RetrievalStrategy::Semantic => {
                vec![RetrievalStrategy::Semantic, RetrievalStrategy::BM25]
            }
            RetrievalStrategy::Hybrid => vec![
                RetrievalStrategy::Hybrid,
                RetrievalStrategy::Episodic,
                RetrievalStrategy::BM25,
            ],
            RetrievalStrategy::BM25 => vec![RetrievalStrategy::BM25, RetrievalStrategy::Episodic],
        };

        debug!("Fallback chain: {:?}", fallback_chain);
        fallback_chain
    }
}

impl Default for StrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_howto_selects_episodic() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::HowTo,
            entities: vec![],
            keywords: vec!["implement".to_string(), "feature".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Episodic);
    }

    #[test]
    fn test_whatis_with_entities_selects_semantic() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::WhatIs,
            entities: vec!["HashMap".to_string(), "BTreeMap".to_string()],
            keywords: vec!["hashmap".to_string(), "btreemap".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Semantic);
    }

    #[test]
    fn test_explain_with_entities_selects_semantic() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Explain,
            entities: vec!["Mutex".to_string(), "RwLock".to_string()],
            keywords: vec!["mutex".to_string(), "rwlock".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Semantic);
    }

    #[test]
    fn test_debug_selects_hybrid() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Debug,
            entities: vec!["Error".to_string()],
            keywords: vec!["error".to_string(), "broken".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Hybrid);
    }

    #[test]
    fn test_whydoes_with_entities_selects_hybrid() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::WhyDoes,
            entities: vec!["Rust".to_string()],
            keywords: vec!["rust".to_string(), "ownership".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Hybrid);
    }

    #[test]
    fn test_complex_query_selects_hybrid() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![
                "HashMap".to_string(),
                "VecDeque".to_string(),
                "BTreeMap".to_string(),
            ],
            keywords: vec!["compare".to_string(), "performance".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Hybrid);
    }

    #[test]
    fn test_simple_query_selects_episodic() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec!["hello".to_string()],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::Episodic);
    }

    #[test]
    fn test_default_fallback_selects_bm25() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec![
                "query".to_string(),
                "search".to_string(),
                "find".to_string(),
            ],
        };

        assert_eq!(selector.select(&understanding), RetrievalStrategy::BM25);
    }

    #[test]
    fn test_hybrid_disabled() {
        let selector = StrategySelector::with_config(false, 2);
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Debug,
            entities: vec![],
            keywords: vec!["error".to_string()],
        };

        // With hybrid disabled, should fall through to BM25
        assert_eq!(selector.select(&understanding), RetrievalStrategy::BM25);
    }

    #[test]
    fn test_semantic_threshold() {
        let selector = StrategySelector::with_config(true, 3);
        let understanding = QueryUnderstanding {
            intent: QueryIntent::WhatIs,
            entities: vec!["HashMap".to_string(), "Vec".to_string()],
            keywords: vec!["hashmap".to_string()],
        };

        // Only 2 entities, threshold is 3, should fall through to BM25
        assert_eq!(selector.select(&understanding), RetrievalStrategy::BM25);
    }

    #[test]
    fn test_fallback_chain_episodic() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::HowTo,
            entities: vec![],
            keywords: vec!["implement".to_string()],
        };

        let fallbacks = selector.select_with_fallback(&understanding);
        assert_eq!(
            fallbacks,
            vec![RetrievalStrategy::Episodic, RetrievalStrategy::BM25]
        );
    }

    #[test]
    fn test_fallback_chain_semantic() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::WhatIs,
            entities: vec!["Rust".to_string(), "Ownership".to_string()],
            keywords: vec!["rust".to_string()],
        };

        let fallbacks = selector.select_with_fallback(&understanding);
        assert_eq!(
            fallbacks,
            vec![RetrievalStrategy::Semantic, RetrievalStrategy::BM25]
        );
    }

    #[test]
    fn test_fallback_chain_hybrid() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Debug,
            entities: vec![],
            keywords: vec!["error".to_string()],
        };

        let fallbacks = selector.select_with_fallback(&understanding);
        assert_eq!(
            fallbacks,
            vec![
                RetrievalStrategy::Hybrid,
                RetrievalStrategy::Episodic,
                RetrievalStrategy::BM25
            ]
        );
    }

    #[test]
    fn test_fallback_chain_bm25() {
        let selector = StrategySelector::new();
        let understanding = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec!["search".to_string(), "query".to_string()],
        };

        let fallbacks = selector.select_with_fallback(&understanding);
        assert_eq!(
            fallbacks,
            vec![RetrievalStrategy::BM25, RetrievalStrategy::Episodic]
        );
    }
}
