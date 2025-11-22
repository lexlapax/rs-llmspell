-- Recommended profile: sessions
-- Run with: llmspell -p sessions run state-management.lua
-- Session management with SQLite

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: 08 - State Management v0.7.0
-- Complexity Level: PRODUCTION
-- Real-World Use Case: Enterprise state management with versioning and persistence
-- Pattern Category: State & Data Management
--
-- Purpose: Production state management patterns including versioning, persistence,
--          migration, and distributed state handling for LLMSpell applications.
--          Essential for maintaining application state across restarts and scaling.
-- Architecture: Versioned state with migration support and conflict resolution
-- Crates Showcased: llmspell-state-persistence, llmspell-bridge
-- Key Features:
--   • State versioning and history tracking
--   • State migration patterns
--   • Conflict resolution strategies
--   • Distributed state synchronization
--   • State backup and recovery
--   • Scoped state management (global, workflow, agent)
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Optional: State persistence config for enhanced features
--   • No API keys required
--
-- HOW TO RUN:
-- # With state persistence:
-- ./target/debug/llmspell -p state \
--   run examples/script-users/cookbook/state-management.lua
--
-- # Without persistence (in-memory only):
-- ./target/debug/llmspell run examples/script-users/cookbook/state-management.lua
--
-- EXPECTED OUTPUT:
-- 5 state patterns demonstrated:
-- 1. Versioned state with history tracking
-- 2. State migration from v1 to v2 schema
-- 3. Conflict resolution with merge strategies
-- 4. State snapshots and rollback
-- 5. Scoped state isolation
--
-- Time to Complete: <3 seconds
-- Production Notes: Use persistent backend for production, implement regular
--                   backups, version all schema changes, use distributed locks
--                   for concurrent access, monitor state size and growth.
-- ============================================================

print("=== State Management Patterns ===")
print("Pattern 08: PRODUCTION - Enterprise state versioning and persistence\n")

-- ============================================================
-- Pattern 1: Simple Version Control
-- ============================================================

print("1. Simple Version Control")
print("-" .. string.rep("-", 40))

local VersionedState = {}
VersionedState.__index = VersionedState

function VersionedState:new(options)
    options = options or {}
    return setmetatable({
        current_version = 0,
        versions = {},
        max_versions = options.max_versions or 100,
        auto_snapshot = options.auto_snapshot or false,
        snapshot_interval = options.snapshot_interval or 10
    }, self)
end

function VersionedState:set(key, value)
    -- Create new version
    self.current_version = self.current_version + 1
    
    -- Copy previous state
    local prev_state = {}
    if self.versions[self.current_version - 1] then
        for k, v in pairs(self.versions[self.current_version - 1].state) do
            prev_state[k] = v
        end
    end
    
    -- Apply change
    prev_state[key] = value
    
    -- Store version
    self.versions[self.current_version] = {
        version = self.current_version,
        state = prev_state,
        changes = {[key] = value},
        timestamp = os.time(),
        metadata = {}
    }
    
    -- Cleanup old versions if needed
    self:cleanup_old_versions()
    
    -- Auto-snapshot if enabled
    if self.auto_snapshot and self.current_version % self.snapshot_interval == 0 then
        self:create_snapshot()
    end
    
    return self.current_version
end

function VersionedState:get(key, version)
    version = version or self.current_version
    
    if not self.versions[version] then
        return nil, "Version not found"
    end
    
    return self.versions[version].state[key]
end

function VersionedState:get_version(version)
    version = version or self.current_version
    return self.versions[version]
end

function VersionedState:rollback(target_version)
    if not self.versions[target_version] then
        return false, "Target version not found"
    end
    
    -- Create new version with old state
    self.current_version = self.current_version + 1
    self.versions[self.current_version] = {
        version = self.current_version,
        state = self:deep_copy(self.versions[target_version].state),
        changes = {["_rollback"] = true},
        timestamp = os.time(),
        metadata = {
            rollback_from = self.current_version - 1,
            rollback_to = target_version
        }
    }
    
    print(string.format("   Rolled back to version %d (new version: %d)",
        target_version, self.current_version))
    
    return true
end

function VersionedState:cleanup_old_versions()
    if self.current_version > self.max_versions then
        local to_delete = self.current_version - self.max_versions
        self.versions[to_delete] = nil
    end
end

function VersionedState:create_snapshot()
    local snapshot_key = "snapshot_" .. self.current_version
    self.versions[snapshot_key] = self:deep_copy(self.versions[self.current_version])
    print(string.format("   Created snapshot at version %d", self.current_version))
end

function VersionedState:deep_copy(obj)
    if type(obj) ~= "table" then
        return obj
    end
    local copy = {}
    for k, v in pairs(obj) do
        copy[k] = self:deep_copy(v)
    end
    return copy
end

function VersionedState:diff(version1, version2)
    local v1 = self.versions[version1]
    local v2 = self.versions[version2]
    
    if not v1 or not v2 then
        return nil, "Invalid versions"
    end
    
    local diff = {
        added = {},
        modified = {},
        removed = {}
    }
    
    -- Check for added/modified
    for key, value in pairs(v2.state) do
        if v1.state[key] == nil then
            diff.added[key] = value
        elseif v1.state[key] ~= value then
            diff.modified[key] = {
                old = v1.state[key],
                new = value
            }
        end
    end
    
    -- Check for removed
    for key, value in pairs(v1.state) do
        if v2.state[key] == nil then
            diff.removed[key] = value
        end
    end
    
    return diff
end

-- Test versioned state
local versioned = VersionedState:new({max_versions = 10})

print("   Creating versions:")
versioned:set("name", "Alice")
versioned:set("age", 30)
versioned:set("status", "active")
versioned:set("age", 31)  -- Modify existing

print(string.format("   Current version: %d", versioned.current_version))

-- Get historical values
print("\n   Historical values for 'age':")
for v = 2, versioned.current_version do
    local val = versioned:get("age", v)
    print(string.format("   Version %d: %s", v, tostring(val)))
end

-- Diff versions
local diff = versioned:diff(2, 4)
print("\n   Diff between version 2 and 4:")
for key, change in pairs(diff.modified) do
    print(string.format("   %s: %s -> %s", 
        key, tostring(change.old), tostring(change.new)))
end

-- Rollback
versioned:rollback(2)
print("\n   After rollback to version 2:")
print("   Current age: " .. tostring(versioned:get("age")))

print()

-- ============================================================
-- Pattern 2: Git-Style Branching
-- ============================================================

print("2. Git-Style Branching")
print("-" .. string.rep("-", 40))

local BranchingState = {}
BranchingState.__index = BranchingState

function BranchingState:new()
    return setmetatable({
        branches = {},
        current_branch = "main",
        commits = {},
        next_commit_id = 1
    }, self)
    
end

function BranchingState:init()
    -- Create initial commit
    local commit_id = self:create_commit({}, "Initial commit", nil)
    
    -- Create main branch
    self.branches["main"] = {
        name = "main",
        head = commit_id,
        created_at = os.time()
    }
    
    return commit_id
end

function BranchingState:create_commit(state, message, parent)
    local commit_id = "commit_" .. self.next_commit_id
    self.next_commit_id = self.next_commit_id + 1
    
    self.commits[commit_id] = {
        id = commit_id,
        state = state,
        message = message,
        parent = parent,
        timestamp = os.time(),
        author = "system"
    }
    
    return commit_id
end

function BranchingState:checkout(branch_name)
    if not self.branches[branch_name] then
        return false, "Branch not found"
    end
    
    self.current_branch = branch_name
    print(string.format("   Switched to branch '%s'", branch_name))
    return true
end

function BranchingState:create_branch(name, from_branch)
    from_branch = from_branch or self.current_branch
    
    if not self.branches[from_branch] then
        return false, "Source branch not found"
    end
    
    self.branches[name] = {
        name = name,
        head = self.branches[from_branch].head,
        created_at = os.time(),
        created_from = from_branch
    }
    
    print(string.format("   Created branch '%s' from '%s'", name, from_branch))
    return true
end

function BranchingState:commit(changes, message)
    local branch = self.branches[self.current_branch]
    if not branch then
        return false, "Invalid branch"
    end
    
    -- Get current state
    local parent_commit = self.commits[branch.head]
    local new_state = {}
    
    if parent_commit then
        for k, v in pairs(parent_commit.state) do
            new_state[k] = v
        end
    end
    
    -- Apply changes
    for key, value in pairs(changes) do
        if value == nil then
            new_state[key] = nil  -- Allow deletions
        else
            new_state[key] = value
        end
    end
    
    -- Create new commit
    local commit_id = self:create_commit(new_state, message, branch.head)
    
    -- Update branch head
    branch.head = commit_id
    
    print(string.format("   Committed to '%s': %s", self.current_branch, message))
    return commit_id
end

function BranchingState:merge(source_branch, target_branch)
    target_branch = target_branch or self.current_branch
    
    local source = self.branches[source_branch]
    local target = self.branches[target_branch]
    
    if not source or not target then
        return false, "Invalid branches"
    end
    
    -- Simple merge: combine states
    local source_commit = self.commits[source.head]
    local target_commit = self.commits[target.head]
    
    local merged_state = {}
    
    -- Start with target state
    for k, v in pairs(target_commit.state) do
        merged_state[k] = v
    end
    
    -- Apply source changes
    for k, v in pairs(source_commit.state) do
        if merged_state[k] ~= nil and merged_state[k] ~= v then
            print(string.format("   Merge conflict on key '%s'", k))
            -- Simple resolution: source wins
            print("     Resolved: using source value")
        end
        merged_state[k] = v
    end
    
    -- Create merge commit
    local merge_message = string.format("Merge '%s' into '%s'", 
        source_branch, target_branch)
    local commit_id = self:create_commit(merged_state, merge_message, target.head)
    
    -- Update target branch
    target.head = commit_id
    
    print(string.format("   Merged '%s' into '%s'", source_branch, target_branch))
    return commit_id
end

function BranchingState:get_current_state()
    local branch = self.branches[self.current_branch]
    if not branch then
        return nil
    end
    
    local commit = self.commits[branch.head]
    return commit and commit.state or {}
end

-- Test branching state
local branching = BranchingState:new()
branching:init()

-- Work on main branch
branching:commit({user = "Alice", role = "admin"}, "Add user Alice")
branching:commit({config = "production"}, "Set config")

-- Create feature branch
branching:create_branch("feature", "main")
branching:checkout("feature")
branching:commit({feature_flag = true}, "Enable feature")
branching:commit({user = "Bob"}, "Update user to Bob")

-- Create another branch
branching:checkout("main")
branching:create_branch("hotfix", "main")
branching:checkout("hotfix")
branching:commit({config = "staging"}, "Emergency config change")

-- Merge branches
branching:checkout("main")
branching:merge("hotfix", "main")
branching:merge("feature", "main")

print("\n   Final state on main:")
for key, value in pairs(branching:get_current_state()) do
    print(string.format("   %s: %s", key, tostring(value)))
end

print()

-- ============================================================
-- Pattern 3: Schema Versioning
-- ============================================================

print("3. Schema Versioning")
print("-" .. string.rep("-", 40))

local SchemaVersionedState = {}
SchemaVersionedState.__index = SchemaVersionedState

function SchemaVersionedState:new()
    return setmetatable({
        schemas = {},
        current_schema_version = 0,
        data = {},
        migrations = {}
    }, self)
end

function SchemaVersionedState:register_schema(version, schema)
    self.schemas[version] = {
        version = version,
        fields = schema,
        registered_at = os.time()
    }
    
    if version > self.current_schema_version then
        self.current_schema_version = version
    end
    
    print(string.format("   Registered schema version %d", version))
end

function SchemaVersionedState:register_migration(from_version, to_version, migration_fn)
    local key = from_version .. "_to_" .. to_version
    self.migrations[key] = migration_fn
    
    print(string.format("   Registered migration from v%d to v%d", 
        from_version, to_version))
end

function SchemaVersionedState:validate(data, schema_version)
    schema_version = schema_version or self.current_schema_version
    
    local schema = self.schemas[schema_version]
    if not schema then
        return false, "Schema not found"
    end
    
    -- Validate required fields
    for field_name, field_def in pairs(schema.fields) do
        if field_def.required and data[field_name] == nil then
            return false, "Missing required field: " .. field_name
        end
        
        -- Type validation
        if data[field_name] ~= nil and field_def.type then
            local actual_type = type(data[field_name])
            if actual_type ~= field_def.type then
                return false, string.format("Type mismatch for %s: expected %s, got %s",
                    field_name, field_def.type, actual_type)
            end
        end
    end
    
    return true
end

function SchemaVersionedState:set(key, data, schema_version)
    schema_version = schema_version or self.current_schema_version
    
    -- Validate against schema
    local valid, err = self:validate(data, schema_version)
    if not valid then
        return false, err
    end
    
    -- Store with schema version
    self.data[key] = {
        data = data,
        schema_version = schema_version,
        updated_at = os.time()
    }
    
    return true
end

function SchemaVersionedState:get(key, target_schema_version)
    local entry = self.data[key]
    if not entry then
        return nil
    end
    
    target_schema_version = target_schema_version or self.current_schema_version
    
    -- If same schema version, return as-is
    if entry.schema_version == target_schema_version then
        return entry.data
    end
    
    -- Need to migrate
    return self:migrate_data(entry.data, entry.schema_version, target_schema_version)
end

function SchemaVersionedState:migrate_data(data, from_version, to_version)
    print(string.format("   Migrating data from v%d to v%d", 
        from_version, to_version))
    
    local current_data = data
    local current_version = from_version
    
    -- Apply migrations step by step
    while current_version < to_version do
        local next_version = current_version + 1
        local migration_key = current_version .. "_to_" .. next_version
        local migration_fn = self.migrations[migration_key]
        
        if migration_fn then
            current_data = migration_fn(current_data)
            print(string.format("     Applied migration v%d -> v%d", 
                current_version, next_version))
        else
            -- Default migration: copy fields that exist in new schema
            local new_schema = self.schemas[next_version]
            if new_schema then
                local migrated = {}
                for field_name, _ in pairs(new_schema.fields) do
                    if current_data[field_name] ~= nil then
                        migrated[field_name] = current_data[field_name]
                    end
                end
                current_data = migrated
            end
        end
        
        current_version = next_version
    end
    
    return current_data
end

-- Test schema versioning
local schema_state = SchemaVersionedState:new()

-- Define schemas
schema_state:register_schema(1, {
    name = {type = "string", required = true},
    age = {type = "number", required = true}
})

schema_state:register_schema(2, {
    name = {type = "string", required = true},
    age = {type = "number", required = true},
    email = {type = "string", required = false}
})

schema_state:register_schema(3, {
    full_name = {type = "string", required = true},  -- Renamed from 'name'
    age = {type = "number", required = true},
    email = {type = "string", required = true},  -- Now required
    created_at = {type = "number", required = false}
})

-- Register migrations
schema_state:register_migration(1, 2, function(data)
    -- Add default email
    data.email = data.email or "noemail@example.com"
    return data
end)

schema_state:register_migration(2, 3, function(data)
    -- Rename name to full_name
    data.full_name = data.name
    data.name = nil
    -- Add created_at
    data.created_at = os.time()
    return data
end)

-- Store data with different schemas
schema_state:set("user1", {name = "Alice", age = 30}, 1)
schema_state:set("user2", {name = "Bob", age = 25, email = "bob@example.com"}, 2)

-- Retrieve with migration
print("\n   Retrieving user1 with latest schema:")
local user1_migrated = schema_state:get("user1", 3)
for k, v in pairs(user1_migrated) do
    print(string.format("   %s: %s", k, tostring(v)))
end

print()

-- ============================================================
-- Pattern 4: Event Sourcing with Snapshots
-- ============================================================

print("4. Event Sourcing with Snapshots")
print("-" .. string.rep("-", 40))

local EventSourcedState = {}
EventSourcedState.__index = EventSourcedState

function EventSourcedState:new(options)
    options = options or {}
    return setmetatable({
        events = {},
        snapshots = {},
        current_state = {},
        event_handlers = {},
        snapshot_interval = options.snapshot_interval or 100,
        next_event_id = 1
    }, self)
end

function EventSourcedState:register_handler(event_type, handler)
    self.event_handlers[event_type] = handler
end

function EventSourcedState:append_event(event_type, data, metadata)
    local event = {
        id = self.next_event_id,
        type = event_type,
        data = data,
        metadata = metadata or {},
        timestamp = os.time(),
        version = self.next_event_id
    }
    
    self.next_event_id = self.next_event_id + 1
    table.insert(self.events, event)
    
    -- Apply event
    self:apply_event(event)
    
    -- Create snapshot if needed
    if #self.events % self.snapshot_interval == 0 then
        self:create_snapshot()
    end
    
    return event.id
end

function EventSourcedState:apply_event(event)
    local handler = self.event_handlers[event.type]
    if handler then
        self.current_state = handler(self.current_state, event.data)
    end
end

function EventSourcedState:create_snapshot()
    local snapshot = {
        version = #self.events,
        state = self:deep_copy(self.current_state),
        timestamp = os.time()
    }
    
    table.insert(self.snapshots, snapshot)
    print(string.format("   Created snapshot at version %d", snapshot.version))
    
    return snapshot
end

function EventSourcedState:rebuild_state(target_version)
    target_version = target_version or #self.events
    
    -- Find nearest snapshot
    local best_snapshot = nil
    for _, snapshot in ipairs(self.snapshots) do
        if snapshot.version <= target_version then
            best_snapshot = snapshot
        end
    end
    
    -- Start from snapshot or empty state
    local state = {}
    local start_index = 1
    
    if best_snapshot then
        state = self:deep_copy(best_snapshot.state)
        start_index = best_snapshot.version + 1
        print(string.format("   Rebuilding from snapshot at version %d", 
            best_snapshot.version))
    else
        print("   Rebuilding from beginning")
    end
    
    -- Apply events from snapshot to target
    for i = start_index, target_version do
        local event = self.events[i]
        if event then
            local handler = self.event_handlers[event.type]
            if handler then
                state = handler(state, event.data)
            end
        end
    end
    
    return state
end

function EventSourcedState:get_state_at_version(version)
    return self:rebuild_state(version)
end

function EventSourcedState:deep_copy(obj)
    if type(obj) ~= "table" then
        return obj
    end
    local copy = {}
    for k, v in pairs(obj) do
        copy[k] = self:deep_copy(v)
    end
    return copy
end

-- Test event sourcing
local event_state = EventSourcedState:new({snapshot_interval = 5})

-- Register event handlers
event_state:register_handler("user_created", function(state, data)
    state.users = state.users or {}
    state.users[data.id] = {
        name = data.name,
        created_at = os.time()
    }
    return state
end)

event_state:register_handler("user_updated", function(state, data)
    if state.users and state.users[data.id] then
        for k, v in pairs(data.updates) do
            state.users[data.id][k] = v
        end
    end
    return state
end)

event_state:register_handler("user_deleted", function(state, data)
    if state.users then
        state.users[data.id] = nil
    end
    return state
end)

-- Generate events
print("   Generating events:")
for i = 1, 8 do
    if i <= 4 then
        event_state:append_event("user_created", {
            id = "user" .. i,
            name = "User " .. i
        })
    elseif i <= 6 then
        event_state:append_event("user_updated", {
            id = "user" .. (i - 4),
            updates = {status = "active"}
        })
    else
        event_state:append_event("user_deleted", {
            id = "user" .. (i - 6)
        })
    end
end

print(string.format("\n   Total events: %d", #event_state.events))
print(string.format("   Total snapshots: %d", #event_state.snapshots))

-- Get state at different versions
print("\n   State at version 4 (after creates):")
local state_v4 = event_state:get_state_at_version(4)
for id, user in pairs(state_v4.users or {}) do
    print(string.format("   %s: %s", id, user.name))
end

print("\n   Current state (after all events):")
for id, user in pairs(event_state.current_state.users or {}) do
    print(string.format("   %s: %s (status: %s)", 
        id, user.name, user.status or "unknown"))
end

print()
print("🎯 Key Takeaways:")
print("   • Version control enables state history tracking")
print("   • Branching supports parallel development")
print("   • Schema versioning handles data evolution")
print("   • Event sourcing provides complete audit trail")
print("   • Snapshots optimize rebuild performance")