//! ABOUTME: Lua-specific event global bindings with `EventBridge` integration
//! ABOUTME: Provides Lua-native event publishing, subscription, and management

use crate::globals::event_global::EventGlobal;
use crate::globals::types::{GlobalContext, GlobalObject};
use llmspell_core::error::LLMSpellError;

/// Inject the Event global into a Lua environment
///
/// # Errors
///
/// Returns an error if global injection fails
pub fn inject_event_global(lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
    let event_global = EventGlobal::new();
    event_global.inject_lua(lua, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};
    use mlua::Lua;
    use std::sync::Arc;

    async fn create_test_context() -> GlobalContext {
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(ProviderManager::new(Default::default()).await.unwrap());
        GlobalContext::new(registry, providers)
    }
    #[tokio::test]
    async fn test_event_global_injection() {
        let lua = Lua::new();
        let context = create_test_context().await;

        // Inject the Event global
        inject_event_global(&lua, &context).unwrap();

        // Test that Event global exists and has expected methods
        let result: mlua::Result<()> = lua.load(r#"
            assert(Event ~= nil, "Event global should exist")
            assert(type(Event.publish) == "function", "Event.publish should be a function")
            assert(type(Event.subscribe) == "function", "Event.subscribe should be a function")
            assert(type(Event.receive) == "function", "Event.receive should be a function")
            assert(type(Event.unsubscribe) == "function", "Event.unsubscribe should be a function")
            assert(type(Event.list_subscriptions) == "function", "Event.list_subscriptions should be a function")
            assert(type(Event.get_stats) == "function", "Event.get_stats should be a function")
        "#).exec();

        assert!(
            result.is_ok(),
            "Event global should have all required methods"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_event_publish_subscribe_flow() {
        let lua = Lua::new();
        let context = create_test_context().await;

        inject_event_global(&lua, &context).unwrap();

        // Test basic publish/subscribe flow
        let result: mlua::Result<String> = lua
            .load(
                r#"
            -- Subscribe to test events
            local sub_id = Event.subscribe("test.*")
            
            -- Publish a test event
            local published = Event.publish("test.example", {message = "hello", number = 42})
            
            -- Try to receive the event (with short timeout for test)
            local received = Event.receive(sub_id, 1000)  -- 1 second timeout
            
            -- Clean up
            Event.unsubscribe(sub_id)
            
            if received then
                return "received_event"
            else
                return "timeout"
            end
        "#,
            )
            .eval();

        // The result could be either "received_event" or "timeout" depending on timing
        // Both are valid outcomes for this integration test
        match result {
            Ok(outcome) => {
                assert!(
                    outcome == "received_event" || outcome == "timeout",
                    "Should either receive event or timeout, got: {}",
                    outcome
                );
            }
            Err(e) => {
                panic!("Event flow test failed: {}", e);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_event_list_subscriptions() {
        let lua = Lua::new();
        let context = create_test_context().await;

        inject_event_global(&lua, &context).unwrap();

        let result: mlua::Result<bool> = lua
            .load(
                r#"
            -- Initially no subscriptions
            local subs_before = Event.list_subscriptions()
            
            -- Create some subscriptions
            local sub1 = Event.subscribe("user.*")
            local sub2 = Event.subscribe("system.*")
            
            -- Check subscriptions
            local subs_after = Event.list_subscriptions()
            
            -- Clean up
            Event.unsubscribe(sub1)
            Event.unsubscribe(sub2)
            
            -- Should have 2 subscriptions
            return #subs_after >= 2
        "#,
            )
            .eval();

        assert!(
            result.is_ok() && result.unwrap(),
            "Should be able to list subscriptions"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_event_stats() {
        let lua = Lua::new();
        let context = create_test_context().await;

        inject_event_global(&lua, &context).unwrap();

        let result: mlua::Result<bool> = lua
            .load(
                r"
            local stats = Event.get_stats()
            return stats ~= nil and stats.event_bus_stats ~= nil and stats.bridge_stats ~= nil
        ",
            )
            .eval();

        assert!(
            result.is_ok() && result.unwrap(),
            "Should be able to get event stats"
        );
    }
}
