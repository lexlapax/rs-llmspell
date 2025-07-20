-- ABOUTME: Practical patterns and best practices for using globals
-- ABOUTME: Real-world examples of error handling, caching, and composition

print("=== Practical Global Usage Patterns ===\n")

-- Pattern 1: Safe Tool Execution with Error Handling
local function safe_tool_execute(tool_name, parameters)
    local tool = Tool.get(tool_name)
    if not tool then
        Logger.error("Tool not found", {tool = tool_name})
        return nil, "Tool not found: " .. tool_name
    end
    
    local success, result = pcall(function()
        return tool:execute(parameters)
    end)
    
    if success then
        Logger.debug("Tool executed successfully", {
            tool = tool_name,
            duration_ms = result.metadata and result.metadata.duration_ms
        })
        return result, nil
    else
        Logger.error("Tool execution failed", {
            tool = tool_name,
            error = tostring(result)
        })
        return nil, tostring(result)
    end
end

-- Example usage
local result, err = safe_tool_execute("calculator", {
    operation = "divide",
    a = 10,
    b = 0  -- This might cause an error
})

if err then
    print("Error:", err)
else
    print("Result:", result.result)
end

-- Pattern 2: Caching Expensive Operations
local function cached_agent_execute(agent_id, prompt)
    -- Create cache key
    local cache_key = "agent_cache:" .. agent_id .. ":" .. Utils.hash(prompt)
    
    -- Check cache first
    local cached = State.get(cache_key)
    if cached and cached.timestamp > os.time() - 300 then  -- 5 minute cache
        Logger.debug("Cache hit", {agent_id = agent_id})
        return cached.response
    end
    
    -- Execute agent
    local agent = Agent.get(agent_id)
    if not agent then
        return nil, "Agent not found"
    end
    
    local response = agent:execute({prompt = prompt})
    
    -- Cache the response
    State.set(cache_key, {
        response = response,
        timestamp = os.time()
    })
    
    return response
end

-- Pattern 3: Workflow Factory Pattern
local WorkflowFactory = {
    -- Create a data processing pipeline
    create_data_pipeline = function(source_url, transformations)
        local steps = {
            -- Step 1: Fetch data
            {
                name = "fetch_data",
                tool = "web_scraper",
                parameters = {url = source_url}
            }
        }
        
        -- Add transformation steps
        for i, transform in ipairs(transformations) do
            table.insert(steps, {
                name = "transform_" .. i,
                tool = transform.tool,
                parameters = transform.params or {},
                input_ref = "$" .. steps[#steps].name .. ".output"
            })
        end
        
        -- Final step: Store results
        table.insert(steps, {
            name = "store_results",
            tool = "state_manager",
            parameters = {
                key = "pipeline_results",
                value = "$" .. steps[#steps].name .. ".output"
            }
        })
        
        return Workflow.sequential({
            name = "data_pipeline_" .. Utils.uuid(),
            description = "Automated data pipeline",
            steps = steps
        })
    end,
    
    -- Create a multi-agent decision workflow
    create_decision_workflow = function(agents, decision_criteria)
        local branches = {}
        
        -- Create a branch for each agent
        for _, agent_config in ipairs(agents) do
            table.insert(branches, {
                name = agent_config.name .. "_analysis",
                tool = "agent_executor",
                parameters = {
                    agent_id = agent_config.id,
                    prompt = decision_criteria
                }
            })
        end
        
        -- Add aggregation step
        local workflow = Workflow.parallel({
            name = "multi_agent_decision",
            branches = branches,
            merge_strategy = "all",  -- Collect all responses
            post_process = {
                tool = "decision_aggregator",
                parameters = {
                    strategy = "majority_vote"
                }
            }
        })
        
        return workflow
    end
}

-- Pattern 4: State-Driven Workflow Execution
local function state_driven_workflow()
    -- Initialize workflow state
    State.set("workflow:status", "initializing")
    State.set("workflow:steps_completed", 0)
    State.set("workflow:errors", {})
    
    -- Define workflow with state updates
    local workflow = Workflow.sequential({
        name = "state_aware_workflow",
        steps = {
            {
                name = "init",
                tool = "logger",
                parameters = {message = "Starting workflow"},
                on_complete = function(result)
                    State.set("workflow:status", "processing")
                    local count = State.get("workflow:steps_completed") or 0
                    State.set("workflow:steps_completed", count + 1)
                end
            },
            {
                name = "process",
                tool = "data_processor",
                parameters = {
                    data = State.get("input_data") or {}
                },
                on_error = function(error)
                    local errors = State.get("workflow:errors") or {}
                    table.insert(errors, {
                        step = "process",
                        error = error,
                        timestamp = os.time()
                    })
                    State.set("workflow:errors", errors)
                end
            },
            {
                name = "complete",
                tool = "logger",
                parameters = {message = "Workflow completed"},
                on_complete = function(result)
                    State.set("workflow:status", "completed")
                    State.set("workflow:completed_at", os.time())
                end
            }
        }
    })
    
    return workflow
end

-- Pattern 5: Dynamic Tool Selection Based on Context
local function context_aware_tool_selection(task_type, data)
    local tool_mapping = {
        text = {
            small = "text_processor",
            large = "streaming_text_processor"
        },
        image = {
            small = "image_processor", 
            large = "batch_image_processor"
        },
        data = {
            structured = "json_processor",
            unstructured = "ml_data_processor"
        }
    }
    
    -- Determine data size
    local data_size = "small"
    if type(data) == "string" and #data > 10000 then
        data_size = "large"
    elseif type(data) == "table" and #JSON.stringify(data) > 10000 then
        data_size = "large"
    end
    
    -- Select appropriate tool
    local tool_category = tool_mapping[task_type]
    if not tool_category then
        Logger.warn("Unknown task type", {type = task_type})
        return nil
    end
    
    local tool_name = tool_category[data_size] or tool_category.small
    return Tool.get(tool_name)
end

-- Pattern 6: Graceful Degradation with Fallback Tools
local function execute_with_fallback(primary_tool, fallback_tools, parameters)
    -- Try primary tool first
    local result, err = safe_tool_execute(primary_tool, parameters)
    if result then
        return result
    end
    
    Logger.warn("Primary tool failed, trying fallbacks", {
        primary = primary_tool,
        error = err
    })
    
    -- Try each fallback in order
    for _, fallback in ipairs(fallback_tools) do
        result, err = safe_tool_execute(fallback, parameters)
        if result then
            Logger.info("Fallback succeeded", {tool = fallback})
            return result
        end
    end
    
    return nil, "All tools failed"
end

-- Pattern 7: Workflow Composition and Reuse
local ComposableWorkflows = {
    -- Store workflow templates
    templates = {},
    
    -- Register a reusable workflow
    register = function(self, name, workflow_fn)
        self.templates[name] = workflow_fn
        State.set("workflow_templates:" .. name, {
            registered_at = os.time(),
            usage_count = 0
        })
    end,
    
    -- Create workflow from template
    create = function(self, template_name, params)
        local template = self.templates[template_name]
        if not template then
            error("Unknown workflow template: " .. template_name)
        end
        
        -- Update usage count
        local meta = State.get("workflow_templates:" .. template_name)
        meta.usage_count = meta.usage_count + 1
        State.set("workflow_templates:" .. template_name, meta)
        
        return template(params)
    end,
    
    -- Compose multiple workflows
    compose = function(self, workflow_names)
        local steps = {}
        
        for _, name in ipairs(workflow_names) do
            table.insert(steps, {
                name = name .. "_step",
                workflow = self.templates[name]()
            })
        end
        
        return Workflow.sequential({
            name = "composed_workflow",
            steps = steps
        })
    end
}

-- Register some templates
ComposableWorkflows:register("validate_data", function(params)
    return Workflow.sequential({
        name = "data_validation",
        steps = {
            {name = "check_format", tool = "validator", parameters = params}
        }
    })
end)

-- Pattern 8: Performance Monitoring Wrapper
local function monitored_execution(name, fn, ...)
    local start_time = os.time()
    local start_memory = collectgarbage("count")
    
    -- Execute function
    local results = {pcall(fn, ...)}
    
    -- Collect metrics
    local duration = os.time() - start_time
    local memory_used = collectgarbage("count") - start_memory
    
    -- Store metrics
    local metrics = State.get("performance_metrics") or {}
    table.insert(metrics, {
        name = name,
        duration_seconds = duration,
        memory_kb = memory_used,
        success = results[1],
        timestamp = os.time()
    })
    State.set("performance_metrics", metrics)
    
    -- Return results
    if results[1] then
        return table.unpack(results, 2)
    else
        error(results[2])
    end
end

-- Example: Monitor tool execution
local function monitored_tool_execute(tool_name, params)
    return monitored_execution(
        "tool:" .. tool_name,
        safe_tool_execute,
        tool_name,
        params
    )
end

print("\n=== Pattern Examples Complete ===")
print("Demonstrated patterns:")
print("1. Safe execution with error handling")
print("2. Caching expensive operations")
print("3. Workflow factory patterns")
print("4. State-driven workflows")
print("5. Dynamic tool selection")
print("6. Graceful degradation")
print("7. Workflow composition")
print("8. Performance monitoring")

-- Clean up state
State.delete("workflow:status")
State.delete("workflow:steps_completed")
State.delete("workflow:errors")
State.delete("performance_metrics")