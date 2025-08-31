//! Hook multiplexer to allow multiple debug hooks to coexist
//!
//! Since Lua only supports one debug hook at a time, this provides
//! a multiplexing layer that allows multiple systems to register hooks.

use crate::debug_state_cache::DebugStateCache;
use mlua::{Debug, DebugEvent, HookTriggers, Lua, Result as LuaResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Priority for hook execution (lower executes first)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HookPriority(pub i32);

impl HookPriority {
    pub const PROFILER: Self = Self(-1000); // Profilers run first (minimal overhead)
    pub const DEBUGGER: Self = Self(0); // Debuggers run second
    pub const MONITOR: Self = Self(1000); // Monitors run last
}

/// A registered hook handler
pub trait HookHandler: Send + Sync {
    /// Handle a debug event
    ///
    /// # Errors
    ///
    /// Returns an error if handling the event fails
    fn handle_event(&mut self, lua: &Lua, ar: &Debug, event: DebugEvent) -> LuaResult<()>;

    /// Get the events this handler is interested in
    fn interested_events(&self) -> HookTriggers;

    /// Check if this handler is currently active
    fn is_active(&self) -> bool {
        true
    }
}

type HandlerMap = HashMap<String, (HookPriority, Box<dyn HookHandler>)>;

/// Hook multiplexer that manages multiple debug hooks
pub struct HookMultiplexer {
    /// Registered handlers by ID
    handlers: Arc<RwLock<HandlerMap>>,
    /// Combined triggers from all handlers
    combined_triggers: Arc<RwLock<HookTriggers>>,
    /// Whether multiplexer is installed
    installed: Arc<RwLock<bool>>,
}

impl HookMultiplexer {
    /// Create a new hook multiplexer
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            combined_triggers: Arc::new(RwLock::new(HookTriggers::default())),
            installed: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a hook handler
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails
    pub fn register_handler(
        &self,
        id: String,
        priority: HookPriority,
        handler: Box<dyn HookHandler>,
    ) -> LuaResult<()> {
        // Add handler
        self.handlers.write().insert(id, (priority, handler));

        // Update combined triggers
        self.update_combined_triggers();

        Ok(())
    }

    /// Unregister a hook handler
    #[must_use]
    pub fn unregister_handler(&self, id: &str) -> bool {
        let removed = self.handlers.write().remove(id).is_some();

        if removed {
            self.update_combined_triggers();
        }

        removed
    }

    /// Update combined triggers from all handlers
    fn update_combined_triggers(&self) {
        let mut combined = HookTriggers::default();
        let mut min_instruction_interval: Option<u32> = None;

        {
            let handlers = self.handlers.read();
            for (_, handler) in handlers.values() {
                if !handler.is_active() {
                    continue;
                }

                let triggers = handler.interested_events();

                // Combine all trigger types
                combined.on_calls |= triggers.on_calls;
                combined.on_returns |= triggers.on_returns;
                combined.every_line |= triggers.every_line;

                // Take minimum instruction interval
                if let Some(interval) = triggers.every_nth_instruction {
                    min_instruction_interval = Some(
                        min_instruction_interval.map_or(interval, |current| current.min(interval)),
                    );
                }
            }
        }

        combined.every_nth_instruction = min_instruction_interval;
        *self.combined_triggers.write() = combined;
    }

    /// Install the multiplexer as the Lua debug hook
    ///
    /// # Errors
    ///
    /// Returns an error if installation fails
    pub fn install(&self, lua: &Lua) -> LuaResult<()> {
        let handlers = self.handlers.clone();
        let triggers = *self.combined_triggers.read();

        // Only install if we have handlers
        if handlers.read().is_empty() {
            return Ok(());
        }

        lua.set_hook(triggers, move |lua, ar| {
            // Determine event type - check actual event first, not line number
            let event = if ar.event() == mlua::DebugEvent::Call {
                DebugEvent::Call
            } else if ar.event() == mlua::DebugEvent::TailCall {
                DebugEvent::TailCall
            } else if ar.event() == mlua::DebugEvent::Ret {
                DebugEvent::Ret
            } else if ar.curr_line() != -1 {
                DebugEvent::Line
            } else {
                return Ok(());
            };

            // Get handlers sorted by priority
            let mut sorted_handlers: Vec<_> = {
                let handlers = handlers.read();
                handlers
                    .iter()
                    .filter(|(_, (_, h))| h.is_active())
                    .map(|(id, (priority, _))| (id.clone(), *priority))
                    .collect()
            };
            sorted_handlers.sort_by_key(|(_, p)| *p);

            // Execute handlers in priority order
            for (id, _) in sorted_handlers {
                if let Some((_, handler)) = handlers.write().get_mut(&id) {
                    // Check if this handler is interested in this event
                    let triggers = handler.interested_events();
                    let interested = match event {
                        DebugEvent::Line => triggers.every_line,
                        DebugEvent::Call | DebugEvent::TailCall => triggers.on_calls,
                        DebugEvent::Ret => triggers.on_returns,
                        _ => false,
                    };

                    if interested {
                        handler.handle_event(lua, &ar, event)?;
                    }
                }
            }

            Ok(())
        });

        *self.installed.write() = true;
        Ok(())
    }

    /// Uninstall the multiplexer
    pub fn uninstall(&self, lua: &Lua) {
        if *self.installed.read() {
            lua.remove_hook();
            *self.installed.write() = false;
        }
    }

    /// Check if any handlers are registered
    #[must_use]
    pub fn has_handlers(&self) -> bool {
        !self.handlers.read().is_empty()
    }

    /// Get number of registered handlers
    #[must_use]
    pub fn handler_count(&self) -> usize {
        self.handlers.read().len()
    }
}

impl Default for HookMultiplexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter to wrap our `LuaExecutionHook` as a `HookHandler`
pub struct DebugHookAdapter {
    pub inner: Arc<parking_lot::Mutex<crate::lua::globals::execution::LuaExecutionHook>>,
}

impl HookHandler for DebugHookAdapter {
    fn handle_event(&mut self, lua: &Lua, ar: &Debug, event: DebugEvent) -> LuaResult<()> {
        self.inner.lock().handle_event(lua, ar, event)
    }

    fn interested_events(&self) -> HookTriggers {
        use crate::debug_state_cache::DebugMode;

        let mode = self.inner.lock().debug_cache().get_debug_mode();
        match mode {
            DebugMode::Disabled => HookTriggers::default(),
            DebugMode::Minimal { check_interval } => HookTriggers {
                every_nth_instruction: Some(check_interval),
                ..Default::default()
            },
            DebugMode::Full => HookTriggers {
                on_calls: true,
                on_returns: true,
                every_line: true,
                ..Default::default()
            },
        }
    }

    fn is_active(&self) -> bool {
        use crate::debug_state_cache::DebugMode;
        !matches!(
            self.inner.lock().debug_cache().get_debug_mode(),
            DebugMode::Disabled
        )
    }
}

/// Example profiler hook handler
pub struct ProfilerHook {
    pub sample_count: Arc<std::sync::atomic::AtomicU64>,
}

impl HookHandler for ProfilerHook {
    fn handle_event(&mut self, _lua: &Lua, _ar: &Debug, event: DebugEvent) -> LuaResult<()> {
        if matches!(event, DebugEvent::Line) {
            self.sample_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    fn interested_events(&self) -> HookTriggers {
        HookTriggers {
            every_line: true, // For testing - use line events
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplexer_priority() {
        let multiplexer = HookMultiplexer::new();

        // Register handlers with different priorities
        let profiler = Box::new(ProfilerHook {
            sample_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        });

        multiplexer
            .register_handler("profiler".to_string(), HookPriority::PROFILER, profiler)
            .unwrap();

        assert_eq!(multiplexer.handler_count(), 1);

        // Remove handler
        assert!(multiplexer.unregister_handler("profiler"));
        assert_eq!(multiplexer.handler_count(), 0);
    }

    #[test]
    fn test_combined_triggers() {
        // Handler wanting line events
        struct LineHandler;
        impl HookHandler for LineHandler {
            fn handle_event(&mut self, _: &Lua, _: &Debug, _: DebugEvent) -> LuaResult<()> {
                Ok(())
            }
            fn interested_events(&self) -> HookTriggers {
                HookTriggers {
                    every_line: true,
                    ..Default::default()
                }
            }
        }

        // Handler wanting instruction count
        struct CountHandler;
        impl HookHandler for CountHandler {
            fn handle_event(&mut self, _: &Lua, _: &Debug, _: DebugEvent) -> LuaResult<()> {
                Ok(())
            }
            fn interested_events(&self) -> HookTriggers {
                HookTriggers {
                    every_nth_instruction: Some(500),
                    ..Default::default()
                }
            }
        }

        let multiplexer = HookMultiplexer::new();

        multiplexer
            .register_handler("line".to_string(), HookPriority(0), Box::new(LineHandler))
            .unwrap();

        multiplexer
            .register_handler("count".to_string(), HookPriority(1), Box::new(CountHandler))
            .unwrap();

        // Combined triggers should have both
        let combined = *multiplexer.combined_triggers.read();
        assert!(combined.every_line);
        assert_eq!(combined.every_nth_instruction, Some(500));
    }
}
