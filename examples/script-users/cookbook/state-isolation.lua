-- Cookbook: State Isolation - Preventing State Contamination
-- Purpose: Implement patterns for isolating state between components and agents
-- Prerequisites: State persistence enabled in config (optional for enhanced features)
-- Expected Output: Demonstration of state isolation patterns
-- Version: 0.7.0
-- Tags: cookbook, state, isolation, sandboxing, production

print("=== State Isolation Patterns ===\n")

-- ============================================================
-- Pattern 1: Sandboxed State Containers
-- ============================================================

print("1. Sandboxed State Containers")
print("-" .. string.rep("-", 40))

local SandboxedContainer = {}
SandboxedContainer.__index = SandboxedContainer

function SandboxedContainer:new(id, options)
    options = options or {}
    return setmetatable({
        id = id,
        private_state = {},
        public_state = {},
        access_log = {},
        read_only = options.read_only or false,
        max_size = options.max_size or 1000,
        current_size = 0
    }, self)
end

function SandboxedContainer:set_private(key, value)
    if self.read_only then
        return false, "Container is read-only"
    end
    
    -- Check size limits
    local value_size = self:estimate_size(value)
    if self.current_size + value_size > self.max_size then
        return false, "Size limit exceeded"
    end
    
    self.private_state[key] = value
    self.current_size = self.current_size + value_size
    
    self:log_access("set_private", key)
    return true
end

function SandboxedContainer:get_private(key)
    self:log_access("get_private", key)
    return self.private_state[key]
end

function SandboxedContainer:set_public(key, value)
    if self.read_only then
        return false, "Container is read-only"
    end
    
    self.public_state[key] = value
    self:log_access("set_public", key)
    return true
end

function SandboxedContainer:get_public(key)
    self:log_access("get_public", key)
    return self.public_state[key]
end

function SandboxedContainer:estimate_size(value)
    -- Simple size estimation
    if type(value) == "string" then
        return #value
    elseif type(value) == "number" then
        return 8
    elseif type(value) == "boolean" then
        return 1
    elseif type(value) == "table" then
        local size = 0
        for k, v in pairs(value) do
            size = size + self:estimate_size(k) + self:estimate_size(v)
        end
        return size
    end
    return 10  -- Default size
end

function SandboxedContainer:log_access(operation, key)
    table.insert(self.access_log, {
        operation = operation,
        key = key,
        timestamp = os.time(),
        container = self.id
    })
end

function SandboxedContainer:clone()
    -- Create isolated copy
    local clone = SandboxedContainer:new(self.id .. "_clone", {
        read_only = self.read_only,
        max_size = self.max_size
    })
    
    -- Deep copy public state only
    for k, v in pairs(self.public_state) do
        clone.public_state[k] = self:deep_copy(v)
    end
    
    return clone
end

function SandboxedContainer:deep_copy(obj)
    if type(obj) ~= "table" then
        return obj
    end
    
    local copy = {}
    for k, v in pairs(obj) do
        copy[k] = self:deep_copy(v)
    end
    return copy
end

-- Test sandboxed containers
local container1 = SandboxedContainer:new("agent1", {max_size = 100})
local container2 = SandboxedContainer:new("agent2", {read_only = false})

print("   Testing sandboxed containers:")
container1:set_private("secret", "agent1_key")
container1:set_public("status", "active")

container2:set_private("secret", "agent2_key")
container2:set_public("status", "idle")

print("   Container1 public status: " .. tostring(container1:get_public("status")))
print("   Container2 public status: " .. tostring(container2:get_public("status")))
print("   Container1 private (isolated): " .. tostring(container1:get_private("secret")))

-- Test cloning
local clone = container1:clone()
print("\n   Cloned container public status: " .. tostring(clone:get_public("status")))
print("   Cloned container private (not copied): " .. tostring(clone:get_private("secret")))

print()

-- ============================================================
-- Pattern 2: Transaction-Based State Isolation
-- ============================================================

print("2. Transaction-Based State Isolation")
print("-" .. string.rep("-", 40))

local TransactionalState = {}
TransactionalState.__index = TransactionalState

function TransactionalState:new()
    return setmetatable({
        committed_state = {},
        transactions = {},
        next_tx_id = 1
    }, self)
end

function TransactionalState:begin_transaction(isolation_level)
    isolation_level = isolation_level or "READ_COMMITTED"
    
    local tx_id = self.next_tx_id
    self.next_tx_id = self.next_tx_id + 1
    
    self.transactions[tx_id] = {
        id = tx_id,
        isolation_level = isolation_level,
        read_set = {},     -- Keys read during transaction
        write_set = {},    -- Pending writes
        snapshot = {},     -- Snapshot for REPEATABLE_READ
        status = "active",
        started_at = os.time()
    }
    
    -- Take snapshot for higher isolation levels
    if isolation_level == "REPEATABLE_READ" or 
       isolation_level == "SERIALIZABLE" then
        for k, v in pairs(self.committed_state) do
            self.transactions[tx_id].snapshot[k] = v
        end
    end
    
    print(string.format("   Transaction %d started (%s)", 
        tx_id, isolation_level))
    
    return tx_id
end

function TransactionalState:read(tx_id, key)
    local tx = self.transactions[tx_id]
    if not tx or tx.status ~= "active" then
        return nil, "Invalid or inactive transaction"
    end
    
    -- Record read
    tx.read_set[key] = true
    
    -- Check write set first (read your own writes)
    if tx.write_set[key] ~= nil then
        return tx.write_set[key]
    end
    
    -- Use snapshot for REPEATABLE_READ/SERIALIZABLE
    if tx.isolation_level == "REPEATABLE_READ" or 
       tx.isolation_level == "SERIALIZABLE" then
        return tx.snapshot[key]
    end
    
    -- READ_COMMITTED reads current committed state
    return self.committed_state[key]
end

function TransactionalState:write(tx_id, key, value)
    local tx = self.transactions[tx_id]
    if not tx or tx.status ~= "active" then
        return false, "Invalid or inactive transaction"
    end
    
    -- Buffer write
    tx.write_set[key] = value
    return true
end

function TransactionalState:commit(tx_id)
    local tx = self.transactions[tx_id]
    if not tx or tx.status ~= "active" then
        return false, "Invalid or inactive transaction"
    end
    
    -- Check for conflicts (simplified)
    if tx.isolation_level == "SERIALIZABLE" then
        -- Check if any read keys were modified by other transactions
        for key, _ in pairs(tx.read_set) do
            if tx.snapshot[key] ~= self.committed_state[key] then
                self:rollback(tx_id)
                return false, "Serialization conflict detected"
            end
        end
    end
    
    -- Apply writes
    for key, value in pairs(tx.write_set) do
        self.committed_state[key] = value
    end
    
    tx.status = "committed"
    print(string.format("   Transaction %d committed (%d writes)", 
        tx_id, self:count_table(tx.write_set)))
    
    return true
end

function TransactionalState:rollback(tx_id)
    local tx = self.transactions[tx_id]
    if not tx then
        return false, "Invalid transaction"
    end
    
    tx.status = "rolled_back"
    tx.write_set = {}  -- Discard writes
    
    print(string.format("   Transaction %d rolled back", tx_id))
    return true
end

function TransactionalState:count_table(t)
    local count = 0
    for _ in pairs(t) do
        count = count + 1
    end
    return count
end

-- Test transactional isolation
local tx_state = TransactionalState:new()

-- Transaction 1: READ_COMMITTED
local tx1 = tx_state:begin_transaction("READ_COMMITTED")
tx_state:write(tx1, "counter", 1)
tx_state:write(tx1, "status", "processing")

-- Transaction 2: REPEATABLE_READ (concurrent)
local tx2 = tx_state:begin_transaction("REPEATABLE_READ")
local value = tx_state:read(tx2, "counter")
print("   TX2 reads counter: " .. tostring(value))

-- Commit TX1
tx_state:commit(tx1)

-- TX2 still sees old value (isolated)
value = tx_state:read(tx2, "counter")
print("   TX2 reads counter again (isolated): " .. tostring(value))

tx_state:write(tx2, "counter", 2)
tx_state:commit(tx2)

print()

-- ============================================================
-- Pattern 3: Namespace-Based Isolation
-- ============================================================

print("3. Namespace-Based Isolation")
print("-" .. string.rep("-", 40))

local NamespacedState = {}
NamespacedState.__index = NamespacedState

function NamespacedState:new()
    return setmetatable({
        namespaces = {},
        global_namespace = "_global",
        access_rules = {}
    }, self)
end

function NamespacedState:create_namespace(name, options)
    options = options or {}
    
    self.namespaces[name] = {
        name = name,
        data = {},
        parent = options.parent,
        private = options.private or false,
        metadata = {
            created_at = os.time(),
            owner = options.owner,
            access_count = 0
        }
    }
    
    print(string.format("   Created namespace: %s%s", 
        name, options.private and " (private)" or ""))
    
    return true
end

function NamespacedState:set(namespace, key, value)
    local ns = self.namespaces[namespace]
    if not ns then
        return false, "Namespace not found"
    end
    
    ns.data[key] = value
    ns.metadata.access_count = ns.metadata.access_count + 1
    
    return true
end

function NamespacedState:get(namespace, key, accessor)
    local ns = self.namespaces[namespace]
    if not ns then
        -- Try parent namespaces
        return self:get_with_inheritance(namespace, key)
    end
    
    -- Check access rules
    if ns.private and accessor ~= ns.metadata.owner then
        return nil, "Access denied to private namespace"
    end
    
    ns.metadata.access_count = ns.metadata.access_count + 1
    
    -- Check current namespace
    if ns.data[key] ~= nil then
        return ns.data[key]
    end
    
    -- Check parent namespace
    if ns.parent then
        return self:get(ns.parent, key, accessor)
    end
    
    return nil
end

function NamespacedState:get_with_inheritance(namespace, key)
    -- Walk up the namespace hierarchy
    local parts = {}
    for part in string.gmatch(namespace, "[^%.]+") do
        table.insert(parts, part)
    end
    
    -- Try progressively shorter namespaces
    for i = #parts, 1, -1 do
        local ns_name = table.concat(parts, ".", 1, i)
        local ns = self.namespaces[ns_name]
        if ns and ns.data[key] ~= nil then
            return ns.data[key]
        end
    end
    
    -- Finally check global
    local global = self.namespaces[self.global_namespace]
    if global then
        return global.data[key]
    end
    
    return nil
end

function NamespacedState:merge_namespaces(source, target)
    local source_ns = self.namespaces[source]
    local target_ns = self.namespaces[target]
    
    if not source_ns or not target_ns then
        return false, "Invalid namespaces"
    end
    
    local merged_count = 0
    for key, value in pairs(source_ns.data) do
        if not target_ns.data[key] then
            target_ns.data[key] = value
            merged_count = merged_count + 1
        end
    end
    
    print(string.format("   Merged %d keys from %s to %s", 
        merged_count, source, target))
    
    return true
end

-- Test namespace isolation
local ns_state = NamespacedState:new()

-- Create namespaces
ns_state:create_namespace("app", {})
ns_state:create_namespace("app.module1", {parent = "app"})
ns_state:create_namespace("app.module2", {parent = "app"})
ns_state:create_namespace("secure", {private = true, owner = "admin"})

-- Set values in different namespaces
ns_state:set("app", "version", "1.0")
ns_state:set("app", "config", "shared_config")
ns_state:set("app.module1", "feature", "enabled")
ns_state:set("app.module2", "feature", "disabled")
ns_state:set("secure", "secret", "private_data")

-- Test inheritance
print("\n   Testing namespace inheritance:")
print("   module1.version (inherited): " .. 
    tostring(ns_state:get("app.module1", "version")))
print("   module1.feature (own): " .. 
    tostring(ns_state:get("app.module1", "feature")))
print("   module2.feature (own): " .. 
    tostring(ns_state:get("app.module2", "feature")))

-- Test private namespace
local val, err = ns_state:get("secure", "secret", "user")
print("   Access secure.secret as user: " .. (err or tostring(val)))

print()

-- ============================================================
-- Pattern 4: Copy-on-Write State Isolation
-- ============================================================

print("4. Copy-on-Write State Isolation")
print("-" .. string.rep("-", 40))

local COWState = {}
COWState.__index = COWState

function COWState:new(base_state)
    return setmetatable({
        base = base_state,
        overlay = {},
        copies = {},
        write_count = 0,
        read_count = 0
    }, self)
end

function COWState:read(key)
    self.read_count = self.read_count + 1
    
    -- Check overlay first
    if self.overlay[key] ~= nil then
        return self.overlay[key]
    end
    
    -- Fall back to base
    if self.base then
        return self.base[key]
    end
    
    return nil
end

function COWState:write(key, value)
    self.write_count = self.write_count + 1
    
    -- Copy on first write
    if self.overlay[key] == nil and self.base and self.base[key] ~= nil then
        self.copies[key] = true
        print(string.format("   COW: Copied key '%s' on write", key))
    end
    
    self.overlay[key] = value
end

function COWState:fork()
    -- Create new COW state with current state as base
    local merged = {}
    
    -- Merge base and overlay for new base
    if self.base then
        for k, v in pairs(self.base) do
            merged[k] = v
        end
    end
    
    for k, v in pairs(self.overlay) do
        merged[k] = v
    end
    
    return COWState:new(merged)
end

function COWState:commit()
    -- Merge overlay back to base
    if not self.base then
        self.base = {}
    end
    
    for key, value in pairs(self.overlay) do
        self.base[key] = value
    end
    
    local committed = self.write_count
    self.overlay = {}
    self.copies = {}
    self.write_count = 0
    
    print(string.format("   Committed %d changes", committed))
    
    return committed
end

function COWState:get_stats()
    local overlay_size = 0
    for _ in pairs(self.overlay) do
        overlay_size = overlay_size + 1
    end
    
    local copy_size = 0
    for _ in pairs(self.copies) do
        copy_size = copy_size + 1
    end
    
    return {
        reads = self.read_count,
        writes = self.write_count,
        overlay_size = overlay_size,
        copies_made = copy_size
    }
end

-- Test COW isolation
local base = {
    config = "production",
    timeout = 30,
    retries = 3,
    servers = {"server1", "server2"}
}

local cow = COWState:new(base)

print("   Testing Copy-on-Write:")
print("   Read config: " .. cow:read("config"))
cow:write("config", "development")
cow:write("new_key", "new_value")

print("\n   COW Stats before commit:")
local stats = cow:get_stats()
print(string.format("   Reads: %d, Writes: %d, Overlay: %d, Copies: %d",
    stats.reads, stats.writes, stats.overlay_size, stats.copies_made))

-- Fork before commit
local fork = cow:fork()
fork:write("fork_key", "fork_value")

-- Commit original
cow:commit()

print("\n   After commit:")
print("   Original config: " .. tostring(base.config))
print("   Fork has fork_key: " .. tostring(fork:read("fork_key")))

print()

-- ============================================================
-- Pattern 5: Capability-Based State Access
-- ============================================================

print("5. Capability-Based State Access")
print("-" .. string.rep("-", 40))

local CapabilityState = {}
CapabilityState.__index = CapabilityState

function CapabilityState:new()
    return setmetatable({
        state = {},
        capabilities = {},
        next_cap_id = 1
    }, self)
end

function CapabilityState:create_capability(permissions)
    local cap_id = "CAP_" .. self.next_cap_id
    self.next_cap_id = self.next_cap_id + 1
    
    self.capabilities[cap_id] = {
        id = cap_id,
        permissions = permissions or {},
        created_at = os.time(),
        usage_count = 0,
        revoked = false
    }
    
    print(string.format("   Created capability %s with permissions: %s",
        cap_id, table.concat(permissions, ", ")))
    
    return cap_id
end

function CapabilityState:check_permission(cap_id, operation, key)
    local cap = self.capabilities[cap_id]
    
    if not cap or cap.revoked then
        return false, "Invalid or revoked capability"
    end
    
    -- Check if operation is allowed
    local op_allowed = false
    for _, perm in ipairs(cap.permissions) do
        if perm == operation or perm == "*" then
            op_allowed = true
            break
        end
        
        -- Check pattern matching (e.g., "read:user:*")
        local pattern = "^" .. perm:gsub("%*", ".*") .. "$"
        if string.match(operation .. ":" .. key, pattern) then
            op_allowed = true
            break
        end
    end
    
    if op_allowed then
        cap.usage_count = cap.usage_count + 1
    end
    
    return op_allowed
end

function CapabilityState:read(cap_id, key)
    local allowed, err = self:check_permission(cap_id, "read", key)
    if not allowed then
        return nil, err or "Read permission denied"
    end
    
    return self.state[key]
end

function CapabilityState:write(cap_id, key, value)
    local allowed, err = self:check_permission(cap_id, "write", key)
    if not allowed then
        return false, err or "Write permission denied"
    end
    
    self.state[key] = value
    return true
end

function CapabilityState:revoke_capability(cap_id)
    if self.capabilities[cap_id] then
        self.capabilities[cap_id].revoked = true
        print(string.format("   Revoked capability %s", cap_id))
        return true
    end
    return false
end

function CapabilityState:delegate_capability(parent_cap_id, new_permissions)
    local parent = self.capabilities[parent_cap_id]
    if not parent or parent.revoked then
        return nil, "Invalid parent capability"
    end
    
    -- New capability can only have subset of parent permissions
    local filtered = {}
    for _, new_perm in ipairs(new_permissions) do
        for _, parent_perm in ipairs(parent.permissions) do
            if new_perm == parent_perm or 
               string.match(new_perm, "^" .. parent_perm:gsub("%*", ".*") .. "$") then
                table.insert(filtered, new_perm)
                break
            end
        end
    end
    
    return self:create_capability(filtered)
end

-- Test capability-based access
local cap_state = CapabilityState:new()

-- Create capabilities with different permissions
local admin_cap = cap_state:create_capability({"*"})  -- All permissions
local read_cap = cap_state:create_capability({"read"})
local user_cap = cap_state:create_capability({"read:user:*", "write:user:profile"})

-- Set some state
cap_state:write(admin_cap, "system_config", "value1")
cap_state:write(admin_cap, "user:profile", "John Doe")

-- Test different capabilities
print("\n   Testing capability-based access:")

local val, err = cap_state:read(read_cap, "system_config")
print("   Read with read_cap: " .. tostring(val))

val, err = cap_state:write(read_cap, "system_config", "new_value")
print("   Write with read_cap: " .. (err or "Success"))

val, err = cap_state:write(user_cap, "user:profile", "Jane Doe")
print("   Write user:profile with user_cap: " .. (val and "Success" or err))

-- Delegate capability
local delegated = cap_state:delegate_capability(user_cap, {"read:user:*"})
print("\n   Created delegated capability from user_cap")

-- Revoke original
cap_state:revoke_capability(user_cap)
val, err = cap_state:read(user_cap, "user:profile")
print("   Read with revoked user_cap: " .. (err or tostring(val)))

print()
print("🎯 Key Takeaways:")
print("   • Sandboxing prevents unauthorized state access")
print("   • Transactions provide ACID guarantees")
print("   • Namespaces organize and isolate state")
print("   • Copy-on-Write minimizes memory usage")
print("   • Capabilities enable fine-grained access control")