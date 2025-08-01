//! ABOUTME: Lua-specific Streaming global implementation  
//! ABOUTME: Provides Lua bindings for streaming utilities and coroutine functionality

use crate::globals::GlobalContext;
use llmspell_core::error::LLMSpellError;
use mlua::{
    AnyUserDataExt, Function, Lua, Result as LuaResult, Table, TableExt, UserData, UserDataMethods,
    Value,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Inject Streaming global into Lua environment
pub fn inject_streaming_global(lua: &Lua, _context: &GlobalContext) -> Result<(), LLMSpellError> {
    // Create the streaming utilities table
    let streaming_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create streaming table: {}", e),
        source: None,
    })?;

    // Create a coroutine wrapper for streaming
    let create_stream_fn = lua
        .create_function(|lua, f: Function| -> LuaResult<Table> {
            // Create a coroutine from the function
            let thread = lua.create_thread(f)?;

            // Create stream object with coroutine
            let stream = lua.create_table()?;
            stream.set("_coroutine", thread)?;
            stream.set("_done", false)?;

            // Add next() method
            stream.set(
                "next",
                lua.create_function(|_lua, stream: Table| -> LuaResult<Value> {
                    let thread: mlua::Thread = stream.get("_coroutine")?;
                    let done: bool = stream.get("_done")?;

                    if done {
                        return Ok(Value::Nil);
                    }

                    match thread.resume::<_, Value>(()) {
                        Ok(value) => {
                            if thread.status() == mlua::ThreadStatus::Resumable {
                                Ok(value)
                            } else {
                                stream.set("_done", true)?;
                                Ok(value)
                            }
                        }
                        Err(e) => {
                            stream.set("_done", true)?;
                            Err(e)
                        }
                    }
                })?,
            )?;

            // Add isDone() method
            stream.set(
                "isDone",
                lua.create_function(|_lua, stream: Table| -> LuaResult<bool> {
                    stream.get("_done")
                })?,
            )?;

            // Add collect() method to get all values
            stream.set(
                "collect",
                lua.create_function(|lua, stream: Table| -> LuaResult<Table> {
                    let results = lua.create_table()?;
                    let mut idx = 1;

                    while !stream.call_method::<_, bool>("isDone", ())? {
                        if let Ok(value) = stream.call_method::<_, Value>("next", ()) {
                            if !value.is_nil() {
                                results.set(idx, value)?;
                                idx += 1;
                            }
                        }
                    }

                    Ok(results)
                })?,
            )?;

            Ok(stream)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create stream constructor: {}", e),
            source: None,
        })?;

    // Add the create function to streaming table
    streaming_table
        .set("create", create_stream_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set streaming.create: {}", e),
            source: None,
        })?;

    // Create yield helper for use inside coroutines
    let yield_fn = lua
        .create_function(|_lua, value: Value| -> LuaResult<()> {
            // In a real coroutine context, this would yield the value
            // For now, this is a placeholder
            mlua::Error::external(format!("Yield called with: {:?}", value));
            Ok(())
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create yield function: {}", e),
            source: None,
        })?;

    streaming_table
        .set("yield", yield_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set streaming.yield: {}", e),
            source: None,
        })?;

    // Set the streaming table as a global
    lua.globals()
        .set("Streaming", streaming_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Streaming global: {}", e),
            source: None,
        })?;

    Ok(())
}

/// Wrapper for tokio mpsc Receiver to work with mlua
#[derive(Clone)]
struct StreamReceiver {
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

impl UserData for StreamReceiver {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Add synchronous next method (using blocking approach)
        methods.add_method("next", |lua, this, ()| {
            let receiver_arc = this.receiver.clone();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut receiver = receiver_arc.lock().await;
                    receiver.recv().await
                })
            });

            match result {
                Some(chunk) => Ok(Value::String(lua.create_string(&chunk)?)),
                None => Ok(Value::Nil),
            }
        });

        // Add try_next for non-blocking receive
        methods.add_method("try_next", |lua, this, ()| {
            let mut receiver = this.receiver.blocking_lock();
            match receiver.try_recv() {
                Ok(chunk) => Ok(Value::String(lua.create_string(&chunk)?)),
                Err(_) => Ok(Value::Nil),
            }
        });

        // Add is_closed method
        methods.add_method("is_closed", |_lua, this, ()| {
            let receiver = this.receiver.blocking_lock();
            Ok(receiver.is_closed())
        });
    }
}

/// Create a Lua-compatible stream from a Rust async stream
pub fn create_lua_stream_bridge(lua: &Lua, receiver: mpsc::Receiver<String>) -> LuaResult<Table> {
    let stream = lua.create_table()?;

    // Create the receiver wrapper
    let receiver_wrapper = StreamReceiver {
        receiver: Arc::new(Mutex::new(receiver)),
    };

    // Store the receiver as userdata
    stream.set("_receiver", receiver_wrapper)?;
    stream.set("_done", false)?;

    // Add synchronous next method that delegates to receiver
    stream.set(
        "next",
        lua.create_function(|_lua, stream: Table| {
            let done: bool = stream.get("_done")?;
            if done {
                return Ok(Value::Nil);
            }

            let receiver_ud: mlua::AnyUserData = stream.get("_receiver")?;
            let result = receiver_ud.call_method::<_, Value>("next", ())?;

            if result.is_nil() {
                stream.set("_done", true)?;
            }

            Ok(result)
        })?,
    )?;

    // Add isDone method
    stream.set(
        "isDone",
        lua.create_function(|_lua, stream: Table| -> LuaResult<bool> { stream.get("_done") })?,
    )?;

    // Add collect method for gathering all values
    stream.set(
        "collect",
        lua.create_function(|lua, stream: Table| {
            let results = lua.create_table()?;
            let mut idx = 1;

            while !stream.call_method::<_, bool>("isDone", ())? {
                let value = stream.call_method::<_, Value>("next", ())?;
                if !value.is_nil() {
                    results.set(idx, value)?;
                    idx += 1;
                }
            }

            Ok(results)
        })?,
    )?;

    Ok(stream)
}

#[cfg(test)]
#[cfg_attr(test_category = "bridge")]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_streaming_global_injection() {
        use crate::providers::{ProviderManager, ProviderManagerConfig};
        use crate::registry::ComponentRegistry;
        use std::sync::Arc;

        let lua = mlua::Lua::new();
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
        let context = GlobalContext::new(registry, providers);

        // Inject global
        inject_streaming_global(&lua, &context).unwrap();

        // Test that Streaming global exists
        lua.load(
            r#"
            assert(Streaming ~= nil, "Streaming global should exist")
            assert(type(Streaming.create) == "function", "Streaming.create should be a function")
            assert(type(Streaming.yield) == "function", "Streaming.yield should be a function")
        "#,
        )
        .exec()
        .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stream_creation() {
        use crate::providers::{ProviderManager, ProviderManagerConfig};
        use crate::registry::ComponentRegistry;
        use std::sync::Arc;

        let lua = mlua::Lua::new();
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
        let context = GlobalContext::new(registry, providers);

        // Inject global
        inject_streaming_global(&lua, &context).unwrap();

        // Test stream creation and usage
        lua.load(
            r#"
            local function generator()
                for i = 1, 3 do
                    coroutine.yield(i)
                end
            end
            
            local stream = Streaming.create(generator)
            assert(type(stream) == "table", "Stream should be a table")
            assert(type(stream.next) == "function", "Stream should have next method")
            assert(type(stream.isDone) == "function", "Stream should have isDone method")
            assert(type(stream.collect) == "function", "Stream should have collect method")
            
            -- Test isDone starts false
            assert(stream:isDone() == false, "Stream should start not done")
        "#,
        )
        .exec()
        .unwrap();
    }
}
