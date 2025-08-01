//! ABOUTME: Event global object providing cross-language event bus functionality
//! ABOUTME: Full implementation with EventBridge integration for Phase 4

use crate::event_bridge::EventBridge;
use crate::event_serialization::EventSerialization;
use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::error::LLMSpellError;
use llmspell_events::{Language, UniversalEvent};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;

/// Event global object providing cross-language event bus functionality
pub struct EventGlobal;

impl EventGlobal {
    /// Create a new Event global
    pub fn new() -> Self {
        Self
    }
}

/// Helper function to get or create an EventBridge from GlobalContext
async fn get_or_create_event_bridge(
    context: &GlobalContext,
) -> Result<Arc<EventBridge>, LLMSpellError> {
    // Try to get existing bridge from context first
    if let Some(bridge) = context.get_bridge::<EventBridge>("event_bridge") {
        return Ok(bridge);
    }

    // Create new bridge and store it in context
    let new_bridge = Arc::new(
        EventBridge::new(Arc::new(context.clone()))
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to initialize EventBridge: {}", e),
                source: None,
            })?,
    );

    // Store for future use
    context.set_bridge("event_bridge", new_bridge.clone());
    Ok(new_bridge)
}

impl GlobalObject for EventGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Event".to_string(),
            version: "4.0.0".to_string(),
            description: "Cross-language event bus with UniversalEvent and EventBridge".to_string(),
            dependencies: vec![],
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        use crate::lua::sync_utils::block_on_async;
        use std::time::Duration;

        let event_table = lua.create_table().map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Event table: {}", e),
            source: None,
        })?;

        // Store references for closures
        let global_context = Arc::new(context.clone());

        // Event.publish(event_type, data, options?)
        let context_publish = global_context.clone();
        let publish_fn =
            lua
                .create_function(
                    move |_,
                          (event_type, data, options): (
                        String,
                        mlua::Value,
                        Option<mlua::Table>,
                    )| {
                        let context = context_publish.clone();

                        block_on_async::<_, _, LLMSpellError>(
                            "event_publish",
                            async move {
                                let bridge = get_or_create_event_bridge(&context).await?;

                                // Convert Lua data to JSON
                                let data_json = crate::lua::conversion::lua_value_to_json(data)
                                    .map_err(|e| LLMSpellError::Component {
                                        message: format!(
                                            "Failed to convert Lua data to JSON: {}",
                                            e
                                        ),
                                        source: None,
                                    })?;

                                // Extract language and optional fields
                                let language = options
                                    .as_ref()
                                    .map(|opts| {
                                        opts.get::<&str, Option<String>>("language")
                                            .unwrap_or(None)
                                            .map(|s| match s.to_lowercase().as_str() {
                                                "lua" => Language::Lua,
                                                "javascript" | "js" => Language::JavaScript,
                                                "python" | "py" => Language::Python,
                                                "unknown" => Language::Unknown,
                                                "rust" => Language::Rust,
                                                _ => Language::Lua,
                                            })
                                            .unwrap_or(Language::Lua)
                                    })
                                    .unwrap_or(Language::Lua);

                                let mut event =
                                    UniversalEvent::new(&event_type, data_json, language);

                                // Set optional fields if provided
                                if let Some(opts) = options {
                                    if let Ok(Some(correlation_id_str)) =
                                        opts.get::<&str, Option<String>>("correlation_id")
                                    {
                                        if let Ok(correlation_id) =
                                            uuid::Uuid::parse_str(&correlation_id_str)
                                        {
                                            event.metadata.correlation_id = correlation_id;
                                        }
                                    }

                                    if let Ok(Some(ttl_secs)) =
                                        opts.get::<&str, Option<u64>>("ttl_seconds")
                                    {
                                        event.metadata.ttl = Some(ttl_secs);
                                    }
                                }

                                bridge.publish_event(event).await.map_err(|e| {
                                    LLMSpellError::Component {
                                        message: format!("Failed to publish event: {}", e),
                                        source: None,
                                    }
                                })?;

                                Ok(true)
                            },
                            Some(Duration::from_secs(30)),
                        )
                    },
                )
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to create Event.publish: {}", e),
                    source: None,
                })?;

        event_table
            .set("publish", publish_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event.publish: {}", e),
                source: None,
            })?;

        // Event.subscribe(pattern, options?)
        let context_subscribe = global_context.clone();
        let subscribe_fn = lua
            .create_function(
                move |_, (pattern, options): (String, Option<mlua::Table>)| {
                    let context = context_subscribe.clone();

                    block_on_async::<_, _, LLMSpellError>(
                        "event_subscribe",
                        async move {
                            let bridge = get_or_create_event_bridge(&context).await?;

                            let language = options
                                .as_ref()
                                .map(|opts| {
                                    opts.get::<&str, Option<String>>("language")
                                        .unwrap_or(None)
                                        .map(|s| match s.to_lowercase().as_str() {
                                            "lua" => Language::Lua,
                                            "javascript" | "js" => Language::JavaScript,
                                            "python" | "py" => Language::Python,
                                            "unknown" => Language::Unknown,
                                            "rust" => Language::Rust,
                                            _ => Language::Lua,
                                        })
                                        .unwrap_or(Language::Lua)
                                })
                                .unwrap_or(Language::Lua);

                            let (subscription_id, receiver) = bridge
                                .subscribe_pattern(&pattern, language)
                                .await
                                .map_err(|e| LLMSpellError::Component {
                                    message: format!("Failed to subscribe to pattern: {}", e),
                                    source: None,
                                })?;

                            // Store the receiver for receive() calls in GlobalContext
                            context.set_bridge(
                                &format!("event_receiver_{}", subscription_id),
                                Arc::new(tokio::sync::RwLock::new(receiver)),
                            );

                            Ok(subscription_id)
                        },
                        Some(Duration::from_secs(10)),
                    )
                },
            )
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create Event.subscribe: {}", e),
                source: None,
            })?;

        event_table
            .set("subscribe", subscribe_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event.subscribe: {}", e),
                source: None,
            })?;

        // Event.receive(subscription_id, timeout_ms?)
        let context_receive = global_context.clone();
        let receive_fn = lua
            .create_function(move |lua, (subscription_id, timeout_ms): (String, Option<u64>)| {
                let context = context_receive.clone();
                let timeout = Duration::from_millis(timeout_ms.unwrap_or(5000));
                block_on_async::<_, _, LLMSpellError>(
                    "event_receive",
                    async move {
                        // Get the receiver from GlobalContext
                        let receiver_key = format!("event_receiver_{}", subscription_id);
                        let receiver_arc = context.get_bridge::<tokio::sync::RwLock<UnboundedReceiver<UniversalEvent>>>(&receiver_key)
                            .ok_or_else(|| LLMSpellError::Component {
                                message: format!("No active subscription found: {}", subscription_id),
                                source: None,
                            })?;

                        // Wait for an event with timeout (using async-aware RwLock)
                        let result = {
                            let mut receiver_guard = receiver_arc.write().await;
                            tokio::time::timeout(timeout, receiver_guard.recv()).await
                        };

                        match result {
                            Ok(Some(event)) => {
                                // Serialize the event for Lua
                                let serialized = EventSerialization::serialize_for_language(&event, Language::Lua)
                                    .map_err(|e| LLMSpellError::Component {
                                        message: format!("Failed to serialize event: {}", e),
                                        source: None,
                                    })?;

                                // Convert JSON to Lua value
                                crate::lua::conversion::json_to_lua_value(lua, &serialized)
                                    .map_err(|e| LLMSpellError::Component {
                                        message: format!("Failed to convert event to Lua: {}", e),
                                        source: None,
                                    })
                            }
                            Ok(None) => {
                                // Channel closed
                                Ok(mlua::Value::Nil)
                            }
                            Err(_) => {
                                // Timeout - return nil
                                Ok(mlua::Value::Nil)
                            }
                        }
                    },
                    Some(timeout + Duration::from_secs(1)) // Add buffer to async timeout
                )
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create Event.receive: {}", e),
                source: None,
            })?;

        event_table
            .set("receive", receive_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event.receive: {}", e),
                source: None,
            })?;

        // Event.unsubscribe(subscription_id)
        let context_unsubscribe = global_context.clone();
        let unsubscribe_fn = lua
            .create_function(move |_, subscription_id: String| {
                let context = context_unsubscribe.clone();

                block_on_async::<_, _, LLMSpellError>(
                    "event_unsubscribe",
                    async move {
                        let bridge = get_or_create_event_bridge(&context).await?;

                        // Remove the receiver from GlobalContext (we can't easily remove from GlobalContext,
                        // so we'll just check if it exists)
                        let receiver_key = format!("event_receiver_{}", subscription_id);
                        let had_receiver = context
                            .get_bridge::<tokio::sync::RwLock<UnboundedReceiver<UniversalEvent>>>(
                                &receiver_key,
                            )
                            .is_some();

                        // Unsubscribe from the bridge
                        let unsubscribed =
                            bridge.unsubscribe(&subscription_id).await.map_err(|e| {
                                LLMSpellError::Component {
                                    message: format!("Failed to unsubscribe: {}", e),
                                    source: None,
                                }
                            })?;

                        Ok(unsubscribed && had_receiver)
                    },
                    Some(Duration::from_secs(5)),
                )
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create Event.unsubscribe: {}", e),
                source: None,
            })?;

        event_table
            .set("unsubscribe", unsubscribe_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event.unsubscribe: {}", e),
                source: None,
            })?;

        // Event.list_subscriptions()
        let context_list = global_context.clone();
        let list_fn = lua
            .create_function(move |lua, ()| {
                let context = context_list.clone();

                block_on_async::<_, _, LLMSpellError>(
                    "event_list_subscriptions",
                    async move {
                        let bridge = get_or_create_event_bridge(&context).await?;
                        let subscriptions = bridge.list_subscriptions();

                        // Convert to Lua-friendly format
                        let lua_subscriptions: Vec<serde_json::Value> = subscriptions
                            .into_iter()
                            .map(|handle| {
                                serde_json::json!({
                                    "id": handle.id,
                                    "pattern": handle.pattern,
                                    "language": format!("{:?}", handle.language)
                                })
                            })
                            .collect();

                        // Convert JSON array to Lua value
                        crate::lua::conversion::json_to_lua_value(
                            lua,
                            &serde_json::Value::Array(lua_subscriptions),
                        )
                        .map_err(|e| LLMSpellError::Component {
                            message: format!("Failed to convert subscriptions to Lua: {}", e),
                            source: None,
                        })
                    },
                    Some(Duration::from_secs(5)),
                )
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create Event.list_subscriptions: {}", e),
                source: None,
            })?;

        event_table
            .set("list_subscriptions", list_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event.list_subscriptions: {}", e),
                source: None,
            })?;

        // Event.get_stats()
        let context_stats = global_context.clone();
        let stats_fn = lua
            .create_function(move |lua, ()| {
                let context = context_stats.clone();

                block_on_async::<_, _, LLMSpellError>(
                    "event_get_stats",
                    async move {
                        let bridge = get_or_create_event_bridge(&context).await?;
                        let stats = bridge.get_stats().await;

                        // Convert JSON stats to Lua value
                        crate::lua::conversion::json_to_lua_value(lua, &stats).map_err(|e| {
                            LLMSpellError::Component {
                                message: format!("Failed to convert stats to Lua: {}", e),
                                source: None,
                            }
                        })
                    },
                    Some(Duration::from_secs(5)),
                )
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create Event.get_stats: {}", e),
                source: None,
            })?;

        event_table
            .set("get_stats", stats_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event.get_stats: {}", e),
                source: None,
            })?;

        lua.globals()
            .set("Event", event_table)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set Event global: {}", e),
                source: None,
            })?;

        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO (Phase 15): JavaScript Event implementation - stub for now
        Ok(())
    }
}

impl Default for EventGlobal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_global_metadata() {
        let global = EventGlobal::new();
        let metadata = global.metadata();
        assert_eq!(metadata.name, "Event");
        assert_eq!(metadata.version, "4.0.0"); // Placeholder version
    }
}
