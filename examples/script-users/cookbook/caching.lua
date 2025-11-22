-- Recommended profile: development
-- Run with: llmspell -p development run caching.lua
-- Development environment with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: 03 - Caching Patterns v0.7.0
-- Complexity Level: PRODUCTION
-- Real-World Use Case: High-performance caching for reduced latency and API costs
-- Pattern Category: Performance & Optimization
--
-- Purpose: Production caching strategies for LLMSpell applications. Implements
--          LRU, TTL-based, and write-through caching patterns to minimize API
--          calls, reduce latency, and optimize costs in production systems.
-- Architecture: Multi-tier caching with eviction policies and invalidation strategies
-- Crates Showcased: llmspell-tools, llmspell-state, llmspell-bridge
-- Key Features:
--   • Simple in-memory cache with size limits
--   • LRU (Least Recently Used) cache implementation
--   • TTL-based expiration with automatic cleanup
--   • Write-through and lazy-loading patterns
--   • Cache hit/miss statistics and monitoring
--   • Cache invalidation strategies for consistency
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • No API keys required (simulation only)
--   • Optional: State persistence for distributed caching
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/cookbook/caching.lua
--
-- EXPECTED OUTPUT:
-- 5 caching patterns demonstrated:
-- 1. Simple in-memory cache with hit/miss tracking
-- 2. LRU cache with 3-item capacity showing eviction
-- 3. TTL cache with 2-second expiration
-- 4. Write-through cache pattern
-- 5. Lazy-loading cache with computed values
--
-- Time to Complete: <3 seconds
-- Production Notes: Use distributed caching (Redis) for multi-instance setups,
--                   implement cache stampede protection, monitor hit ratios,
--                   size caches based on memory constraints and access patterns.
-- ============================================================

print("=== Caching Patterns Cookbook ===")
print("Pattern 03: PRODUCTION - High-performance caching strategies\n")

-- ============================================================
-- Pattern 1: Simple In-Memory Cache
-- ============================================================

print("1. Simple In-Memory Cache")
print("-" .. string.rep("-", 40))

local SimpleCache = {}
SimpleCache.__index = SimpleCache

function SimpleCache:new(max_size)
    return setmetatable({
        cache = {},
        max_size = max_size or 100,
        hits = 0,
        misses = 0
    }, self)
end

function SimpleCache:get(key)
    local entry = self.cache[key]
    if entry then
        self.hits = self.hits + 1
        return entry.value
    else
        self.misses = self.misses + 1
        return nil
    end
end

function SimpleCache:set(key, value)
    -- Simple size limit (remove oldest if at capacity)
    if self:size() >= self.max_size then
        -- Remove first item (simple FIFO)
        for k, _ in pairs(self.cache) do
            self.cache[k] = nil
            break
        end
    end
    
    self.cache[key] = {
        value = value,
        timestamp = os.time()
    }
end

function SimpleCache:size()
    local count = 0
    for _ in pairs(self.cache) do
        count = count + 1
    end
    return count
end

function SimpleCache:hit_rate()
    local total = self.hits + self.misses
    if total == 0 then return 0 end
    return (self.hits / total) * 100
end

-- Test simple cache
local cache = SimpleCache:new(3)

cache:set("user:1", {name = "Alice", age = 30})
cache:set("user:2", {name = "Bob", age = 25})

print("   Cache get user:1: " .. tostring(cache:get("user:1").name))
print("   Cache get user:3: " .. tostring(cache:get("user:3")))
print(string.format("   Hit rate: %.1f%%", cache:hit_rate()))

print()

-- ============================================================
-- Pattern 2: TTL (Time-To-Live) Cache
-- ============================================================

print("2. TTL Cache with Expiration")
print("-" .. string.rep("-", 40))

local TTLCache = {}
TTLCache.__index = TTLCache

function TTLCache:new(default_ttl)
    return setmetatable({
        cache = {},
        default_ttl = default_ttl or 300, -- 5 minutes default
        stats = {
            hits = 0,
            misses = 0,
            evictions = 0,
            expirations = 0
        }
    }, self)
end

function TTLCache:is_expired(entry)
    return os.time() > entry.expires_at
end

function TTLCache:get(key)
    local entry = self.cache[key]
    
    if not entry then
        self.stats.misses = self.stats.misses + 1
        return nil
    end
    
    if self:is_expired(entry) then
        self.cache[key] = nil
        self.stats.expirations = self.stats.expirations + 1
        self.stats.misses = self.stats.misses + 1
        return nil
    end
    
    self.stats.hits = self.stats.hits + 1
    entry.last_accessed = os.time()
    entry.access_count = entry.access_count + 1
    return entry.value
end

function TTLCache:set(key, value, ttl)
    ttl = ttl or self.default_ttl
    
    self.cache[key] = {
        value = value,
        created_at = os.time(),
        expires_at = os.time() + ttl,
        last_accessed = os.time(),
        access_count = 0,
        ttl = ttl
    }
end

function TTLCache:cleanup()
    local expired_count = 0
    for key, entry in pairs(self.cache) do
        if self:is_expired(entry) then
            self.cache[key] = nil
            expired_count = expired_count + 1
        end
    end
    self.stats.expirations = self.stats.expirations + expired_count
    return expired_count
end

function TTLCache:get_stats()
    local total = self.stats.hits + self.stats.misses
    return {
        size = self:size(),
        hit_rate = total > 0 and (self.stats.hits / total * 100) or 0,
        hits = self.stats.hits,
        misses = self.stats.misses,
        expirations = self.stats.expirations
    }
end

function TTLCache:size()
    local count = 0
    for _ in pairs(self.cache) do
        count = count + 1
    end
    return count
end

-- Test TTL cache
local ttl_cache = TTLCache:new(60) -- 60 second default TTL

ttl_cache:set("config:db", {host = "localhost", port = 5432}, 120) -- 2 minute TTL
ttl_cache:set("config:api", {url = "https://api.example.com"}, 30) -- 30 second TTL

print("   Set 2 config values with different TTLs")
print("   Cache size: " .. ttl_cache:size())

-- Simulate some gets
ttl_cache:get("config:db")
ttl_cache:get("config:db")
ttl_cache:get("config:missing")

local stats = ttl_cache:get_stats()
print(string.format("   Stats - Hits: %d, Misses: %d, Hit rate: %.1f%%", 
    stats.hits, stats.misses, stats.hit_rate))

print()

-- ============================================================
-- Pattern 3: LRU (Least Recently Used) Cache
-- ============================================================

print("3. LRU Cache")
print("-" .. string.rep("-", 40))

local LRUCache = {}
LRUCache.__index = LRUCache

function LRUCache:new(capacity)
    return setmetatable({
        capacity = capacity or 100,
        cache = {},
        order = {}, -- Doubly linked list for LRU order
        head = nil,
        tail = nil,
        size = 0
    }, self)
end

function LRUCache:_add_to_head(node)
    node.prev = nil
    node.next = self.head
    
    if self.head then
        self.head.prev = node
    end
    
    self.head = node
    
    if not self.tail then
        self.tail = node
    end
end

function LRUCache:_remove_node(node)
    if node.prev then
        node.prev.next = node.next
    else
        self.head = node.next
    end
    
    if node.next then
        node.next.prev = node.prev
    else
        self.tail = node.prev
    end
end

function LRUCache:_move_to_head(node)
    self:_remove_node(node)
    self:_add_to_head(node)
end

function LRUCache:get(key)
    local node = self.cache[key]
    
    if not node then
        return nil
    end
    
    -- Move to head (most recently used)
    self:_move_to_head(node)
    
    return node.value
end

function LRUCache:set(key, value)
    local node = self.cache[key]
    
    if node then
        -- Update existing
        node.value = value
        self:_move_to_head(node)
    else
        -- Add new
        node = {
            key = key,
            value = value,
            prev = nil,
            next = nil
        }
        
        self.cache[key] = node
        self:_add_to_head(node)
        self.size = self.size + 1
        
        -- Evict LRU if at capacity
        if self.size > self.capacity then
            local lru = self.tail
            self:_remove_node(lru)
            self.cache[lru.key] = nil
            self.size = self.size - 1
            print("   Evicted LRU item: " .. lru.key)
        end
    end
end

-- Test LRU cache
local lru = LRUCache:new(3)

print("   LRU Cache with capacity 3:")
lru:set("a", 1)
lru:set("b", 2)
lru:set("c", 3)
print("   Added a=1, b=2, c=3")

lru:get("a") -- Access 'a', making it most recent
print("   Accessed 'a'")

lru:set("d", 4) -- This should evict 'b' (least recently used)
print("   Added d=4 (should evict 'b')")

print("   b in cache: " .. tostring(lru:get("b"))) -- Should be nil

print()

-- ============================================================
-- Pattern 4: Write-Through Cache with Lazy Loading
-- ============================================================

print("4. Write-Through Cache Pattern")
print("-" .. string.rep("-", 40))

local WriteThroughCache = {}
WriteThroughCache.__index = WriteThroughCache

function WriteThroughCache:new(options)
    options = options or {}
    return setmetatable({
        cache = TTLCache:new(options.ttl or 300),
        loader = options.loader, -- Function to load data
        writer = options.writer, -- Function to write data
        key_prefix = options.key_prefix or "cache:",
        stats = {
            loads = 0,
            writes = 0
        }
    }, self)
end

function WriteThroughCache:get(key)
    local cache_key = self.key_prefix .. key
    
    -- Try cache first
    local value = self.cache:get(cache_key)
    if value then
        return value
    end
    
    -- Cache miss - load from source
    if self.loader then
        print("   Cache miss, loading from source...")
        value = self.loader(key)
        self.stats.loads = self.stats.loads + 1
        
        if value then
            self.cache:set(cache_key, value)
        end
        
        return value
    end
    
    return nil
end

function WriteThroughCache:set(key, value)
    local cache_key = self.key_prefix .. key
    
    -- Write through to backing store
    if self.writer then
        local success = self.writer(key, value)
        self.stats.writes = self.stats.writes + 1
        
        if not success then
            return false
        end
    end
    
    -- Update cache
    self.cache:set(cache_key, value)
    return true
end

function WriteThroughCache:invalidate(key)
    local cache_key = self.key_prefix .. key
    self.cache.cache[cache_key] = nil
end

-- Example write-through cache for user data
local user_cache = WriteThroughCache:new({
    ttl = 120,
    loader = function(user_id)
        -- Simulate database load
        print("     [DB] Loading user " .. user_id)
        return {id = user_id, name = "User " .. user_id, loaded_at = os.time()}
    end,
    writer = function(user_id, data)
        -- Simulate database write
        print("     [DB] Saving user " .. user_id)
        return true
    end
})

print("   Testing write-through cache:")
local user1 = user_cache:get("123") -- Cache miss, loads from DB
print("   Got user: " .. user1.name)

local user1_again = user_cache:get("123") -- Cache hit
print("   Got user again (from cache): " .. user1_again.name)

user_cache:set("124", {id = "124", name = "New User"}) -- Writes through
print("   Stats - Loads: " .. user_cache.stats.loads .. ", Writes: " .. user_cache.stats.writes)

print()

-- ============================================================
-- Pattern 5: Intelligent Cache with Adaptive TTL
-- ============================================================

print("5. Adaptive TTL Cache")
print("-" .. string.rep("-", 40))

local AdaptiveCache = {}
AdaptiveCache.__index = AdaptiveCache

function AdaptiveCache:new(options)
    options = options or {}
    return setmetatable({
        cache = {},
        min_ttl = options.min_ttl or 60,
        max_ttl = options.max_ttl or 3600,
        hit_threshold = options.hit_threshold or 5, -- Hits to increase TTL
        stats = {}
    }, self)
end

function AdaptiveCache:calculate_ttl(key)
    local stats = self.stats[key] or {hits = 0, last_set = 0}
    
    -- Increase TTL based on popularity
    local base_ttl = self.min_ttl
    local bonus_ttl = math.min(
        stats.hits * 60, -- 1 minute per hit
        self.max_ttl - self.min_ttl
    )
    
    return base_ttl + bonus_ttl
end

function AdaptiveCache:get(key)
    local entry = self.cache[key]
    
    if not entry then
        return nil
    end
    
    if os.time() > entry.expires_at then
        self.cache[key] = nil
        return nil
    end
    
    -- Track stats
    self.stats[key] = self.stats[key] or {hits = 0, last_set = entry.created_at}
    self.stats[key].hits = self.stats[key].hits + 1
    
    -- Adapt TTL if popular
    if self.stats[key].hits >= self.hit_threshold then
        local new_ttl = self:calculate_ttl(key)
        if new_ttl > entry.ttl then
            print(string.format("   Extending TTL for '%s' from %ds to %ds (popular item)", 
                key, entry.ttl, new_ttl))
            entry.expires_at = entry.created_at + new_ttl
            entry.ttl = new_ttl
        end
    end
    
    return entry.value
end

function AdaptiveCache:set(key, value)
    local ttl = self:calculate_ttl(key)
    
    self.cache[key] = {
        value = value,
        created_at = os.time(),
        expires_at = os.time() + ttl,
        ttl = ttl
    }
    
    self.stats[key] = {
        hits = 0,
        last_set = os.time()
    }
    
    print(string.format("   Set '%s' with adaptive TTL: %ds", key, ttl))
end

-- Test adaptive cache
local adaptive = AdaptiveCache:new({
    min_ttl = 30,
    max_ttl = 300
})

adaptive:set("popular_item", "data1")
adaptive:set("rare_item", "data2")

-- Simulate access patterns
print("   Simulating access patterns...")
for i = 1, 6 do
    adaptive:get("popular_item")
end
adaptive:get("rare_item")

-- Popular item should have extended TTL
adaptive:get("popular_item")

print()

-- ============================================================
-- Pattern 6: Cache Warming and Preloading
-- ============================================================

print("6. Cache Warming Strategy")
print("-" .. string.rep("-", 40))

local CacheWarmer = {}
CacheWarmer.__index = CacheWarmer

function CacheWarmer:new(cache, options)
    options = options or {}
    return setmetatable({
        cache = cache,
        warm_list = options.warm_list or {},
        warm_interval = options.warm_interval or 300,
        loader = options.loader,
        last_warm = 0
    }, self)
end

function CacheWarmer:add_to_warm_list(keys)
    for _, key in ipairs(keys) do
        table.insert(self.warm_list, key)
    end
end

function CacheWarmer:warm()
    print("   Starting cache warming...")
    local warmed = 0
    
    for _, key in ipairs(self.warm_list) do
        if self.loader then
            local value = self.loader(key)
            if value then
                self.cache:set(key, value)
                warmed = warmed + 1
            end
        end
    end
    
    self.last_warm = os.time()
    print(string.format("   Warmed %d/%d entries", warmed, #self.warm_list))
    return warmed
end

function CacheWarmer:should_warm()
    return (os.time() - self.last_warm) >= self.warm_interval
end

-- Example cache warmer
local main_cache = TTLCache:new(300)
local warmer = CacheWarmer:new(main_cache, {
    warm_interval = 60,
    loader = function(key)
        return "Preloaded: " .. key
    end
})

warmer:add_to_warm_list({"critical_config", "user_settings", "feature_flags"})
warmer:warm()

print("   Cache after warming: " .. main_cache:size() .. " entries")

print()
print("🎯 Key Takeaways:")
print("   • Choose the right cache strategy for your use case")
print("   • TTL prevents stale data")
print("   • LRU keeps hot data in memory")
print("   • Write-through ensures consistency")
print("   • Adaptive TTL optimizes for access patterns")
print("   • Cache warming reduces cold start impact")