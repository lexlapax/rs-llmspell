-- ABOUTME: Events triggered by hooks and hook-event integration patterns
-- ABOUTME: Demonstrates bidirectional hook-event communication, event-driven hooks, and integrated patterns

print("=== Event Hook Integration Example ===")
print("Demonstrates: Hook-event integration, bidirectional communication, event-driven hooks, and advanced patterns")
print()

local subscriptions = {}
local hook_handles = {}
local integration_stats = {
    hooks_registered = 0,
    events_from_hooks = 0,
    events_to_hooks = 0,
    hook_event_cycles = 0,
    integration_patterns = {},
    performance_metrics = {}
}

-- Helper function to track integration event
local function track_integration_event(event_type, source, target, details)
    local entry = {
        timestamp = os.time(),
        event_type = event_type,
        source = source,
        target = target,
        details = details or {}
    }
    
    if event_type == "HOOK_TO_EVENT" then
        integration_stats.events_from_hooks = integration_stats.events_from_hooks + 1
    elseif event_type == "EVENT_TO_HOOK" then
        integration_stats.events_to_hooks = integration_stats.events_to_hooks + 1
    elseif event_type == "HOOK_EVENT_CYCLE" then
        integration_stats.hook_event_cycles = integration_stats.hook_event_cycles + 1
    end
    
    print(string.format("   🔄 [%s] %s → %s", event_type:gsub("_", " "), source, target))
end

print("1. Setting up hook-event integration infrastructure:")

print("   📡 Creating event subscriptions for hook integration:")

-- Create subscriptions for hook-event integration patterns
local integration_patterns = {
    hook_notifications = "hook.notification.*",
    hook_data_events = "hook.data.*",
    hook_lifecycle = "hook.lifecycle.*", 
    hook_coordination = "hook.coordination.*",
    event_triggers = "event.trigger.*",
    workflow_hooks = "workflow.hook.*",
    system_events = "system.event.*",
    integration_monitoring = "integration.monitor.*"
}

for pattern_name, pattern in pairs(integration_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    integration_stats.integration_patterns[pattern_name] = 0
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Hook-event integration infrastructure ready")

print()
print("2. Hooks that publish events:")

print("   🪝 Registering hooks that publish events:")

-- Hook that publishes events when agents are initialized
hook_handles.agent_lifecycle = Hook.register("BeforeAgentInit", function(context)
    track_integration_event("HOOK_TO_EVENT", "BeforeAgentInit", "hook.lifecycle.agent_init")
    
    -- Publish event about agent initialization
    Event.publish("hook.lifecycle.agent_init", {
        agent_id = context.component_id.name,
        initialization_timestamp = os.time(),
        hook_trigger = "BeforeAgentInit",
        context_data = {
            correlation_id = context.correlation_id,
            metadata = context.metadata or {}
        }
    })
    
    print(string.format("   📤 Hook published agent init event for: %s", context.component_id.name))
    return "continue"
end, "normal")

integration_stats.hooks_registered = integration_stats.hooks_registered + 1
print("   ✅ Agent lifecycle hook registered")

-- Hook that publishes events during tool execution
hook_handles.tool_monitor = Hook.register("BeforeToolExecution", function(context)
    track_integration_event("HOOK_TO_EVENT", "BeforeToolExecution", "hook.notification.tool_start")
    
    -- Publish event about tool execution start
    Event.publish("hook.notification.tool_start", {
        tool_name = context.component_id.name,
        execution_start = os.time(),
        hook_source = "BeforeToolExecution",
        execution_context = {
            correlation_id = context.correlation_id,
            expected_duration = math.random(100, 1000),
            priority = "normal"
        }
    })
    
    print(string.format("   📤 Hook published tool start event for: %s", context.component_id.name))
    return "continue"
end, "high")

hook_handles.tool_completion = Hook.register("AfterToolExecution", function(context)
    track_integration_event("HOOK_TO_EVENT", "AfterToolExecution", "hook.notification.tool_complete")
    
    -- Publish event about tool execution completion
    Event.publish("hook.notification.tool_complete", {
        tool_name = context.component_id.name,
        execution_end = os.time(),
        hook_source = "AfterToolExecution",
        execution_result = {
            correlation_id = context.correlation_id,
            success = true,
            duration_estimate = math.random(50, 500),
            performance_score = 0.8 + math.random() * 0.2
        }
    })
    
    print(string.format("   📤 Hook published tool completion event for: %s", context.component_id.name))
    return "continue"
end, "high")

integration_stats.hooks_registered = integration_stats.hooks_registered + 2
print("   ✅ Tool monitoring hooks registered")

-- Hook that publishes data events with processing results
hook_handles.data_processor = Hook.register("BeforeAgentExecution", function(context)
    track_integration_event("HOOK_TO_EVENT", "BeforeAgentExecution", "hook.data.processing")
    
    -- Simulate data processing and publish results
    local processing_results = {
        agent_id = context.component_id.name,
        processing_timestamp = os.time(),
        data_metrics = {
            records_processed = math.random(100, 1000),
            processing_time_ms = math.random(50, 200),
            quality_score = 0.7 + math.random() * 0.3,
            throughput_rps = math.random(10, 100)
        },
        hook_metadata = {
            correlation_id = context.correlation_id,
            hook_point = "BeforeAgentExecution"
        }
    }
    
    Event.publish("hook.data.processing", processing_results)
    
    print(string.format("   📤 Hook published data processing event: %d records", 
          processing_results.data_metrics.records_processed))
    
    return "continue"
end, "low")

integration_stats.hooks_registered = integration_stats.hooks_registered + 1
print("   ✅ Data processing hook registered")

print()
print("3. Events that trigger hooks:")

print("   📡 Publishing events that will trigger hook responses:")

-- Simulate external events that should trigger hook responses
local trigger_events = {
    {
        name = "event.trigger.system_alert",
        data = {
            alert_type = "high_cpu_usage",
            severity = "warning",
            threshold_exceeded = 85.5,
            current_value = 92.1,
            timestamp = os.time()
        }
    },
    {
        name = "event.trigger.user_action",
        data = {
            user_id = "user_001",
            action = "critical_operation",
            requires_approval = true,
            request_timestamp = os.time()
        }
    },
    {
        name = "event.trigger.workflow_milestone",
        data = {
            workflow_id = "wf_integration_001",
            milestone = "data_processing_complete",
            next_phase = "analysis",
            completion_percentage = 75.0
        }
    }
}

for i, trigger_event in ipairs(trigger_events) do
    Event.publish(trigger_event.name, trigger_event.data)
    print(string.format("   %d. ✅ Published trigger event: %s", i, trigger_event.name))
end

print()
print("4. Event-driven hook registration:")

print("   🎯 Implementing event-driven hook registration:")

-- Hook that registers other hooks based on events
hook_handles.dynamic_hook_manager = Hook.register("BeforeWorkflowStart", function(context)
    track_integration_event("EVENT_TO_HOOK", "workflow_event", "dynamic_hook_registration")
    
    -- Register dynamic hooks based on workflow type
    local workflow_name = context.component_id.name
    
    if workflow_name:find("critical") then
        -- Register additional monitoring hooks for critical workflows
        local critical_monitor = Hook.register("BeforeWorkflowStep", function(step_context)
            track_integration_event("HOOK_TO_EVENT", "dynamic_critical_monitor", "hook.monitoring.critical_step")
            
            Event.publish("hook.monitoring.critical_step", {
                workflow_id = context.correlation_id,
                step_name = step_context.component_id.name,
                monitoring_level = "critical",
                timestamp = os.time(),
                dynamic_registration = true
            })
            
            print(string.format("   📊 Critical step monitor: %s", step_context.component_id.name))
            return "continue"
        end, "highest")
        
        -- Store handle for later cleanup
        hook_handles["dynamic_critical_" .. context.correlation_id] = critical_monitor
        integration_stats.hooks_registered = integration_stats.hooks_registered + 1
        
        print(string.format("   ✅ Registered dynamic critical monitoring for workflow: %s", workflow_name))
    end
    
    return "continue"
end, "normal")

print("   ✅ Dynamic hook manager registered")
integration_stats.hooks_registered = integration_stats.hooks_registered + 1

print()
print("5. Bidirectional hook-event communication:")

print("   🔄 Implementing bidirectional communication patterns:")

-- Hook that both responds to events and publishes new events
hook_handles.bidirectional_coordinator = Hook.register("BeforeAgentExecution", function(context)
    local agent_name = context.component_id.name
    
    -- Step 1: Check for coordination events (simulated)
    track_integration_event("EVENT_TO_HOOK", "coordination_check", "bidirectional_coordinator")
    
    -- Simulate checking for coordination events
    local coordination_needed = math.random() > 0.7 -- 30% chance
    
    if coordination_needed then
        -- Step 2: Publish coordination request event
        track_integration_event("HOOK_TO_EVENT", "bidirectional_coordinator", "hook.coordination.request")
        
        Event.publish("hook.coordination.request", {
            requesting_agent = agent_name,
            coordination_type = "resource_allocation",
            priority = "high",
            request_timestamp = os.time(),
            hook_initiated = true,
            expected_response_time = 5000 -- 5 seconds
        })
        
        print(string.format("   🤝 Agent %s requested coordination", agent_name))
        
        -- Step 3: Wait for coordination response (simulated)
        -- In a real system, this would be asynchronous
        local coordination_granted = math.random() > 0.3 -- 70% success rate
        
        if coordination_granted then
            -- Step 4: Publish coordination success event
            track_integration_event("HOOK_TO_EVENT", "bidirectional_coordinator", "hook.coordination.granted")
            
            Event.publish("hook.coordination.granted", {
                requesting_agent = agent_name,
                coordination_id = "coord_" .. os.time(),
                granted_resources = {"cpu", "memory", "network"},
                grant_timestamp = os.time(),
                expires_at = os.time() + 300 -- 5 minutes
            })
            
            print(string.format("   ✅ Coordination granted for agent %s", agent_name))
            integration_stats.hook_event_cycles = integration_stats.hook_event_cycles + 1
        else
            -- Step 4: Publish coordination failure event
            track_integration_event("HOOK_TO_EVENT", "bidirectional_coordinator", "hook.coordination.denied")
            
            Event.publish("hook.coordination.denied", {
                requesting_agent = agent_name,
                denial_reason = "resource_unavailable",
                retry_after_seconds = 30,
                denial_timestamp = os.time()
            })
            
            print(string.format("   ❌ Coordination denied for agent %s", agent_name))
        end
    end
    
    return "continue"
end, "normal")

print("   ✅ Bidirectional coordinator hook registered")
integration_stats.hooks_registered = integration_stats.hooks_registered + 1

print()
print("6. Event-driven hook chaining:")

print("   ⛓️  Implementing event-driven hook chaining:")

-- Chain of hooks that communicate via events
local chain_state = {current_step = 0, max_steps = 3}

-- First hook in chain
hook_handles.chain_step_1 = Hook.register("BeforeToolExecution", function(context)
    if chain_state.current_step == 0 then
        chain_state.current_step = 1
        track_integration_event("HOOK_TO_EVENT", "chain_step_1", "hook.chain.step_1")
        
        Event.publish("hook.chain.step_1", {
            chain_id = "chain_001",
            step = 1,
            tool_name = context.component_id.name,
            step_data = {
                validation_passed = true,
                pre_processing_complete = true
            },
            next_step = "step_2",
            timestamp = os.time()
        })
        
        print("   ⛓️  Chain Step 1: Pre-processing completed")
        integration_stats.hook_event_cycles = integration_stats.hook_event_cycles + 1
    end
    
    return "continue"
end, "highest")

-- Second hook in chain (triggered by step 1)
hook_handles.chain_step_2 = Hook.register("AfterToolExecution", function(context)
    if chain_state.current_step == 1 then
        chain_state.current_step = 2
        track_integration_event("HOOK_TO_EVENT", "chain_step_2", "hook.chain.step_2")
        
        Event.publish("hook.chain.step_2", {
            chain_id = "chain_001",
            step = 2,
            tool_name = context.component_id.name,
            step_data = {
                execution_completed = true,
                results_validated = true,
                performance_score = 0.95
            },
            next_step = "step_3",
            timestamp = os.time()
        })
        
        print("   ⛓️  Chain Step 2: Execution completed")
        integration_stats.hook_event_cycles = integration_stats.hook_event_cycles + 1
    end
    
    return "continue"
end, "high")

-- Third hook in chain (triggered by step 2)
hook_handles.chain_step_3 = Hook.register("BeforeAgentShutdown", function(context)
    if chain_state.current_step == 2 then
        chain_state.current_step = 3
        track_integration_event("HOOK_TO_EVENT", "chain_step_3", "hook.chain.complete")
        
        Event.publish("hook.chain.complete", {
            chain_id = "chain_001",
            step = 3,
            agent_name = context.component_id.name,
            chain_result = {
                total_steps = chain_state.max_steps,
                completion_successful = true,
                chain_duration = os.time() - (os.time() - 10), -- Simulated
                final_status = "success"
            },
            timestamp = os.time()
        })
        
        print("   ⛓️  Chain Step 3: Chain completed successfully")
        integration_stats.hook_event_cycles = integration_stats.hook_event_cycles + 1
        
        -- Reset chain for potential reuse
        chain_state.current_step = 0
    end
    
    return "continue"
end, "low")

print("   ✅ Hook chain registered (3 steps)")
integration_stats.hooks_registered = integration_stats.hooks_registered + 3

print()
print("7. Event aggregation from multiple hooks:")

print("   📊 Implementing event aggregation patterns:")

-- Aggregator that collects events from multiple hooks
local aggregation_buffer = {
    events = {},
    buffer_size = 5,
    flush_interval = 10 -- seconds
}

-- Hooks that feed into aggregation
for i = 1, 3 do
    hook_handles["aggregation_source_" .. i] = Hook.register("BeforeAgentExecution", function(context)
        track_integration_event("HOOK_TO_EVENT", "aggregation_source_" .. i, "hook.data.sample")
        
        -- Generate sample data for aggregation
        local sample_data = {
            source_hook = "aggregation_source_" .. i,
            agent_name = context.component_id.name,
            timestamp = os.time(),
            metrics = {
                cpu_usage = math.random(10, 90),
                memory_usage = math.random(20, 80),
                response_time = math.random(50, 300)
            },
            correlation_id = context.correlation_id
        }
        
        Event.publish("hook.data.sample", sample_data)
        
        -- Add to aggregation buffer
        table.insert(aggregation_buffer.events, sample_data)
        
        print(string.format("   📈 Aggregation source %d: CPU=%.1f%%, Memory=%.1f%%", 
              i, sample_data.metrics.cpu_usage, sample_data.metrics.memory_usage))
        
        -- Check if buffer should be flushed
        if #aggregation_buffer.events >= aggregation_buffer.buffer_size then
            track_integration_event("HOOK_TO_EVENT", "aggregation_flush", "hook.data.aggregated")
            
            -- Calculate aggregated metrics
            local total_cpu = 0
            local total_memory = 0
            local total_response_time = 0
            
            for _, event in ipairs(aggregation_buffer.events) do
                total_cpu = total_cpu + event.metrics.cpu_usage
                total_memory = total_memory + event.metrics.memory_usage
                total_response_time = total_response_time + event.metrics.response_time
            end
            
            local count = #aggregation_buffer.events
            
            Event.publish("hook.data.aggregated", {
                aggregation_id = "agg_" .. os.time(),
                sample_count = count,
                time_window = {
                    start = aggregation_buffer.events[1].timestamp,
                    end = aggregation_buffer.events[count].timestamp
                },
                aggregated_metrics = {
                    avg_cpu_usage = total_cpu / count,
                    avg_memory_usage = total_memory / count,
                    avg_response_time = total_response_time / count,
                    max_cpu = math.max(table.unpack((function()
                        local cpu_values = {}
                        for _, e in ipairs(aggregation_buffer.events) do
                            table.insert(cpu_values, e.metrics.cpu_usage)
                        end
                        return cpu_values
                    end)())),
                    min_response_time = math.min(table.unpack((function()
                        local response_values = {}
                        for _, e in ipairs(aggregation_buffer.events) do
                            table.insert(response_values, e.metrics.response_time)
                        end
                        return response_values
                    end)()))
                },
                sources = (function()
                    local sources = {}
                    for _, event in ipairs(aggregation_buffer.events) do
                        sources[event.source_hook] = true
                    end
                    local source_list = {}
                    for source in pairs(sources) do
                        table.insert(source_list, source)
                    end
                    return source_list
                end)()
            })
            
            print(string.format("   📊 Aggregated %d events - Avg CPU: %.1f%%, Avg Memory: %.1f%%", 
                  count, total_cpu / count, total_memory / count))
            
            -- Clear buffer
            aggregation_buffer.events = {}
        end
        
        return "continue"
    end, "low")
end

print("   ✅ Event aggregation pattern registered (3 sources)")
integration_stats.hooks_registered = integration_stats.hooks_registered + 3

print()
print("8. Processing hook-event integration patterns:")

print("   📥 Processing integration events:")

-- Process events from all integration patterns
local integration_events_processed = 0

for pattern_name, sub_id in pairs(subscriptions) do
    local pattern_events = 0
    print(string.format("   🔍 Processing %s events:", pattern_name))
    
    -- Receive events for this pattern
    for attempt = 1, 5 do
        local received = Event.receive(sub_id, 200) -- 200ms timeout
        if received then
            pattern_events = pattern_events + 1
            integration_events_processed = integration_events_processed + 1
            
            integration_stats.integration_patterns[pattern_name] = 
                integration_stats.integration_patterns[pattern_name] + 1
            
            -- Analyze event content
            local event_type = received.event_type or "unknown"
            local source_info = ""
            
            if received.data then
                if received.data.hook_source then
                    source_info = " (from " .. received.data.hook_source .. ")"
                elseif received.data.source_hook then
                    source_info = " (from " .. received.data.source_hook .. ")"
                end
            end
            
            print(string.format("     %d. %s%s", pattern_events, event_type, source_info))
        else
            break -- No more events for this pattern
        end
    end
    
    if pattern_events > 0 then
        print(string.format("   📊 %s: processed %d events", pattern_name, pattern_events))
    else
        print(string.format("   ⏰ %s: no events received", pattern_name))
    end
end

print(string.format("   ✅ Total integration events processed: %d", integration_events_processed))

print()
print("9. Integration performance analysis:")

print("   ⚡ Integration Performance Analysis:")

-- Calculate integration statistics
local hook_to_event_ratio = integration_stats.events_from_hooks > 0 and
                           integration_stats.events_to_hooks / integration_stats.events_from_hooks or 0

print(string.format("   • Hooks registered: %d", integration_stats.hooks_registered))
print(string.format("   • Events from hooks: %d", integration_stats.events_from_hooks))
print(string.format("   • Events to hooks: %d", integration_stats.events_to_hooks))
print(string.format("   • Hook-event cycles: %d", integration_stats.hook_event_cycles))
print(string.format("   • Event-to-hook ratio: %.2f", hook_to_event_ratio))

-- Pattern usage analysis
print("   📈 Integration Pattern Usage:")
for pattern_name, event_count in pairs(integration_stats.integration_patterns) do
    if event_count > 0 then
        print(string.format("   • %s: %d events", pattern_name, event_count))
    end
end

-- Integration efficiency
local total_integration_events = integration_stats.events_from_hooks + integration_stats.events_to_hooks
local integration_efficiency = integration_stats.hooks_registered > 0 and
                              total_integration_events / integration_stats.hooks_registered or 0

print(string.format("   • Integration efficiency: %.2f events per hook", integration_efficiency))

print()
print("10. Advanced integration scenarios:")

print("   🎛️  Advanced Integration Scenarios:")

-- Self-modifying hook system
hook_handles.self_modifier = Hook.register("BeforeAgentInit", function(context)
    track_integration_event("HOOK_TO_EVENT", "self_modifier", "hook.system.self_modify")
    
    -- Publish event that might trigger hook modifications
    Event.publish("hook.system.self_modify", {
        modifier_hook = "self_modifier",
        agent_context = context.component_id.name,
        modification_request = {
            action = "optimize_priority",
            current_priority = "normal",
            suggested_priority = "high",
            reason = "performance_improvement"
        },
        timestamp = os.time()
    })
    
    print("   🔧 Self-modifying hook published optimization request")
    
    return "continue"
end, "normal")

-- Recursive event-hook pattern
hook_handles.recursive_pattern = Hook.register("BeforeToolExecution", function(context)
    local recursion_depth = context.metadata and context.metadata.recursion_depth or 0
    
    if recursion_depth < 3 then -- Prevent infinite recursion
        track_integration_event("HOOK_TO_EVENT", "recursive_pattern", "hook.recursion.level_" .. (recursion_depth + 1))
        
        Event.publish("hook.recursion.level_" .. (recursion_depth + 1), {
            recursion_depth = recursion_depth + 1,
            original_tool = context.component_id.name,
            recursion_data = {
                level = recursion_depth + 1,
                max_depth = 3,
                pattern_type = "recursive_hook_event"
            },
            timestamp = os.time()
        })
        
        print(string.format("   🔄 Recursive pattern: depth %d", recursion_depth + 1))
        integration_stats.hook_event_cycles = integration_stats.hook_event_cycles + 1
    else
        print("   🛑 Recursive pattern: max depth reached")
    end
    
    return "continue"
end, "low")

print("   ✅ Advanced integration scenarios registered")
integration_stats.hooks_registered = integration_stats.hooks_registered + 2

print()
print("11. Integration monitoring and health checks:")

print("   🏥 Integration Health Monitoring:")

-- Health check for hook-event integration
local health_metrics = {
    hooks_responsive = 0,
    events_flowing = 0,
    integration_cycles_healthy = 0,
    total_hooks = integration_stats.hooks_registered
}

-- Check hook responsiveness (simulate health check)
for hook_name, handle in pairs(hook_handles) do
    if handle and handle:id() then
        health_metrics.hooks_responsive = health_metrics.hooks_responsive + 1
    end
end

-- Check event flow health
health_metrics.events_flowing = integration_events_processed

-- Check integration cycle health
health_metrics.integration_cycles_healthy = integration_stats.hook_event_cycles

print(string.format("   • Hook responsiveness: %d/%d (%.1f%%)", 
      health_metrics.hooks_responsive, health_metrics.total_hooks,
      (health_metrics.hooks_responsive / health_metrics.total_hooks) * 100))

print(string.format("   • Event flow health: %d events processed", health_metrics.events_flowing))
print(string.format("   • Integration cycles: %d completed", health_metrics.integration_cycles_healthy))

-- Overall health score
local health_score = ((health_metrics.hooks_responsive / health_metrics.total_hooks) * 0.4 +
                     (math.min(health_metrics.events_flowing / 10, 1)) * 0.3 +
                     (math.min(health_metrics.integration_cycles_healthy / 5, 1)) * 0.3) * 100

print(string.format("   • Overall integration health: %.1f/100", health_score))

if health_score >= 80 then
    print("   ✅ Integration system is healthy")
elseif health_score >= 60 then
    print("   ⚠️  Integration system has minor issues")
else
    print("   ❌ Integration system needs attention")
end

print()
print("12. Integration best practices:")

print("   💡 Hook-Event Integration Best Practices:")
print("   • Design hooks and events with loose coupling")
print("   • Use correlation IDs for tracking hook-event relationships")
print("   • Implement proper error handling in bidirectional communication")
print("   • Monitor integration performance and health metrics")
print("   • Avoid infinite loops in recursive hook-event patterns")
print("   • Use event aggregation for high-frequency hook data")
print("   • Implement proper cleanup for dynamic hook registration")
print("   • Design event schemas that support both hook and external sources")
print("   • Use appropriate event patterns for different integration scenarios")
print("   • Test integration patterns thoroughly under load")

print()
print("13. Cleaning up integration infrastructure:")

-- Cleanup hooks
local hook_cleanup_count = 0
for hook_name, handle in pairs(hook_handles) do
    if handle and handle:id() then
        Hook.unregister(handle)
        hook_cleanup_count = hook_cleanup_count + 1
        print(string.format("   🧹 Unregistered hook: %s", hook_name))
    end
end

-- Cleanup subscriptions
local subscription_cleanup_count = 0
for pattern_name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        subscription_cleanup_count = subscription_cleanup_count + 1
        print(string.format("   🧹 Unsubscribed from: %s", pattern_name))
    end
end

local final_hooks = #Hook.list()
local final_subs = Event.list_subscriptions()

print(string.format("   ✅ Cleaned up %d hooks", hook_cleanup_count))
print(string.format("   ✅ Cleaned up %d subscriptions", subscription_cleanup_count))
print(string.format("   ✅ Final hook count: %d", final_hooks))
print(string.format("   ✅ Final subscription count: %d", #final_subs))

print()
print("14. Final integration statistics:")

print("   📊 Final Integration Statistics:")
print(string.format("   • Total hooks registered: %d", integration_stats.hooks_registered))
print(string.format("   • Events from hooks: %d", integration_stats.events_from_hooks))
print(string.format("   • Events to hooks: %d", integration_stats.events_to_hooks))
print(string.format("   • Hook-event cycles completed: %d", integration_stats.hook_event_cycles))
print(string.format("   • Integration patterns tested: %d", (function()
    local count = 0
    for _ in pairs(integration_stats.integration_patterns) do count = count + 1 end
    return count
end)()))
print(string.format("   • Integration health score: %.1f/100", health_score))

print()
print("✨ Event hook integration example complete!")
print("   Key concepts demonstrated:")
print("   • Hooks that publish events for external notification")
print("   • Events that trigger hook registration and execution")
print("   • Bidirectional hook-event communication patterns")
print("   • Event-driven hook chaining and coordination")
print("   • Event aggregation from multiple hook sources")
print("   • Advanced integration scenarios (self-modifying, recursive)")
print("   • Integration performance monitoring and health checks")
print("   • Dynamic hook management based on event triggers")
print("   • Best practices for robust hook-event integration")
print("   • Comprehensive cleanup and resource management")