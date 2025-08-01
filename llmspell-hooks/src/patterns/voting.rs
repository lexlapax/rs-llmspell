// ABOUTME: Voting hook pattern implementation with configurable thresholds
// ABOUTME: Executes all hooks and uses majority voting to determine the final result

use crate::composite::{CompositeHook, CompositionPattern};
use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{ArcHook, Hook};
use crate::types::HookMetadata;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// A voting hook that uses majority consensus
pub struct VotingHook {
    inner: CompositeHook,
    threshold: f64,
    tie_breaker: TieBreaker,
}

/// Strategy for breaking ties in voting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TieBreaker {
    /// Use the first result in case of tie
    FirstResult,
    /// Prefer Continue in case of tie
    PreferContinue,
    /// Prefer non-Continue in case of tie  
    PreferAction,
    /// Use priority ordering
    ByPriority,
}

impl VotingHook {
    /// Create a new voting hook with default 50% threshold
    pub fn new(name: &str) -> Self {
        Self::with_threshold(name, 0.5)
    }

    /// Create a voting hook with specific threshold
    pub fn with_threshold(name: &str, threshold: f64) -> Self {
        assert!(threshold > 0.0 && threshold <= 1.0, "Threshold must be between 0 and 1");
        Self {
            inner: CompositeHook::new(name, CompositionPattern::Voting { threshold }),
            threshold,
            tie_breaker: TieBreaker::FirstResult,
        }
    }

    /// Set the tie breaker strategy
    pub fn with_tie_breaker(mut self, tie_breaker: TieBreaker) -> Self {
        self.tie_breaker = tie_breaker;
        self
    }

    /// Add a hook to vote
    pub fn add_hook(mut self, hook: impl Hook + 'static) -> Self {
        self.inner = self.inner.add_hook(Arc::new(hook));
        self
    }

    /// Add an Arc'd hook
    pub fn add_arc_hook(mut self, hook: ArcHook) -> Self {
        self.inner = self.inner.add_hook(hook);
        self
    }

    /// Add multiple hooks
    pub fn add_hooks(mut self, hooks: Vec<ArcHook>) -> Self {
        self.inner = self.inner.add_hooks(hooks);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.inner = self.inner.with_metadata(metadata);
        self
    }

    /// Get the number of voting hooks
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Create a builder
    pub fn builder(name: &str) -> VotingHookBuilder {
        VotingHookBuilder::new(name)
    }
}

#[async_trait]
impl Hook for VotingHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Use the inner composite hook's voting implementation
        self.inner.execute(context).await
    }

    fn metadata(&self) -> HookMetadata {
        self.inner.metadata()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        self.inner.should_execute(context)
    }
}

/// Builder for voting hooks
pub struct VotingHookBuilder {
    name: String,
    hooks: Vec<ArcHook>,
    metadata: Option<HookMetadata>,
    threshold: f64,
    tie_breaker: TieBreaker,
    require_all: bool,
    min_votes: Option<usize>,
}

impl VotingHookBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hooks: Vec::new(),
            metadata: None,
            threshold: 0.5,
            tie_breaker: TieBreaker::FirstResult,
            require_all: false,
            min_votes: None,
        }
    }

    pub fn add_hook(mut self, hook: impl Hook + 'static) -> Self {
        self.hooks.push(Arc::new(hook));
        self
    }

    pub fn add_arc_hook(mut self, hook: ArcHook) -> Self {
        self.hooks.push(hook);
        self
    }

    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        assert!(threshold > 0.0 && threshold <= 1.0, "Threshold must be between 0 and 1");
        self.threshold = threshold;
        self
    }

    pub fn with_tie_breaker(mut self, tie_breaker: TieBreaker) -> Self {
        self.tie_breaker = tie_breaker;
        self
    }

    pub fn require_all_votes(mut self, require: bool) -> Self {
        self.require_all = require;
        self
    }

    pub fn with_min_votes(mut self, min: usize) -> Self {
        self.min_votes = Some(min);
        self
    }

    pub fn build(self) -> VotingHook {
        let mut hook = VotingHook::with_threshold(&self.name, self.threshold)
            .with_tie_breaker(self.tie_breaker);
        
        if let Some(metadata) = self.metadata {
            hook = hook.with_metadata(metadata);
        }
        
        hook = hook.add_hooks(self.hooks);
        hook
    }
}

/// Advanced voting aggregator with detailed vote tracking
#[derive(Debug)]
pub struct VotingAggregator {
    votes: HashMap<String, Vec<usize>>,
    results: Vec<HookResult>,
    total_votes: usize,
}

impl VotingAggregator {
    pub fn new() -> Self {
        Self {
            votes: HashMap::new(),
            results: Vec::new(),
            total_votes: 0,
        }
    }

    pub fn add_vote(&mut self, hook_idx: usize, result: HookResult) {
        let vote_key = format!("{:?}", result.variant_name());
        self.votes.entry(vote_key).or_insert_with(Vec::new).push(hook_idx);
        self.results.push(result);
        self.total_votes += 1;
    }

    pub fn get_winner(&self, threshold: f64, tie_breaker: TieBreaker) -> HookResult {
        let required_votes = (self.total_votes as f64 * threshold).ceil() as usize;
        
        // Find all results that meet the threshold
        let mut candidates: Vec<(&String, &Vec<usize>)> = self.votes
            .iter()
            .filter(|(_, voters)| voters.len() >= required_votes)
            .collect();

        if candidates.is_empty() {
            debug!("No voting winner, defaulting to Continue");
            return HookResult::Continue;
        }

        // Sort by vote count (descending)
        candidates.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        // Check for ties
        let max_votes = candidates[0].1.len();
        let tied_candidates: Vec<_> = candidates
            .iter()
            .take_while(|(_, voters)| voters.len() == max_votes)
            .collect();

        let winner_key = if tied_candidates.len() > 1 {
            // Apply tie breaker
            match tie_breaker {
                TieBreaker::FirstResult => {
                    // Find the first result among tied candidates
                    let first_idx = tied_candidates
                        .iter()
                        .map(|(_, voters)| voters.iter().min().unwrap())
                        .min()
                        .unwrap();
                    tied_candidates
                        .iter()
                        .find(|(_, voters)| voters.contains(first_idx))
                        .unwrap()
                        .0
                }
                TieBreaker::PreferContinue => {
                    tied_candidates
                        .iter()
                        .find(|(key, _)| key.contains("Continue"))
                        .map(|(key, _)| key)
                        .unwrap_or(&tied_candidates[0].0)
                }
                TieBreaker::PreferAction => {
                    tied_candidates
                        .iter()
                        .find(|(key, _)| !key.contains("Continue"))
                        .map(|(key, _)| key)
                        .unwrap_or(&tied_candidates[0].0)
                }
                TieBreaker::ByPriority => {
                    // Would need access to hook priorities here
                    tied_candidates[0].0
                }
            }
        } else {
            candidates[0].0
        };

        // Find the first result matching this vote
        for (idx, result) in self.results.iter().enumerate() {
            if format!("{:?}", result.variant_name()) == *winner_key {
                info!(
                    "Voting winner: {} with {}/{} votes (>= {} required)",
                    result.description(),
                    self.votes[winner_key].len(),
                    self.total_votes,
                    required_votes
                );
                return result.clone();
            }
        }

        HookResult::Continue
    }

    pub fn get_vote_summary(&self) -> HashMap<String, usize> {
        self.votes
            .iter()
            .map(|(key, voters)| (key.clone(), voters.len()))
            .collect()
    }
}

impl Default for VotingAggregator {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait to get variant name
trait VariantName {
    fn variant_name(&self) -> &'static str;
}

impl VariantName for HookResult {
    fn variant_name(&self) -> &'static str {
        match self {
            HookResult::Continue => "Continue",
            HookResult::Modified(_) => "Modified",
            HookResult::Cancel(_) => "Cancel",
            HookResult::Replace(_) => "Replace",
            HookResult::Redirect(_) => "Redirect",
            HookResult::Retry { .. } => "Retry",
            HookResult::Cache { .. } => "Cache",
            HookResult::Fork { .. } => "Fork",
            HookResult::Skipped => "Skipped",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::FnHook;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    #[tokio::test]
    async fn test_simple_majority() {
        let hook = VotingHook::builder("test_majority")
            .add_hook(FnHook::new("h1", |_| Ok(HookResult::Continue)))
            .add_hook(FnHook::new("h2", |_| Ok(HookResult::Continue)))
            .add_hook(FnHook::new("h3", |_| Ok(HookResult::Cancel("Veto".to_string()))))
            .with_threshold(0.5)
            .build();

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        // 2 out of 3 voted Continue (66.7% > 50%)
        assert!(matches!(result, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_unanimous_requirement() {
        let hook = VotingHook::builder("test_unanimous")
            .add_hook(FnHook::new("h1", |_| Ok(HookResult::Continue)))
            .add_hook(FnHook::new("h2", |_| Ok(HookResult::Continue)))
            .add_hook(FnHook::new("h3", |_| Ok(HookResult::Modified(serde_json::json!({"value": 1})))))
            .with_threshold(1.0) // Require unanimous vote
            .build();

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        // No unanimous winner, defaults to Continue
        assert!(matches!(result, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_supermajority() {
        let hook = VotingHook::builder("test_supermajority")
            .add_hook(FnHook::new("h1", |_| Ok(HookResult::Modified(serde_json::json!({"v": 1})))))
            .add_hook(FnHook::new("h2", |_| Ok(HookResult::Modified(serde_json::json!({"v": 1})))))
            .add_hook(FnHook::new("h3", |_| Ok(HookResult::Modified(serde_json::json!({"v": 1})))))
            .add_hook(FnHook::new("h4", |_| Ok(HookResult::Continue)))
            .with_threshold(0.75) // 75% supermajority
            .build();

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        // 3 out of 4 voted Modified (75% >= 75%)
        assert!(matches!(result, HookResult::Modified(_)));
    }
    #[test]
    fn test_voting_aggregator() {
        let mut aggregator = VotingAggregator::new();
        
        aggregator.add_vote(0, HookResult::Continue);
        aggregator.add_vote(1, HookResult::Continue);
        aggregator.add_vote(2, HookResult::Cancel("Stop".to_string()));
        aggregator.add_vote(3, HookResult::Continue);
        
        let winner = aggregator.get_winner(0.5, TieBreaker::FirstResult);
        assert!(matches!(winner, HookResult::Continue)); // 3/4 = 75% > 50%
        
        let summary = aggregator.get_vote_summary();
        assert_eq!(summary["Continue"], 3);
        assert_eq!(summary["Cancel"], 1);
    }
    #[test]
    fn test_tie_breaker_strategies() {
        let mut aggregator = VotingAggregator::new();
        
        // Create a tie: 2 Continue, 2 Cancel
        aggregator.add_vote(0, HookResult::Continue);
        aggregator.add_vote(1, HookResult::Cancel("Stop".to_string()));
        aggregator.add_vote(2, HookResult::Continue);
        aggregator.add_vote(3, HookResult::Cancel("Stop".to_string()));
        
        // Test FirstResult tie breaker
        let winner = aggregator.get_winner(0.5, TieBreaker::FirstResult);
        assert!(matches!(winner, HookResult::Continue)); // First vote was Continue
        
        // Test PreferAction tie breaker
        let winner = aggregator.get_winner(0.5, TieBreaker::PreferAction);
        assert!(matches!(winner, HookResult::Cancel(_))); // Prefer non-Continue
        
        // Test PreferContinue tie breaker  
        let winner = aggregator.get_winner(0.5, TieBreaker::PreferContinue);
        assert!(matches!(winner, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_empty_voting() {
        let hook = VotingHook::new("empty_voting");
        
        assert!(hook.is_empty());
        assert_eq!(hook.len(), 0);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }
}
