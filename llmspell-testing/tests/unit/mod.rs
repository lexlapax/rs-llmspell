// ABOUTME: Unit tests module for all llmspell crates
// ABOUTME: Organizes unit tests by crate for easy navigation and execution

//! Unit test suite for llmspell framework
//!
//! This module consolidates unit tests from all llmspell crates.
//! Tests are organized by crate and can be run selectively using features.

// Core crate unit tests
#[cfg(test)]
mod core_tests;

// Agents crate unit tests
#[cfg(test)]
mod agents_tests;

// Tools crate unit tests
#[cfg(test)]
mod tools_tests;

// Workflows crate unit tests
#[cfg(test)]
mod workflows_tests;

// Bridge crate unit tests
#[cfg(test)]
mod bridge_tests;

// State persistence unit tests
#[cfg(test)]
mod state_tests;

// Hooks unit tests
#[cfg(test)]
mod hooks_tests;

// Events unit tests
#[cfg(test)]
mod events_tests;

// Storage unit tests
#[cfg(test)]
mod storage_tests;

// Utils unit tests
#[cfg(test)]
mod utils_tests;
