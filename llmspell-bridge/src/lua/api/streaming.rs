//! ABOUTME: Lua streaming API implementation using coroutines
//! ABOUTME: Provides async generator-like functionality for streaming LLM responses

use crate::engine::types::StreamingApiDefinition;
use llmspell_core::error::LLMSpellError;
use mlua::{
    AnyUserDataExt, Function, Lua, Result as LuaResult, Table, TableExt, UserData, UserDataMethods,
    Value,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Inject streaming API into Lua environment
pub fn inject_streaming_api(
    lua: &Lua,
    _api_def: &StreamingApiDefinition,
) -> Result<(), LLMSpellError> {
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
        // Add async next method
        methods.add_async_method("next", |lua, this, ()| async move {
            let mut receiver = this.receiver.lock().await;
            match receiver.recv().await {
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

    // Add next method that delegates to receiver
    stream.set(
        "next",
        lua.create_async_function(|_lua, stream: Table| async move {
            let done: bool = stream.get("_done")?;
            if done {
                return Ok(Value::Nil);
            }

            let receiver_ud: mlua::AnyUserData = stream.get("_receiver")?;
            let result = receiver_ud
                .call_async_method::<_, Value>("next", ())
                .await?;

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
        lua.create_async_function(|lua, stream: Table| async move {
            let results = lua.create_table()?;
            let mut idx = 1;

            while !stream.call_method::<_, bool>("isDone", ())? {
                let value = stream.call_async_method::<_, Value>("next", ()).await?;
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
