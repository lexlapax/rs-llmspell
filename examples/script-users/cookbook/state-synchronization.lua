-- Cookbook: State Synchronization - Keeping State Consistent Across Systems
-- Purpose: Implement patterns for synchronizing state between distributed components
-- Prerequisites: State persistence enabled in config (optional for enhanced features)
-- Expected Output: Demonstration of state synchronization patterns
-- Version: 0.7.0
-- Tags: cookbook, state, synchronization, distributed, production

print("=== State Synchronization Patterns ===\n")

-- ============================================================
-- Pattern 1: Leader-Follower Synchronization
-- ============================================================

print("1. Leader-Follower Synchronization")
print("-" .. string.rep("-", 40))

local LeaderFollowerSync = {}
LeaderFollowerSync.__index = LeaderFollowerSync

function LeaderFollowerSync:new(node_id, is_leader)
    return setmetatable({
        node_id = node_id,
        is_leader = is_leader or false,
        state = {},
        followers = {},
        leader = nil,
        sync_queue = {},
        last_sync = 0,
        version = 0
    }, self)
end

function LeaderFollowerSync:register_follower(follower)
    if not self.is_leader then
        return false, "Only leader can register followers"
    end
    
    self.followers[follower.node_id] = follower
    print(string.format("   Leader %s: Registered follower %s", 
        self.node_id, follower.node_id))
    
    -- Send initial state
    self:sync_to_follower(follower)
    return true
end

function LeaderFollowerSync:set_leader(leader)
    if self.is_leader then
        return false, "Node is already a leader"
    end
    
    self.leader = leader
    print(string.format("   Follower %s: Set leader to %s", 
        self.node_id, leader.node_id))
    return true
end

function LeaderFollowerSync:write(key, value)
    if not self.is_leader then
        -- Forward to leader
        if self.leader then
            return self.leader:write(key, value)
        else
            return false, "No leader available"
        end
    end
    
    -- Leader writes
    self.state[key] = value
    self.version = self.version + 1
    
    -- Queue sync to followers
    table.insert(self.sync_queue, {
        operation = "set",
        key = key,
        value = value,
        version = self.version
    })
    
    -- Sync to followers
    self:sync_to_followers()
    
    return true
end

function LeaderFollowerSync:read(key)
    -- Can read from any node
    return self.state[key]
end

function LeaderFollowerSync:sync_to_followers()
    if not self.is_leader or #self.sync_queue == 0 then
        return
    end
    
    for _, follower in pairs(self.followers) do
        self:sync_to_follower(follower)
    end
    
    self.sync_queue = {}
    self.last_sync = os.time()
end

function LeaderFollowerSync:sync_to_follower(follower)
    -- Send sync queue
    for _, op in ipairs(self.sync_queue) do
        follower:apply_sync(op)
    end
end

function LeaderFollowerSync:apply_sync(operation)
    if self.is_leader then
        return false, "Leader doesn't apply sync"
    end
    
    if operation.version <= self.version then
        -- Already applied
        return true
    end
    
    if operation.operation == "set" then
        self.state[operation.key] = operation.value
        self.version = operation.version
    end
    
    return true
end

function LeaderFollowerSync:promote_to_leader()
    if self.is_leader then
        return false, "Already a leader"
    end
    
    self.is_leader = true
    self.leader = nil
    self.followers = {}
    
    print(string.format("   Node %s promoted to leader", self.node_id))
    return true
end

-- Test leader-follower
local leader = LeaderFollowerSync:new("node1", true)
local follower1 = LeaderFollowerSync:new("node2", false)
local follower2 = LeaderFollowerSync:new("node3", false)

follower1:set_leader(leader)
follower2:set_leader(leader)

leader:register_follower(follower1)
leader:register_follower(follower2)

print("\n   Testing writes:")
leader:write("config", "production")
leader:write("users", 100)

print(string.format("   Leader state: config=%s, users=%s",
    leader:read("config"), leader:read("users")))
print(string.format("   Follower1 state: config=%s, users=%s",
    follower1:read("config"), follower1:read("users")))
print(string.format("   Follower2 state: config=%s, users=%s",
    follower2:read("config"), follower2:read("users")))

print()

-- ============================================================
-- Pattern 2: Optimistic Replication with Conflict Resolution
-- ============================================================

print("2. Optimistic Replication")
print("-" .. string.rep("-", 40))

local OptimisticReplication = {}
OptimisticReplication.__index = OptimisticReplication

function OptimisticReplication:new(node_id)
    return setmetatable({
        node_id = node_id,
        local_state = {},
        vector_clock = {},
        pending_sync = {},
        conflict_log = {}
    }, self)
end

function OptimisticReplication:write(key, value)
    -- Update vector clock
    self.vector_clock[self.node_id] = (self.vector_clock[self.node_id] or 0) + 1
    
    -- Store with metadata
    self.local_state[key] = {
        value = value,
        version = self.vector_clock[self.node_id],
        node = self.node_id,
        timestamp = os.time(),
        vector_clock = self:copy_vector_clock()
    }
    
    -- Queue for sync
    table.insert(self.pending_sync, {
        key = key,
        entry = self.local_state[key]
    })
    
    return true
end

function OptimisticReplication:read(key)
    local entry = self.local_state[key]
    return entry and entry.value or nil
end

function OptimisticReplication:sync_with(other_node)
    print(string.format("   Syncing %s <-> %s", self.node_id, other_node.node_id))
    
    -- Exchange states
    local my_changes = {}
    local their_changes = {}
    
    -- Find my changes to send
    for key, entry in pairs(self.local_state) do
        my_changes[key] = entry
    end
    
    -- Get their changes
    for key, entry in pairs(other_node.local_state) do
        their_changes[key] = entry
    end
    
    -- Apply their changes
    for key, their_entry in pairs(their_changes) do
        self:merge_entry(key, their_entry)
    end
    
    -- Send my changes to them
    for key, my_entry in pairs(my_changes) do
        other_node:merge_entry(key, my_entry)
    end
    
    -- Update vector clocks
    self:merge_vector_clock(other_node.vector_clock)
    other_node:merge_vector_clock(self.vector_clock)
end

function OptimisticReplication:merge_entry(key, remote_entry)
    local local_entry = self.local_state[key]
    
    if not local_entry then
        -- No local version, accept remote
        self.local_state[key] = remote_entry
        return
    end
    
    -- Compare vector clocks
    local comparison = self:compare_vector_clocks(
        local_entry.vector_clock,
        remote_entry.vector_clock
    )
    
    if comparison == "older" then
        -- Local is older, accept remote
        self.local_state[key] = remote_entry
    elseif comparison == "concurrent" then
        -- Conflict! Resolve using timestamp (last-write-wins)
        if remote_entry.timestamp > local_entry.timestamp then
            table.insert(self.conflict_log, {
                key = key,
                local_value = local_entry.value,
                remote_value = remote_entry.value,
                resolution = "remote_wins"
            })
            self.local_state[key] = remote_entry
            print(string.format("     Conflict on %s: remote wins", key))
        else
            table.insert(self.conflict_log, {
                key = key,
                local_value = local_entry.value,
                remote_value = remote_entry.value,
                resolution = "local_wins"
            })
            print(string.format("     Conflict on %s: local wins", key))
        end
    end
    -- If "newer", keep local
end

function OptimisticReplication:compare_vector_clocks(vc1, vc2)
    local all_nodes = {}
    for node in pairs(vc1) do all_nodes[node] = true end
    for node in pairs(vc2) do all_nodes[node] = true end
    
    local vc1_greater = false
    local vc2_greater = false
    
    for node in pairs(all_nodes) do
        local v1 = vc1[node] or 0
        local v2 = vc2[node] or 0
        
        if v1 > v2 then vc1_greater = true end
        if v2 > v1 then vc2_greater = true end
    end
    
    if vc1_greater and not vc2_greater then
        return "newer"
    elseif vc2_greater and not vc1_greater then
        return "older"
    elseif vc1_greater and vc2_greater then
        return "concurrent"
    else
        return "equal"
    end
end

function OptimisticReplication:copy_vector_clock()
    local copy = {}
    for k, v in pairs(self.vector_clock) do
        copy[k] = v
    end
    return copy
end

function OptimisticReplication:merge_vector_clock(other_vc)
    for node, version in pairs(other_vc) do
        self.vector_clock[node] = math.max(
            self.vector_clock[node] or 0,
            version
        )
    end
end

-- Test optimistic replication
local opt_node1 = OptimisticReplication:new("node1")
local opt_node2 = OptimisticReplication:new("node2")

-- Independent writes
opt_node1:write("doc1", "version_A")
opt_node1:write("shared", "node1_data")

opt_node2:write("doc2", "version_B")
opt_node2:write("shared", "node2_data")  -- Conflict!

print("\n   Before sync:")
print("   Node1 shared: " .. tostring(opt_node1:read("shared")))
print("   Node2 shared: " .. tostring(opt_node2:read("shared")))

-- Sync nodes
opt_node1:sync_with(opt_node2)

print("\n   After sync:")
print("   Node1 shared: " .. tostring(opt_node1:read("shared")))
print("   Node2 shared: " .. tostring(opt_node2:read("shared")))

print()

-- ============================================================
-- Pattern 3: Two-Phase Commit (2PC)
-- ============================================================

print("3. Two-Phase Commit Protocol")
print("-" .. string.rep("-", 40))

local TwoPhaseCommit = {}
TwoPhaseCommit.__index = TwoPhaseCommit

function TwoPhaseCommit:new(node_id, is_coordinator)
    return setmetatable({
        node_id = node_id,
        is_coordinator = is_coordinator or false,
        state = {},
        participants = {},
        pending_tx = nil,
        tx_log = {}
    }, self)
end

function TwoPhaseCommit:add_participant(participant)
    if not self.is_coordinator then
        return false, "Only coordinator can add participants"
    end
    
    self.participants[participant.node_id] = participant
    return true
end

function TwoPhaseCommit:begin_transaction(tx_id, changes)
    if not self.is_coordinator then
        return false, "Only coordinator can begin transaction"
    end
    
    print(string.format("   Coordinator: Beginning transaction %s", tx_id))
    
    self.pending_tx = {
        id = tx_id,
        changes = changes,
        votes = {},
        phase = "voting",
        timestamp = os.time()
    }
    
    -- Phase 1: Voting (Prepare)
    local all_votes = true
    
    for _, participant in pairs(self.participants) do
        local vote = participant:prepare(tx_id, changes)
        self.pending_tx.votes[participant.node_id] = vote
        
        if not vote then
            all_votes = false
            print(string.format("     %s voted: NO", participant.node_id))
        else
            print(string.format("     %s voted: YES", participant.node_id))
        end
    end
    
    -- Phase 2: Commit or Abort
    if all_votes then
        print("   Coordinator: All votes YES, committing...")
        
        -- Commit on all participants
        for _, participant in pairs(self.participants) do
            participant:commit(tx_id)
        end
        
        -- Commit locally
        self:apply_changes(changes)
        self.pending_tx.phase = "committed"
        
        return true, "committed"
    else
        print("   Coordinator: Some votes NO, aborting...")
        
        -- Abort on all participants
        for _, participant in pairs(self.participants) do
            participant:abort(tx_id)
        end
        
        self.pending_tx.phase = "aborted"
        
        return false, "aborted"
    end
end

function TwoPhaseCommit:prepare(tx_id, changes)
    -- Participant: Check if can commit
    print(string.format("   %s: Preparing transaction %s", self.node_id, tx_id))
    
    -- Simple validation: check if we have capacity
    local can_commit = true
    
    -- Check for conflicts or constraints
    for key, value in pairs(changes) do
        if key == "balance" and type(value) == "number" then
            -- Example: Don't allow negative balance
            if value < 0 then
                can_commit = false
                break
            end
        end
    end
    
    if can_commit then
        self.pending_tx = {
            id = tx_id,
            changes = changes,
            status = "prepared"
        }
    end
    
    return can_commit
end

function TwoPhaseCommit:commit(tx_id)
    if not self.pending_tx or self.pending_tx.id ~= tx_id then
        return false
    end
    
    print(string.format("   %s: Committing transaction %s", self.node_id, tx_id))
    
    self:apply_changes(self.pending_tx.changes)
    self.pending_tx.status = "committed"
    
    table.insert(self.tx_log, {
        id = tx_id,
        status = "committed",
        timestamp = os.time()
    })
    
    return true
end

function TwoPhaseCommit:abort(tx_id)
    if not self.pending_tx or self.pending_tx.id ~= tx_id then
        return false
    end
    
    print(string.format("   %s: Aborting transaction %s", self.node_id, tx_id))
    
    self.pending_tx.status = "aborted"
    
    table.insert(self.tx_log, {
        id = tx_id,
        status = "aborted",
        timestamp = os.time()
    })
    
    return true
end

function TwoPhaseCommit:apply_changes(changes)
    for key, value in pairs(changes) do
        self.state[key] = value
    end
end

-- Test 2PC
local coordinator = TwoPhaseCommit:new("coordinator", true)
local participant1 = TwoPhaseCommit:new("participant1", false)
local participant2 = TwoPhaseCommit:new("participant2", false)

coordinator:add_participant(participant1)
coordinator:add_participant(participant2)

print("\n   Transaction 1: Valid changes")
local success, status = coordinator:begin_transaction("tx1", {
    balance = 100,
    status = "active"
})
print("   Result: " .. status)

print("\n   Transaction 2: Invalid changes (negative balance)")
success, status = coordinator:begin_transaction("tx2", {
    balance = -50,
    status = "overdrawn"
})
print("   Result: " .. status)

print()

-- ============================================================
-- Pattern 4: CRDT-Based Synchronization
-- ============================================================

print("4. CRDT-Based Synchronization")
print("-" .. string.rep("-", 40))

-- G-Counter (Grow-only Counter) CRDT
local GCounter = {}
GCounter.__index = GCounter

function GCounter:new(node_id)
    return setmetatable({
        node_id = node_id,
        counts = {}
    }, self)
end

function GCounter:increment(amount)
    amount = amount or 1
    self.counts[self.node_id] = (self.counts[self.node_id] or 0) + amount
end

function GCounter:value()
    local sum = 0
    for _, count in pairs(self.counts) do
        sum = sum + count
    end
    return sum
end

function GCounter:merge(other)
    -- Merge by taking maximum for each node
    for node, count in pairs(other.counts) do
        self.counts[node] = math.max(
            self.counts[node] or 0,
            count
        )
    end
end

-- PN-Counter (Positive-Negative Counter) CRDT
local PNCounter = {}
PNCounter.__index = PNCounter

function PNCounter:new(node_id)
    return setmetatable({
        node_id = node_id,
        positive = GCounter:new(node_id),
        negative = GCounter:new(node_id)
    }, self)
end

function PNCounter:increment(amount)
    amount = amount or 1
    if amount >= 0 then
        self.positive:increment(amount)
    else
        self.negative:increment(-amount)
    end
end

function PNCounter:decrement(amount)
    amount = amount or 1
    self.negative:increment(amount)
end

function PNCounter:value()
    return self.positive:value() - self.negative:value()
end

function PNCounter:merge(other)
    self.positive:merge(other.positive)
    self.negative:merge(other.negative)
end

-- OR-Set (Observed-Remove Set) CRDT
local ORSet = {}
ORSet.__index = ORSet

function ORSet:new(node_id)
    return setmetatable({
        node_id = node_id,
        elements = {},  -- element -> {unique_id -> true}
        tombstones = {} -- removed unique_ids
    }, self)
end

function ORSet:add(element)
    local unique_id = self.node_id .. "_" .. os.time() .. "_" .. math.random(1000)
    
    if not self.elements[element] then
        self.elements[element] = {}
    end
    
    self.elements[element][unique_id] = true
    print(string.format("   %s: Added '%s'", self.node_id, element))
end

function ORSet:remove(element)
    if self.elements[element] then
        -- Mark all unique_ids as tombstones
        for unique_id in pairs(self.elements[element]) do
            self.tombstones[unique_id] = true
        end
        print(string.format("   %s: Removed '%s'", self.node_id, element))
    end
end

function ORSet:contains(element)
    if not self.elements[element] then
        return false
    end
    
    -- Check if any unique_id is not tombstoned
    for unique_id in pairs(self.elements[element]) do
        if not self.tombstones[unique_id] then
            return true
        end
    end
    
    return false
end

function ORSet:merge(other)
    -- Merge elements
    for element, ids in pairs(other.elements) do
        if not self.elements[element] then
            self.elements[element] = {}
        end
        for id in pairs(ids) do
            self.elements[element][id] = true
        end
    end
    
    -- Merge tombstones
    for id in pairs(other.tombstones) do
        self.tombstones[id] = true
    end
end

function ORSet:values()
    local result = {}
    for element in pairs(self.elements) do
        if self:contains(element) then
            table.insert(result, element)
        end
    end
    return result
end

-- Test CRDTs
print("\n   Testing G-Counter CRDT:")
local gc1 = GCounter:new("node1")
local gc2 = GCounter:new("node2")

gc1:increment(5)
gc2:increment(3)

print("   Node1 counter: " .. gc1:value())
print("   Node2 counter: " .. gc2:value())

gc1:merge(gc2)
gc2:merge(gc1)

print("   After merge - Both nodes: " .. gc1:value())

print("\n   Testing OR-Set CRDT:")
local set1 = ORSet:new("node1")
local set2 = ORSet:new("node2")

set1:add("apple")
set1:add("banana")
set2:add("cherry")
set2:remove("banana")  -- Won't affect node1's banana

set1:merge(set2)
set2:merge(set1)

print("   After merge - Set contents:")
for _, value in ipairs(set1:values()) do
    print("     " .. value)
end

print()

-- ============================================================
-- Pattern 5: Gossip Protocol Synchronization
-- ============================================================

print("5. Gossip Protocol Synchronization")
print("-" .. string.rep("-", 40))

local GossipNode = {}
GossipNode.__index = GossipNode

function GossipNode:new(node_id)
    return setmetatable({
        node_id = node_id,
        state = {},
        version_vector = {},
        peers = {},
        gossip_interval = 1,  -- seconds
        fanout = 2,  -- number of peers to gossip to
        heard_from = {}
    }, self)
end

function GossipNode:add_peer(peer)
    self.peers[peer.node_id] = peer
    peer.peers[self.node_id] = self
end

function GossipNode:update(key, value)
    -- Update local state and version
    self.version_vector[self.node_id] = (self.version_vector[self.node_id] or 0) + 1
    
    self.state[key] = {
        value = value,
        version = self.version_vector[self.node_id],
        source = self.node_id,
        timestamp = os.time()
    }
    
    -- Trigger gossip
    self:gossip()
end

function GossipNode:gossip()
    -- Select random peers to gossip to
    local peer_list = {}
    for id, peer in pairs(self.peers) do
        table.insert(peer_list, peer)
    end
    
    -- Shuffle and select fanout peers
    for i = #peer_list, 2, -1 do
        local j = math.random(i)
        peer_list[i], peer_list[j] = peer_list[j], peer_list[i]
    end
    
    local gossip_count = math.min(self.fanout, #peer_list)
    
    for i = 1, gossip_count do
        local peer = peer_list[i]
        if peer then
            print(string.format("   %s gossiping to %s", 
                self.node_id, peer.node_id))
            self:send_gossip(peer)
        end
    end
end

function GossipNode:send_gossip(peer)
    -- Send state digest
    local digest = {
        sender = self.node_id,
        version_vector = self.version_vector,
        state_summary = {}
    }
    
    for key, entry in pairs(self.state) do
        digest.state_summary[key] = {
            version = entry.version,
            source = entry.source
        }
    end
    
    peer:receive_gossip(digest, self)
end

function GossipNode:receive_gossip(digest, sender)
    -- Update heard_from
    self.heard_from[sender.node_id] = os.time()
    
    -- Compare version vectors and request missing updates
    local need_updates = {}
    
    for key, summary in pairs(digest.state_summary) do
        local local_entry = self.state[key]
        
        if not local_entry or 
           (local_entry.source == summary.source and 
            local_entry.version < summary.version) then
            table.insert(need_updates, key)
        end
    end
    
    -- Request missing updates
    if #need_updates > 0 then
        self:request_updates(sender, need_updates)
    end
    
    -- Update version vector
    for node, version in pairs(digest.version_vector) do
        self.version_vector[node] = math.max(
            self.version_vector[node] or 0,
            version
        )
    end
end

function GossipNode:request_updates(sender, keys)
    for _, key in ipairs(keys) do
        local entry = sender.state[key]
        if entry then
            self.state[key] = {
                value = entry.value,
                version = entry.version,
                source = entry.source,
                timestamp = entry.timestamp
            }
            print(string.format("     %s updated %s from %s", 
                self.node_id, key, sender.node_id))
        end
    end
end

function GossipNode:get(key)
    local entry = self.state[key]
    return entry and entry.value or nil
end

-- Test gossip protocol
local gossip1 = GossipNode:new("gossip1")
local gossip2 = GossipNode:new("gossip2")
local gossip3 = GossipNode:new("gossip3")
local gossip4 = GossipNode:new("gossip4")

-- Create network topology
gossip1:add_peer(gossip2)
gossip1:add_peer(gossip3)
gossip2:add_peer(gossip3)
gossip2:add_peer(gossip4)
gossip3:add_peer(gossip4)

print("\n   Initial updates:")
gossip1:update("config", "v1")
gossip4:update("status", "ready")

print("\n   After gossip propagation:")
print(string.format("   Node1 status: %s", tostring(gossip1:get("status"))))
print(string.format("   Node4 config: %s", tostring(gossip4:get("config"))))

print()
print("🎯 Key Takeaways:")
print("   • Leader-follower ensures consistency with single writer")
print("   • Optimistic replication allows concurrent updates")
print("   • 2PC provides strong consistency guarantees")
print("   • CRDTs enable conflict-free replication")
print("   • Gossip protocols scale to large networks")