//! ABOUTME: Agent configuration management and loading
//! ABOUTME: Provides configuration templates and environment-based configuration

pub mod persistence_config;

pub use persistence_config::{presets, PersistenceConfigBuilder};
