//! ABOUTME: Testing utilities and helpers for rs-llmspell framework
//! ABOUTME: Provides mocks, generators, benchmarks, and test fixtures

//! Testing utilities for the LLMSpell framework.
//!
//! This crate provides comprehensive testing utilities including:
//! - Mock implementations of core traits for unit testing
//! - Property-based test generators using proptest
//! - Benchmark helpers for criterion performance testing
//! - Common test fixtures and data generators
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::mocks::MockBaseAgent;
//! use llmspell_testing::generators::component_id_strategy;
//! use proptest::prelude::*;
//! use llmspell_core::traits::base_agent::AgentOutput;
//!
//! // Use mock agent in tests
//! let mut mock = MockBaseAgent::new();
//! mock.expect_execute()
//!     .returning(|_, _| Ok(AgentOutput::new("test".to_string())));
//!
//! // Generate test data with proptest
//! proptest! {
//!     #[test]
//!     fn test_with_random_component_id(id in component_id_strategy()) {
//!         // Test with generated component ID
//!     }
//! }
//! ```

pub mod benchmarks;
pub mod fixtures;
pub mod generators;
pub mod mocks;
