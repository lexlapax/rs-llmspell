-- ABOUTME: Helper functions for agent examples
-- ABOUTME: Provides utilities for handling async agent operations

-- Helper to create agents using the synchronous wrapper
function createAgent(config)
    -- Agent.createAsync is a synchronous wrapper provided by the API
    local success, agent = pcall(function()
        return Agent.createAsync(config)
    end)
    
    if success then
        return agent, nil
    else
        return nil, tostring(agent)
    end
end

-- Helper to register agents with full configuration
function registerAgent(config)
    -- Ensure all required fields are present
    if not config.agent_type then
        config.agent_type = "llm"
    end
    
    -- Convert simple model string to full model config
    if type(config.model) == "string" then
        local provider, model_id = config.model:match("^([^/]+)/(.+)$")
        if not provider then
            provider = "openai"
            model_id = config.model
        end
        config.model = {
            provider = provider,
            model_id = model_id,
            temperature = config.temperature or 0.7,
            max_tokens = config.max_tokens or 200,
            settings = config.model_settings or {}
        }
    end
    
    -- Ensure other required fields
    if not config.allowed_tools then
        config.allowed_tools = {}
    end
    if not config.resource_limits then
        config.resource_limits = {
            max_execution_time_secs = 60,
            max_memory_mb = 256,
            max_tool_calls = 10,
            max_recursion_depth = 5
        }
    end
    if not config.custom_config and config.system_prompt then
        config.custom_config = {
            system_prompt = config.system_prompt
        }
    end
    
    local success, result = pcall(function()
        return Agent.register(config)
    end)
    
    if success then
        return result, nil
    else
        return nil, tostring(result)
    end
end

-- Helper to invoke agents asynchronously
function invokeAgent(agent, input)
    -- Create coroutine for async execution
    local co = coroutine.create(function()
        return agent:invoke(input)
    end)
    
    -- Resume until complete
    local success, result = coroutine.resume(co)
    
    -- Handle async operations that yield
    while success and coroutine.status(co) ~= "dead" do
        success, result = coroutine.resume(co, result)
    end
    
    if success then
        return result, nil
    else
        return nil, tostring(result)
    end
end

-- Helper to invoke agent streaming
function invokeAgentStream(agent, input, callback)
    -- Create coroutine for async streaming
    local co = coroutine.create(function()
        return agent:invokeStream(input, callback)
    end)
    
    -- Resume until complete
    local success, result = coroutine.resume(co)
    
    while success and coroutine.status(co) ~= "dead" do
        success, result = coroutine.resume(co, result)
    end
    
    return success, result
end

-- Export helpers
return {
    createAgent = createAgent,
    registerAgent = registerAgent,
    invokeAgent = invokeAgent,
    invokeAgentStream = invokeAgentStream
}