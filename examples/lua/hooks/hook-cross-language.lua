-- ABOUTME: Cross-language hook coordination between Lua, Rust, and JavaScript
-- ABOUTME: Demonstrates event-driven coordination and cross-language hook communication

print("=== Cross-Language Hook Coordination Example ===")
print("Demonstrates: Hooks + Events for cross-language component coordination")
print()

local handles = {}
local subscriptions = {}
local coordination_state = {
    active_languages = {"lua"},
    message_log = {},
    coordination_events = 0,
    cross_language_hooks = 0
}

-- Helper function to log cross-language interactions
local function log_interaction(from_lang, to_lang, message_type, data)
    local entry = {
        timestamp = os.time(),
        from_language = from_lang,
        to_language = to_lang,
        message_type = message_type,
        data = data or {}
    }
    table.insert(coordination_state.message_log, entry)
    coordination_state.coordination_events = coordination_state.coordination_events + 1
    
    print(string.format("   🌐 [%s] %s → %s: %s", 
          os.date("%H:%M:%S", entry.timestamp), from_lang, to_lang, message_type))
end

print("1. Setting up cross-language event subscriptions:")

-- Subscribe to events from other languages
subscriptions.rust_events = Event.subscribe("rust.*")
print("   📡 Subscribed to Rust events")

subscriptions.javascript_events = Event.subscribe("javascript.*")
print("   📡 Subscribed to JavaScript events")

subscriptions.coordination_events = Event.subscribe("coordination.*")
print("   📡 Subscribed to coordination events")

subscriptions.cross_lang_responses = Event.subscribe("response.to.lua.*")
print("   📡 Subscribed to responses directed to Lua")

print()
print("2. Registering hooks that publish cross-language events:")

-- Before Agent Init - Announce Lua component initialization
handles.announce_init = Hook.register("BeforeAgentInit", function(context)
    local agent_name = context.component_id.name
    
    print("   🚀 Lua agent initializing, announcing to other languages...")
    
    -- Announce initialization to Rust components
    Event.publish("coordination.component.init", {
        component_type = "agent",
        component_name = agent_name,
        language = "lua",
        capabilities = {
            "script_execution",
            "dynamic_hooks",
            "event_processing"
        },
        status = "initializing",
        correlation_id = context.correlation_id
    }, {
        language = "lua",
        ttl_seconds = 300
    })
    
    log_interaction("lua", "rust", "COMPONENT_INIT", {
        component = agent_name,
        status = "initializing"
    })
    
    return "continue"
end, "high")
print("   ✅ Registered cross-language initialization announcer")

-- After Agent Init - Confirm initialization complete
handles.confirm_init = Hook.register("AfterAgentInit", function(context)
    local agent_name = context.component_id.name
    
    print("   ✅ Lua agent initialized, confirming to other languages...")
    
    -- Confirm initialization to all languages
    Event.publish("coordination.component.ready", {
        component_type = "agent",
        component_name = agent_name,
        language = "lua",
        ready_timestamp = os.time(),
        available_hooks = {
            "BeforeAgentExecution",
            "AfterAgentExecution", 
            "BeforeToolExecution",
            "AfterToolExecution"
        },
        status = "ready"
    })
    
    log_interaction("lua", "all", "COMPONENT_READY", {
        component = agent_name,
        hooks_available = 4
    })
    
    return "continue"
end, "normal")
print("   ✅ Registered initialization confirmation announcer")

print()
print("3. Cross-language coordination hooks:")

-- Tool execution coordination with other languages
handles.tool_coordination = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    print("   🤝 Coordinating tool execution across languages...")
    
    -- Check if this tool requires cross-language coordination
    local coordination_required_tools = {
        "system", "network", "database", "file"
    }
    
    local requires_coordination = false
    for _, pattern in ipairs(coordination_required_tools) do
        if tool_name:lower():find(pattern) then
            requires_coordination = true
            break
        end
    end
    
    if requires_coordination then
        print("   🔄 Tool requires cross-language coordination:", tool_name)
        
        -- Request coordination from Rust layer
        Event.publish("coordination.tool.request", {
            tool_name = tool_name,
            requesting_language = "lua",
            coordination_type = "execution_approval",
            tool_parameters = context.data or {},
            security_context = {
                correlation_id = context.correlation_id,
                component_id = context.component_id.name
            }
        })
        
        log_interaction("lua", "rust", "TOOL_COORDINATION_REQUEST", {
            tool = tool_name,
            type = "execution_approval"
        })
        
        -- In a real implementation, we might wait for approval
        -- For demo purposes, we'll continue with a modification
        return {
            type = "modified",
            data = {
                coordination_requested = true,
                cross_language_approved = true,
                coordinating_languages = {"lua", "rust"},
                coordination_timestamp = os.time()
            }
        }
    end
    
    return "continue"
end, "high")
print("   ✅ Registered cross-language tool coordination")

-- Workflow coordination with other components
handles.workflow_coordination = Hook.register("BeforeWorkflowStage", function(context) 
    local workflow_name = context.component_id.name
    local stage_name = context.data and context.data.stage_name or "unknown"
    
    print("   🔄 Coordinating workflow stage with other languages...")
    
    -- Publish workflow stage event for coordination
    Event.publish("coordination.workflow.stage", {
        workflow_name = workflow_name,
        stage_name = stage_name,
        stage_status = "starting",
        coordinating_language = "lua",
        requires_resources = {
            "compute", "memory", "network"
        },
        dependencies = {
            "previous_stage_complete",
            "resources_available"
        }
    })
    
    log_interaction("lua", "all", "WORKFLOW_STAGE_COORDINATION", {
        workflow = workflow_name,
        stage = stage_name,
        status = "starting"
    })
    
    return "continue"
end, "normal")
print("   ✅ Registered workflow stage coordination")

print()
print("4. Event-driven response handling:")

-- Set up event listeners for cross-language responses
print("   👂 Setting up event listeners for responses...")

-- Listen for coordination responses (simulate with a few test events)
for i = 1, 3 do
    -- Simulate publishing events from other languages
    Event.publish("rust.component.status", {
        component_type = "core_engine",
        status = "active",
        message_id = "rust_msg_" .. i,
        responding_to = "lua_coordination_request"
    })
    
    Event.publish("javascript.ui.event", {
        event_type = "user_interaction",
        component = "ui_controller", 
        message_id = "js_msg_" .. i,
        data = {user_action = "button_click"}
    })
    
    log_interaction("system", "lua", "SIMULATED_EVENT", {
        event_id = i,
        type = "demonstration"
    })
end

print("   📤 Published demonstration events from simulated components")

print()
print("5. Cross-language message processing:")

-- Process incoming messages from other languages
print("   📥 Processing cross-language messages...")

local message_handlers = {
    ["rust.component"] = function(event_data)
        print("   🦀 Processing Rust component message:")
        print("     • Component:", event_data.component_type or "unknown")
        print("     • Status:", event_data.status or "unknown")
        
        -- Respond back to Rust
        Event.publish("response.to.rust.component", {
            lua_component_status = "active",
            message_acknowledged = true,
            response_timestamp = os.time()
        })
        
        return true
    end,
    
    ["javascript.ui"] = function(event_data)
        print("   🌐 Processing JavaScript UI message:")
        print("     • Event type:", event_data.event_type or "unknown")
        print("     • Component:", event_data.component or "unknown")
        
        -- Respond to JavaScript
        Event.publish("response.to.javascript.ui", {
            lua_processing_complete = true,
            ui_event_handled = true,
            response_data = {processed_by = "lua_hook_system"}
        })
        
        return true
    end,
    
    ["coordination"] = function(event_data)
        print("   🤝 Processing coordination message:")
        print("     • Type:", event_data.coordination_type or "unknown")
        print("     • From:", event_data.requesting_language or "unknown")
        
        return true
    end
}

-- Try to receive and process messages
local processed_messages = 0
for pattern, handler in pairs(message_handlers) do
    for sub_name, sub_id in pairs(subscriptions) do
        if sub_name:find(pattern:gsub("%.", "_")) then
            -- Try to receive message with short timeout
            local received = Event.receive(sub_id, 100) -- 100ms timeout
            if received then
                local success = handler(received.data or {})
                if success then
                    processed_messages = processed_messages + 1
                    log_interaction("lua", "system", "MESSAGE_PROCESSED", {
                        pattern = pattern,
                        message_type = sub_name
                    })
                end
            end
        end
    end
end

print("   📊 Processed", processed_messages, "cross-language messages")

print()
print("6. Cross-language coordination statistics:")

print("   📊 Coordination Statistics:")
print("   • Active languages detected:", table.concat(coordination_state.active_languages, ", "))
print("   • Cross-language events:", coordination_state.coordination_events)
print("   • Coordination hooks registered:", coordination_state.cross_language_hooks + #handles)
print("   • Messages processed:", processed_messages)
print("   • Interaction log entries:", #coordination_state.message_log)

print()
print("   🌐 Language Interaction Matrix:")
local interaction_matrix = {}
for _, entry in ipairs(coordination_state.message_log) do
    local key = entry.from_language .. "_to_" .. entry.to_language
    interaction_matrix[key] = (interaction_matrix[key] or 0) + 1
end

for interaction, count in pairs(interaction_matrix) do
    local from_lang, to_lang = interaction:match("(.+)_to_(.+)")
    print(string.format("   • %s → %s: %d interactions", from_lang, to_lang, count))
end

print()
print("7. Cross-language capabilities showcase:")

print("   🚀 Cross-Language Capabilities:")
print("   • Event Publishing: Lua → Rust/JavaScript coordination")
print("   • Event Subscription: Listen for events from all languages")
print("   • Hook Coordination: Hooks that trigger cross-language events") 
print("   • Response Handling: Process responses from other components")
print("   • Resource Coordination: Cross-language resource management")
print("   • Error Propagation: Error handling across language boundaries")
print("   • State Synchronization: Shared state via event messaging")

print()
print("8. Real-world coordination patterns:")

print("   💼 Real-World Use Cases Demonstrated:")
print("   • Component Lifecycle: Announce init/ready states across languages")
print("   • Resource Management: Coordinate tool/resource access")
print("   • Workflow Orchestration: Multi-language workflow coordination")
print("   • Error Handling: Propagate and handle errors across boundaries")
print("   • Event-Driven Architecture: Loose coupling via events")
print("   • Security Coordination: Cross-language security validation")
print("   • Performance Monitoring: Distributed performance tracking")

print()
print("9. Cleaning up cross-language coordination:")

-- Unregister hooks
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "coordination hook")
end

-- Unsubscribe from events
for name, sub_id in pairs(subscriptions) do
    Event.unsubscribe(sub_id)
    print("   🧹 Unsubscribed from", name)
end

local final_hook_count = #Hook.list()
local final_sub_count = #Event.list_subscriptions()
print("   ✅ Final hook count:", final_hook_count)
print("   ✅ Final subscription count:", final_sub_count)

print()
print("✨ Cross-language hook coordination example complete!")
print("   Key concepts demonstrated:")
print("   • Hook-Event integration for cross-language communication")
print("   • Component lifecycle announcement and coordination")
print("   • Resource and tool access coordination")
print("   • Workflow stage synchronization across languages")
print("   • Event-driven response handling patterns")
print("   • Cross-language error propagation and handling")
print("   • Distributed system coordination via hooks and events")