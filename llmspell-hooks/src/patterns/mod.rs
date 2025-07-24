// ABOUTME: Pattern-specific hook implementations for common composition patterns
// ABOUTME: Exports specialized implementations of Sequential, Parallel, FirstMatch, and Voting patterns

pub mod sequential;
pub mod parallel;
pub mod voting;

pub use sequential::SequentialHook;
pub use parallel::ParallelHook;
pub use voting::VotingHook;

// Re-export common types
pub use crate::composite::{CompositionPattern, CompositeHook, CompositeHookBuilder};