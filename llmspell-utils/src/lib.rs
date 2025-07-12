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

/// Encoding and hashing utilities
pub mod encoding;

/// Time and date utilities
pub mod time;

/// File system monitoring utilities
pub mod file_monitor;

/// File and content search utilities
pub mod search;

/// Parameter extraction and validation utilities
pub mod params;

/// Common validation functions
pub mod validators;

/// Response building utilities
pub mod response;

/// Retry logic with exponential backoff
pub mod retry;

/// Rate limiting utilities
pub mod rate_limiter;

/// Timeout management utilities
pub mod timeout;

/// Connection pooling abstraction
pub mod connection_pool;

/// Progress reporting framework
pub mod progress;

/// Security utilities for DoS protection and resource limits
pub mod security;

/// API key management system
pub mod api_key_manager;

/// Persistent storage for API keys
pub mod api_key_persistent_storage;

// Re-export commonly used types and functions
pub use async_utils::{
    concurrent_map, race_to_success, retry_async, timeout, timeout_with_default, AsyncError,
    AsyncResult, BoxedResultFuture, Cancellable, RetryConfig,
};
pub use encoding::{
    base64_decode, base64_decode_url_safe, base64_encode, base64_encode_url_safe, from_hex_string,
    hash_data, hash_file, hash_string, to_hex_string, verify_hash, HashAlgorithm,
};
pub use error_builders::{templates, BuiltError, ErrorBuilder, WithContext};
pub use file_monitor::{debounce_events, should_watch_path, FileEvent, FileEventType, WatchConfig};
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
pub use params::{
    extract_bool_with_default, extract_direct_parameters, extract_optional_array,
    extract_optional_bool, extract_optional_f64, extract_optional_i64, extract_optional_object,
    extract_optional_string, extract_optional_typed, extract_optional_u64, extract_parameters,
    extract_required_array, extract_required_bool, extract_required_f64, extract_required_i64,
    extract_required_object, extract_required_string, extract_required_typed, extract_required_u64,
    extract_string_with_default, require_all_of, require_one_of,
};
pub use response::{
    error_response, file_operation_response, list_response, success_response, validation_response,
    ErrorDetails, ResponseBuilder, ValidationError,
};
pub use search::{
    search_in_directory, search_in_file, should_search_file, SearchMatch, SearchOptions,
    SearchResult,
};
pub use serialization::{
    convert_format, from_json, from_toml, from_yaml, json, merge_json, to_json, to_json_pretty,
    to_toml, to_yaml, Format,
};
pub use string_utils::{
    dedent, indent, is_valid_identifier, join_with, normalize_whitespace, replace_all, reverse,
    sanitize, split_by, substring, to_camel_case, to_lowercase, to_pascal_case, to_snake_case,
    to_uppercase, trim, truncate, word_wrap,
};
pub use system_info::{
    find_executable, format_bytes, get_cpu_count, get_home_directory, get_hostname,
    get_system_info, get_username, OperatingSystem, SystemInfo,
};
pub use time::{
    add_duration, convert_timezone, days_in_month, duration_between, end_of_day, format_datetime,
    format_duration, is_leap_year, now_local, now_utc, parse_datetime, start_of_day,
    subtract_duration, weekday_name, TimeError, TimeResult, DATE_FORMATS,
};
pub use validators::{
    sanitize_string, validate_date_format, validate_email, validate_enum, validate_file_size,
    validate_identifier, validate_is_directory, validate_is_file, validate_json_schema,
    validate_no_shell_injection, validate_not_empty, validate_not_empty_collection,
    validate_path_exists, validate_pattern, validate_range, validate_regex_pattern,
    validate_required_field, validate_resource_limit, validate_safe_path, validate_string_length,
    validate_url,
};

#[cfg(unix)]
pub use validators::validate_file_permissions;

// Re-export retry utilities
pub use retry::{
    retry, retry_default, AlwaysRetry, HttpStatusRetryPolicy, RetryBuilder, RetryError, RetryPolicy,
};

// Re-export rate limiter utilities
pub use rate_limiter::{
    RateLimitAlgorithm, RateLimitError, RateLimiter, RateLimiterBuilder, RateLimiterConfig,
};

// Re-export timeout utilities
pub use timeout::{
    with_timeout, with_timeout_config, CancellableTimeout, TimeoutBuilder, TimeoutConfig,
    TimeoutError, TimeoutExt, TimeoutManager,
};

// Re-export connection pool utilities
pub use connection_pool::{
    ConnectionFactory, ConnectionPool, PoolBuilder, PoolConfig, PoolError, PoolGuard, PoolStats,
    PoolableConnection,
};

// Re-export progress utilities
pub use progress::{
    ProgressBuilder, ProgressError, ProgressEvent, ProgressIteratorExt, ProgressReporter,
    ProgressTracker,
};

// Re-export security utilities
pub use security::{
    path::{PathSecurityConfig, PathSecurityValidator},
    ExpressionAnalyzer, ExpressionComplexity, ExpressionComplexityConfig,
};

// Re-export API key management utilities
pub use api_key_manager::{
    ApiKeyAction, ApiKeyAuditEntry, ApiKeyManager, ApiKeyMetadata, ApiKeyStorage, InMemoryStorage,
};
pub use api_key_persistent_storage::PersistentApiKeyStorage;
