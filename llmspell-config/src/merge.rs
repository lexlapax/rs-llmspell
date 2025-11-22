//! Configuration merging logic for layer composition
//!
//! Provides deep merging of LLMSpellConfig instances, enabling layer-based
//! configuration composition. Later layers override earlier ones.
//!
//! # Merge Strategy
//!
//! - **Primitive fields**: Override if non-default in source
//! - **Option fields**: Override if Some in source, keep base if None
//! - **Collections**: Merge HashMaps by key, override Vecs entirely
//! - **Nested structs**: Recursively merge all fields

use crate::{
    debug::DebugConfig, engines::EngineConfigs, EventFilterConfig, EventsConfig,
    GlobalRuntimeConfig, HookConfig, LLMSpellConfig, ProviderManagerConfig, RAGConfig,
    SessionConfig, StatePersistenceConfig, ToolsConfig,
};

/// Merge configuration layers
///
/// Applies `source` on top of `base`, with source values taking precedence.
/// This is the main entry point for layer composition.
///
/// # Arguments
///
/// * `base` - Base configuration to merge into (modified in place)
/// * `source` - Source configuration to merge from
///
/// # Examples
///
/// ```no_run
/// use llmspell_config::{LLMSpellConfig, merge::merge_config};
///
/// let mut base = LLMSpellConfig::default();
/// let source = LLMSpellConfig::default();
/// merge_config(&mut base, source);
/// ```
pub fn merge_config(base: &mut LLMSpellConfig, source: LLMSpellConfig) {
    // Merge simple fields - override if non-default
    if source.default_engine != LLMSpellConfig::default().default_engine {
        base.default_engine = source.default_engine;
    }

    // Merge complex fields
    merge_engines(&mut base.engines, source.engines);
    merge_providers(&mut base.providers, source.providers);
    merge_runtime(&mut base.runtime, source.runtime);
    merge_tools(&mut base.tools, source.tools);
    merge_hooks(&mut base.hooks, source.hooks);
    merge_events(&mut base.events, source.events);
    merge_debug(&mut base.debug, source.debug);
    merge_rag(&mut base.rag, source.rag);
}

/// Merge engine configurations
fn merge_engines(base: &mut EngineConfigs, source: EngineConfigs) {
    // Merge Lua config fields individually
    let default_lua = crate::engines::LuaConfig::default();
    if source.lua.stdlib != default_lua.stdlib {
        base.lua.stdlib = source.lua.stdlib;
    }
    if source.lua.max_memory_bytes.is_some() {
        base.lua.max_memory_bytes = source.lua.max_memory_bytes;
    }
    if source.lua.enable_debug != default_lua.enable_debug {
        base.lua.enable_debug = source.lua.enable_debug;
    }
    if source.lua.timeout_ms.is_some() {
        base.lua.timeout_ms = source.lua.timeout_ms;
    }

    // Merge JavaScript config fields
    let default_js = crate::engines::JSConfig::default();
    if source.javascript.strict_mode != default_js.strict_mode {
        base.javascript.strict_mode = source.javascript.strict_mode;
    }
    if source.javascript.max_heap_size_bytes.is_some() {
        base.javascript.max_heap_size_bytes = source.javascript.max_heap_size_bytes;
    }
    if source.javascript.enable_console != default_js.enable_console {
        base.javascript.enable_console = source.javascript.enable_console;
    }
    if source.javascript.timeout_ms.is_some() {
        base.javascript.timeout_ms = source.javascript.timeout_ms;
    }

    // Merge custom engines (HashMap merge - source entries override)
    for (name, config) in source.custom {
        base.custom.insert(name, config);
    }
}

/// Merge provider configurations
fn merge_providers(base: &mut ProviderManagerConfig, source: ProviderManagerConfig) {
    // Override default provider if set
    if source.default_provider.is_some() {
        base.default_provider = source.default_provider;
    }

    // Merge individual providers (HashMap merge - source entries override)
    for (name, provider) in source.providers {
        base.providers.insert(name, provider);
    }
}

/// Merge runtime configurations
fn merge_runtime(base: &mut GlobalRuntimeConfig, source: GlobalRuntimeConfig) {
    let default_runtime = GlobalRuntimeConfig::default();

    // Override simple fields if non-default
    if source.max_concurrent_scripts != default_runtime.max_concurrent_scripts {
        base.max_concurrent_scripts = source.max_concurrent_scripts;
    }
    if source.script_timeout_seconds != default_runtime.script_timeout_seconds {
        base.script_timeout_seconds = source.script_timeout_seconds;
    }
    if source.enable_streaming != default_runtime.enable_streaming {
        base.enable_streaming = source.enable_streaming;
    }

    // Merge nested configs
    merge_security(&mut base.security, source.security);
    merge_state_persistence(&mut base.state_persistence, source.state_persistence);
    merge_sessions(&mut base.sessions, source.sessions);
    merge_memory(&mut base.memory, source.memory);
}

/// Merge security configurations
fn merge_security(base: &mut crate::SecurityConfig, source: crate::SecurityConfig) {
    let default_security = crate::SecurityConfig::default();

    if source.allow_file_access != default_security.allow_file_access {
        base.allow_file_access = source.allow_file_access;
    }
    if source.allow_network_access != default_security.allow_network_access {
        base.allow_network_access = source.allow_network_access;
    }
    if source.allow_process_spawn != default_security.allow_process_spawn {
        base.allow_process_spawn = source.allow_process_spawn;
    }
    if source.max_memory_bytes.is_some() {
        base.max_memory_bytes = source.max_memory_bytes;
    }
    if source.max_execution_time_ms.is_some() {
        base.max_execution_time_ms = source.max_execution_time_ms;
    }
}

/// Merge state persistence configurations
fn merge_state_persistence(base: &mut StatePersistenceConfig, source: StatePersistenceConfig) {
    let default_state = StatePersistenceConfig::default();

    if source.enabled != default_state.enabled {
        base.enabled = source.enabled;
    }
    if source.migration_enabled != default_state.migration_enabled {
        base.migration_enabled = source.migration_enabled;
    }
    if source.backup_on_migration != default_state.backup_on_migration {
        base.backup_on_migration = source.backup_on_migration;
    }
    if source.backup_enabled != default_state.backup_enabled {
        base.backup_enabled = source.backup_enabled;
    }
    if source.backend_type != default_state.backend_type {
        base.backend_type = source.backend_type;
    }
    if source.schema_directory.is_some() {
        base.schema_directory = source.schema_directory;
    }
    if source.max_state_size_bytes.is_some() {
        base.max_state_size_bytes = source.max_state_size_bytes;
    }
    if source.backup.is_some() {
        base.backup = source.backup;
    }
}

/// Merge session configurations
fn merge_sessions(base: &mut SessionConfig, source: SessionConfig) {
    let default_sessions = SessionConfig::default();

    if source.enabled != default_sessions.enabled {
        base.enabled = source.enabled;
    }
    if source.max_sessions != default_sessions.max_sessions {
        base.max_sessions = source.max_sessions;
    }
    if source.max_artifacts_per_session != default_sessions.max_artifacts_per_session {
        base.max_artifacts_per_session = source.max_artifacts_per_session;
    }
    if source.artifact_compression_threshold != default_sessions.artifact_compression_threshold {
        base.artifact_compression_threshold = source.artifact_compression_threshold;
    }
    if source.session_timeout_seconds != default_sessions.session_timeout_seconds {
        base.session_timeout_seconds = source.session_timeout_seconds;
    }
    if source.storage_backend != default_sessions.storage_backend {
        base.storage_backend = source.storage_backend;
    }
}

/// Merge memory configurations
fn merge_memory(base: &mut crate::memory::MemoryConfig, source: crate::memory::MemoryConfig) {
    let default_memory = crate::memory::MemoryConfig::default();

    if source.enabled != default_memory.enabled {
        base.enabled = source.enabled;
    }

    // Merge consolidation config
    merge_consolidation(&mut base.consolidation, source.consolidation);

    // Merge daemon config
    merge_daemon(&mut base.daemon, source.daemon);
}

/// Merge consolidation configurations
fn merge_consolidation(
    base: &mut crate::memory::ConsolidationConfig,
    source: crate::memory::ConsolidationConfig,
) {
    let default_consolidation = crate::memory::ConsolidationConfig::default();

    if source.provider_name.is_some() {
        base.provider_name = source.provider_name;
    }
    if source.batch_size != default_consolidation.batch_size {
        base.batch_size = source.batch_size;
    }
    if source.max_concurrent != default_consolidation.max_concurrent {
        base.max_concurrent = source.max_concurrent;
    }
    if source.active_session_threshold_secs != default_consolidation.active_session_threshold_secs {
        base.active_session_threshold_secs = source.active_session_threshold_secs;
    }
}

/// Merge daemon configurations
fn merge_daemon(base: &mut crate::memory::DaemonConfig, source: crate::memory::DaemonConfig) {
    let default_daemon = crate::memory::DaemonConfig::default();

    if source.enabled != default_daemon.enabled {
        base.enabled = source.enabled;
    }
    if source.fast_interval_secs != default_daemon.fast_interval_secs {
        base.fast_interval_secs = source.fast_interval_secs;
    }
    if source.normal_interval_secs != default_daemon.normal_interval_secs {
        base.normal_interval_secs = source.normal_interval_secs;
    }
    if source.slow_interval_secs != default_daemon.slow_interval_secs {
        base.slow_interval_secs = source.slow_interval_secs;
    }
    if source.queue_threshold_fast != default_daemon.queue_threshold_fast {
        base.queue_threshold_fast = source.queue_threshold_fast;
    }
    if source.queue_threshold_slow != default_daemon.queue_threshold_slow {
        base.queue_threshold_slow = source.queue_threshold_slow;
    }
    if source.shutdown_max_wait_secs != default_daemon.shutdown_max_wait_secs {
        base.shutdown_max_wait_secs = source.shutdown_max_wait_secs;
    }
    if source.health_check_interval_secs != default_daemon.health_check_interval_secs {
        base.health_check_interval_secs = source.health_check_interval_secs;
    }
}

/// Merge tools configurations
fn merge_tools(base: &mut ToolsConfig, source: ToolsConfig) {
    // Merge file operations config
    let default_file_ops = crate::tools::FileOperationsConfig::default();
    if source.file_operations.enabled != default_file_ops.enabled {
        base.file_operations.enabled = source.file_operations.enabled;
    }
    if source.file_operations.max_file_size != default_file_ops.max_file_size {
        base.file_operations.max_file_size = source.file_operations.max_file_size;
    }
    if !source.file_operations.allowed_paths.is_empty() {
        base.file_operations.allowed_paths = source.file_operations.allowed_paths;
    }

    // Merge network config if present
    if source.network.is_some() {
        base.network = source.network;
    }

    // Override rate limit if set
    if source.rate_limit_per_minute.is_some() {
        base.rate_limit_per_minute = source.rate_limit_per_minute;
    }
}

/// Merge hook configurations
fn merge_hooks(base: &mut Option<HookConfig>, source: Option<HookConfig>) {
    match (base.as_mut(), source) {
        (Some(base_hook), Some(source_hook)) => {
            let default_hook = HookConfig::default();

            if source_hook.enabled != default_hook.enabled {
                base_hook.enabled = source_hook.enabled;
            }
            if source_hook.rate_limit_per_minute.is_some() {
                base_hook.rate_limit_per_minute = source_hook.rate_limit_per_minute;
            }
            if source_hook.timeout_ms.is_some() {
                base_hook.timeout_ms = source_hook.timeout_ms;
            }
            if source_hook.circuit_breaker_threshold.is_some() {
                base_hook.circuit_breaker_threshold = source_hook.circuit_breaker_threshold;
            }
        }
        (None, Some(source_hook)) => {
            *base = Some(source_hook);
        }
        _ => {} // Keep base as-is if source is None
    }
}

/// Merge events configurations
fn merge_events(base: &mut EventsConfig, source: EventsConfig) {
    let default_events = EventsConfig::default();

    if source.enabled != default_events.enabled {
        base.enabled = source.enabled;
    }
    if source.buffer_size != default_events.buffer_size {
        base.buffer_size = source.buffer_size;
    }
    if source.emit_timing_events != default_events.emit_timing_events {
        base.emit_timing_events = source.emit_timing_events;
    }
    if source.emit_state_events != default_events.emit_state_events {
        base.emit_state_events = source.emit_state_events;
    }
    if source.emit_debug_events != default_events.emit_debug_events {
        base.emit_debug_events = source.emit_debug_events;
    }
    if source.max_events_per_second.is_some() {
        base.max_events_per_second = source.max_events_per_second;
    }

    // Merge filtering
    merge_event_filter(&mut base.filtering, source.filtering);

    // Merge export
    merge_event_export(&mut base.export, source.export);
}

/// Merge event filter configurations
fn merge_event_filter(base: &mut EventFilterConfig, source: EventFilterConfig) {
    let default_filter = EventFilterConfig::default();

    if source.include_types != default_filter.include_types {
        base.include_types = source.include_types;
    }
    if !source.exclude_types.is_empty() {
        base.exclude_types = source.exclude_types;
    }
    if source.include_components != default_filter.include_components {
        base.include_components = source.include_components;
    }
    if !source.exclude_components.is_empty() {
        base.exclude_components = source.exclude_components;
    }
}

/// Merge event export configurations
fn merge_event_export(base: &mut crate::EventExportConfig, source: crate::EventExportConfig) {
    let default_export = crate::EventExportConfig::default();

    if source.stdout != default_export.stdout {
        base.stdout = source.stdout;
    }
    if source.file.is_some() {
        base.file = source.file;
    }
    if source.webhook.is_some() {
        base.webhook = source.webhook;
    }
    if source.pretty_json != default_export.pretty_json {
        base.pretty_json = source.pretty_json;
    }
}

/// Merge debug configurations
fn merge_debug(base: &mut DebugConfig, source: DebugConfig) {
    let default_debug = DebugConfig::default();

    if source.enabled != default_debug.enabled {
        base.enabled = source.enabled;
    }
    if source.level != default_debug.level {
        base.level = source.level;
    }

    // For nested configs, override entirely if they differ from default
    // More granular merging could be added if needed
    base.output = source.output;
    base.module_filters = source.module_filters;
    base.performance = source.performance;
    base.stack_trace = source.stack_trace;
}

/// Merge RAG configurations
fn merge_rag(base: &mut RAGConfig, source: RAGConfig) {
    let default_rag = RAGConfig::default();

    if source.enabled != default_rag.enabled {
        base.enabled = source.enabled;
    }
    if source.multi_tenant != default_rag.multi_tenant {
        base.multi_tenant = source.multi_tenant;
    }

    // Merge vector storage config
    merge_vector_storage(&mut base.vector_storage, source.vector_storage);

    // Merge embedding config
    merge_embedding(&mut base.embedding, source.embedding);

    // Merge chunking config
    merge_chunking(&mut base.chunking, source.chunking);

    // Merge cache config
    merge_rag_cache(&mut base.cache, source.cache);
}

/// Merge vector storage configurations
fn merge_vector_storage(
    base: &mut crate::rag::VectorStorageConfig,
    source: crate::rag::VectorStorageConfig,
) {
    let default_vector = crate::rag::VectorStorageConfig::default();

    if source.dimensions != default_vector.dimensions {
        base.dimensions = source.dimensions;
    }
    if source.backend != default_vector.backend {
        base.backend = source.backend;
    }
    if source.persistence_path.is_some() {
        base.persistence_path = source.persistence_path;
    }
    if source.max_memory_mb.is_some() {
        base.max_memory_mb = source.max_memory_mb;
    }

    // Merge HNSW config
    merge_hnsw(&mut base.hnsw, source.hnsw);
}

/// Merge HNSW configurations
fn merge_hnsw(base: &mut crate::rag::HNSWConfig, source: crate::rag::HNSWConfig) {
    let default_hnsw = crate::rag::HNSWConfig::default();

    if source.m != default_hnsw.m {
        base.m = source.m;
    }
    if source.ef_construction != default_hnsw.ef_construction {
        base.ef_construction = source.ef_construction;
    }
    if source.ef_search != default_hnsw.ef_search {
        base.ef_search = source.ef_search;
    }
    if source.max_elements != default_hnsw.max_elements {
        base.max_elements = source.max_elements;
    }
    if source.seed.is_some() {
        base.seed = source.seed;
    }
    if source.metric != default_hnsw.metric {
        base.metric = source.metric;
    }
    if source.allow_replace_deleted != default_hnsw.allow_replace_deleted {
        base.allow_replace_deleted = source.allow_replace_deleted;
    }
    if source.num_threads.is_some() {
        base.num_threads = source.num_threads;
    }
}

/// Merge embedding configurations
fn merge_embedding(base: &mut crate::rag::EmbeddingConfig, source: crate::rag::EmbeddingConfig) {
    let default_embedding = crate::rag::EmbeddingConfig::default();

    if source.default_provider != default_embedding.default_provider {
        base.default_provider = source.default_provider;
    }
    if source.cache_enabled != default_embedding.cache_enabled {
        base.cache_enabled = source.cache_enabled;
    }
    if source.cache_size != default_embedding.cache_size {
        base.cache_size = source.cache_size;
    }
    if source.cache_ttl_seconds != default_embedding.cache_ttl_seconds {
        base.cache_ttl_seconds = source.cache_ttl_seconds;
    }
    if source.batch_size != default_embedding.batch_size {
        base.batch_size = source.batch_size;
    }
    if source.timeout_seconds != default_embedding.timeout_seconds {
        base.timeout_seconds = source.timeout_seconds;
    }
    if source.max_retries != default_embedding.max_retries {
        base.max_retries = source.max_retries;
    }
}

/// Merge chunking configurations
fn merge_chunking(base: &mut crate::rag::ChunkingConfig, source: crate::rag::ChunkingConfig) {
    let default_chunking = crate::rag::ChunkingConfig::default();

    if source.strategy != default_chunking.strategy {
        base.strategy = source.strategy;
    }
    if source.chunk_size != default_chunking.chunk_size {
        base.chunk_size = source.chunk_size;
    }
    if source.overlap != default_chunking.overlap {
        base.overlap = source.overlap;
    }
    if source.max_chunk_size != default_chunking.max_chunk_size {
        base.max_chunk_size = source.max_chunk_size;
    }
    if source.min_chunk_size != default_chunking.min_chunk_size {
        base.min_chunk_size = source.min_chunk_size;
    }
}

/// Merge RAG cache configurations
fn merge_rag_cache(base: &mut crate::rag::RAGCacheConfig, source: crate::rag::RAGCacheConfig) {
    let default_cache = crate::rag::RAGCacheConfig::default();

    if source.search_cache_enabled != default_cache.search_cache_enabled {
        base.search_cache_enabled = source.search_cache_enabled;
    }
    if source.search_cache_size != default_cache.search_cache_size {
        base.search_cache_size = source.search_cache_size;
    }
    if source.search_cache_ttl_seconds != default_cache.search_cache_ttl_seconds {
        base.search_cache_ttl_seconds = source.search_cache_ttl_seconds;
    }
    if source.document_cache_enabled != default_cache.document_cache_enabled {
        base.document_cache_enabled = source.document_cache_enabled;
    }
    if source.document_cache_size_mb != default_cache.document_cache_size_mb {
        base.document_cache_size_mb = source.document_cache_size_mb;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_simple_fields() {
        let mut base = LLMSpellConfig::default();
        let source = LLMSpellConfig {
            default_engine: "javascript".to_string(),
            ..Default::default()
        };

        merge_config(&mut base, source);

        assert_eq!(base.default_engine, "javascript");
    }

    #[test]
    fn test_merge_preserves_base_when_source_default() {
        let mut base = LLMSpellConfig {
            default_engine: "custom".to_string(),
            ..Default::default()
        };

        let source = LLMSpellConfig::default(); // default_engine = "lua"

        merge_config(&mut base, source);

        // Should preserve base since source has default value
        assert_eq!(base.default_engine, "custom");
    }

    #[test]
    fn test_merge_runtime_config() {
        let mut base = LLMSpellConfig::default();
        let mut source = LLMSpellConfig::default();
        source.runtime.max_concurrent_scripts = 20;
        source.runtime.script_timeout_seconds = 600;

        merge_config(&mut base, source);

        assert_eq!(base.runtime.max_concurrent_scripts, 20);
        assert_eq!(base.runtime.script_timeout_seconds, 600);
    }

    #[test]
    fn test_merge_security_config() {
        let mut base = LLMSpellConfig::default();
        let mut source = LLMSpellConfig::default();
        source.runtime.security.allow_file_access = true;
        source.runtime.security.max_memory_bytes = Some(100_000_000);

        merge_config(&mut base, source);

        assert!(base.runtime.security.allow_file_access);
        assert_eq!(base.runtime.security.max_memory_bytes, Some(100_000_000));
    }

    #[test]
    fn test_merge_option_fields() {
        let mut base = LLMSpellConfig {
            hooks: None,
            ..Default::default()
        };

        let mut source = LLMSpellConfig::default();
        let hook_config = HookConfig {
            enabled: true,
            ..Default::default()
        };
        source.hooks = Some(hook_config);

        merge_config(&mut base, source);

        assert!(base.hooks.is_some());
        assert!(base.hooks.unwrap().enabled);
    }

    #[test]
    fn test_merge_rag_config() {
        let mut base = LLMSpellConfig::default();
        let mut source = LLMSpellConfig::default();
        source.rag.enabled = true;
        source.rag.vector_storage.dimensions = 768;
        source.rag.vector_storage.hnsw.m = 32;

        merge_config(&mut base, source);

        assert!(base.rag.enabled);
        assert_eq!(base.rag.vector_storage.dimensions, 768);
        assert_eq!(base.rag.vector_storage.hnsw.m, 32);
    }

    #[test]
    fn test_merge_preserves_unset_fields() {
        let mut base = LLMSpellConfig {
            default_engine: "custom".to_string(),
            ..Default::default()
        };
        base.runtime.max_concurrent_scripts = 15;

        let mut source = LLMSpellConfig::default();
        source.runtime.script_timeout_seconds = 600;
        // source.default_engine remains "lua" (default)

        merge_config(&mut base, source);

        // Should preserve base.default_engine since source has default
        assert_eq!(base.default_engine, "custom");
        // Should update timeout
        assert_eq!(base.runtime.script_timeout_seconds, 600);
        // Should preserve base.max_concurrent_scripts since source has default
        assert_eq!(base.runtime.max_concurrent_scripts, 15);
    }
}
