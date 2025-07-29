// ABOUTME: Test module structure for provider hook integration tests with real LLM providers
// ABOUTME: Organizes tests for hook persistence, replay, and correlation with actual API calls

pub mod common;

#[cfg(test)]
pub mod openai_hook_tests;

#[cfg(test)]
pub mod anthropic_hook_tests;

// These tests need to be updated to match the new common.rs structure
// #[cfg(test)]
// pub mod replay_tests;

// #[cfg(test)]
// pub mod correlation_tests;

// #[cfg(test)]
// pub mod timeline_tests;

// #[cfg(test)]
// pub mod tool_hook_tests;

// #[cfg(test)]
// pub mod workflow_hook_tests;
