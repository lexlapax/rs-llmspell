// ABOUTME: Shared utilities for synchronous wrappers around async operations
// ABOUTME: Provides consistent error handling and panic safety for Lua bridge

use mlua::prelude::*;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, trace};

/// Execute an async operation synchronously with proper error handling and panic safety
///
/// This function provides a consistent way to execute async Rust code from synchronous
/// Lua contexts. It handles:
/// - Proper tokio runtime integration using `block_in_place`
/// - Panic safety with `catch_unwind`
/// - Error transformation from any error type to `mlua::Error`
/// - Optional timeout support
/// - Tracing/logging for debugging
///
/// # Type Parameters
/// - `F`: The future type to execute
/// - `T`: The success value type (must be convertible to Lua)
/// - `E`: The error type (must implement `std::error::Error`)
///
/// # Arguments
/// - `operation_name`: Name of the operation for logging/debugging
/// - `future`: The async operation to execute
/// - `timeout`: Optional timeout duration
///
/// # Returns
/// - `Ok(T)` on success
/// - `Err(mlua::Error)` on failure (including panics, timeouts, and operation errors)
///
/// # Example
/// ```rust,no_run
/// # use llmspell_bridge::lua::sync_utils::block_on_async;
/// # use std::time::Duration;
/// # async fn create_agent_internal(config: &str) -> Result<String, std::io::Error> { Ok("agent".to_string()) }
/// # fn example() -> mlua::Result<()> {
/// let result = block_on_async(
///     "agent_create",
///     async { create_agent_internal("config").await },
///     Some(Duration::from_secs(30))
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn block_on_async<F, T, E>(
    operation_name: &str,
    future: F,
    timeout: Option<Duration>,
) -> LuaResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    trace!(
        "Starting synchronous execution of async operation: {}",
        operation_name
    );

    // Capture the operation name for use in error messages
    let op_name = operation_name.to_string();

    // Use catch_unwind to protect against panics in async code
    let result = catch_unwind(AssertUnwindSafe(|| {
        // block_in_place tells tokio this thread will block
        tokio::task::block_in_place(|| {
            // Get current runtime handle
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle,
                Err(e) => {
                    error!("No tokio runtime available for {}: {}", op_name, e);
                    return Err(mlua::Error::RuntimeError(format!(
                        "No async runtime available for {op_name}: {e}"
                    )));
                }
            };

            // Execute the future with optional timeout
            let result = if let Some(duration) = timeout {
                debug!("Executing {} with timeout of {:?}", op_name, duration);
                handle.block_on(async { tokio::time::timeout(duration, future).await })
            } else {
                debug!("Executing {} without timeout", op_name);
                handle.block_on(async {
                    // Wrap in Ok to match timeout signature
                    Ok(future.await)
                })
            };

            // Handle timeout and operation results
            match result {
                Ok(Ok(value)) => {
                    trace!("{} completed successfully", op_name);
                    Ok(value)
                }
                Ok(Err(e)) => {
                    error!("{} failed with error: {}", op_name, e);
                    Err(mlua::Error::ExternalError(Arc::new(e)))
                }
                Err(_timeout_err) => {
                    error!("{} timed out after {:?}", op_name, timeout);
                    Err(mlua::Error::RuntimeError(format!(
                        "{op_name} timed out after {timeout:?}"
                    )))
                }
            }
        })
    }));

    // Handle panic results
    match result {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(e),
        Err(panic_err) => {
            error!("{} panicked: {:?}", operation_name, panic_err);
            Err(mlua::Error::RuntimeError(format!(
                "Runtime panic in {operation_name}: operation failed unexpectedly"
            )))
        }
    }
}

/// Execute an async operation that returns a Lua value directly
///
/// This is a convenience wrapper for operations that already return `LuaResult<LuaValue>`
/// and don't need additional error transformation.
///
/// # Example
/// ```rust,no_run
/// # use llmspell_bridge::lua::sync_utils::block_on_async_lua;
/// # use mlua::{Lua, Value as LuaValue};
/// # async fn execute_internal() -> mlua::Result<LuaValue<'static>> {
/// #     let lua = Lua::new();
/// #     Ok(LuaValue::Boolean(true))
/// # }
/// # fn example() -> mlua::Result<()> {
/// let value = block_on_async_lua(
///     "tool_execute",
///     async { execute_internal().await },
///     None
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn block_on_async_lua<'lua, F>(
    operation_name: &str,
    future: F,
    timeout: Option<Duration>,
) -> LuaResult<LuaValue<'lua>>
where
    F: Future<Output = LuaResult<LuaValue<'lua>>>,
{
    trace!(
        "Starting synchronous execution of async Lua operation: {}",
        operation_name
    );

    let op_name = operation_name.to_string();

    let result = catch_unwind(AssertUnwindSafe(|| {
        tokio::task::block_in_place(|| {
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle,
                Err(e) => {
                    error!("No tokio runtime available for {}: {}", op_name, e);
                    return Err(mlua::Error::RuntimeError(format!(
                        "No async runtime available for {op_name}: {e}"
                    )));
                }
            };

            let result = if let Some(duration) = timeout {
                debug!("Executing {} with timeout of {:?}", op_name, duration);
                handle.block_on(async {
                    match tokio::time::timeout(duration, future).await {
                        Ok(result) => result,
                        Err(_) => Err(mlua::Error::RuntimeError(format!(
                            "{op_name} timed out after {duration:?}"
                        ))),
                    }
                })
            } else {
                debug!("Executing {} without timeout", op_name);
                handle.block_on(future)
            };

            match result {
                Ok(value) => {
                    trace!("{} completed successfully", op_name);
                    Ok(value)
                }
                Err(e) => {
                    error!("{} failed with error: {}", op_name, e);
                    Err(e)
                }
            }
        })
    }));

    match result {
        Ok(result) => result,
        Err(panic_err) => {
            error!("{} panicked: {:?}", operation_name, panic_err);
            Err(mlua::Error::RuntimeError(format!(
                "Runtime panic in {operation_name}: operation failed unexpectedly"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    #[test]
    fn test_block_on_async_success() {
        let rt = Runtime::new().unwrap();
        let _guard = rt.enter();

        let result: Result<i32, mlua::Error> = block_on_async(
            "test_success",
            async { Ok::<i32, std::io::Error>(42) },
            None,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    #[test]
    fn test_block_on_async_error() {
        let rt = Runtime::new().unwrap();
        let _guard = rt.enter();

        let result: Result<i32, mlua::Error> = block_on_async(
            "test_error",
            async {
                Err::<i32, std::io::Error>(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "test error",
                ))
            },
            None,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, mlua::Error::ExternalError(_)));
    }
    #[test]
    fn test_block_on_async_timeout() {
        let rt = Runtime::new().unwrap();
        let _guard = rt.enter();

        let result: Result<i32, mlua::Error> = block_on_async(
            "test_timeout",
            async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok::<i32, std::io::Error>(42)
            },
            Some(Duration::from_millis(100)),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, mlua::Error::RuntimeError(_)));
        if let mlua::Error::RuntimeError(msg) = err {
            assert!(msg.contains("timed out"));
        }
    }
    #[test]
    fn test_block_on_async_panic_safety() {
        let rt = Runtime::new().unwrap();
        let _guard = rt.enter();

        let result: Result<i32, mlua::Error> = block_on_async(
            "test_panic",
            async {
                panic!("deliberate panic for testing");
                #[allow(unreachable_code)]
                Ok::<i32, std::io::Error>(42)
            },
            None,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, mlua::Error::RuntimeError(_)));
        if let mlua::Error::RuntimeError(msg) = err {
            assert!(msg.contains("Runtime panic"));
        }
    }
    #[test]
    fn test_block_on_async_lua_success() {
        let rt = Runtime::new().unwrap();
        let _guard = rt.enter();

        let lua = Lua::new();
        let value = lua.create_string("test").unwrap();

        let result = block_on_async_lua(
            "test_lua_success",
            async { Ok(LuaValue::String(value)) },
            None,
        );

        assert!(result.is_ok());
        if let Ok(LuaValue::String(s)) = result {
            assert_eq!(s.to_str().unwrap(), "test");
        } else {
            panic!("Expected string value");
        }
    }
}
