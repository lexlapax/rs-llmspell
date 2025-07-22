-- ABOUTME: Helper functions for workflow examples
-- ABOUTME: Provides utilities for handling async workflow execution

-- Helper to execute async workflow methods synchronously
function executeWorkflow(workflow, input)
    local result = nil
    local error = nil
    
    -- Create coroutine for async execution
    local co = coroutine.create(function()
        return workflow:execute(input)
    end)
    
    -- Resume until complete
    local ok, value = coroutine.resume(co)
    
    -- Handle async operations that yield
    while ok and coroutine.status(co) ~= "dead" do
        ok, value = coroutine.resume(co, value)
    end
    
    if ok then
        result = value
    else
        error = value
    end
    
    return result, error
end

-- Helper to list workflows synchronously
function listWorkflows()
    local co = coroutine.create(function()
        return Workflow.list()
    end)
    
    local ok, value = coroutine.resume(co)
    while ok and coroutine.status(co) ~= "dead" do
        ok, value = coroutine.resume(co, value)
    end
    
    return ok and value or {}
end

-- Helper to get workflow by ID
function getWorkflow(id)
    -- Workflow.get() is synchronous
    return Workflow.get(id)
end

-- Export helpers
return {
    executeWorkflow = executeWorkflow,
    listWorkflows = listWorkflows,
    getWorkflow = getWorkflow
}