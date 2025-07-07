// ABOUTME: Shared utilities library for the LLMSpell framework
// ABOUTME: Provides async helpers, file operations, string manipulation, system info, error builders, ID generation, and serialization utilities

//! # `LLMSpell` Utilities
//!
//! This crate provides shared utility functions and helpers used across the `LLMSpell` framework.
//!
//! ## Features
//!
//! - **Async Utilities**: Helpers for working with async operations, timeouts, and concurrency
//! - **File Utilities**: Cross-platform file operations and path manipulation
//! - **String Utilities**: String manipulation, formatting, and validation helpers
//! - **System Info**: System information gathering and environment utilities
//! - **Error Builders**: Convenient error construction helpers
//! - **ID Generator**: UUID-based ID generation for components
//! - **Serialization**: Common serialization/deserialization utilities
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llmspell_utils::{async_utils, file_utils, id_generator};
//! use std::time::Duration;
//!
//! // Generate a unique component ID
//! let id = id_generator::generate_component_id("agent");
//!
//! // Use async utilities for timeout operations (when implemented)
//! let result = async_utils::timeout(Duration::from_secs(30), async {
//!     // Your async operation
//! }).await?;
//! ```

#![warn(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Async operation utilities and helpers
pub mod async_utils;

/// File system operations and path utilities
pub mod file_utils;

/// String manipulation and formatting helpers
pub mod string_utils;

/// System information gathering utilities
pub mod system_info;

/// Error construction and builder utilities
pub mod error_builders;

/// Component ID generation utilities
pub mod id_generator;

/// Serialization and deserialization helpers
pub mod serialization;

// Re-export commonly used types and functions
pub use async_utils::{
    concurrent_map, race_to_success, retry_async, timeout, timeout_with_default, AsyncError,
    AsyncResult, BoxedResultFuture, Cancellable, RetryConfig,
};
pub use error_builders::{templates, BuiltError, ErrorBuilder, WithContext};
pub use file_utils::{
    append_file, copy_file, ensure_dir, expand_path, file_exists, get_metadata, is_absolute_path,
    join_paths, list_dir, move_file, normalize_path, parent_dir, read_file,
    remove_dir_all_if_exists, remove_file_if_exists, write_file, write_file_atomic, DirEntry,
    FileMetadata,
};
pub use id_generator::{
    generate_component_id, generate_deterministic_id, generate_short_id, validate_component_id,
    ComponentId, ComponentIdBuilder, NAMESPACE_AGENT, NAMESPACE_TOOL, NAMESPACE_WORKFLOW,
};
pub use serialization::{
    convert_format, from_json, from_toml, from_yaml, json, merge_json, to_json, to_json_pretty,
    to_toml, to_yaml, Format,
};
pub use string_utils::{
    dedent, indent, is_valid_identifier, normalize_whitespace, sanitize, to_camel_case,
    to_pascal_case, to_snake_case, truncate, word_wrap,
};
pub use system_info::{
    find_executable, format_bytes, get_cpu_count, get_home_directory, get_hostname,
    get_system_info, get_username, OperatingSystem, SystemInfo,
};
