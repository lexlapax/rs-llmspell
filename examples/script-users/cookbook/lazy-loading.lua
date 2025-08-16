-- Cookbook: Lazy Loading - Load Resources on Demand
-- Purpose: Implement lazy loading patterns to optimize resource usage and startup time
-- Prerequisites: Tools optional for real resource loading
-- Expected Output: Demonstration of lazy loading patterns
-- Version: 0.7.0
-- Tags: cookbook, lazy-loading, performance, optimization, resources

print("=== Lazy Loading Patterns ===\n")

-- ============================================================
-- Pattern 1: Simple Lazy Initialization
-- ============================================================

print("1. Simple Lazy Initialization")
print("-" .. string.rep("-", 40))

local LazyResource = {}
LazyResource.__index = LazyResource

function LazyResource:new(name, initializer)
    return setmetatable({
        name = name,
        initializer = initializer,
        value = nil,
        initialized = false,
        access_count = 0
    }, self)
end

function LazyResource:get()
    self.access_count = self.access_count + 1
    
    if not self.initialized then
        print(string.format("   💾 Initializing lazy resource: %s", self.name))
        self.value = self.initializer()
        self.initialized = true
    else
        print(string.format("   📦 Using cached resource: %s (access #%d)", 
            self.name, self.access_count))
    end
    
    return self.value
end

function LazyResource:is_initialized()
    return self.initialized
end

function LazyResource:invalidate()
    print(string.format("   🔄 Invalidating resource: %s", self.name))
    self.initialized = false
    self.value = nil
end

-- Test simple lazy initialization
local expensive_config = LazyResource:new("database_config", function()
    -- Simulate expensive initialization
    local end_time = os.clock() + 0.05 -- 50ms delay
    while os.clock() < end_time do end
    
    return {
        host = "db.example.com",
        port = 5432,
        pool_size = 10,
        timeout = 30
    }
end)

local api_client = LazyResource:new("api_client", function()
    -- Simulate API client initialization
    local end_time = os.clock() + 0.03 -- 30ms delay
    while os.clock() < end_time do end
    
    return {
        base_url = "https://api.example.com",
        auth_token = "lazy_token_123",
        rate_limit = 1000
    }
end)

print("   Testing lazy resource access:")
print("   Before initialization:")
print(string.format("     Config initialized: %s", expensive_config:is_initialized()))
print(string.format("     API client initialized: %s", api_client:is_initialized()))

print("\n   First access (triggers initialization):")
local config1 = expensive_config:get()
print(string.format("     Config host: %s", config1.host))

print("\n   Second access (uses cache):")
local config2 = expensive_config:get()
print(string.format("     Config timeout: %s", config2.timeout))

print("\n   API client access:")
local client = api_client:get()
print(string.format("     API base URL: %s", client.base_url))

print()

-- ============================================================
-- Pattern 2: Lazy Collection Loading
-- ============================================================

print("2. Lazy Collection Loading")
print("-" .. string.rep("-", 40))

local LazyCollection = {}
LazyCollection.__index = LazyCollection

function LazyCollection:new(name, loader_func, page_size)
    return setmetatable({
        name = name,
        loader_func = loader_func,
        page_size = page_size or 10,
        loaded_pages = {},
        total_items = nil,
        cache = {}
    }, self)
end

function LazyCollection:get_item(index)
    local page_number = math.floor((index - 1) / self.page_size) + 1
    
    if not self.loaded_pages[page_number] then
        self:load_page(page_number)
    end
    
    local item_in_page = ((index - 1) % self.page_size) + 1
    local page_data = self.loaded_pages[page_number]
    
    if page_data and page_data.items and page_data.items[item_in_page] then
        return page_data.items[item_in_page]
    end
    
    return nil
end

function LazyCollection:load_page(page_number)
    print(string.format("   📄 Loading page %d of %s", page_number, self.name))
    
    local start_index = (page_number - 1) * self.page_size + 1
    local page_data = self.loader_func(start_index, self.page_size)
    
    self.loaded_pages[page_number] = page_data
    
    if page_data.total_items then
        self.total_items = page_data.total_items
    end
    
    return page_data
end

function LazyCollection:get_loaded_pages_info()
    local loaded_count = 0
    for _ in pairs(self.loaded_pages) do
        loaded_count = loaded_count + 1
    end
    
    local total_pages = self.total_items and math.ceil(self.total_items / self.page_size) or "unknown"
    
    return {
        loaded_pages = loaded_count,
        total_pages = total_pages,
        page_size = self.page_size
    }
end

function LazyCollection:iterate_range(start_index, end_index)
    local results = {}
    for i = start_index, end_index do
        local item = self:get_item(i)
        if item then
            table.insert(results, item)
        end
    end
    return results
end

-- Test lazy collection loading
local user_collection = LazyCollection:new("users", function(start_index, page_size)
    -- Simulate database query
    local end_time = os.clock() + 0.02 -- 20ms delay
    while os.clock() < end_time do end
    
    local items = {}
    for i = 1, page_size do
        local user_id = start_index + i - 1
        if user_id <= 47 then -- Simulate 47 total users
            table.insert(items, {
                id = user_id,
                name = "User" .. user_id,
                email = "user" .. user_id .. "@example.com"
            })
        end
    end
    
    return {
        items = items,
        total_items = 47,
        page_number = math.floor((start_index - 1) / page_size) + 1
    }
end, 5) -- Page size of 5

print("   Testing lazy collection access:")

-- Access specific items (should trigger page loads)
local user1 = user_collection:get_item(1)
print(string.format("   User 1: %s (%s)", user1.name, user1.email))

local user7 = user_collection:get_item(7)
print(string.format("   User 7: %s (%s)", user7.name, user7.email))

local user3 = user_collection:get_item(3)
print(string.format("   User 3: %s (%s)", user3.name, user3.email))

local info = user_collection:get_loaded_pages_info()
print(string.format("   Loaded: %d/%s pages", info.loaded_pages, tostring(info.total_pages)))

-- Iterate over a range (may trigger additional page loads)
print("\n   Iterating over range 12-16:")
local range_users = user_collection:iterate_range(12, 16)
for _, user in ipairs(range_users) do
    print(string.format("     %s", user.name))
end

local final_info = user_collection:get_loaded_pages_info()
print(string.format("   Final loaded: %d/%s pages", final_info.loaded_pages, tostring(final_info.total_pages)))

print()

-- ============================================================
-- Pattern 3: Lazy Component Loading
-- ============================================================

print("3. Lazy Component Loading")
print("-" .. string.rep("-", 40))

local ComponentLoader = {}
ComponentLoader.__index = ComponentLoader

function ComponentLoader:new()
    return setmetatable({
        components = {},
        dependencies = {},
        loading_stack = {}
    }, self)
end

function ComponentLoader:register_component(name, loader_func, dependencies)
    self.components[name] = {
        name = name,
        loader = loader_func,
        instance = nil,
        loaded = false,
        loading = false
    }
    
    self.dependencies[name] = dependencies or {}
    print(string.format("   📋 Registered component: %s", name))
end

function ComponentLoader:load_component(name)
    local component = self.components[name]
    if not component then
        error("Component not found: " .. name)
    end
    
    if component.loaded then
        return component.instance
    end
    
    if component.loading then
        error("Circular dependency detected for component: " .. name)
    end
    
    -- Check for circular dependencies
    for _, loading_component in ipairs(self.loading_stack) do
        if loading_component == name then
            error("Circular dependency detected: " .. table.concat(self.loading_stack, " -> ") .. " -> " .. name)
        end
    end
    
    table.insert(self.loading_stack, name)
    component.loading = true
    
    print(string.format("   🔧 Loading component: %s", name))
    
    -- Load dependencies first
    local resolved_dependencies = {}
    for _, dep_name in ipairs(self.dependencies[name]) do
        print(string.format("     📦 Loading dependency: %s", dep_name))
        resolved_dependencies[dep_name] = self:load_component(dep_name)
    end
    
    -- Load the component itself
    component.instance = component.loader(resolved_dependencies)
    component.loaded = true
    component.loading = false
    
    table.remove(self.loading_stack)
    
    print(string.format("   ✅ Loaded component: %s", name))
    return component.instance
end

function ComponentLoader:get_component(name)
    return self:load_component(name)
end

function ComponentLoader:is_loaded(name)
    local component = self.components[name]
    return component and component.loaded
end

function ComponentLoader:get_load_order()
    local order = {}
    for name, component in pairs(self.components) do
        if component.loaded then
            table.insert(order, name)
        end
    end
    return order
end

-- Test lazy component loading
local loader = ComponentLoader:new()

-- Register components with dependencies
loader:register_component("config", function(deps)
    return {
        database_url = "postgresql://localhost:5432/app",
        cache_ttl = 300,
        debug_mode = true
    }
end)

loader:register_component("logger", function(deps)
    local config = deps.config
    return {
        level = config.debug_mode and "debug" or "info",
        output = "console",
        format = "json"
    }
end, {"config"})

loader:register_component("database", function(deps)
    local config = deps.config
    local logger = deps.logger
    
    -- Simulate database connection setup
    return {
        url = config.database_url,
        pool_size = 10,
        connected = true,
        logger = logger
    }
end, {"config", "logger"})

loader:register_component("cache", function(deps)
    local config = deps.config
    return {
        ttl = config.cache_ttl,
        size_limit = 1000,
        type = "memory"
    }
end, {"config"})

loader:register_component("user_service", function(deps)
    local database = deps.database
    local cache = deps.cache
    local logger = deps.logger
    
    return {
        db = database,
        cache = cache,
        logger = logger,
        methods = {"create", "read", "update", "delete"}
    }
end, {"database", "cache", "logger"})

print("   Testing lazy component loading:")
print("   Component loading status before use:")
for _, name in ipairs({"config", "logger", "database", "cache", "user_service"}) do
    print(string.format("     %s: %s", name, loader:is_loaded(name)))
end

print("\n   Requesting user_service (should trigger dependency chain):")
local user_service = loader:get_component("user_service")
print(string.format("   User service has %d methods", #user_service.methods))
print(string.format("   Database connected: %s", user_service.db.connected))

print("\n   Load order:", table.concat(loader:get_load_order(), " -> "))

print()

-- ============================================================
-- Pattern 4: Smart Caching with TTL
-- ============================================================

print("4. Smart Caching with TTL and Refresh")
print("-" .. string.rep("-", 40))

local SmartCache = {}
SmartCache.__index = SmartCache

function SmartCache:new(default_ttl)
    return setmetatable({
        cache = {},
        default_ttl = default_ttl or 300, -- 5 minutes
        access_stats = {}
    }, self)
end

function SmartCache:get(key, loader_func, ttl)
    ttl = ttl or self.default_ttl
    local current_time = os.time()
    
    -- Track access
    if not self.access_stats[key] then
        self.access_stats[key] = {count = 0, last_access = current_time}
    end
    self.access_stats[key].count = self.access_stats[key].count + 1
    self.access_stats[key].last_access = current_time
    
    local cached_item = self.cache[key]
    
    -- Check if item exists and is still valid
    if cached_item and (current_time - cached_item.timestamp) < ttl then
        print(string.format("   📦 Cache hit: %s (age: %ds)", key, 
            current_time - cached_item.timestamp))
        return cached_item.value
    end
    
    -- Load/reload the item
    if cached_item then
        print(string.format("   🔄 Cache expired: %s (age: %ds, ttl: %ds)", key,
            current_time - cached_item.timestamp, ttl))
    else
        print(string.format("   💾 Cache miss: %s", key))
    end
    
    local value = loader_func()
    
    self.cache[key] = {
        value = value,
        timestamp = current_time,
        ttl = ttl,
        size = self:estimate_size(value)
    }
    
    return value
end

function SmartCache:estimate_size(value)
    -- Simple size estimation
    if type(value) == "string" then
        return #value
    elseif type(value) == "table" then
        local size = 0
        for k, v in pairs(value) do
            size = size + self:estimate_size(k) + self:estimate_size(v)
        end
        return size
    else
        return 8 -- Rough estimate for other types
    end
end

function SmartCache:invalidate(key)
    if self.cache[key] then
        print(string.format("   🗑️  Invalidated: %s", key))
        self.cache[key] = nil
        return true
    end
    return false
end

function SmartCache:get_cache_stats()
    local current_time = os.time()
    local stats = {
        total_items = 0,
        expired_items = 0,
        total_size = 0,
        access_stats = self.access_stats
    }
    
    for key, item in pairs(self.cache) do
        stats.total_items = stats.total_items + 1
        stats.total_size = stats.total_size + (item.size or 0)
        
        if (current_time - item.timestamp) >= item.ttl then
            stats.expired_items = stats.expired_items + 1
        end
    end
    
    return stats
end

function SmartCache:cleanup_expired()
    local current_time = os.time()
    local removed = 0
    
    for key, item in pairs(self.cache) do
        if (current_time - item.timestamp) >= item.ttl then
            self.cache[key] = nil
            removed = removed + 1
        end
    end
    
    if removed > 0 then
        print(string.format("   🧹 Cleaned up %d expired items", removed))
    end
    
    return removed
end

-- Test smart caching
local cache = SmartCache:new(3) -- 3 second TTL

print("   Testing smart cache with TTL:")

-- Define some expensive operations
local function load_user_data(user_id)
    -- Simulate expensive database query
    local end_time = os.clock() + 0.03 -- 30ms
    while os.clock() < end_time do end
    
    return {
        id = user_id,
        name = "User " .. user_id,
        email = "user" .. user_id .. "@example.com",
        last_login = os.date("%Y-%m-%d %H:%M:%S")
    }
end

local function load_analytics_data()
    -- Simulate expensive analytics computation
    local end_time = os.clock() + 0.05 -- 50ms
    while os.clock() < end_time do end
    
    return {
        daily_users = math.random(1000, 5000),
        page_views = math.random(10000, 50000),
        conversion_rate = math.random(100, 500) / 100
    }
end

-- Test caching behavior
local user1 = cache:get("user:123", function() return load_user_data(123) end)
print(string.format("   User: %s", user1.name))

local user1_again = cache:get("user:123", function() return load_user_data(123) end)
print(string.format("   User again: %s", user1_again.name))

local analytics = cache:get("analytics:daily", function() return load_analytics_data() end, 5) -- Custom TTL
print(string.format("   Daily users: %d", analytics.daily_users))

-- Wait and test expiration
print("\n   Waiting for cache expiration...")
local wait_end = os.time() + 4 -- Wait 4 seconds
while os.time() < wait_end do end

local stats_before = cache:get_cache_stats()
print(string.format("   Before cleanup: %d items (%d expired)", 
    stats_before.total_items, stats_before.expired_items))

cache:cleanup_expired()

local user1_after_expire = cache:get("user:123", function() return load_user_data(123) end)
print(string.format("   User after expiry: %s", user1_after_expire.name))

local final_stats = cache:get_cache_stats()
print(string.format("   Final stats: %d items, %d bytes", 
    final_stats.total_items, final_stats.total_size))

print()
print("🎯 Key Takeaways:")
print("   • Use lazy loading to defer expensive operations")
print("   • Implement pagination for large collections")
print("   • Manage component dependencies carefully")
print("   • Cache with appropriate TTL based on data volatility")
print("   • Track access patterns to optimize cache strategies")
print("   • Clean up expired cache entries regularly")