//! ABOUTME: Capability aggregation and management for composite agents
//! ABOUTME: Provides mechanisms to aggregate, match, and manage agent capabilities

use super::traits::{Capability, CapabilityCategory};
use llmspell_core::{LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Capability aggregator that manages and matches capabilities
pub struct CapabilityAggregator {
    /// All registered capabilities
    capabilities: RwLock<HashMap<String, CapabilityEntry>>,
    /// Index by category for fast lookup
    category_index: RwLock<HashMap<CapabilityCategory, HashSet<String>>>,
    /// Capability requirements
    requirements: RwLock<Vec<CapabilityRequirement>>,
    /// Capability scoring function
    scorer: Arc<dyn CapabilityScorer>,
}

/// Entry for a registered capability
#[derive(Debug, Clone)]
pub struct CapabilityEntry {
    /// The capability itself
    pub capability: Capability,
    /// Provider of this capability
    pub provider_id: String,
    /// Score for this capability (0.0 to 1.0)
    pub score: f64,
    /// Whether this capability is currently available
    pub available: bool,
    /// Usage statistics
    pub usage_stats: CapabilityUsageStats,
}

/// Usage statistics for a capability
#[derive(Debug, Clone, Default)]
pub struct CapabilityUsageStats {
    /// Total number of times invoked
    pub invocations: u64,
    /// Number of successful invocations
    pub successes: u64,
    /// Number of failed invocations
    pub failures: u64,
    /// Average execution time
    pub avg_execution_time: std::time::Duration,
    /// Last invocation time
    pub last_invocation: Option<chrono::DateTime<chrono::Utc>>,
}

/// Requirement for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Name pattern (supports wildcards)
    pub name_pattern: String,
    /// Required category
    pub category: Option<CapabilityCategory>,
    /// Minimum version
    pub min_version: Option<String>,
    /// Required metadata fields
    pub required_metadata: HashMap<String, String>,
    /// Whether this requirement is mandatory
    pub mandatory: bool,
    /// Minimum score required
    pub min_score: Option<f64>,
}

/// Trait for scoring capabilities
pub trait CapabilityScorer: Send + Sync {
    /// Score a capability (0.0 to 1.0)
    fn score(&self, capability: &Capability, stats: &CapabilityUsageStats) -> f64;
}

/// Default capability scorer based on usage statistics
#[derive(Debug)]
pub struct DefaultCapabilityScorer;

impl CapabilityScorer for DefaultCapabilityScorer {
    fn score(&self, _capability: &Capability, stats: &CapabilityUsageStats) -> f64 {
        if stats.invocations == 0 {
            return 0.5; // Neutral score for unused capabilities
        }

        let success_rate = stats.successes as f64 / stats.invocations as f64;
        let recency_score = if let Some(last) = stats.last_invocation {
            let hours_ago = (chrono::Utc::now() - last).num_hours();
            (1.0 / (1.0 + hours_ago as f64 / 24.0)).min(1.0)
        } else {
            0.0
        };

        // Weighted average of success rate and recency
        (success_rate * 0.7 + recency_score * 0.3).clamp(0.0, 1.0)
    }
}

/// Result of a capability match
#[derive(Debug, Clone)]
pub struct CapabilityMatch {
    /// The matched capability
    pub capability: Capability,
    /// Provider of the capability
    pub provider_id: String,
    /// Match score (0.0 to 1.0)
    pub score: f64,
    /// Which requirements were satisfied
    pub satisfied_requirements: Vec<String>,
}

impl CapabilityAggregator {
    /// Create a new capability aggregator
    pub fn new() -> Self {
        Self::with_scorer(Arc::new(DefaultCapabilityScorer))
    }

    /// Create with a custom scorer
    pub fn with_scorer(scorer: Arc<dyn CapabilityScorer>) -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
            category_index: RwLock::new(HashMap::new()),
            requirements: RwLock::new(Vec::new()),
            scorer,
        }
    }

    /// Register a capability
    pub fn register_capability(
        &self,
        capability: Capability,
        provider_id: impl Into<String>,
    ) -> Result<()> {
        let provider_id = provider_id.into();
        let capability_id = format!("{}::{}", provider_id, capability.name);

        // Calculate initial score
        let score = self
            .scorer
            .score(&capability, &CapabilityUsageStats::default());

        let entry = CapabilityEntry {
            capability: capability.clone(),
            provider_id,
            score,
            available: true,
            usage_stats: CapabilityUsageStats::default(),
        };

        // Add to main registry
        let mut capabilities = self.capabilities.write().unwrap();
        capabilities.insert(capability_id.clone(), entry);

        // Update category index
        let mut index = self.category_index.write().unwrap();
        index
            .entry(capability.category.clone())
            .or_default()
            .insert(capability_id);

        Ok(())
    }

    /// Unregister a capability
    pub fn unregister_capability(&self, provider_id: &str, capability_name: &str) -> Result<()> {
        let capability_id = format!("{}::{}", provider_id, capability_name);

        let mut capabilities = self.capabilities.write().unwrap();
        if let Some(entry) = capabilities.remove(&capability_id) {
            // Remove from category index
            let mut index = self.category_index.write().unwrap();
            if let Some(set) = index.get_mut(&entry.capability.category) {
                set.remove(&capability_id);
            }
        }

        Ok(())
    }

    /// Add a capability requirement
    pub fn add_requirement(&self, requirement: CapabilityRequirement) {
        let mut requirements = self.requirements.write().unwrap();
        requirements.push(requirement);
    }

    /// Clear all requirements
    pub fn clear_requirements(&self) {
        let mut requirements = self.requirements.write().unwrap();
        requirements.clear();
    }

    /// Find capabilities matching requirements
    pub fn find_matches(&self) -> Vec<CapabilityMatch> {
        let capabilities = self.capabilities.read().unwrap();
        let requirements = self.requirements.read().unwrap();
        let mut matches = Vec::new();

        for (_cap_id, entry) in capabilities.iter() {
            if !entry.available {
                continue;
            }

            let mut satisfied = Vec::new();
            let mut total_score = 0.0;
            let mut requirement_count = 0;

            for (idx, req) in requirements.iter().enumerate() {
                if self.matches_requirement(&entry.capability, req) {
                    satisfied.push(format!("req-{}", idx));
                    total_score += entry.score;
                    requirement_count += 1;
                }
            }

            if !satisfied.is_empty() {
                let avg_score = if requirement_count > 0 {
                    total_score / requirement_count as f64
                } else {
                    entry.score
                };

                matches.push(CapabilityMatch {
                    capability: entry.capability.clone(),
                    provider_id: entry.provider_id.clone(),
                    score: avg_score,
                    satisfied_requirements: satisfied,
                });
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        matches
    }

    /// Check if a capability matches a requirement
    fn matches_requirement(
        &self,
        capability: &Capability,
        requirement: &CapabilityRequirement,
    ) -> bool {
        // Check name pattern
        if !self.matches_pattern(&capability.name, &requirement.name_pattern) {
            return false;
        }

        // Check category
        if let Some(ref req_category) = requirement.category {
            if &capability.category != req_category {
                return false;
            }
        }

        // Check version (simplified)
        if let Some(ref min_version) = requirement.min_version {
            if let Some(ref cap_version) = capability.version {
                if cap_version < min_version {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check metadata
        for (key, value) in &requirement.required_metadata {
            if capability.metadata.get(key) != Some(value) {
                return false;
            }
        }

        // Check score
        if let Some(min_score) = requirement.min_score {
            let cap_id = format!("{}::{}", "unknown", capability.name);
            if let Some(entry) = self.capabilities.read().unwrap().get(&cap_id) {
                if entry.score < min_score {
                    return false;
                }
            }
        }

        true
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.is_empty() {
                return true;
            }

            let mut text_pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }

                if i == 0 && !text.starts_with(part) {
                    return false;
                }

                if i == parts.len() - 1 && !pattern.ends_with('*') && !text.ends_with(part) {
                    return false;
                }

                if let Some(pos) = text[text_pos..].find(part) {
                    text_pos += pos + part.len();
                } else {
                    return false;
                }
            }
            true
        } else {
            text == pattern
        }
    }

    /// Update usage statistics for a capability
    pub fn update_usage(
        &self,
        provider_id: &str,
        capability_name: &str,
        success: bool,
        duration: std::time::Duration,
    ) -> Result<()> {
        let capability_id = format!("{}::{}", provider_id, capability_name);

        let mut capabilities = self.capabilities.write().unwrap();
        if let Some(entry) = capabilities.get_mut(&capability_id) {
            let stats = &mut entry.usage_stats;
            stats.invocations += 1;
            if success {
                stats.successes += 1;
            } else {
                stats.failures += 1;
            }

            // Update average execution time
            let total_time =
                stats.avg_execution_time.as_secs() * stats.invocations + duration.as_secs();
            stats.avg_execution_time =
                std::time::Duration::from_secs(total_time / (stats.invocations + 1));

            stats.last_invocation = Some(chrono::Utc::now());

            // Recalculate score
            entry.score = self.scorer.score(&entry.capability, stats);

            Ok(())
        } else {
            Err(LLMSpellError::Component {
                message: format!("Capability not found: {}", capability_id),
                source: None,
            })
        }
    }

    /// Get all capabilities for a provider
    pub fn get_provider_capabilities(&self, provider_id: &str) -> Vec<Capability> {
        let capabilities = self.capabilities.read().unwrap();
        capabilities
            .values()
            .filter(|entry| entry.provider_id == provider_id)
            .map(|entry| entry.capability.clone())
            .collect()
    }

    /// Get capabilities by category
    pub fn get_by_category(&self, category: &CapabilityCategory) -> Vec<Capability> {
        let index = self.category_index.read().unwrap();
        let capabilities = self.capabilities.read().unwrap();

        if let Some(cap_ids) = index.get(category) {
            cap_ids
                .iter()
                .filter_map(|id| capabilities.get(id))
                .map(|entry| entry.capability.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Set availability for a capability
    pub fn set_availability(
        &self,
        provider_id: &str,
        capability_name: &str,
        available: bool,
    ) -> Result<()> {
        let capability_id = format!("{}::{}", provider_id, capability_name);

        let mut capabilities = self.capabilities.write().unwrap();
        if let Some(entry) = capabilities.get_mut(&capability_id) {
            entry.available = available;
            Ok(())
        } else {
            Err(LLMSpellError::Component {
                message: format!("Capability not found: {}", capability_id),
                source: None,
            })
        }
    }

    /// Get statistics for all capabilities
    pub fn get_statistics(&self) -> CapabilityStatistics {
        let capabilities = self.capabilities.read().unwrap();
        let total = capabilities.len();
        let available = capabilities.values().filter(|e| e.available).count();

        let mut by_category = HashMap::new();
        for entry in capabilities.values() {
            *by_category
                .entry(entry.capability.category.clone())
                .or_insert(0) += 1;
        }

        let avg_score = if total > 0 {
            capabilities.values().map(|e| e.score).sum::<f64>() / total as f64
        } else {
            0.0
        };

        CapabilityStatistics {
            total_capabilities: total,
            available_capabilities: available,
            capabilities_by_category: by_category,
            average_score: avg_score,
        }
    }
}

impl Default for CapabilityAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about capabilities
#[derive(Debug, Clone)]
pub struct CapabilityStatistics {
    /// Total number of registered capabilities
    pub total_capabilities: usize,
    /// Number of available capabilities
    pub available_capabilities: usize,
    /// Count by category
    pub capabilities_by_category: HashMap<CapabilityCategory, usize>,
    /// Average capability score
    pub average_score: f64,
}

/// Builder for capability requirements
pub struct CapabilityRequirementBuilder {
    requirement: CapabilityRequirement,
}

impl CapabilityRequirementBuilder {
    /// Create a new requirement builder
    pub fn new(name_pattern: impl Into<String>) -> Self {
        Self {
            requirement: CapabilityRequirement {
                name_pattern: name_pattern.into(),
                category: None,
                min_version: None,
                required_metadata: HashMap::new(),
                mandatory: false,
                min_score: None,
            },
        }
    }

    /// Set the category requirement
    pub fn category(mut self, category: CapabilityCategory) -> Self {
        self.requirement.category = Some(category);
        self
    }

    /// Set minimum version
    pub fn min_version(mut self, version: impl Into<String>) -> Self {
        self.requirement.min_version = Some(version.into());
        self
    }

    /// Add required metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.requirement
            .required_metadata
            .insert(key.into(), value.into());
        self
    }

    /// Set as mandatory
    pub fn mandatory(mut self) -> Self {
        self.requirement.mandatory = true;
        self
    }

    /// Set minimum score
    pub fn min_score(mut self, score: f64) -> Self {
        self.requirement.min_score = Some(score);
        self
    }

    /// Build the requirement
    pub fn build(self) -> CapabilityRequirement {
        self.requirement
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "agent")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_capability_aggregator() {
        let aggregator = CapabilityAggregator::new();

        // Register a capability
        let cap = Capability {
            name: "text-processing".to_string(),
            category: CapabilityCategory::DataProcessing,
            version: Some("1.0.0".to_string()),
            metadata: HashMap::new(),
        };

        aggregator
            .register_capability(cap.clone(), "agent-1")
            .unwrap();

        // Check it was registered
        let provider_caps = aggregator.get_provider_capabilities("agent-1");
        assert_eq!(provider_caps.len(), 1);
        assert_eq!(provider_caps[0].name, "text-processing");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_pattern_matching() {
        let aggregator = CapabilityAggregator::new();

        assert!(aggregator.matches_pattern("text-processing", "text-processing"));
        assert!(aggregator.matches_pattern("text-processing", "text-*"));
        assert!(aggregator.matches_pattern("text-processing", "*-processing"));
        assert!(aggregator.matches_pattern("text-processing", "*"));
        assert!(!aggregator.matches_pattern("text-processing", "image-*"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_capability_matching() {
        let aggregator = CapabilityAggregator::new();

        // Register capabilities
        let cap1 = Capability {
            name: "text-analysis".to_string(),
            category: CapabilityCategory::DataProcessing,
            version: Some("2.0.0".to_string()),
            metadata: HashMap::new(),
        };

        let cap2 = Capability {
            name: "image-analysis".to_string(),
            category: CapabilityCategory::DataProcessing,
            version: Some("1.0.0".to_string()),
            metadata: HashMap::new(),
        };

        aggregator.register_capability(cap1, "agent-1").unwrap();
        aggregator.register_capability(cap2, "agent-2").unwrap();

        // Add requirement
        let requirement = CapabilityRequirementBuilder::new("*-analysis")
            .category(CapabilityCategory::DataProcessing)
            .min_version("1.5.0")
            .build();

        aggregator.add_requirement(requirement);

        // Find matches
        let matches = aggregator.find_matches();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].capability.name, "text-analysis");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_usage_statistics() {
        let aggregator = CapabilityAggregator::new();

        let cap = Capability {
            name: "test-cap".to_string(),
            category: CapabilityCategory::Custom("test".to_string()),
            version: None,
            metadata: HashMap::new(),
        };

        aggregator.register_capability(cap, "provider-1").unwrap();

        // Update usage
        aggregator
            .update_usage(
                "provider-1",
                "test-cap",
                true,
                std::time::Duration::from_secs(1),
            )
            .unwrap();

        // Check statistics
        let stats = aggregator.get_statistics();
        assert_eq!(stats.total_capabilities, 1);
        assert_eq!(stats.available_capabilities, 1);
    }
}
