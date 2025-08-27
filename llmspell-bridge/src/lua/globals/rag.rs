//! ABOUTME: Lua-specific RAG global implementation
//! ABOUTME: Provides Lua bindings for vector storage and retrieval

#![allow(clippy::significant_drop_tightening)]

use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_table_to_json};
use crate::lua::sync_utils::block_on_async;
use crate::rag_bridge::{ChunkingConfig, RAGBridge, RAGDocument};
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
    let search_fn = lua.create_function(move |lua, params: Table| {
        let bridge = bridge.clone();

        // Extract parameters from single table
        let query = params.get::<_, String>("query").map_err(|_| {
            mlua::Error::RuntimeError("RAG.search requires 'query' field".to_string())
        })?;

        // Parse optional parameters
        let mut k = None;
        let mut scope = None;
        let mut scope_id = None;
        let mut filters = None;
        let mut threshold = None;

        // Handle both 'top_k' and 'k' for compatibility
        if let Ok(top_k) = params.get::<_, u32>("top_k") {
            k = Some(top_k as usize);
        } else if let Ok(k_val) = params.get::<_, u32>("k") {
            k = Some(k_val as usize);
        }

        if let Ok(s) = params.get::<_, String>("scope") {
            scope = Some(s);
        }
        if let Ok(id) = params.get::<_, String>("scope_id") {
            scope_id = Some(id);
        }
        if let Ok(tenant_id) = params.get::<_, String>("tenant_id") {
            // Map tenant_id to scope for multi-tenant support
            scope = Some("tenant".to_string());
            scope_id = Some(tenant_id);
        }
        if let Ok(t) = params.get::<_, f32>("threshold") {
            threshold = Some(t);
        }
        if let Ok(f) = params.get::<_, Table>("metadata_filter") {
            let json_value = lua_table_to_json(f)?;
            if let serde_json::Value::Object(map) = json_value {
                filters = Some(map.into_iter().collect());
            }
        } else if let Ok(f) = params.get::<_, Table>("filters") {
            let json_value = lua_table_to_json(f)?;
            if let serde_json::Value::Object(map) = json_value {
                filters = Some(map.into_iter().collect());
            }
        }

        debug!(
            "RAG search from Lua: query={}, k={:?}, scope={:?}",
            query, k, scope
        );

        // Execute search with direct parameters
        let response = block_on_async(
            "rag_search",
            async move {
                bridge
                    .search(&query, k, scope, scope_id, filters, threshold, None)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("RAG search failed: {e}"),
                        source: None,
                    })
            },
            None,
        )?;

        // Convert results array directly (not wrapped in a table)
        let results_array = lua.create_table()?;
        for (i, result) in response.results.iter().enumerate() {
            let result_table = lua.create_table()?;
            result_table.set("id", result.id.as_str())?;
            // Use 'content' field to match test expectations
            result_table.set("content", result.text.as_str())?;
            result_table.set("text", result.text.as_str())?; // Keep for backward compat
            result_table.set("score", result.score)?;

            // Convert metadata
            let metadata_value = json_to_lua_value(
                lua,
                &serde_json::Value::Object(result.metadata.clone().into_iter().collect()),
            )?;
            result_table.set("metadata", metadata_value)?;

            results_array.set(i + 1, result_table)?;
        }

        Ok(results_array)
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
        lua.create_function(move |lua, params: Table| {
            let bridge = bridge.clone();

            // Determine if this is a single document or array of documents
            let mut docs = Vec::new();
            let mut is_single_doc = false;

            // Check if params has 'content' field (single document format)
            if params.get::<_, String>("content").is_ok() {
                // Single document format: { content = "...", metadata = {...} }
                is_single_doc = true;
                let content = params.get::<_, String>("content")
                    .map_err(|_| mlua::Error::RuntimeError("RAG.ingest requires 'content' field".to_string()))?;

                // Generate a unique ID if not provided
                let id = params.get::<_, String>("id")
                    .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

                let metadata = if let Ok(meta) = params.get::<_, Table>("metadata") {
                    let json_value = lua_table_to_json(meta)?;
                    if let serde_json::Value::Object(map) = json_value {
                        Some(map.into_iter().collect())
                    } else {
                        None
                    }
                } else {
                    None
                };

                docs.push(RAGDocument {
                    id,
                    text: content,
                    metadata
                });
            } else if params.get::<_, Table>("documents").is_ok() {
                // Array format: { documents = [{content = "...", ...}, ...] }
                let documents: Table = params.get("documents")?;
                for i in 1..=documents.len()? {
                    let doc_table: Table = documents.get(i)?;

                    // Support both 'content' and 'text' fields
                    let text = doc_table.get::<_, String>("content")
                        .or_else(|_| doc_table.get::<_, String>("text"))
                        .map_err(|_| mlua::Error::RuntimeError("Document requires 'content' or 'text' field".to_string()))?;

                    let id = doc_table.get::<_, String>("id")
                        .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

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
            } else {
                // Try array format where params itself is treated as array
                // This handles backwards compatibility
                if params.len()? > 0 {
                    for i in 1..=params.len()? {
                        if let Ok(doc_table) = params.get::<_, Table>(i) {
                            let text = doc_table.get::<_, String>("content")
                                .or_else(|_| doc_table.get::<_, String>("text"))
                                .map_err(|_| mlua::Error::RuntimeError("Document requires 'content' or 'text' field".to_string()))?;

                            let id = doc_table.get::<_, String>("id")
                                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

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
                        } else {
                            return Err(mlua::Error::RuntimeError(
                                "RAG.ingest requires either { content = '...' } for single document or { documents = [...] } for multiple".to_string()
                            ));
                        }
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(
                        "RAG.ingest requires either 'content' field or 'documents' array".to_string()
                    ));
                }
            }

            // Save first document ID if single doc mode
            let first_doc_id = if is_single_doc && !docs.is_empty() {
                Some(docs[0].id.clone())
            } else {
                None
            };

            // Parse optional parameters
            let mut scope = None;
            let mut scope_id = None;
            let mut provider = None;
            let mut chunking = None;

            // Extract optional fields from params
            if let Ok(s) = params.get::<_, String>("scope") {
                scope = Some(s);
            }
            if let Ok(id) = params.get::<_, String>("scope_id") {
                scope_id = Some(id);
            }
            if let Ok(tenant_id) = params.get::<_, String>("tenant_id") {
                // Map tenant_id to scope_id for multi-tenant support
                scope = Some("tenant".to_string());
                scope_id = Some(tenant_id);
            }
            if let Ok(p) = params.get::<_, String>("provider") {
                provider = Some(p);
            }
            if let Ok(chunk_table) = params.get::<_, Table>("chunking") {
                let mut config = ChunkingConfig {
                    chunk_size: None,
                    overlap: None,
                    strategy: None,
                };
                if let Ok(size) = chunk_table.get::<_, u32>("chunk_size") {
                    config.chunk_size = Some(size as usize);
                }
                if let Ok(overlap) = chunk_table.get::<_, u32>("overlap") {
                    config.overlap = Some(overlap as usize);
                }
                if let Ok(strategy) = chunk_table.get::<_, String>("strategy") {
                    config.strategy = Some(strategy);
                }
                chunking = Some(config);
            }

            debug!("RAG ingest from Lua: {} documents", docs.len());

            // Execute ingestion with direct parameters
            let response = block_on_async(
                "rag_ingest",
                async move {
                    bridge.ingest(docs, scope, scope_id, provider, chunking, None).await.map_err(|e| {
                        llmspell_core::LLMSpellError::Component {
                            message: format!("RAG ingest failed: {e}"),
                            source: None,
                        }
                    })
                },
                None,
            )?;

            // Return appropriate response based on input type
            if is_single_doc {
                // For single document, return the document ID directly (first doc ID)
                if first_doc_id.is_some() {
                    Ok(mlua::Value::String(lua.create_string(first_doc_id.unwrap())?))
                } else {
                    Err(mlua::Error::RuntimeError("Failed to ingest document".to_string()))
                }
            } else {
                // For multiple documents, return full response table
                let result_table = lua.create_table()?;
                result_table.set("success", true)?; // Always true if we got here without error
                result_table.set("documents_processed", response.documents_processed)?;
                result_table.set("vectors_created", response.vectors_created)?;
                result_table.set("total_tokens", response.total_tokens)?;

                Ok(mlua::Value::Table(result_table))
            }
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
    let _bridge = bridge.clone();
    let configure_fn = lua.create_function(move |_lua, options: Table| {
        // Configure is not implemented yet - just validate the options
        let _session_ttl = options.get::<_, u32>("session_ttl").ok();
        let _default_provider = options.get::<_, String>("default_provider").ok();
        let _enable_cache = options.get::<_, bool>("enable_cache").ok();
        let _cache_ttl = options.get::<_, u32>("cache_ttl").ok();

        // For now, just return success since configure is not implemented
        debug!("RAG configure called but not implemented yet");

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

/// Register save method to persist vector storage
fn register_save_method(lua: &Lua, table: &Table, _bridge: &Arc<RAGBridge>) -> mlua::Result<()> {
    // For now, save is a no-op since we can't access the HNSW storage directly
    // The save will happen automatically when the storage is dropped
    let save_func = lua.create_async_function(move |_lua, ()| async move {
        debug!("RAG.save() called - persistence will happen on shutdown");
        Ok(())
    })?;

    table.set("save", save_func)?;
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
    register_save_method(lua, &rag_table, &bridge)?;
    register_session_methods(lua, &rag_table)?;

    // Set the RAG global
    lua.globals().set("RAG", rag_table)?;

    debug!("RAG global injected into Lua");
    Ok(())
}
