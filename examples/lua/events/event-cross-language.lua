-- ABOUTME: Cross-language event communication between Lua, Rust, JavaScript, and Python
-- ABOUTME: Demonstrates language-specific event publishing, inter-language messaging, and coordination

print("=== Cross-Language Event Communication Example ===")
print("Demonstrates: Event-driven communication across language boundaries")
print()

local subscriptions = {}
local cross_lang_stats = {
    messages_sent = 0,
    messages_received = 0,
    languages_coordinated = {"lua"},
    coordination_patterns = {}
}

print("1. Setting up cross-language event subscriptions:")

-- Subscribe to events from different languages
local language_subscriptions = {
    rust_events = "rust.*",
    javascript_events = "javascript.*", 
    python_events = "python.*",
    native_events = "native.*",
    
    -- Cross-language coordination channels
    coordination_events = "coordination.*",
    broadcast_events = "broadcast.*",
    response_events = "response.*",
    
    -- Language-specific response channels
    responses_to_lua = "response.to.lua.*",
    requests_from_any = "request.*",
    
    -- Inter-service communication
    service_discovery = "service.*",
    health_checks = "health.*"
}

print("   📡 Creating cross-language subscriptions:")
for channel_name, pattern in pairs(language_subscriptions) do
    subscriptions[channel_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", channel_name, pattern))
end

print("   ✅ Cross-language event channels established")

print()
print("2. Publishing events to simulate other languages:")

-- Simulate events as if coming from different language components
print("   🦀 Simulating Rust component events:")

local rust_events = {
    {
        name = "rust.core.startup", 
        data = {
            component = "core_engine",
            version = "1.0.0",
            capabilities = {"high_performance", "memory_safe", "concurrent"},
            rust_specific = {
                memory_usage_mb = 45,
                thread_count = 4,
                compiled_at = "2024-01-15T10:30:00Z"
            }
        },
        options = {language = "rust"}
    },
    {
        name = "rust.tool.execution", 
        data = {
            tool_name = "filesystem_scanner",
            execution_time_us = 1250, -- microseconds for Rust precision
            result_count = 1547,
            memory_allocated_bytes = 8192
        },
        options = {language = "rust"}
    },
    {
        name = "rust.performance.metric",
        data = {
            metric_type = "throughput",
            operations_per_second = 95000,
            cpu_utilization = 23.4,
            memory_efficiency = 0.98
        },
        options = {language = "rust"}
    }
}

for i, event in ipairs(rust_events) do
    local published = Event.publish(event.name, event.data, event.options)
    if published then
        cross_lang_stats.messages_sent = cross_lang_stats.messages_sent + 1
        print(string.format("   %d. ✅ %s", i, event.name))
    end
end

print("   🌐 Simulating JavaScript component events:")

local javascript_events = {
    {
        name = "javascript.ui.interaction",
        data = {
            event_type = "click",
            element_id = "submit_button",
            user_id = "user_123",
            timestamp = os.time() * 1000, -- JavaScript uses milliseconds
            browser_info = {
                user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
                viewport = {width = 1920, height = 1080},
                cookies_enabled = true
            }
        },
        options = {language = "javascript"}
    },
    {
        name = "javascript.api.request",
        data = {
            method = "POST",
            endpoint = "/api/v1/users",
            status_code = 201,
            response_time_ms = 245,
            payload_size_bytes = 1024
        },
        options = {language = "javascript"}
    },
    {
        name = "javascript.state.update",
        data = {
            component = "UserDashboard",
            state_change = {
                from = {user_count = 150, active_sessions = 23},
                to = {user_count = 151, active_sessions = 24}
            },
            react_version = "18.2.0"
        },
        options = {language = "javascript"}
    }
}

for i, event in ipairs(javascript_events) do
    local published = Event.publish(event.name, event.data, event.options)
    if published then
        cross_lang_stats.messages_sent = cross_lang_stats.messages_sent + 1
        print(string.format("   %d. ✅ %s", i, event.name))
    end
end

print("   🐍 Simulating Python component events:")

local python_events = {
    {
        name = "python.ml.training",
        data = {
            model_type = "neural_network",
            dataset_size = 50000,
            epochs_completed = 25,
            accuracy = 0.94,
            loss = 0.12,
            training_time_seconds = 3600,
            python_version = "3.11.2",
            libraries = {"tensorflow", "numpy", "pandas"}
        },
        options = {language = "python"}
    },
    {
        name = "python.data.analysis",
        data = {
            analysis_type = "statistical_summary",
            records_processed = 1000000,
            results = {
                mean = 45.7,
                median = 42.1,
                std_dev = 12.3,
                min = 0.1,
                max = 99.9
            },
            processing_time_ms = 5432
        },
        options = {language = "python"}
    },
    {
        name = "python.automation.task",
        data = {
            task_name = "data_pipeline",
            status = "completed",
            files_processed = 247,
            total_size_mb = 1536,
            script_path = "/scripts/automation/data_pipeline.py"
        },
        options = {language = "python"}
    }
}

for i, event in ipairs(python_events) do
    local published = Event.publish(event.name, event.data, event.options)
    if published then
        cross_lang_stats.messages_sent = cross_lang_stats.messages_sent + 1
        print(string.format("   %d. ✅ %s", i, event.name))
    end
end

print("   📊 Total cross-language events published:", cross_lang_stats.messages_sent)

print()
print("3. Publishing Lua coordination events:")

-- Lua announces its capabilities and coordinates with other languages
print("   📢 Lua announcing capabilities to other languages:")

local lua_announcements = {
    {
        name = "coordination.language.announce",
        data = {
            language = "lua",
            component_type = "scripting_engine",
            capabilities = {
                "dynamic_scripting",
                "hook_system",
                "event_processing",
                "rapid_prototyping"
            },
            version = "5.4",
            memory_usage_kb = 2048,
            available_hooks = 45,
            active_subscriptions = #subscriptions
        }
    },
    {
        name = "coordination.service.register",
        data = {
            service_name = "lua_event_processor",
            service_type = "event_handler",
            endpoints = {
                health_check = "lua.health",
                process_event = "lua.process",
                get_stats = "lua.stats"
            },
            status = "active",
            registered_at = os.time()
        }
    },
    {
        name = "broadcast.system.status",
        data = {
            from_language = "lua",
            message_type = "system_status",
            system_health = {
                status = "operational",
                uptime_seconds = 3600,
                memory_usage = "normal",
                event_processing_rate = "high"
            },
            broadcast_timestamp = os.time()
        }
    }
}

for i, event in ipairs(lua_announcements) do
    local published = Event.publish(event.name, event.data)
    if published then
        cross_lang_stats.messages_sent = cross_lang_stats.messages_sent + 1
        print(string.format("   %d. ✅ %s", i, event.name))
    end
end

print()
print("4. Receiving and processing cross-language events:")

-- Process events from other languages
print("   📥 Processing cross-language messages:")

local message_processors = {
    rust_events = function(event)
        print("   🦀 Processing Rust event:")
        if event.data then
            if event.data.component then
                print("     • Component:", event.data.component)
            end
            if event.data.execution_time_us then
                print(string.format("     • Execution time: %.2fms (from %dμs)", 
                      event.data.execution_time_us / 1000, event.data.execution_time_us))
            end
            if event.data.operations_per_second then
                print(string.format("     • Performance: %d ops/sec", event.data.operations_per_second))
            end
        end
        
        -- Respond to Rust component
        Event.publish("response.to.rust.component", {
            lua_response = "acknowledged",
            processed_at = os.time(),
            response_type = "cross_language_ack"
        })
        
        return true
    end,
    
    javascript_events = function(event)
        print("   🌐 Processing JavaScript event:")
        if event.data then
            if event.data.event_type and event.data.element_id then
                print("     • UI Event:", event.data.event_type, "on", event.data.element_id)
            end
            if event.data.method and event.data.endpoint then
                print("     • API Call:", event.data.method, event.data.endpoint)
            end
            if event.data.component then
                print("     • React Component:", event.data.component)
            end
        end
        
        -- Respond to JavaScript component
        Event.publish("response.to.javascript.ui", {
            lua_processing = "complete",
            ui_state_synchronized = true,
            processed_at = os.time()
        })
        
        return true
    end,
    
    python_events = function(event)
        print("   🐍 Processing Python event:")
        if event.data then
            if event.data.model_type then
                print("     • ML Model:", event.data.model_type, 
                      string.format("(%.2f accuracy)", event.data.accuracy or 0))
            end
            if event.data.analysis_type then
                print("     • Data Analysis:", event.data.analysis_type)
                if event.data.results then
                    print(string.format("       Mean: %.1f, Std Dev: %.1f", 
                          event.data.results.mean or 0, event.data.results.std_dev or 0))
                end
            end
            if event.data.task_name then
                print("     • Automation Task:", event.data.task_name, "(" .. (event.data.status or "unknown") .. ")")
            end
        end
        
        -- Respond to Python component
        Event.publish("response.to.python.processor", {
            lua_analysis = "received",
            data_validated = true,
            processed_at = os.time()
        })
        
        return true
    end
}

-- Process events from each language
local processed_events = 0
for subscription_name, processor in pairs(message_processors) do
    local sub_id = subscriptions[subscription_name]
    if sub_id then
        -- Try to receive multiple events from this language
        for attempt = 1, 3 do
            local received = Event.receive(sub_id, 500) -- 500ms timeout
            if received then
                local success = processor(received)
                if success then
                    processed_events = processed_events + 1
                    cross_lang_stats.messages_received = cross_lang_stats.messages_received + 1
                end
            else
                break -- No more events from this language
            end
        end
    end
end

print("   📊 Processed", processed_events, "cross-language events")

print()
print("5. Cross-language request-response patterns:")

-- Demonstrate request-response patterns across languages
print("   💬 Implementing request-response patterns:")

-- Lua sends requests to other languages
local cross_lang_requests = {
    {
        name = "request.rust.performance_stats",
        data = {
            requesting_language = "lua",
            request_type = "performance_metrics",
            requested_metrics = {"cpu_usage", "memory_usage", "throughput"},
            request_id = "req_" .. os.time(),
            timeout_seconds = 30
        }
    },
    {
        name = "request.javascript.ui_state",
        data = {
            requesting_language = "lua",
            request_type = "ui_state_snapshot",
            components = {"UserDashboard", "NavigationBar", "Footer"},
            request_id = "req_ui_" .. os.time()
        }
    },
    {
        name = "request.python.data_summary",
        data = {
            requesting_language = "lua",
            request_type = "statistical_analysis",
            dataset = "user_interactions",
            analysis_depth = "full",
            request_id = "req_data_" .. os.time()
        }
    }
}

print("   📤 Sending cross-language requests:")
for i, request in ipairs(cross_lang_requests) do
    local published = Event.publish(request.name, request.data)
    if published then
        print(string.format("   %d. ✅ %s", i, request.name))
        cross_lang_stats.messages_sent = cross_lang_stats.messages_sent + 1
    end
end

-- Try to receive responses (simulate other languages responding)
print("   📥 Checking for cross-language responses:")

-- Simulate responses from other languages
local simulated_responses = {
    {
        name = "response.to.lua.performance_stats",
        data = {
            responding_language = "rust",
            request_id = "req_" .. (os.time() - 1),
            performance_metrics = {
                cpu_usage = 15.2,
                memory_usage_mb = 128,
                throughput_ops_sec = 87500
            },
            response_timestamp = os.time()
        }
    },
    {
        name = "response.to.lua.ui_state",
        data = {
            responding_language = "javascript",
            ui_components = {
                UserDashboard = {active_users = 45, notifications = 3},
                NavigationBar = {current_page = "dashboard"},
                Footer = {status = "loaded"}
            },
            response_timestamp = os.time()
        }
    }
}

-- Publish simulated responses
for _, response in ipairs(simulated_responses) do
    Event.publish(response.name, response.data)
end

-- Try to receive the responses
local responses_received = 0
for attempt = 1, 5 do
    local response = Event.receive(subscriptions.responses_to_lua, 300)
    if response then
        responses_received = responses_received + 1
        print(string.format("   📨 Response #%d received:", responses_received))
        if response.data and response.data.responding_language then
            print("     • From:", response.data.responding_language)
        end
        if response.data and response.data.request_id then
            print("     • Request ID:", response.data.request_id)
        end
    else
        break
    end
end

print("   📊 Responses received:", responses_received)

print()
print("6. Cross-language coordination patterns:")

-- Demonstrate advanced coordination patterns
print("   🤝 Advanced cross-language coordination:")

local coordination_patterns = {
    {
        name = "coordination.workflow.distributed",
        data = {
            workflow_id = "wf_cross_lang_" .. os.time(),
            pattern_type = "distributed_processing",
            participating_languages = {"lua", "rust", "python", "javascript"},
            coordination_steps = {
                {step = 1, language = "lua", action = "initiate_workflow"},
                {step = 2, language = "rust", action = "high_performance_processing"},
                {step = 3, language = "python", action = "data_analysis"},
                {step = 4, language = "javascript", action = "ui_visualization"},
                {step = 5, language = "lua", action = "finalize_results"}
            },
            coordinator = "lua"
        }
    },
    {
        name = "coordination.health.distributed_check",
        data = {
            check_id = "health_" .. os.time(),
            initiating_language = "lua",
            health_check_targets = {
                "rust.core.engine",
                "javascript.ui.frontend", 
                "python.ml.backend"
            },
            check_timeout_seconds = 10,
            aggregation_required = true
        }
    },
    {
        name = "coordination.resource.shared_pool",
        data = {
            resource_type = "database_connections",
            pool_manager = "lua",
            available_connections = 50,
            requesting_languages = {"rust", "python"},
            allocation_strategy = "fair_share",
            expires_at = os.time() + 3600
        }
    }
}

print("   📤 Publishing coordination patterns:")
for i, pattern in ipairs(coordination_patterns) do
    local published = Event.publish(pattern.name, pattern.data)
    if published then
        print(string.format("   %d. ✅ %s", i, pattern.name))
        cross_lang_stats.coordination_patterns[pattern.data.pattern_type or "unknown"] = true
    end
end

print()
print("7. Cross-language event statistics:")

print("   📊 Cross-Language Communication Statistics:")
print("   • Messages sent by Lua:", cross_lang_stats.messages_sent)
print("   • Messages received by Lua:", cross_lang_stats.messages_received)
print("   • Languages involved:", table.concat(cross_lang_stats.languages_coordinated, ", "))
print("   • Active subscriptions:", #subscriptions)

print()
print("   🌐 Coordination Patterns Demonstrated:")
for pattern_type, _ in pairs(cross_lang_stats.coordination_patterns) do
    print("   • " .. pattern_type:gsub("_", " "))
end

print()
print("8. Cross-language best practices:")

print("   💡 Cross-Language Event Best Practices:")
print("   • Use language prefixes for event organization (rust.*, javascript.*)")
print("   • Include language metadata in event options")
print("   • Implement request-response patterns for synchronous communication")
print("   • Use coordination events for distributed workflow management")
print("   • Include version information in cross-language messages")
print("   • Handle language-specific data types and formats")
print("   • Implement health check patterns for distributed systems")
print("   • Use correlation IDs for request tracing across languages")
print("   • Design graceful degradation when languages are unavailable")

print()
print("9. Service discovery and health monitoring:")

-- Simulate service discovery across languages
print("   🔍 Cross-language service discovery:")

local service_discovery_events = {
    {name = "service.discover.request", data = {from = "lua", seeking = "data_processors"}},
    {name = "service.register.rust_engine", data = {language = "rust", capabilities = ["high_performance"]}},
    {name = "service.register.python_ml", data = {language = "python", capabilities = ["machine_learning"]}},
    {name = "health.check.broadcast", data = {from = "lua", timestamp = os.time()}}
}

for _, event in ipairs(service_discovery_events) do
    Event.publish(event.name, event.data)
end

print("   ✅ Service discovery events published")

print()
print("10. Cleaning up cross-language subscriptions:")

-- Clean up all cross-language subscriptions
local cleanup_count = 0
for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name)
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "cross-language subscriptions")
print("   ✅ Final subscription count:", #final_subs)

-- Final coordination message
Event.publish("coordination.language.shutdown", {
    language = "lua",
    shutdown_type = "graceful",
    final_message = "Cross-language example completed successfully",
    statistics = cross_lang_stats,
    shutdown_timestamp = os.time()
})

print()
print("✨ Cross-language event communication example complete!")
print("   Key concepts demonstrated:")
print("   • Language-specific event channels (rust.*, javascript.*, python.*)")
print("   • Cross-language message publishing with language metadata")
print("   • Request-response patterns across language boundaries")
print("   • Distributed workflow coordination via events")
print("   • Service discovery and health monitoring patterns")
print("   • Language capability announcement and coordination")
print("   • Graceful cross-language communication handling")