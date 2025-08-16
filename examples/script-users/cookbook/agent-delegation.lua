-- Cookbook: Agent Delegation - Task Distribution Patterns
-- Purpose: Implement patterns for delegating tasks between agents efficiently
-- Prerequisites: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable, cookbook config
-- Expected Output: Demonstration of agent delegation patterns
-- Version: 0.7.0
-- Tags: cookbook, agent-delegation, task-distribution, multi-agent, coordination

print("=== Agent Delegation Patterns ===\n")

-- ============================================================
-- Pattern 1: Capability-Based Delegation
-- ============================================================

print("1. Capability-Based Delegation")
print("-" .. string.rep("-", 40))

local CapabilityDelegator = {}
CapabilityDelegator.__index = CapabilityDelegator

function CapabilityDelegator:new()
    return setmetatable({
        agents = {},
        capabilities = {},
        delegation_history = {}
    }, self)
end

function CapabilityDelegator:register_agent(agent_name, capabilities, config)
    self.agents[agent_name] = {
        name = agent_name,
        capabilities = capabilities,
        load = 0,
        max_concurrent = config.max_concurrent or 3,
        performance_score = config.performance_score or 1.0,
        available = true
    }
    
    -- Index by capability
    for _, capability in ipairs(capabilities) do
        if not self.capabilities[capability] then
            self.capabilities[capability] = {}
        end
        table.insert(self.capabilities[capability], agent_name)
    end
    
    print(string.format("   📋 Registered agent %s with capabilities: %s", 
        agent_name, table.concat(capabilities, ", ")))
end

function CapabilityDelegator:find_best_agent(required_capability, task_priority)
    task_priority = task_priority or "normal"
    local available_agents = self.capabilities[required_capability]
    
    if not available_agents or #available_agents == 0 then
        return nil, "No agents available for capability: " .. required_capability
    end
    
    -- Score agents based on load, performance, and availability
    local scored_agents = {}
    
    for _, agent_name in ipairs(available_agents) do
        local agent = self.agents[agent_name]
        
        if agent.available and agent.load < agent.max_concurrent then
            local load_factor = 1 - (agent.load / agent.max_concurrent)
            local score = agent.performance_score * load_factor
            
            -- Boost score for high priority tasks
            if task_priority == "high" then
                score = score * 1.5
            elseif task_priority == "low" then
                score = score * 0.8
            end
            
            table.insert(scored_agents, {
                name = agent_name,
                score = score,
                load = agent.load
            })
        end
    end
    
    if #scored_agents == 0 then
        return nil, "No available agents for capability: " .. required_capability
    end
    
    -- Sort by score (descending)
    table.sort(scored_agents, function(a, b) return a.score > b.score end)
    
    return scored_agents[1].name, nil
end

function CapabilityDelegator:delegate_task(task_name, required_capability, task_data, priority)
    priority = priority or "normal"
    
    local agent_name, error_msg = self:find_best_agent(required_capability, priority)
    
    if not agent_name then
        return {
            success = false,
            error = error_msg,
            agent = nil
        }
    end
    
    -- Update agent load
    local agent = self.agents[agent_name]
    agent.load = agent.load + 1
    
    -- Record delegation
    local delegation = {
        task_name = task_name,
        agent = agent_name,
        capability = required_capability,
        priority = priority,
        timestamp = os.time(),
        status = "assigned"
    }
    table.insert(self.delegation_history, delegation)
    
    print(string.format("   ✅ Delegated '%s' to %s (load: %d/%d)", 
        task_name, agent_name, agent.load, agent.max_concurrent))
    
    -- Simulate task execution
    local result = self:execute_task(agent_name, task_name, task_data)
    
    -- Update agent load after completion
    agent.load = math.max(0, agent.load - 1)
    delegation.status = result.success and "completed" or "failed"
    
    return {
        success = result.success,
        result = result.result,
        agent = agent_name,
        execution_time = result.execution_time
    }
end

function CapabilityDelegator:execute_task(agent_name, task_name, task_data)
    -- Simulate task execution time based on task complexity
    local complexity_delays = {
        simple = 0.01,   -- 10ms
        moderate = 0.03, -- 30ms
        complex = 0.05   -- 50ms
    }
    
    local complexity = task_data.complexity or "moderate"
    local delay = complexity_delays[complexity] or 0.03
    
    local start_time = os.clock()
    local end_time = start_time + delay
    while os.clock() < end_time do end
    
    local execution_time = (os.clock() - start_time) * 1000
    
    -- Simulate occasional failures
    local success = math.random() > 0.1 -- 90% success rate
    
    local result = success and {
        task = task_name,
        agent = agent_name,
        data = task_data,
        completed_at = os.date("%Y-%m-%d %H:%M:%S")
    } or nil
    
    return {
        success = success,
        result = result,
        execution_time = execution_time
    }
end

function CapabilityDelegator:get_delegation_stats()
    local stats = {
        total_delegations = #self.delegation_history,
        by_agent = {},
        by_capability = {},
        by_status = {}
    }
    
    for _, delegation in ipairs(self.delegation_history) do
        stats.by_agent[delegation.agent] = (stats.by_agent[delegation.agent] or 0) + 1
        stats.by_capability[delegation.capability] = (stats.by_capability[delegation.capability] or 0) + 1
        stats.by_status[delegation.status] = (stats.by_status[delegation.status] or 0) + 1
    end
    
    return stats
end

-- Test capability-based delegation
local delegator = CapabilityDelegator:new()

-- Register agents with different capabilities
delegator:register_agent("research_agent", {"research", "analysis"}, {
    max_concurrent = 2,
    performance_score = 1.2
})

delegator:register_agent("content_agent", {"writing", "editing"}, {
    max_concurrent = 3,
    performance_score = 1.0
})

delegator:register_agent("data_agent", {"analysis", "computation"}, {
    max_concurrent = 4,
    performance_score = 0.9
})

delegator:register_agent("general_agent", {"research", "writing", "analysis"}, {
    max_concurrent = 2,
    performance_score = 0.8
})

-- Test task delegation
print("   Testing capability-based delegation:")

local tasks = {
    {name = "market_research", capability = "research", complexity = "complex", priority = "high"},
    {name = "data_analysis", capability = "analysis", complexity = "moderate", priority = "normal"},
    {name = "blog_post", capability = "writing", complexity = "simple", priority = "low"},
    {name = "financial_analysis", capability = "computation", complexity = "complex", priority = "high"},
    {name = "competitor_research", capability = "research", complexity = "moderate", priority = "normal"}
}

for _, task_config in ipairs(tasks) do
    local result = delegator:delegate_task(
        task_config.name,
        task_config.capability,
        {complexity = task_config.complexity},
        task_config.priority
    )
    
    if result.success then
        print(string.format("     ✅ %s completed by %s (%.1fms)", 
            task_config.name, result.agent, result.execution_time))
    else
        print(string.format("     ❌ %s failed: %s", task_config.name, result.error))
    end
end

local stats = delegator:get_delegation_stats()
print(string.format("\n   Delegation Summary: %d total tasks", stats.total_delegations))
print("   By agent:")
for agent, count in pairs(stats.by_agent) do
    print(string.format("     %s: %d tasks", agent, count))
end

print()

-- ============================================================
-- Pattern 2: Load-Balanced Delegation
-- ============================================================

print("2. Load-Balanced Delegation")
print("-" .. string.rep("-", 40))

local LoadBalancer = {}
LoadBalancer.__index = LoadBalancer

function LoadBalancer:new(strategy)
    return setmetatable({
        strategy = strategy or "round_robin", -- round_robin, least_connections, weighted
        agents = {},
        current_index = 1,
        task_queue = {},
        processing_tasks = {}
    }, self)
end

function LoadBalancer:add_agent(agent_name, weight)
    local agent = {
        name = agent_name,
        weight = weight or 1,
        active_tasks = 0,
        total_tasks = 0,
        last_used = 0
    }
    
    table.insert(self.agents, agent)
    print(string.format("   ⚖️  Added agent to load balancer: %s (weight: %d)", 
        agent_name, weight or 1))
end

function LoadBalancer:select_agent()
    if #self.agents == 0 then
        return nil
    end
    
    if self.strategy == "round_robin" then
        return self:select_round_robin()
    elseif self.strategy == "least_connections" then
        return self:select_least_connections()
    elseif self.strategy == "weighted" then
        return self:select_weighted()
    else
        return self.agents[1] -- Fallback
    end
end

function LoadBalancer:select_round_robin()
    local agent = self.agents[self.current_index]
    self.current_index = (self.current_index % #self.agents) + 1
    return agent
end

function LoadBalancer:select_least_connections()
    local best_agent = nil
    local min_connections = math.huge
    
    for _, agent in ipairs(self.agents) do
        if agent.active_tasks < min_connections then
            min_connections = agent.active_tasks
            best_agent = agent
        end
    end
    
    return best_agent
end

function LoadBalancer:select_weighted()
    local total_weight = 0
    for _, agent in ipairs(self.agents) do
        total_weight = total_weight + agent.weight
    end
    
    local random_weight = math.random(total_weight)
    local current_weight = 0
    
    for _, agent in ipairs(self.agents) do
        current_weight = current_weight + agent.weight
        if random_weight <= current_weight then
            return agent
        end
    end
    
    return self.agents[1] -- Fallback
end

function LoadBalancer:assign_task(task_name, task_data)
    local agent = self:select_agent()
    
    if not agent then
        return {success = false, error = "No agents available"}
    end
    
    agent.active_tasks = agent.active_tasks + 1
    agent.total_tasks = agent.total_tasks + 1
    agent.last_used = os.time()
    
    local task_id = task_name .. "_" .. os.time() .. "_" .. math.random(1000)
    
    self.processing_tasks[task_id] = {
        task_id = task_id,
        task_name = task_name,
        agent = agent.name,
        start_time = os.clock(),
        data = task_data
    }
    
    print(string.format("   📋 Assigned %s to %s (active: %d)", 
        task_name, agent.name, agent.active_tasks))
    
    -- Simulate task processing
    local result = self:process_task(task_id)
    
    -- Clean up
    agent.active_tasks = math.max(0, agent.active_tasks - 1)
    self.processing_tasks[task_id] = nil
    
    return result
end

function LoadBalancer:process_task(task_id)
    local task = self.processing_tasks[task_id]
    if not task then
        return {success = false, error = "Task not found"}
    end
    
    -- Simulate processing time
    local processing_time = math.random(20, 100) / 1000 -- 20-100ms
    local end_time = os.clock() + processing_time
    while os.clock() < end_time do end
    
    local execution_time = (os.clock() - task.start_time) * 1000
    
    return {
        success = true,
        task_id = task_id,
        agent = task.agent,
        execution_time = execution_time,
        result = "Task " .. task.task_name .. " completed successfully"
    }
end

function LoadBalancer:get_load_stats()
    local stats = {
        total_agents = #self.agents,
        strategy = self.strategy,
        agents = {}
    }
    
    for _, agent in ipairs(self.agents) do
        table.insert(stats.agents, {
            name = agent.name,
            weight = agent.weight,
            active_tasks = agent.active_tasks,
            total_tasks = agent.total_tasks,
            last_used = agent.last_used
        })
    end
    
    return stats
end

-- Test load balancing strategies
local strategies = {"round_robin", "least_connections", "weighted"}

for _, strategy in ipairs(strategies) do
    print(string.format("   Testing %s strategy:", strategy))
    
    local balancer = LoadBalancer:new(strategy)
    
    -- Add agents with different weights for weighted strategy
    balancer:add_agent("agent_1", 3)
    balancer:add_agent("agent_2", 2)
    balancer:add_agent("agent_3", 1)
    
    -- Assign multiple tasks
    for i = 1, 6 do
        local result = balancer:assign_task("task_" .. i, {complexity = "moderate"})
        if result.success then
            print(string.format("     Task %d: %s (%.1fms)", 
                i, result.agent, result.execution_time))
        end
    end
    
    local stats = balancer:get_load_stats()
    print(string.format("     Load distribution:"))
    for _, agent_stats in ipairs(stats.agents) do
        print(string.format("       %s: %d total tasks", 
            agent_stats.name, agent_stats.total_tasks))
    end
    
    print()
end

-- ============================================================
-- Pattern 3: Priority-Based Delegation
-- ============================================================

print("3. Priority-Based Delegation")
print("-" .. string.rep("-", 40))

local PriorityDelegator = {}
PriorityDelegator.__index = PriorityDelegator

function PriorityDelegator:new()
    return setmetatable({
        priority_queues = {
            critical = {},
            high = {},
            normal = {},
            low = {}
        },
        agents = {},
        processing = false
    }, self)
end

function PriorityDelegator:add_agent(agent_name, config)
    self.agents[agent_name] = {
        name = agent_name,
        available = true,
        current_task = nil,
        capabilities = config.capabilities or {},
        max_priority = config.max_priority or "critical"
    }
    
    print(string.format("   👤 Added priority agent: %s (max priority: %s)", 
        agent_name, config.max_priority or "critical"))
end

function PriorityDelegator:queue_task(task_name, priority, task_data, required_capability)
    priority = priority or "normal"
    
    local task = {
        name = task_name,
        priority = priority,
        data = task_data,
        required_capability = required_capability,
        queued_at = os.time(),
        id = task_name .. "_" .. os.time()
    }
    
    table.insert(self.priority_queues[priority], task)
    
    print(string.format("   📥 Queued task: %s [%s priority]", task_name, priority))
    
    -- Try to process immediately
    self:process_queue()
    
    return task.id
end

function PriorityDelegator:find_available_agent(required_capability, max_priority)
    local priority_levels = {"critical", "high", "normal", "low"}
    local max_priority_level = 4 -- Default to low
    
    for i, level in ipairs(priority_levels) do
        if level == max_priority then
            max_priority_level = i
            break
        end
    end
    
    for _, agent in pairs(self.agents) do
        if agent.available then
            -- Check capability
            if required_capability then
                local has_capability = false
                for _, cap in ipairs(agent.capabilities) do
                    if cap == required_capability then
                        has_capability = true
                        break
                    end
                end
                if not has_capability then
                    goto continue
                end
            end
            
            -- Check if agent can handle this priority level
            local agent_max_level = 4
            for i, level in ipairs(priority_levels) do
                if level == agent.max_priority then
                    agent_max_level = i
                    break
                end
            end
            
            if agent_max_level <= max_priority_level then
                return agent
            end
        end
        
        ::continue::
    end
    
    return nil
end

function PriorityDelegator:process_queue()
    if self.processing then
        return
    end
    
    self.processing = true
    
    -- Process queues in priority order
    local priority_order = {"critical", "high", "normal", "low"}
    
    for _, priority in ipairs(priority_order) do
        local queue = self.priority_queues[priority]
        
        while #queue > 0 do
            local task = table.remove(queue, 1) -- FIFO within priority
            local agent = self:find_available_agent(task.required_capability, priority)
            
            if agent then
                self:assign_task_to_agent(agent, task)
            else
                -- Put task back at front of queue
                table.insert(queue, 1, task)
                break -- No available agents, try later
            end
        end
    end
    
    self.processing = false
end

function PriorityDelegator:assign_task_to_agent(agent, task)
    agent.available = false
    agent.current_task = task
    
    print(string.format("   🎯 Assigned %s [%s] to %s", 
        task.name, task.priority, agent.name))
    
    -- Simulate task execution
    local execution_time = self:execute_task(agent, task)
    
    -- Free up agent
    agent.available = true
    agent.current_task = nil
    
    print(string.format("   ✅ Completed %s by %s (%.1fms)", 
        task.name, agent.name, execution_time))
    
    -- Process more tasks
    self:process_queue()
end

function PriorityDelegator:execute_task(agent, task)
    -- Simulate execution time based on priority (higher priority = more resources)
    local priority_multipliers = {
        critical = 0.5, -- Faster execution
        high = 0.7,
        normal = 1.0,
        low = 1.5       -- Slower execution
    }
    
    local base_time = 0.03 -- 30ms
    local multiplier = priority_multipliers[task.priority] or 1.0
    local execution_time = base_time * multiplier
    
    local end_time = os.clock() + execution_time
    while os.clock() < end_time do end
    
    return execution_time * 1000 -- Return in milliseconds
end

function PriorityDelegator:get_queue_status()
    local status = {
        queued_tasks = 0,
        by_priority = {}
    }
    
    for priority, queue in pairs(self.priority_queues) do
        local count = #queue
        status.by_priority[priority] = count
        status.queued_tasks = status.queued_tasks + count
    end
    
    status.busy_agents = 0
    for _, agent in pairs(self.agents) do
        if not agent.available then
            status.busy_agents = status.busy_agents + 1
        end
    end
    
    return status
end

-- Test priority-based delegation
local priority_delegator = PriorityDelegator:new()

-- Add agents with different priority handling capabilities
priority_delegator:add_agent("critical_agent", {
    capabilities = {"security", "urgent_analysis"},
    max_priority = "critical"
})

priority_delegator:add_agent("general_agent_1", {
    capabilities = {"analysis", "reporting"},
    max_priority = "high"
})

priority_delegator:add_agent("general_agent_2", {
    capabilities = {"analysis", "reporting", "research"},
    max_priority = "normal"
})

priority_delegator:add_agent("background_agent", {
    capabilities = {"research", "data_collection"},
    max_priority = "low"
})

print("   Testing priority-based delegation:")

-- Queue tasks with different priorities
local test_tasks = {
    {name = "security_breach", priority = "critical", capability = "security"},
    {name = "user_report", priority = "low", capability = "reporting"},
    {name = "market_analysis", priority = "high", capability = "analysis"},
    {name = "background_research", priority = "low", capability = "research"},
    {name = "urgent_bug_fix", priority = "critical", capability = "urgent_analysis"},
    {name = "weekly_report", priority = "normal", capability = "reporting"}
}

for _, task_config in ipairs(test_tasks) do
    priority_delegator:queue_task(
        task_config.name,
        task_config.priority,
        {type = "standard"},
        task_config.capability
    )
end

local final_status = priority_delegator:get_queue_status()
print(string.format("   Final queue status: %d total queued, %d busy agents", 
    final_status.queued_tasks, final_status.busy_agents))

print()
print("🎯 Key Takeaways:")
print("   • Match agent capabilities to task requirements")
print("   • Use load balancing to distribute work evenly")
print("   • Implement priority queues for urgent tasks")
print("   • Monitor agent performance and adjust assignments")
print("   • Provide fallback mechanisms for failed delegations")
print("   • Track delegation patterns for optimization")