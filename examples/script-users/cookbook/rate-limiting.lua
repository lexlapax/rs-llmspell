-- Recommended profile: development
-- Run with: llmspell -p development run rate-limiting.lua
-- Development environment with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: 02 - Rate Limiting Patterns v0.7.0
-- Complexity Level: PRODUCTION
-- Real-World Use Case: API rate limit management for enterprise integrations
-- Pattern Category: Performance & Resource Management
--
-- Purpose: Production-ready rate limiting patterns for API protection and quota
--          management. Implements token bucket, sliding window, and adaptive
--          rate limiting algorithms essential for preventing API abuse and
--          ensuring fair resource usage in multi-tenant systems.
-- Architecture: Token bucket and sliding window algorithms with statistics
-- Crates Showcased: llmspell-tools, llmspell-bridge
-- Key Features:
--   • Token bucket algorithm for burst handling
--   • Sliding window rate limiter for smooth traffic
--   • Adaptive rate limiting based on response codes
--   • Multi-tier rate limiting (per-user, per-API)
--   • Rate limit statistics and monitoring
--   • Backoff strategies for rate limit responses
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • No API keys required (simulation only)
--   • No external dependencies
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/cookbook/rate-limiting.lua
--
-- EXPECTED OUTPUT:
-- 4 rate limiting patterns demonstrated:
-- 1. Token bucket showing 10 requests with refill
-- 2. Sliding window with smooth request distribution
-- 3. Adaptive rate limiting adjusting to errors
-- 4. Multi-tier limiting with user and API quotas
--
-- Time to Complete: <2 seconds
-- Production Notes: Configure limits based on API provider SLAs, implement
--                   distributed rate limiting for multi-instance deployments,
--                   use Redis or similar for shared state across instances.
-- ============================================================

print("=== Rate Limiting Patterns ===")
print("Pattern 02: PRODUCTION - API rate limit management\n")

-- ============================================================
-- Pattern 1: Token Bucket Algorithm
-- ============================================================

print("1. Token Bucket Rate Limiter")
print("-" .. string.rep("-", 40))

local TokenBucket = {}
TokenBucket.__index = TokenBucket

function TokenBucket:new(options)
    options = options or {}
    return setmetatable({
        capacity = options.capacity or 10,        -- Max tokens
        refill_rate = options.refill_rate or 1,   -- Tokens per second
        tokens = options.capacity or 10,          -- Current tokens
        last_refill = os.clock(),
        stats = {
            allowed = 0,
            rejected = 0,
            total_requests = 0
        }
    }, self)
end

function TokenBucket:refill()
    local now = os.clock()
    local elapsed = now - self.last_refill
    local tokens_to_add = elapsed * self.refill_rate
    
    self.tokens = math.min(self.capacity, self.tokens + tokens_to_add)
    self.last_refill = now
end

function TokenBucket:try_consume(tokens)
    tokens = tokens or 1
    self:refill()
    
    self.stats.total_requests = self.stats.total_requests + 1
    
    if self.tokens >= tokens then
        self.tokens = self.tokens - tokens
        self.stats.allowed = self.stats.allowed + 1
        return true, self.tokens
    else
        self.stats.rejected = self.stats.rejected + 1
        return false, self.tokens
    end
end

function TokenBucket:get_wait_time(tokens)
    tokens = tokens or 1
    self:refill()
    
    if self.tokens >= tokens then
        return 0
    end
    
    local tokens_needed = tokens - self.tokens
    return tokens_needed / self.refill_rate
end

function TokenBucket:get_stats()
    return {
        current_tokens = self.tokens,
        capacity = self.capacity,
        fill_rate = (self.tokens / self.capacity) * 100,
        allowed = self.stats.allowed,
        rejected = self.stats.rejected,
        total = self.stats.total_requests,
        acceptance_rate = self.stats.total_requests > 0 and 
            (self.stats.allowed / self.stats.total_requests * 100) or 0
    }
end

-- Test token bucket
local bucket = TokenBucket:new({
    capacity = 5,
    refill_rate = 2  -- 2 tokens per second
})

print("   Testing token bucket (capacity=5, refill=2/sec):")

-- Rapid requests
for i = 1, 8 do
    local allowed, tokens_left = bucket:try_consume()
    print(string.format("   Request %d: %s (tokens left: %.1f)", 
        i, allowed and "✅ Allowed" or "❌ Rejected", tokens_left))
end

-- Check wait time
local wait = bucket:get_wait_time(3)
print(string.format("   Wait time for 3 tokens: %.2fs", wait))

local stats = bucket:get_stats()
print(string.format("\n   Stats: %d allowed, %d rejected (%.1f%% acceptance)", 
    stats.allowed, stats.rejected, stats.acceptance_rate))

print()

-- ============================================================
-- Pattern 2: Sliding Window Rate Limiter
-- ============================================================

print("2. Sliding Window Rate Limiter")
print("-" .. string.rep("-", 40))

local SlidingWindow = {}
SlidingWindow.__index = SlidingWindow

function SlidingWindow:new(options)
    options = options or {}
    return setmetatable({
        max_requests = options.max_requests or 10,
        window_size = options.window_size or 60,  -- seconds
        requests = {},  -- Timestamp list
        stats = {
            allowed = 0,
            rejected = 0
        }
    }, self)
end

function SlidingWindow:cleanup()
    local now = os.time()
    local cutoff = now - self.window_size
    
    -- Remove old requests
    local new_requests = {}
    for _, timestamp in ipairs(self.requests) do
        if timestamp > cutoff then
            table.insert(new_requests, timestamp)
        end
    end
    
    self.requests = new_requests
end

function SlidingWindow:try_request()
    self:cleanup()
    
    if #self.requests < self.max_requests then
        table.insert(self.requests, os.time())
        self.stats.allowed = self.stats.allowed + 1
        return true, #self.requests
    else
        self.stats.rejected = self.stats.rejected + 1
        return false, #self.requests
    end
end

function SlidingWindow:get_reset_time()
    if #self.requests == 0 then
        return 0
    end
    
    -- Time until oldest request expires
    local oldest = self.requests[1]
    local reset_at = oldest + self.window_size
    local now = os.time()
    
    return math.max(0, reset_at - now)
end

function SlidingWindow:get_status()
    self:cleanup()
    return {
        current_requests = #self.requests,
        max_requests = self.max_requests,
        remaining = math.max(0, self.max_requests - #self.requests),
        reset_in = self:get_reset_time(),
        window_size = self.window_size
    }
end

-- Test sliding window
local window = SlidingWindow:new({
    max_requests = 5,
    window_size = 10  -- 10 second window
})

print("   Testing sliding window (5 requests per 10 seconds):")

for i = 1, 7 do
    local allowed, count = window:try_request()
    local status = window:get_status()
    print(string.format("   Request %d: %s (%d/%d used, %d remaining)", 
        i, 
        allowed and "✅" or "❌", 
        count, 
        window.max_requests,
        status.remaining))
end

print()

-- ============================================================
-- Pattern 3: Leaky Bucket Algorithm
-- ============================================================

print("3. Leaky Bucket Rate Limiter")
print("-" .. string.rep("-", 40))

local LeakyBucket = {}
LeakyBucket.__index = LeakyBucket

function LeakyBucket:new(options)
    options = options or {}
    return setmetatable({
        capacity = options.capacity or 10,
        leak_rate = options.leak_rate or 1,  -- Requests per second
        queue = {},
        last_leak = os.clock(),
        processing = false
    }, self)
end

function LeakyBucket:leak()
    local now = os.clock()
    local elapsed = now - self.last_leak
    local to_leak = math.floor(elapsed * self.leak_rate)
    
    if to_leak > 0 then
        for i = 1, math.min(to_leak, #self.queue) do
            local request = table.remove(self.queue, 1)
            if request.callback then
                request.callback(true)
            end
        end
        self.last_leak = now
    end
end

function LeakyBucket:add_request(request)
    self:leak()
    
    if #self.queue >= self.capacity then
        if request.callback then
            request.callback(false, "Bucket full")
        end
        return false
    end
    
    table.insert(self.queue, {
        id = request.id or os.time(),
        timestamp = os.clock(),
        callback = request.callback
    })
    
    return true
end

function LeakyBucket:get_queue_status()
    self:leak()
    return {
        queue_size = #self.queue,
        capacity = self.capacity,
        utilization = (#self.queue / self.capacity) * 100,
        leak_rate = self.leak_rate
    }
end

-- Test leaky bucket
local leaky = LeakyBucket:new({
    capacity = 5,
    leak_rate = 2
})

print("   Testing leaky bucket (capacity=5, leak=2/sec):")

for i = 1, 6 do
    local added = leaky:add_request({
        id = i,
        callback = function(success, error)
            if success then
                print("     Request " .. i .. " processed")
            else
                print("     Request " .. i .. " rejected: " .. (error or "unknown"))
            end
        end
    })
    
    local status = leaky:get_queue_status()
    print(string.format("   Request %d: %s (queue: %d/%d)", 
        i, 
        added and "Queued" or "Rejected",
        status.queue_size,
        status.capacity))
end

print()

-- ============================================================
-- Pattern 4: Multi-Tier Rate Limiting
-- ============================================================

print("4. Multi-Tier Rate Limiting")
print("-" .. string.rep("-", 40))

local MultiTierRateLimiter = {}
MultiTierRateLimiter.__index = MultiTierRateLimiter

function MultiTierRateLimiter:new()
    return setmetatable({
        tiers = {},
        user_tiers = {}  -- Map users to tiers
    }, self)
end

function MultiTierRateLimiter:add_tier(name, config)
    self.tiers[name] = {
        name = name,
        limiter = TokenBucket:new({
            capacity = config.capacity,
            refill_rate = config.refill_rate
        }),
        priority = config.priority or 1,
        burst_allowed = config.burst_allowed or false
    }
end

function MultiTierRateLimiter:set_user_tier(user_id, tier_name)
    self.user_tiers[user_id] = tier_name
end

function MultiTierRateLimiter:try_request(user_id, tokens)
    tokens = tokens or 1
    
    local tier_name = self.user_tiers[user_id] or "default"
    local tier = self.tiers[tier_name]
    
    if not tier then
        return false, "Unknown tier: " .. tier_name
    end
    
    local allowed = tier.limiter:try_consume(tokens)
    
    -- Check for burst capability
    if not allowed and tier.burst_allowed then
        -- Try to borrow from higher tier
        for name, other_tier in pairs(self.tiers) do
            if other_tier.priority > tier.priority then
                allowed = other_tier.limiter:try_consume(tokens)
                if allowed then
                    print("   Burst allowed from " .. name .. " tier")
                    break
                end
            end
        end
    end
    
    return allowed, tier_name
end

function MultiTierRateLimiter:get_user_status(user_id)
    local tier_name = self.user_tiers[user_id] or "default"
    local tier = self.tiers[tier_name]
    
    if not tier then
        return nil
    end
    
    local stats = tier.limiter:get_stats()
    return {
        tier = tier_name,
        priority = tier.priority,
        tokens_available = stats.current_tokens,
        capacity = stats.capacity,
        burst_allowed = tier.burst_allowed
    }
end

-- Test multi-tier rate limiting
local multi_tier = MultiTierRateLimiter:new()

-- Define tiers
multi_tier:add_tier("free", {
    capacity = 10,
    refill_rate = 0.5,  -- 0.5 tokens per second
    priority = 1,
    burst_allowed = false
})

multi_tier:add_tier("premium", {
    capacity = 100,
    refill_rate = 5,
    priority = 2,
    burst_allowed = true
})

multi_tier:add_tier("enterprise", {
    capacity = 1000,
    refill_rate = 50,
    priority = 3,
    burst_allowed = true
})

-- Assign users to tiers
multi_tier:set_user_tier("user1", "free")
multi_tier:set_user_tier("user2", "premium")
multi_tier:set_user_tier("user3", "enterprise")

print("   Testing multi-tier rate limiting:")

local users = {"user1", "user2", "user3"}
for _, user in ipairs(users) do
    local status = multi_tier:get_user_status(user)
    print(string.format("   %s (%s tier): %d/%d tokens", 
        user, status.tier, status.tokens_available, status.capacity))
    
    -- Try some requests
    for i = 1, 3 do
        local allowed, tier = multi_tier:try_request(user)
        print(string.format("     Request %d: %s", 
            i, allowed and "✅" or "❌"))
    end
end

print()

-- ============================================================
-- Pattern 5: Distributed Rate Limiting
-- ============================================================

print("5. Distributed Rate Limiting (Simulation)")
print("-" .. string.rep("-", 40))

local DistributedRateLimiter = {}
DistributedRateLimiter.__index = DistributedRateLimiter

function DistributedRateLimiter:new(options)
    options = options or {}
    return setmetatable({
        node_id = options.node_id or "node1",
        nodes = options.nodes or 3,
        global_limit = options.global_limit or 100,
        sync_interval = options.sync_interval or 1,  -- seconds
        local_counter = 0,
        global_counter = 0,
        last_sync = os.time(),
        node_quotas = {}
    }, self)
end

function DistributedRateLimiter:calculate_local_quota()
    -- Simple equal distribution
    return math.floor(self.global_limit / self.nodes)
end

function DistributedRateLimiter:sync_with_cluster()
    -- Simulate cluster synchronization
    local now = os.time()
    if (now - self.last_sync) >= self.sync_interval then
        print("   [" .. self.node_id .. "] Syncing with cluster...")
        
        -- Simulate getting global count
        self.global_counter = self.local_counter * self.nodes  -- Simplified
        
        -- Recalculate quota based on usage
        local remaining = self.global_limit - self.global_counter
        local local_quota = math.floor(remaining / self.nodes)
        
        -- Reset local counter
        self.local_counter = 0
        self.last_sync = now
        
        return local_quota
    end
    
    return self:calculate_local_quota()
end

function DistributedRateLimiter:try_request()
    local quota = self:sync_with_cluster()
    
    if self.local_counter < quota then
        self.local_counter = self.local_counter + 1
        return true, {
            node = self.node_id,
            local_count = self.local_counter,
            local_quota = quota,
            global_estimate = self.global_counter + self.local_counter
        }
    else
        return false, {
            node = self.node_id,
            reason = "Local quota exceeded",
            local_quota = quota
        }
    end
end

-- Test distributed rate limiter
local dist_limiter = DistributedRateLimiter:new({
    node_id = "api-server-1",
    nodes = 3,
    global_limit = 30,
    sync_interval = 5
})

print("   Testing distributed rate limiter (3 nodes, 30 global limit):")
print("   Node: " .. dist_limiter.node_id)
print("   Local quota: " .. dist_limiter:calculate_local_quota())

for i = 1, 12 do
    local allowed, info = dist_limiter:try_request()
    if allowed then
        print(string.format("   Request %d: ✅ (local: %d, global est: %d)", 
            i, info.local_count, info.global_estimate))
    else
        print(string.format("   Request %d: ❌ (%s)", 
            i, info.reason))
    end
end

print()
print("🎯 Key Takeaways:")
print("   • Token bucket allows burst traffic")
print("   • Sliding window provides accurate rate limiting")
print("   • Leaky bucket smooths traffic")
print("   • Multi-tier supports different user levels")
print("   • Distributed limiting scales horizontally")
print("   • Choose based on your specific needs")