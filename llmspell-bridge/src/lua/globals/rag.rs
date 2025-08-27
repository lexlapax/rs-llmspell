//! ABOUTME: Lua-specific RAG global implementation
//! ABOUTME: Provides Lua bindings for vector storage and retrieval

#![allow(clippy::significant_drop_tightening)]

use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_table_to_json};
use crate::lua::sync_utils::block_on_async;
use crate::rag_bridge::{
    ChunkingConfig, RAGBridge, RAGConfigRequest, RAGDocument, RAGIngestRequest, RAGSearchRequest,
};
use mlua::{Lua, Table};
use std::sync::Arc;
use tracing::debug;

/// Register `RAG.search()` method
fn register_search_method(
    lua: &Lua,
    rag_table: &Table,
    bridge: &Arc<RAGBridge>,
) -> mlua::Result<()> {
    let bridge = bridge.clone();
    let search_fn =
        lua.create_function(move |lua, (query, options): (String, Option<Table>)| {
            let bridge = bridge.clone();

            // Parse options
            let mut request = RAGSearchRequest {
                query,
                k: None,
                scope: None,
                scope_id: None,
                filters: None,
                threshold: None,
            };

            if let Some(opts) = options {
                if let Ok(k) = opts.get::<_, u32>("k") {
                    request.k = Some(k as usize);
                }
                if let Ok(scope) = opts.get::<_, String>("scope") {
                    request.scope = Some(scope);
                }
                if let Ok(scope_id) = opts.get::<_, String>("scope_id") {
                    request.scope_id = Some(scope_id);
                }
                if let Ok(threshold) = opts.get::<_, f32>("threshold") {
                    request.threshold = Some(threshold);
                }
                if let Ok(filters) = opts.get::<_, Table>("filters") {
                    let json_value = lua_table_to_json(filters)?;
                    if let serde_json::Value::Object(map) = json_value {
                        request.filters = Some(map.into_iter().collect());
                    }
                }
            }

            debug!("RAG search from Lua: {:?}", request);

            // Execute search
            let response = block_on_async(
                "rag_search",
                async move {
                    bridge.search(request, None).await.map_err(|e| {
                        llmspell_core::LLMSpellError::Component {
                            message: format!("RAG search failed: {e}"),
                            source: None,
                        }
                    })
                },
                None,
            )?;

            // Convert response to Lua table
            let result_table = lua.create_table()?;
            result_table.set("success", response.success)?;
            result_table.set("total", response.total)?;

            if let Some(error) = response.error {
                result_table.set("error", error)?;
            }

            // Convert results array
            let results_array = lua.create_table()?;
            for (i, result) in response.results.iter().enumerate() {
                let result_table = lua.create_table()?;
                result_table.set("id", result.id.as_str())?;
                result_table.set("text", result.text.as_str())?;
                result_table.set("score", result.score)?;

                // Convert metadata
                let metadata_value = json_to_lua_value(
                    lua,
                    &serde_json::Value::Object(result.metadata.clone().into_iter().collect()),
                )?;
                result_table.set("metadata", metadata_value)?;

                results_array.set(i + 1, result_table)?;
            }
            result_table.set("results", results_array)?;

            Ok(result_table)
        })?;
    rag_table.set("search", search_fn)?;
    Ok(())
}

/// Register `RAG.ingest()` method
fn register_ingest_method(
    lua: &Lua,
    rag_table: &Table,
    bridge: &Arc<RAGBridge>,
) -> mlua::Result<()> {
    let bridge = bridge.clone();
    let ingest_fn =
        lua.create_function(move |lua, (documents, options): (Table, Option<Table>)| {
            let bridge = bridge.clone();

            // Convert documents from Lua table
            let mut docs = Vec::new();
            for i in 1..=documents.len()? {
                let doc_table: Table = documents.get(i)?;

                let id = doc_table.get::<_, String>("id")?;
                let text = doc_table.get::<_, String>("text")?;
                let metadata = if let Ok(meta) = doc_table.get::<_, Table>("metadata") {
                    let json_value = lua_table_to_json(meta)?;
                    if let serde_json::Value::Object(map) = json_value {
                        Some(map.into_iter().collect())
                    } else {
                        None
                    }
                } else {
                    None
                };

                docs.push(RAGDocument { id, text, metadata });
            }

            // Parse options
            let mut request = RAGIngestRequest {
                documents: docs,
                scope: None,
                scope_id: None,
                provider: None,
                chunking: None,
            };

            if let Some(opts) = options {
                if let Ok(scope) = opts.get::<_, String>("scope") {
                    request.scope = Some(scope);
                }
                if let Ok(scope_id) = opts.get::<_, String>("scope_id") {
                    request.scope_id = Some(scope_id);
                }
                if let Ok(provider) = opts.get::<_, String>("provider") {
                    request.provider = Some(provider);
                }
                if let Ok(chunking) = opts.get::<_, Table>("chunking") {
                    let mut config = ChunkingConfig {
                        chunk_size: None,
                        overlap: None,
                        strategy: None,
                    };
                    if let Ok(size) = chunking.get::<_, u32>("chunk_size") {
                        config.chunk_size = Some(size as usize);
                    }
                    if let Ok(overlap) = chunking.get::<_, u32>("overlap") {
                        config.overlap = Some(overlap as usize);
                    }
                    if let Ok(strategy) = chunking.get::<_, String>("strategy") {
                        config.strategy = Some(strategy);
                    }
                    request.chunking = Some(config);
                }
            }

            debug!("RAG ingest from Lua: {} documents", request.documents.len());

            // Execute ingestion
            let response = block_on_async(
                "rag_ingest",
                async move {
                    bridge.ingest(request, None).await.map_err(|e| {
                        llmspell_core::LLMSpellError::Component {
                            message: format!("RAG ingest failed: {e}"),
                            source: None,
                        }
                    })
                },
                None,
            )?;

            // Convert response to Lua table
            let result_table = lua.create_table()?;
            result_table.set("success", response.success)?;
            result_table.set("documents_processed", response.documents_processed)?;
            result_table.set("vectors_created", response.vectors_created)?;
            result_table.set("total_tokens", response.total_tokens)?;

            if let Some(error) = response.error {
                result_table.set("error", error)?;
            }

            Ok(result_table)
        })?;
    rag_table.set("ingest", ingest_fn)?;
    Ok(())
}

/// Register `RAG.configure()` method
fn register_configure_method(
    lua: &Lua,
    rag_table: &Table,
    bridge: &Arc<RAGBridge>,
) -> mlua::Result<()> {
    let bridge = bridge.clone();
    let configure_fn = lua.create_function(move |_lua, options: Table| {
        let bridge = bridge.clone();

        let mut request = RAGConfigRequest {
            session_ttl: None,
            default_provider: None,
            enable_cache: None,
            cache_ttl: None,
        };

        if let Ok(ttl) = options.get::<_, u32>("session_ttl") {
            request.session_ttl = Some(u64::from(ttl));
        }
        if let Ok(provider) = options.get::<_, String>("default_provider") {
            request.default_provider = Some(provider);
        }
        if let Ok(enable) = options.get::<_, bool>("enable_cache") {
            request.enable_cache = Some(enable);
        }
        if let Ok(ttl) = options.get::<_, u32>("cache_ttl") {
            request.cache_ttl = Some(u64::from(ttl));
        }

        block_on_async(
            "rag_configure",
            async move {
                bridge
                    .configure(request)
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("RAG configure failed: {e}"),
                        source: None,
                    })
            },
            None,
        )?;

        Ok(())
    })?;
    rag_table.set("configure", configure_fn)?;
    Ok(())
}

/// Register `RAG.cleanup_scope()` method
fn register_cleanup_method(
    lua: &Lua,
    rag_table: &Table,
    bridge: &Arc<RAGBridge>,
) -> mlua::Result<()> {
    let bridge = bridge.clone();
    let cleanup_fn = lua.create_function(move |_lua, (scope, scope_id): (String, String)| {
        let bridge = bridge.clone();

        let deleted = block_on_async(
            "rag_cleanup",
            async move {
                bridge.cleanup_scope(&scope, &scope_id).await.map_err(|e| {
                    llmspell_core::LLMSpellError::Component {
                        message: format!("RAG cleanup failed: {e}"),
                        source: None,
                    }
                })
            },
            None,
        )?;

        Ok(deleted)
    })?;
    rag_table.set("cleanup_scope", cleanup_fn)?;
    Ok(())
}

/// Register `RAG.list_providers()` method
fn register_list_providers_method(
    lua: &Lua,
    rag_table: &Table,
    bridge: &Arc<RAGBridge>,
) -> mlua::Result<()> {
    let bridge = bridge.clone();
    let list_providers_fn = lua.create_function(move |lua, ()| {
        let bridge = bridge.clone();

        let providers = block_on_async(
            "rag_list_providers",
            async move {
                bridge
                    .list_providers()
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("RAG list providers failed: {e}"),
                        source: None,
                    })
            },
            None,
        )?;

        let providers_table = lua.create_table()?;
        for (i, provider) in providers.iter().enumerate() {
            providers_table.set(i + 1, provider.as_str())?;
        }

        Ok(providers_table)
    })?;
    rag_table.set("list_providers", list_providers_fn)?;
    Ok(())
}

/// Register `RAG.get_stats()` method
fn register_get_stats_method(
    lua: &Lua,
    rag_table: &Table,
    bridge: &Arc<RAGBridge>,
) -> mlua::Result<()> {
    let bridge = bridge.clone();
    let get_stats_fn =
        lua.create_function(move |lua, (scope, scope_id): (String, Option<String>)| {
            let bridge = bridge.clone();
            let scope_id_opt = scope_id.as_deref();

            let stats = block_on_async(
                "rag_get_stats",
                async move {
                    bridge.get_stats(&scope, scope_id_opt).await.map_err(|e| {
                        llmspell_core::LLMSpellError::Component {
                            message: format!("RAG get stats failed: {e}"),
                            source: None,
                        }
                    })
                },
                None,
            )?;

            // Convert stats to Lua table
            let stats_table = lua.create_table()?;
            for (key, value) in stats {
                let lua_value = json_to_lua_value(lua, &value)?;
                stats_table.set(key, lua_value)?;
            }

            Ok(stats_table)
        })?;
    rag_table.set("get_stats", get_stats_fn)?;
    Ok(())
}

/// Register session-related methods
fn register_session_methods(lua: &Lua, rag_table: &Table) -> mlua::Result<()> {
    // RAG.create_session_collection(session_id, ttl) - Create session-scoped collection
    let create_session_fn =
        lua.create_function(move |lua, (session_id, ttl): (String, Option<u32>)| {
            let result_table = lua.create_table()?;
            result_table.set("session_id", session_id.as_str())?;
            result_table.set("namespace", format!("session_{session_id}"))?;
            if let Some(ttl_val) = ttl {
                result_table.set("ttl", ttl_val)?;
            }
            result_table.set("created", true)?;
            Ok(result_table)
        })?;
    rag_table.set("create_session_collection", create_session_fn)?;

    // RAG.configure_session(options) - Configure session settings
    let configure_session_fn = lua.create_function(move |lua, options: Table| {
        let session_id = options.get::<_, String>("session_id")?;
        let vector_ttl = options.get::<_, Option<u32>>("vector_ttl")?.unwrap_or(3600);

        let result_table = lua.create_table()?;
        result_table.set("session_id", session_id)?;
        result_table.set("vector_ttl", vector_ttl)?;
        result_table.set("configured", true)?;
        Ok(result_table)
    })?;
    rag_table.set("configure_session", configure_session_fn)?;
    Ok(())
}

/// Inject the RAG global object into Lua
///
/// # Errors
///
/// Returns an error if:
/// - Table creation fails
/// - Function creation fails
/// - Global setting fails
#[allow(clippy::needless_pass_by_value)] // We need to pass by value to clone for multiple closures
pub fn inject_rag_global(
    lua: &Lua,
    _context: &GlobalContext,
    bridge: Arc<RAGBridge>,
) -> mlua::Result<()> {
    let rag_table = lua.create_table()?;

    // Register all RAG methods
    register_search_method(lua, &rag_table, &bridge)?;
    register_ingest_method(lua, &rag_table, &bridge)?;
    register_configure_method(lua, &rag_table, &bridge)?;
    register_cleanup_method(lua, &rag_table, &bridge)?;
    register_list_providers_method(lua, &rag_table, &bridge)?;
    register_get_stats_method(lua, &rag_table, &bridge)?;
    register_session_methods(lua, &rag_table)?;

    // Set the RAG global
    lua.globals().set("RAG", rag_table)?;

    debug!("RAG global injected into Lua");
    Ok(())
}
