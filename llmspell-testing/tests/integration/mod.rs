// ABOUTME: Integration tests module for cross-crate functionality
// ABOUTME: Tests interactions between different llmspell components

//! Integration test suite for llmspell framework
//!
//! This module contains tests that verify the integration between
//! different components of the llmspell framework.

// Component integration tests
#[cfg(test)]
mod component_state_integration;

// State migration tests
#[cfg(test)]
mod state_migration;

// Backup and recovery tests
#[cfg(test)]
mod backup_recovery;

// TODO: Add these test modules as part of Task 5.7.4
// Tool integration tests
// #[cfg(test)]
// mod tool_integration;

// Workflow integration tests
// #[cfg(test)]
// mod workflow_integration;

// Agent-tool integration
// #[cfg(test)]
// mod agent_tool_integration;

// Hook system integration
// #[cfg(test)]
// mod hook_integration;

// Event system integration
// #[cfg(test)]
// mod event_integration;

// Lua bridge integration
// #[cfg(test)]
// mod lua_bridge_integration;

// End-to-end integration scenarios
// #[cfg(test)]
// mod e2e_integration;
