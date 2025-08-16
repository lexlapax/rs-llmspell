-- Cookbook: State Sharing - Multi-Agent State Coordination
-- Purpose: Implement patterns for sharing state between agents and components
-- Prerequisites: State persistence enabled in config (optional for enhanced features)
-- Expected Output: Demonstration of state sharing patterns
-- Version: 0.7.0
-- Tags: cookbook, state, sharing, coordination, production

print("=== State Sharing Patterns ===\n")

-- ============================================================
-- Pattern 1: Shared State Store
-- ============================================================

print("1. Shared State Store")
print("-" .. string.rep("-", 40))

local SharedStateStore = {}
SharedStateStore.__index = SharedStateStore

function SharedStateStore:new()
    return setmetatable({
        state = {},
        listeners = {},
        locks = {},
        version = 0
    }, self)
end

function SharedStateStore:get(key)
    return self.state[key]
end

function SharedStateStore:set(key, value)
    local old_value = self.state[key]
    self.state[key] = value
    self.version = self.version + 1
    
    -- Notify listeners
    self:notify_listeners(key, value, old_value)
    
    return true
end

function SharedStateStore:update(key, updater)
    -- Atomic update with function
    local current = self.state[key]
    local new_value = updater(current)
    return self:set(key, new_value)
end

function SharedStateStore:subscribe(key, callback)
    if not self.listeners[key] then
        self.listeners[key] = {}
    end
    table.insert(self.listeners[key], callback)
    
    -- Return unsubscribe function
    local listener_index = #self.listeners[key]
    return function()
        table.remove(self.listeners[key], listener_index)
    end
end

function SharedStateStore:notify_listeners(key, new_value, old_value)
    local listeners = self.listeners[key] or {}
    for _, callback in ipairs(listeners) do
        callback(new_value, old_value, key)
    end
end

function SharedStateStore:acquire_lock(key, owner_id, timeout)
    timeout = timeout or 30  -- Default 30 second timeout
    
    local lock = self.locks[key]
    local now = os.time()
    
    -- Check if lock exists and is still valid
    if lock and (now < lock.expires_at) and lock.owner ~= owner_id then
        return false, "Lock held by " .. lock.owner
    end
    
    -- Acquire or refresh lock
    self.locks[key] = {
        owner = owner_id,
        acquired_at = now,
        expires_at = now + timeout
    }
    
    return true
end

function SharedStateStore:release_lock(key, owner_id)
    local lock = self.locks[key]
    
    if not lock then
        return true
    end
    
    if lock.owner ~= owner_id then
        return false, "Not lock owner"
    end
    
    self.locks[key] = nil
    return true
end

-- Test shared state store
local store = SharedStateStore:new()

-- Subscribe to changes
local unsubscribe = store:subscribe("counter", function(new_val, old_val)
    print(string.format("   Counter changed: %s -> %s", 
        tostring(old_val), tostring(new_val)))
end)

store:set("counter", 0)
store:update("counter", function(val) return (val or 0) + 1 end)
store:update("counter", function(val) return val + 1 end)

-- Test locking
print("\n   Testing locks:")
local acquired, err = store:acquire_lock("resource", "agent1", 10)
print("   Agent1 acquire lock: " .. tostring(acquired))

acquired, err = store:acquire_lock("resource", "agent2", 10)
print("   Agent2 acquire lock: " .. tostring(acquired) .. 
    (err and " (" .. err .. ")" or ""))

store:release_lock("resource", "agent1")
print("   Agent1 released lock")

print()

-- ============================================================
-- Pattern 2: Event-Based State Sharing
-- ============================================================

print("2. Event-Based State Sharing")
print("-" .. string.rep("-", 40))

local EventBasedState = {}
EventBasedState.__index = EventBasedState

function EventBasedState:new()
    return setmetatable({
        state = {},
        event_log = {},
        subscribers = {},
        replay_position = {}
    }, self)
end

function EventBasedState:emit_event(event_type, data)
    local event = {
        id = #self.event_log + 1,
        type = event_type,
        data = data,
        timestamp = os.time(),
        applied = false
    }
    
    table.insert(self.event_log, event)
    
    -- Apply event to state
    self:apply_event(event)
    
    -- Notify subscribers
    self:notify_subscribers(event)
    
    return event.id
end

function EventBasedState:apply_event(event)
    if event.applied then
        return
    end
    
    -- Event handlers
    local handlers = {
        ["state.set"] = function(data)
            self.state[data.key] = data.value
        end,
        ["state.increment"] = function(data)
            self.state[data.key] = (self.state[data.key] or 0) + data.amount
        end,
        ["state.append"] = function(data)
            if not self.state[data.key] then
                self.state[data.key] = {}
            end
            table.insert(self.state[data.key], data.value)
        end
    }
    
    local handler = handlers[event.type]
    if handler then
        handler(event.data)
        event.applied = true
    end
end

function EventBasedState:replay_from(position)
    position = position or 1
    
    print(string.format("   Replaying events from position %d", position))
    
    -- Clear state
    self.state = {}
    
    -- Replay events
    for i = position, #self.event_log do
        local event = self.event_log[i]
        event.applied = false  -- Reset applied flag
        self:apply_event(event)
    end
    
    return #self.event_log - position + 1
end

function EventBasedState:subscribe(pattern, callback)
    table.insert(self.subscribers, {
        pattern = pattern,
        callback = callback
    })
end

function EventBasedState:notify_subscribers(event)
    for _, sub in ipairs(self.subscribers) do
        if string.find(event.type, sub.pattern) then
            sub.callback(event)
        end
    end
end

function EventBasedState:get_state()
    return self.state
end

-- Test event-based state
local event_state = EventBasedState:new()

-- Subscribe to state changes
event_state:subscribe("state%.", function(event)
    print(string.format("   Event: %s - %s", 
        event.type, 
        event.data.key or ""))
end)

-- Emit events
event_state:emit_event("state.set", {key = "user", value = "Alice"})
event_state:emit_event("state.increment", {key = "score", amount = 10})
event_state:emit_event("state.increment", {key = "score", amount = 5})
event_state:emit_event("state.append", {key = "log", value = "Action 1"})

print("\n   Current state:")
for k, v in pairs(event_state:get_state()) do
    print(string.format("     %s: %s", k, tostring(v)))
end

-- Test replay
event_state:replay_from(2)
print("\n   State after replay from position 2:")
for k, v in pairs(event_state:get_state()) do
    print(string.format("     %s: %s", k, tostring(v)))
end

print()

-- ============================================================
-- Pattern 3: Distributed State with Conflict Resolution
-- ============================================================

print("3. Distributed State with Conflict Resolution")
print("-" .. string.rep("-", 40))

local DistributedState = {}
DistributedState.__index = DistributedState

function DistributedState:new(node_id)
    return setmetatable({
        node_id = node_id,
        state = {},
        vector_clock = {},
        pending_merges = {}
    }, self)
end

function DistributedState:increment_clock()
    self.vector_clock[self.node_id] = (self.vector_clock[self.node_id] or 0) + 1
    return self.vector_clock[self.node_id]
end

function DistributedState:set(key, value)
    self:increment_clock()
    
    self.state[key] = {
        value = value,
        clock = self:copy_clock(),
        node = self.node_id,
        timestamp = os.time()
    }
end

function DistributedState:copy_clock()
    local copy = {}
    for k, v in pairs(self.vector_clock) do
        copy[k] = v
    end
    return copy
end

function DistributedState:compare_clocks(clock1, clock2)
    -- Returns: -1 (clock1 < clock2), 0 (concurrent), 1 (clock1 > clock2)
    local less = false
    local greater = false
    
    -- Check all nodes mentioned in either clock
    local all_nodes = {}
    for node, _ in pairs(clock1) do all_nodes[node] = true end
    for node, _ in pairs(clock2) do all_nodes[node] = true end
    
    for node, _ in pairs(all_nodes) do
        local v1 = clock1[node] or 0
        local v2 = clock2[node] or 0
        
        if v1 < v2 then
            less = true
        elseif v1 > v2 then
            greater = true
        end
    end
    
    if less and not greater then
        return -1  -- clock1 happened before clock2
    elseif greater and not less then
        return 1   -- clock1 happened after clock2
    else
        return 0   -- Concurrent (conflict)
    end
end

function DistributedState:merge(other_node_state)
    print(string.format("   Node %s merging state from node %s", 
        self.node_id, other_node_state.node_id))
    
    -- Update vector clock
    for node, version in pairs(other_node_state.vector_clock) do
        self.vector_clock[node] = math.max(
            self.vector_clock[node] or 0,
            version
        )
    end
    
    -- Merge state with conflict resolution
    for key, remote_entry in pairs(other_node_state.state) do
        local local_entry = self.state[key]
        
        if not local_entry then
            -- No local value, accept remote
            self.state[key] = remote_entry
            print(string.format("     Added %s from remote", key))
        else
            -- Compare vector clocks
            local comparison = self:compare_clocks(
                local_entry.clock,
                remote_entry.clock
            )
            
            if comparison == -1 then
                -- Local is older, accept remote
                self.state[key] = remote_entry
                print(string.format("     Updated %s (remote newer)", key))
            elseif comparison == 0 then
                -- Concurrent modification - conflict!
                self:resolve_conflict(key, local_entry, remote_entry)
            end
            -- If comparison == 1, local is newer, keep local
        end
    end
end

function DistributedState:resolve_conflict(key, local_entry, remote_entry)
    print(string.format("     Conflict on %s!", key))
    
    -- Simple resolution: Last-write-wins based on timestamp
    if remote_entry.timestamp > local_entry.timestamp then
        self.state[key] = remote_entry
        print("       Resolved: used remote (newer timestamp)")
    else
        print("       Resolved: kept local (newer timestamp)")
    end
    
    -- Could also merge values, keep both, or use custom resolution
end

-- Test distributed state
local node1 = DistributedState:new("node1")
local node2 = DistributedState:new("node2")

node1:set("config", "value1")
node1:set("shared", "node1_data")

node2:set("config", "value2")
node2:set("unique", "node2_only")

-- Simulate network partition and concurrent updates
node1:set("shared", "node1_updated")
node2:set("shared", "node2_updated")

-- Merge states
node1:merge(node2)

print("\n   Node1 state after merge:")
for key, entry in pairs(node1.state) do
    print(string.format("     %s: %s (from %s)", 
        key, entry.value, entry.node))
end

print()

-- ============================================================
-- Pattern 4: Hierarchical State Management
-- ============================================================

print("4. Hierarchical State Management")
print("-" .. string.rep("-", 40))

local HierarchicalState = {}
HierarchicalState.__index = HierarchicalState

function HierarchicalState:new(name, parent)
    return setmetatable({
        name = name,
        parent = parent,
        local_state = {},
        children = {},
        interceptors = {}
    }, self)
end

function HierarchicalState:create_child(name)
    local child = HierarchicalState:new(name, self)
    self.children[name] = child
    return child
end

function HierarchicalState:get(key)
    -- Check local state first
    if self.local_state[key] ~= nil then
        return self.local_state[key]
    end
    
    -- Check parent state
    if self.parent then
        return self.parent:get(key)
    end
    
    return nil
end

function HierarchicalState:set(key, value)
    -- Check interceptors
    for _, interceptor in ipairs(self.interceptors) do
        local modified_value = interceptor(key, value, "set")
        if modified_value ~= nil then
            value = modified_value
        end
    end
    
    self.local_state[key] = value
    
    -- Propagate to children if needed
    self:propagate_to_children(key, value)
end

function HierarchicalState:set_global(key, value)
    -- Set at root level
    if self.parent then
        self.parent:set_global(key, value)
    else
        self:set(key, value)
    end
end

function HierarchicalState:propagate_to_children(key, value)
    for _, child in pairs(self.children) do
        -- Children can override propagation
        if not child.local_state[key] then
            child:propagate_to_children(key, value)
        end
    end
end

function HierarchicalState:add_interceptor(interceptor_fn)
    table.insert(self.interceptors, interceptor_fn)
end

function HierarchicalState:get_path()
    if self.parent then
        return self.parent:get_path() .. "/" .. self.name
    else
        return self.name
    end
end

function HierarchicalState:debug_print(indent)
    indent = indent or ""
    print(indent .. self.name .. ":")
    
    for key, value in pairs(self.local_state) do
        print(indent .. "  " .. key .. ": " .. tostring(value))
    end
    
    for _, child in pairs(self.children) do
        child:debug_print(indent .. "  ")
    end
end

-- Test hierarchical state
local root = HierarchicalState:new("root")
local app = root:create_child("app")
local module1 = app:create_child("module1")
local module2 = app:create_child("module2")

-- Set values at different levels
root:set("global_config", "production")
app:set("app_version", "1.0.0")
module1:set("feature_enabled", true)
module2:set("feature_enabled", false)

print("   Hierarchical state structure:")
root:debug_print("   ")

print("\n   Module1 sees:")
print("     global_config: " .. tostring(module1:get("global_config")))
print("     app_version: " .. tostring(module1:get("app_version")))
print("     feature_enabled: " .. tostring(module1:get("feature_enabled")))

print()

-- ============================================================
-- Pattern 5: State Channels for Communication
-- ============================================================

print("5. State Channels for Communication")
print("-" .. string.rep("-", 40))

local StateChannel = {}
StateChannel.__index = StateChannel

function StateChannel:new(name)
    return setmetatable({
        name = name,
        subscribers = {},
        message_queue = {},
        state = {},
        processing = false
    }, self)
end

function StateChannel:subscribe(subscriber_id, callback, filter)
    self.subscribers[subscriber_id] = {
        callback = callback,
        filter = filter or function() return true end,
        received_count = 0
    }
    
    print(string.format("   %s subscribed to channel %s", 
        subscriber_id, self.name))
end

function StateChannel:unsubscribe(subscriber_id)
    self.subscribers[subscriber_id] = nil
end

function StateChannel:publish(message, sender_id)
    table.insert(self.message_queue, {
        id = #self.message_queue + 1,
        message = message,
        sender = sender_id,
        timestamp = os.time()
    })
    
    -- Process immediately if not already processing
    if not self.processing then
        self:process_messages()
    end
end

function StateChannel:process_messages()
    self.processing = true
    
    while #self.message_queue > 0 do
        local msg_wrapper = table.remove(self.message_queue, 1)
        
        -- Update channel state
        if msg_wrapper.message.state_update then
            for k, v in pairs(msg_wrapper.message.state_update) do
                self.state[k] = v
            end
        end
        
        -- Deliver to subscribers
        for sub_id, subscriber in pairs(self.subscribers) do
            if sub_id ~= msg_wrapper.sender and 
               subscriber.filter(msg_wrapper.message) then
                subscriber.callback(msg_wrapper.message, msg_wrapper.sender)
                subscriber.received_count = subscriber.received_count + 1
            end
        end
    end
    
    self.processing = false
end

function StateChannel:get_state()
    return self.state
end

function StateChannel:get_stats()
    local stats = {
        subscribers = 0,
        total_received = 0
    }
    
    for _, sub in pairs(self.subscribers) do
        stats.subscribers = stats.subscribers + 1
        stats.total_received = stats.total_received + sub.received_count
    end
    
    return stats
end

-- Test state channels
local channel = StateChannel:new("coordination")

-- Create subscribers
channel:subscribe("agent1", function(msg, sender)
    print(string.format("   [Agent1] Received from %s: %s", 
        sender, msg.content or ""))
end)

channel:subscribe("agent2", function(msg, sender)
    print(string.format("   [Agent2] Received from %s: %s", 
        sender, msg.content or ""))
end, function(msg) 
    -- Filter: only messages with priority
    return msg.priority ~= nil
end)

-- Publish messages
channel:publish({
    content = "Hello all agents",
    state_update = {status = "active"}
}, "coordinator")

channel:publish({
    content = "High priority task",
    priority = 1,
    state_update = {task_assigned = true}
}, "coordinator")

local stats = channel:get_stats()
print(string.format("\n   Channel stats: %d subscribers, %d messages delivered", 
    stats.subscribers, stats.total_received))

print()
print("🎯 Key Takeaways:")
print("   • Shared stores enable centralized state management")
print("   • Event sourcing provides audit trail and replay")
print("   • Distributed state requires conflict resolution")
print("   • Hierarchical state supports modular applications")
print("   • State channels facilitate pub/sub communication")