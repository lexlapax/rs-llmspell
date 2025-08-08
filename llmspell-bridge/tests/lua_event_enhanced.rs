//! ABOUTME: Tests for enhanced Lua Event API functionality
//! ABOUTME: Tests Event.publish, Event.subscribe, cross-language events, and patterns

use llmspell_bridge::globals::types::GlobalContext;
use llmspell_bridge::lua::globals::event::inject_event_global;
use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
use mlua::Lua;
use std::sync::Arc;

async fn create_test_environment() -> (Lua, GlobalContext) {
    let lua = Lua::new();
    let registry = Arc::new(ComponentRegistry::new());
    let providers = Arc::new(
        ProviderManager::new(ProviderManagerConfig::default())
            .await
            .unwrap(),
    );
    let context = GlobalContext::new(registry, providers);

    inject_event_global(&lua, &context).unwrap();

    (lua, context)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_publish_and_subscribe() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Subscribe to test events
        local sub_id = Event.subscribe("test.*")
        
        -- Publish a test event
        local published = Event.publish("test.example", {
            message = "hello from lua",
            number = 42,
            nested = {
                data = "nested value"
            }
        })
        
        -- Try to receive the event (with timeout)
        local received = Event.receive(sub_id, 2000)  -- 2 second timeout
        
        -- Clean up
        Event.unsubscribe(sub_id)
        
        return published and (received ~= nil)
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should publish and receive events successfully"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_publish_with_options() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Subscribe to events
        local sub_id = Event.subscribe("custom.*")
        
        -- Publish event with full options
        local published = Event.publish("custom.event", {
            data = "test data"
        }, {
            language = "lua",
            correlation_id = "12345678-1234-1234-1234-123456789abc",
            ttl_seconds = 300
        })
        
        -- Clean up
        Event.unsubscribe(sub_id)
        
        return published
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should publish events with options"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_pattern_matching() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Test different pattern subscriptions
        local sub1 = Event.subscribe("user.*")
        local sub2 = Event.subscribe("system.*")
        local sub3 = Event.subscribe("*.error")
        
        -- Publish different events
        Event.publish("user.login", {action = "login"})
        Event.publish("system.startup", {action = "startup"})
        Event.publish("app.error", {error = "test error"})
        Event.publish("unmatched.event", {data = "should not match"})
        
        -- Try to receive events (short timeout for testing)
        local user_event = Event.receive(sub1, 500)
        local system_event = Event.receive(sub2, 500)
        local error_event = Event.receive(sub3, 500)
        
        -- Clean up
        Event.unsubscribe(sub1)
        Event.unsubscribe(sub2)
        Event.unsubscribe(sub3)
        
        -- At least one event should be received
        return (user_event ~= nil) or (system_event ~= nil) or (error_event ~= nil)
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should support pattern matching for events"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_list_subscriptions() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Initially no subscriptions
        local subs_before = Event.list_subscriptions()
        
        -- Create some subscriptions
        local sub1 = Event.subscribe("user.*")
        local sub2 = Event.subscribe("system.*")
        local sub3 = Event.subscribe("error.*")
        
        -- Check subscriptions
        local subs_after = Event.list_subscriptions()
        
        -- Verify subscription structure
        local has_proper_structure = true
        for _, sub in ipairs(subs_after) do
            if not (sub.id and sub.pattern and sub.language) then
                has_proper_structure = false
                break
            end
        end
        
        -- Clean up
        Event.unsubscribe(sub1)
        Event.unsubscribe(sub2)
        Event.unsubscribe(sub3)
        
        -- Should have more subscriptions after creating them
        return (#subs_after > #subs_before) and has_proper_structure
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should list subscriptions with proper structure"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_stats() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r"
        local stats = Event.get_stats()
        
        -- Verify stats structure
        return stats ~= nil and 
               stats.event_bus_stats ~= nil and 
               stats.bridge_stats ~= nil
    ",
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should provide event system statistics"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_unsubscribe() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Create subscription
        local sub_id = Event.subscribe("test.*")
        
        -- Verify it exists
        local subs_before = Event.list_subscriptions()
        local found_before = false
        for _, sub in ipairs(subs_before) do
            if sub.id == sub_id then
                found_before = true
                break
            end
        end
        
        -- Unsubscribe
        local unsubscribed = Event.unsubscribe(sub_id)
        
        -- Verify it's gone
        local subs_after = Event.list_subscriptions()
        local found_after = false
        for _, sub in ipairs(subs_after) do
            if sub.id == sub_id then
                found_after = true
                break
            end
        end
        
        return found_before and unsubscribed and (not found_after)
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should unsubscribe from events successfully"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_timeout_behavior() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Subscribe to events
        local sub_id = Event.subscribe("timeout.*")
        
        -- Verify subscription was created
        local subs = Event.list_subscriptions()
        local found_sub = false
        for _, sub in ipairs(subs) do
            if sub.id == sub_id then
                found_sub = true
                break
            end
        end
        
        -- Try to receive with short timeout (no events published)
        local received = Event.receive(sub_id, 100)  -- 100ms timeout
        
        -- Clean up
        Event.unsubscribe(sub_id)
        
        -- Should find subscription and receive should return nil (timeout)
        return found_sub and (received == nil)
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should handle event receive timeouts correctly (subscription found, receive returns nil)"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_event_cross_language_simulation() {
    let (lua, _context) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Simulate cross-language communication
        local rust_sub = Event.subscribe("rust.*")
        local js_sub = Event.subscribe("javascript.*")
        
        -- Publish events as if from different languages
        local rust_published = Event.publish("rust.computation", {
            result = 42,
            computation_time = 0.001
        }, {
            language = "rust"
        })
        
        local js_published = Event.publish("javascript.ui_event", {
            event_type = "click",
            element_id = "button1"
        }, {
            language = "javascript"
        })
        
        -- Try to receive cross-language events
        local rust_event = Event.receive(rust_sub, 1000)
        local js_event = Event.receive(js_sub, 1000)
        
        -- Clean up
        Event.unsubscribe(rust_sub)
        Event.unsubscribe(js_sub)
        
        return rust_published and js_published and 
               ((rust_event ~= nil) or (js_event ~= nil))
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should support cross-language event simulation"
    );
}
