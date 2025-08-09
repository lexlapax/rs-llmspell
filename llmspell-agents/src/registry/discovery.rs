//! ABOUTME: Agent discovery and search capabilities
//! ABOUTME: Provides advanced search, filtering, and recommendation features

#![allow(clippy::significant_drop_tightening)]

use super::{
    metadata::{CapabilityType, ExtendedAgentMetadata, HealthState},
    AgentMetadata, AgentQuery, AgentRegistry, AgentStatus,
};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};

/// Advanced search criteria
#[derive(Debug, Clone, Default)]
pub struct SearchCriteria {
    /// Basic query from `AgentQuery`
    pub base_query: AgentQuery,

    /// Search by capabilities
    pub capabilities: Vec<CapabilityType>,

    /// Search by health state
    pub health_state: Option<HealthState>,

    /// Minimum success rate
    pub min_success_rate: Option<f64>,

    /// Maximum execution time
    pub max_execution_time_ms: Option<f64>,

    /// Sort options
    pub sort_by: Option<SortField>,

    /// Sort order
    pub sort_order: SortOrder,
}

/// Fields to sort by
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortField {
    /// Sort by name
    Name,

    /// Sort by creation date
    CreatedAt,

    /// Sort by last update
    UpdatedAt,

    /// Sort by execution count
    ExecutionCount,

    /// Sort by success rate
    SuccessRate,

    /// Sort by average execution time
    AvgExecutionTime,
}

/// Sort order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order
    Ascending,

    /// Descending order
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Agent metadata
    pub metadata: AgentMetadata,

    /// Extended metadata if available
    pub extended_metadata: Option<ExtendedAgentMetadata>,

    /// Relevance score (0.0 - 1.0)
    pub relevance_score: f64,

    /// Match reasons
    pub match_reasons: Vec<String>,
}

/// Agent discovery
pub struct Discovery<R: AgentRegistry> {
    registry: Arc<R>,
    metadata_cache: Arc<tokio::sync::RwLock<HashMap<String, ExtendedAgentMetadata>>>,
}

impl<R: AgentRegistry> Discovery<R> {
    /// Create new discovery instance
    pub fn new(registry: Arc<R>) -> Self {
        Self {
            registry,
            metadata_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Search agents with advanced criteria
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent querying fails
    /// - Metadata retrieval fails
    /// - Scoring calculation fails
    pub async fn search(&self, criteria: &SearchCriteria) -> Result<Vec<SearchResult>> {
        // First, get basic results from registry
        let basic_results = self.registry.query_agents(&criteria.base_query).await?;

        // Score and filter results
        let mut scored_results = Vec::new();

        for metadata in basic_results {
            let mut score = 1.0;
            let mut reasons = Vec::new();

            // Check success rate
            if let Some(min_rate) = criteria.min_success_rate {
                if metadata.metrics.success_rate < min_rate {
                    continue; // Skip this agent
                }
                reasons.push(format!(
                    "Success rate: {:.2}%",
                    metadata.metrics.success_rate * 100.0
                ));
            }

            // Check execution time
            if let Some(max_time) = criteria.max_execution_time_ms {
                if metadata.metrics.avg_execution_time_ms > max_time {
                    continue; // Skip this agent
                }
                score *= 1.0 - (metadata.metrics.avg_execution_time_ms / max_time);
                reasons.push(format!(
                    "Avg execution time: {:.2}ms",
                    metadata.metrics.avg_execution_time_ms
                ));
            }

            // Check extended metadata if needed
            let extended = self.get_extended_metadata(&metadata.id).await;

            // Check capabilities
            if !criteria.capabilities.is_empty() {
                if let Some(ext) = &extended {
                    let has_all_capabilities = criteria.capabilities.iter().all(|required| {
                        ext.capabilities
                            .iter()
                            .any(|cap| match (&cap.capability_type, required) {
                                (CapabilityType::Custom(a), CapabilityType::Custom(b)) => a == b,
                                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
                            })
                    });

                    if !has_all_capabilities {
                        continue; // Skip this agent
                    }

                    score *= 0.9; // Slightly lower score for capability match
                    reasons.push("Has required capabilities".to_string());
                }
            }

            // Check health state
            if let Some(required_state) = &criteria.health_state {
                if let Some(ext) = &extended {
                    if &ext.health.state != required_state {
                        continue; // Skip this agent
                    }
                    reasons.push(format!("Health: {:?}", ext.health.state));
                }
            }

            // Add name match bonus
            if let Some(name_filter) = &criteria.base_query.name_filter {
                if metadata
                    .name
                    .to_lowercase()
                    .contains(&name_filter.to_lowercase())
                {
                    score *= 1.2; // Bonus for name match
                    reasons.push("Name match".to_string());
                }
            }

            scored_results.push(SearchResult {
                metadata,
                extended_metadata: extended,
                relevance_score: score.min(1.0),
                match_reasons: reasons,
            });
        }

        // Sort results
        Self::sort_results(&mut scored_results, &criteria.sort_by, &criteria.sort_order);

        Ok(scored_results)
    }

    /// Find similar agents
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reference agent metadata is not found
    /// - Agent similarity search fails
    pub async fn find_similar(&self, agent_id: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Get the reference agent
        let reference = match self.registry.get_metadata(agent_id).await? {
            Some(metadata) => metadata,
            None => anyhow::bail!("Agent '{}' not found", agent_id),
        };

        // Build search criteria based on reference
        let criteria = SearchCriteria {
            base_query: AgentQuery {
                type_filter: Some(reference.agent_type.clone()),
                category_filter: reference.categories.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        // Search and filter out the reference agent
        let mut results = self.search(&criteria).await?;
        results.retain(|r| r.metadata.id != agent_id);
        results.truncate(limit);

        Ok(results)
    }

    /// Get recommended agents based on usage patterns
    ///
    /// # Errors
    ///
    /// Returns an error if recommendation generation fails
    ///
    /// # Panics
    ///
    /// Panics if DateTime conversion fails
    pub async fn get_recommendations(
        &self,
        context: &RecommendationContext,
    ) -> Result<Vec<SearchResult>> {
        let mut criteria = SearchCriteria {
            capabilities: context.required_capabilities.clone(),
            ..Default::default()
        };

        // Set performance requirements
        if let Some(max_time) = context.max_response_time_ms {
            criteria.max_execution_time_ms = Some(max_time);
        }

        if let Some(min_rate) = context.min_success_rate {
            criteria.min_success_rate = Some(min_rate);
        }

        // Sort by relevance
        criteria.sort_by = Some(SortField::SuccessRate);
        criteria.sort_order = SortOrder::Descending;

        let mut results = self.search(&criteria).await?;

        // Apply additional scoring based on context
        for result in &mut results {
            // Boost score for frequently used agents
            if context.frequently_used_agents.contains(&result.metadata.id) {
                result.relevance_score *= 1.5;
            }

            // Boost score for recently successful agents
            if result.metadata.metrics.last_execution_time.is_some() {
                let hours_since = chrono::Utc::now()
                    .signed_duration_since(result.metadata.metrics.last_execution_time.unwrap())
                    .num_hours();

                if hours_since < 24 {
                    result.relevance_score *= 1.2;
                }
            }
        }

        // Re-sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(context.max_recommendations);

        Ok(results)
    }

    /// Cache extended metadata
    pub async fn cache_extended_metadata(&self, id: String, metadata: ExtendedAgentMetadata) {
        let mut cache = self.metadata_cache.write().await;
        cache.insert(id, metadata);
    }

    /// Get extended metadata from cache
    async fn get_extended_metadata(&self, id: &str) -> Option<ExtendedAgentMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.get(id).cloned()
    }

    /// Sort search results
    fn sort_results(results: &mut [SearchResult], sort_by: &Option<SortField>, order: &SortOrder) {
        let field = sort_by.as_ref().unwrap_or(&SortField::Name);

        results.sort_by(|a, b| {
            let cmp = match field {
                SortField::Name => a.metadata.name.cmp(&b.metadata.name),
                SortField::CreatedAt => a.metadata.created_at.cmp(&b.metadata.created_at),
                SortField::UpdatedAt => a.metadata.updated_at.cmp(&b.metadata.updated_at),
                SortField::ExecutionCount => a
                    .metadata
                    .metrics
                    .execution_count
                    .cmp(&b.metadata.metrics.execution_count),
                SortField::SuccessRate => a
                    .metadata
                    .metrics
                    .success_rate
                    .partial_cmp(&b.metadata.metrics.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortField::AvgExecutionTime => a
                    .metadata
                    .metrics
                    .avg_execution_time_ms
                    .partial_cmp(&b.metadata.metrics.avg_execution_time_ms)
                    .unwrap_or(std::cmp::Ordering::Equal),
            };

            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }
}

/// Context for agent recommendations
#[derive(Debug, Clone)]
pub struct RecommendationContext {
    /// Required capabilities
    pub required_capabilities: Vec<CapabilityType>,

    /// Maximum response time requirement
    pub max_response_time_ms: Option<f64>,

    /// Minimum success rate requirement
    pub min_success_rate: Option<f64>,

    /// Previously used agents (for collaborative filtering)
    pub frequently_used_agents: Vec<String>,

    /// Maximum number of recommendations
    pub max_recommendations: usize,
}

impl Default for RecommendationContext {
    fn default() -> Self {
        Self {
            required_capabilities: Vec::new(),
            max_response_time_ms: None,
            min_success_rate: Some(0.8),
            frequently_used_agents: Vec::new(),
            max_recommendations: 10,
        }
    }
}

/// Search builder for fluent API
pub struct SearchBuilder {
    criteria: SearchCriteria,
}

impl SearchBuilder {
    /// Create new search builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            criteria: SearchCriteria::default(),
        }
    }

    /// Filter by name
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.criteria.base_query.name_filter = Some(name.to_string());
        self
    }

    /// Filter by type
    #[must_use]
    pub fn with_type(mut self, agent_type: &str) -> Self {
        self.criteria.base_query.type_filter = Some(agent_type.to_string());
        self
    }

    /// Filter by category
    #[must_use]
    pub fn in_category(mut self, category: &str) -> Self {
        self.criteria
            .base_query
            .category_filter
            .push(category.to_string());
        self
    }

    /// Filter by status
    #[must_use]
    pub fn with_status(mut self, status: AgentStatus) -> Self {
        self.criteria.base_query.status_filter = Some(status);
        self
    }

    /// Require capability
    #[must_use]
    pub fn with_capability(mut self, capability: CapabilityType) -> Self {
        self.criteria.capabilities.push(capability);
        self
    }

    /// Set minimum success rate
    #[must_use]
    pub const fn min_success_rate(mut self, rate: f64) -> Self {
        self.criteria.min_success_rate = Some(rate);
        self
    }

    /// Set maximum execution time
    #[must_use]
    pub const fn max_execution_time(mut self, ms: f64) -> Self {
        self.criteria.max_execution_time_ms = Some(ms);
        self
    }

    /// Set sort field
    #[must_use]
    pub const fn sort_by(mut self, field: SortField) -> Self {
        self.criteria.sort_by = Some(field);
        self
    }

    /// Set sort order
    #[must_use]
    pub const fn order(mut self, order: SortOrder) -> Self {
        self.criteria.sort_order = order;
        self
    }

    /// Set result limit
    #[must_use]
    pub const fn limit(mut self, limit: usize) -> Self {
        self.criteria.base_query.limit = Some(limit);
        self
    }

    /// Build search criteria
    #[must_use]
    pub fn build(self) -> SearchCriteria {
        self.criteria
    }
}

impl Default for SearchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_search_builder() {
        let criteria = SearchBuilder::new()
            .with_name("test")
            .with_type("llm")
            .in_category("research-agents")
            .with_capability(CapabilityType::ToolUsage)
            .min_success_rate(0.9)
            .max_execution_time(100.0)
            .sort_by(SortField::SuccessRate)
            .order(SortOrder::Descending)
            .limit(10)
            .build();

        assert_eq!(criteria.base_query.name_filter, Some("test".to_string()));
        assert_eq!(criteria.base_query.type_filter, Some("llm".to_string()));
        assert_eq!(criteria.capabilities.len(), 1);
        assert_eq!(criteria.min_success_rate, Some(0.9));
        assert_eq!(criteria.sort_order, SortOrder::Descending);
    }
}
