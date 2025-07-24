//! ABOUTME: Lua bindings for the cross-language hook system
//! ABOUTME: Provides Hook.register, Hook.unregister, and Hook.list functions for Lua scripts

use crate::globals::types::GlobalContext;
use crate::hook_bridge::{HookBridge, HookHandle};
use crate::lua::conversion::{json_to_lua_value, lua_table_to_json};
use crate::lua::sync_utils::block_on_async;
use llmspell_core::error::LLMSpellError;
use llmspell_core::Result;
use llmspell_hooks::{HookContext, HookPoint, HookResult, Language, Priority};
use mlua::{Function, Lua, Table, UserData, UserDataMethods, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Lua representation of a hook handle for cleanup
struct LuaHookHandle {
    handle: Arc<RwLock<Option<HookHandle>>>,
    hook_bridge: Arc<HookBridge>,
}

impl UserData for LuaHookHandle {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Get hook ID
        methods.add_method("id", |_, this, ()| {
            let handle = block_on_async::<_, _, std::io::Error>(
                "hook_handle_id",
                async { Ok(this.handle.read().await.clone()) },
                None,
            )?;

            match handle {
                Some(h) => Ok(h.id),
                None => Err(mlua::Error::RuntimeError(
                    "Hook already unregistered".to_string(),
                )),
            }
        });

        // Get hook point
        methods.add_method("hook_point", |_, this, ()| {
            let handle = block_on_async::<_, _, std::io::Error>(
                "hook_handle_hook_point",
                async { Ok(this.handle.read().await.clone()) },
                None,
            )?;

            match handle {
                Some(h) => Ok(format!("{:?}", h.hook_point)),
                None => Err(mlua::Error::RuntimeError(
                    "Hook already unregistered".to_string(),
                )),
            }
        });

        // Unregister the hook
        methods.add_method("unregister", |_, this, ()| {
            block_on_async(
                "hook_unregister",
                async {
                    let mut handle_lock = this.handle.write().await;
                    if let Some(handle) = handle_lock.take() {
                        this.hook_bridge.unregister_hook(&handle).await?;
                        Ok::<bool, LLMSpellError>(true)
                    } else {
                        Ok::<bool, LLMSpellError>(false)
                    }
                },
                None,
            )
        });
    }
}

impl Drop for LuaHookHandle {
    fn drop(&mut self) {
        // Auto-unregister when the handle is dropped
        let handle = self.handle.clone();
        let bridge = self.hook_bridge.clone();
        tokio::spawn(async move {
            let mut handle_lock = handle.write().await;
            if let Some(h) = handle_lock.take() {
                let _ = bridge.unregister_hook(&h).await;
            }
        });
    }
}

/// Convert HookContext to Lua table
#[allow(dead_code)]
fn hook_context_to_lua_table<'lua>(
    lua: &'lua Lua,
    context: &HookContext,
) -> mlua::Result<Table<'lua>> {
    let table = lua.create_table()?;

    // Set basic fields
    table.set("point", format!("{:?}", context.point))?;
    table.set("correlation_id", context.correlation_id.to_string())?;

    // Component ID
    let component_table = lua.create_table()?;
    component_table.set("type", format!("{:?}", context.component_id.component_type))?;
    component_table.set("name", context.component_id.name.clone())?;
    table.set("component_id", component_table)?;

    // Language
    table.set("language", context.language.to_string())?;

    // Metadata
    let metadata_table = lua.create_table()?;
    for (key, value) in &context.metadata {
        metadata_table.set(key.as_str(), value.as_str())?;
    }
    table.set("metadata", metadata_table)?;

    // Data (convert JSON to Lua)
    let data_table = lua.create_table()?;
    for (key, value) in &context.data {
        if let Ok(lua_value) = json_to_lua_value(lua, value) {
            data_table.set(key.as_str(), lua_value)?;
        }
    }
    table.set("data", data_table)?;

    Ok(table)
}

/// Convert Lua value to HookResult
#[allow(dead_code)]
fn lua_value_to_hook_result(value: Value) -> mlua::Result<HookResult> {
    match value {
        Value::Nil => Ok(HookResult::Continue),
        Value::String(s) => {
            let s = s.to_str()?;
            match s {
                "continue" => Ok(HookResult::Continue),
                "skip" | "skipped" => Ok(HookResult::Skipped("Skipped by Lua hook".to_string())),
                _ => Ok(HookResult::Cancel(s.to_string())),
            }
        }
        Value::Table(table) => {
            // Check for result type
            if let Ok(Value::String(result_type)) = table.get::<_, Value>("type") {
                let result_type = result_type.to_str()?;
                match result_type {
                    "continue" => Ok(HookResult::Continue),
                    "modified" => {
                        // Get the modified data
                        if let Ok(data) = table.get::<_, Table>("data") {
                            let json_data = lua_table_to_json(data)?;
                            Ok(HookResult::Modified(json_data))
                        } else {
                            Ok(HookResult::Modified(serde_json::json!({})))
                        }
                    }
                    "cancel" => {
                        let reason = table
                            .get::<_, String>("reason")
                            .unwrap_or_else(|_| "Cancelled by Lua hook".to_string());
                        Ok(HookResult::Cancel(reason))
                    }
                    "redirect" => {
                        let target = table.get::<_, String>("target").map_err(|_| {
                            mlua::Error::FromLuaConversionError {
                                from: "table",
                                to: "HookResult::Redirect",
                                message: Some("Missing 'target' field for redirect".to_string()),
                            }
                        })?;
                        Ok(HookResult::Redirect(target))
                    }
                    "replace" => {
                        if let Ok(data) = table.get::<_, Table>("data") {
                            let json_data = lua_table_to_json(data)?;
                            Ok(HookResult::Replace(json_data))
                        } else {
                            Ok(HookResult::Replace(serde_json::json!({})))
                        }
                    }
                    "retry" => {
                        let delay_ms = table.get::<_, u64>("delay_ms").unwrap_or(1000);
                        let max_attempts = table.get::<_, u32>("max_attempts").unwrap_or(3);
                        Ok(HookResult::Retry {
                            delay: std::time::Duration::from_millis(delay_ms),
                            max_attempts,
                        })
                    }
                    "skipped" => {
                        let reason = table
                            .get::<_, String>("reason")
                            .unwrap_or_else(|_| "Skipped by Lua hook".to_string());
                        Ok(HookResult::Skipped(reason))
                    }
                    _ => Ok(HookResult::Continue),
                }
            } else {
                Ok(HookResult::Continue)
            }
        }
        _ => Ok(HookResult::Continue),
    }
}

/// Parse hook point from string
fn parse_hook_point(s: &str) -> mlua::Result<HookPoint> {
    match s {
        "SystemStartup" => Ok(HookPoint::SystemStartup),
        "SystemShutdown" => Ok(HookPoint::SystemShutdown),
        "BeforeAgentInit" => Ok(HookPoint::BeforeAgentInit),
        "AfterAgentInit" => Ok(HookPoint::AfterAgentInit),
        "BeforeAgentExecution" => Ok(HookPoint::BeforeAgentExecution),
        "AfterAgentExecution" => Ok(HookPoint::AfterAgentExecution),
        "BeforeAgentShutdown" => Ok(HookPoint::BeforeAgentShutdown),
        "AfterAgentShutdown" => Ok(HookPoint::AfterAgentShutdown),
        "BeforeToolDiscovery" => Ok(HookPoint::BeforeToolDiscovery),
        "AfterToolDiscovery" => Ok(HookPoint::AfterToolDiscovery),
        "BeforeToolExecution" => Ok(HookPoint::BeforeToolExecution),
        "AfterToolExecution" => Ok(HookPoint::AfterToolExecution),
        "ToolValidation" => Ok(HookPoint::ToolValidation),
        "ToolError" => Ok(HookPoint::ToolError),
        "AgentError" => Ok(HookPoint::AgentError),
        "BeforeWorkflowStart" => Ok(HookPoint::BeforeWorkflowStart),
        "WorkflowStageTransition" => Ok(HookPoint::WorkflowStageTransition),
        "BeforeWorkflowStage" => Ok(HookPoint::BeforeWorkflowStage),
        "AfterWorkflowStage" => Ok(HookPoint::AfterWorkflowStage),
        "WorkflowCheckpoint" => Ok(HookPoint::WorkflowCheckpoint),
        "WorkflowRollback" => Ok(HookPoint::WorkflowRollback),
        "AfterWorkflowComplete" => Ok(HookPoint::AfterWorkflowComplete),
        "WorkflowError" => Ok(HookPoint::WorkflowError),
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown hook point: {}",
            s
        ))),
    }
}

/// Parse priority from string or use default
fn parse_priority(s: Option<String>) -> Priority {
    match s.as_deref() {
        Some("highest") => Priority::HIGHEST,
        Some("high") => Priority::HIGH,
        Some("normal") => Priority::NORMAL,
        Some("low") => Priority::LOW,
        Some("lowest") => Priority::LOWEST,
        _ => Priority::NORMAL,
    }
}

/// Inject the Hook global into Lua
pub fn inject_hook_global(
    lua: &Lua,
    _context: &GlobalContext,
    hook_bridge: Arc<HookBridge>,
) -> Result<()> {
    // Create the Hook table
    let hook_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Hook table: {}", e),
        source: None,
    })?;

    // Register a Lua hook adapter with the bridge
    {
        let adapter = Arc::new(crate::lua::hook_adapter::LuaHookAdapter::new());

        block_on_async(
            "register_lua_adapter",
            async {
                hook_bridge
                    .register_adapter(
                        Language::Lua,
                        adapter
                            as Arc<
                                dyn llmspell_hooks::HookAdapter<
                                    Context = Box<dyn std::any::Any>,
                                    Result = Box<dyn std::any::Any>,
                                >,
                            >,
                    )
                    .await
            },
            None,
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to register Lua adapter: {}", e),
            source: None,
        })?;
    }

    // Hook.register(hook_point, callback, priority) -> handle
    let hook_bridge_clone = hook_bridge.clone();
    let register_fn = lua
        .create_function(
            move |lua, (hook_point, callback, priority): (String, Function, Option<String>)| {
                let hook_point = parse_hook_point(&hook_point)?;
                let priority = parse_priority(priority);
                let hook_bridge = hook_bridge_clone.clone();

                // Store the callback
                let callback_ref = lua.create_registry_value(callback)?;

                block_on_async::<_, _, LLMSpellError>(
                    "hook_register",
                    async move {
                        let handle = hook_bridge
                            .register_hook(
                                Language::Lua,
                                hook_point,
                                priority,
                                Box::new(callback_ref),
                            )
                            .await?;

                        Ok::<LuaHookHandle, LLMSpellError>(LuaHookHandle {
                            handle: Arc::new(RwLock::new(Some(handle))),
                            hook_bridge,
                        })
                    },
                    None,
                )
            },
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Hook.register: {}", e),
            source: None,
        })?;

    hook_table
        .set("register", register_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Hook.register: {}", e),
            source: None,
        })?;

    // Hook.list(filter) -> array of hook info
    // filter can be:
    //   - nil: list all hooks
    //   - string: hook point name
    //   - table: {hook_point?, language?, priority?, tag?}
    let hook_bridge_clone = hook_bridge.clone();
    let list_fn = lua
        .create_function(move |lua, filter: Option<mlua::Value>| {
            let hook_bridge = hook_bridge_clone.clone();

            // Parse filter
            let hook_point_filter = match &filter {
                Some(mlua::Value::String(s)) => Some(parse_hook_point(s.to_str()?)?),
                Some(mlua::Value::Table(table)) => {
                    if let Ok(hook_point_str) = table.get::<_, String>("hook_point") {
                        Some(parse_hook_point(&hook_point_str)?)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let hooks = block_on_async::<_, _, LLMSpellError>(
                "hook_list",
                async move { hook_bridge.list_hooks(hook_point_filter).await },
                None,
            )?;

            // Apply additional filters if provided
            let filtered_hooks: Vec<_> = if let Some(mlua::Value::Table(filter_table)) = &filter {
                hooks
                    .into_iter()
                    .filter(|hook| {
                        // Language filter
                        if let Ok(language_filter) = filter_table.get::<_, String>("language") {
                            if hook.language.to_string().to_lowercase()
                                != language_filter.to_lowercase()
                            {
                                return false;
                            }
                        }

                        // Priority filter
                        if let Ok(priority_filter) = filter_table.get::<_, String>("priority") {
                            let matches_priority = match priority_filter.to_lowercase().as_str() {
                                "highest" => hook.priority.0 == i32::MIN,
                                "high" => hook.priority.0 == -100,
                                "normal" => hook.priority.0 == 0,
                                "low" => hook.priority.0 == 100,
                                "lowest" => hook.priority.0 == i32::MAX,
                                _ => false,
                            };
                            if !matches_priority {
                                return false;
                            }
                        }

                        // Tag filter
                        if let Ok(tag_filter) = filter_table.get::<_, String>("tag") {
                            if !hook.tags.iter().any(|tag| tag.contains(&tag_filter)) {
                                return false;
                            }
                        }

                        true
                    })
                    .collect()
            } else {
                hooks
            };

            let result = lua.create_table()?;
            for (i, hook) in filtered_hooks.iter().enumerate() {
                let hook_info = lua.create_table()?;
                hook_info.set("name", hook.name.clone())?;
                hook_info.set("priority", format!("{:?}", hook.priority))?;
                hook_info.set("language", hook.language.to_string())?;
                hook_info.set("version", hook.version.clone())?;
                if let Some(desc) = &hook.description {
                    hook_info.set("description", desc.clone())?;
                }

                let tags = lua.create_table()?;
                for (j, tag) in hook.tags.iter().enumerate() {
                    tags.set(j + 1, tag.clone())?;
                }
                hook_info.set("tags", tags)?;

                result.set(i + 1, hook_info)?;
            }
            Ok(result)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Hook.list: {}", e),
            source: None,
        })?;

    hook_table
        .set("list", list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Hook.list: {}", e),
            source: None,
        })?;

    // Hook.unregister(handle) -> bool
    let unregister_fn = lua
        .create_function(move |_, handle: mlua::AnyUserData| {
            // Try to cast to LuaHookHandle and call unregister
            if let Ok(lua_handle) = handle.borrow::<LuaHookHandle>() {
                block_on_async::<_, _, LLMSpellError>(
                    "hook_unregister_standalone",
                    async move {
                        let mut handle_lock = lua_handle.handle.write().await;
                        if let Some(h) = handle_lock.take() {
                            lua_handle.hook_bridge.unregister_hook(&h).await?;
                            Ok::<bool, LLMSpellError>(true)
                        } else {
                            Ok::<bool, LLMSpellError>(false)
                        }
                    },
                    None,
                )
            } else {
                Ok(false) // Invalid handle, return false instead of error
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Hook.unregister: {}", e),
            source: None,
        })?;

    hook_table
        .set("unregister", unregister_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Hook.unregister: {}", e),
            source: None,
        })?;

    // Set the Hook global
    lua.globals()
        .set("Hook", hook_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Hook global: {}", e),
            source: None,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_hook_injection() {
        let lua = Lua::new();
        let context = Arc::new(GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(ProviderManager::new(Default::default()).await.unwrap()),
        ));
        let hook_bridge = Arc::new(HookBridge::new(context.clone()).await.unwrap());

        inject_hook_global(&lua, &context, hook_bridge).unwrap();

        // Check that Hook global exists
        let hook: Table = lua.globals().get("Hook").unwrap();
        assert!(hook.contains_key("register").unwrap());
        assert!(hook.contains_key("list").unwrap());
    }
}
