-- Cookbook: API Gateway Patterns - Managing API Interactions
-- Purpose: Implement patterns for API gateway functionality and service orchestration
-- Prerequisites: Tools: http_request (for production use), network access for external APIs
-- Expected Output: Demonstration of API gateway patterns
-- Version: 0.7.0
-- Tags: cookbook, api, gateway, routing, production

print("=== API Gateway Patterns ===\n")

-- ============================================================
-- Pattern 1: Request Routing and Load Balancing
-- ============================================================

print("1. Request Routing and Load Balancing")
print("-" .. string.rep("-", 40))

local APIGateway = {}
APIGateway.__index = APIGateway

function APIGateway:new()
    return setmetatable({
        routes = {},
        backends = {},
        middleware = {},
        load_balancers = {},
        stats = {
            requests = 0,
            errors = 0,
            latency_sum = 0
        }
    }, self)
end

function APIGateway:register_route(pattern, backend_pool, options)
    options = options or {}
    
    self.routes[pattern] = {
        pattern = pattern,
        backend_pool = backend_pool,
        method = options.method or "*",
        auth_required = options.auth_required or false,
        rate_limit = options.rate_limit,
        transform = options.transform
    }
    
    print(string.format("   Registered route: %s -> %s", pattern, backend_pool))
end

function APIGateway:register_backend(pool_name, endpoints, strategy)
    strategy = strategy or "round_robin"
    
    self.backends[pool_name] = {
        endpoints = endpoints,
        current = 1,
        strategy = strategy,
        health = {}
    }
    
    -- Initialize health status
    for i, endpoint in ipairs(endpoints) do
        self.backends[pool_name].health[i] = {
            healthy = true,
            failures = 0,
            last_check = os.time()
        }
    end
    
    print(string.format("   Registered backend pool '%s' with %d endpoints (%s)", 
        pool_name, #endpoints, strategy))
end

function APIGateway:select_backend(pool_name)
    local pool = self.backends[pool_name]
    if not pool then
        return nil, "Backend pool not found"
    end
    
    if pool.strategy == "round_robin" then
        -- Round robin with health check
        local attempts = 0
        while attempts < #pool.endpoints do
            local idx = pool.current
            pool.current = (pool.current % #pool.endpoints) + 1
            
            if pool.health[idx].healthy then
                return pool.endpoints[idx], idx
            end
            
            attempts = attempts + 1
        end
        
        return nil, "No healthy backends"
        
    elseif pool.strategy == "random" then
        -- Random selection
        local healthy = {}
        for i, endpoint in ipairs(pool.endpoints) do
            if pool.health[i].healthy then
                table.insert(healthy, {endpoint = endpoint, idx = i})
            end
        end
        
        if #healthy > 0 then
            local selected = healthy[math.random(1, #healthy)]
            return selected.endpoint, selected.idx
        end
        
        return nil, "No healthy backends"
        
    elseif pool.strategy == "least_connections" then
        -- Simulated least connections
        local best = nil
        local best_idx = nil
        local min_conn = math.huge
        
        for i, endpoint in ipairs(pool.endpoints) do
            if pool.health[i].healthy then
                local connections = pool.health[i].connections or 0
                if connections < min_conn then
                    min_conn = connections
                    best = endpoint
                    best_idx = i
                end
            end
        end
        
        return best, best_idx
    end
    
    return nil, "Unknown strategy"
end

function APIGateway:route_request(request)
    self.stats.requests = self.stats.requests + 1
    local start_time = os.clock()
    
    -- Find matching route
    local route = nil
    for pattern, r in pairs(self.routes) do
        if string.match(request.path, pattern) then
            route = r
            break
        end
    end
    
    if not route then
        self.stats.errors = self.stats.errors + 1
        return {
            status = 404,
            error = "No route found for " .. request.path
        }
    end
    
    -- Apply middleware
    for _, mw in ipairs(self.middleware) do
        local result = mw(request, route)
        if result and result.status ~= 200 then
            return result
        end
    end
    
    -- Select backend
    local backend, idx = self:select_backend(route.backend_pool)
    if not backend then
        self.stats.errors = self.stats.errors + 1
        return {
            status = 503,
            error = "No available backends"
        }
    end
    
    -- Transform request if needed
    if route.transform then
        request = route.transform(request)
    end
    
    -- Simulate forwarding to backend
    print(string.format("   Routing %s %s -> %s", 
        request.method or "GET", request.path, backend))
    
    -- Update stats
    local latency = os.clock() - start_time
    self.stats.latency_sum = self.stats.latency_sum + latency
    
    return {
        status = 200,
        backend = backend,
        latency = latency
    }
end

function APIGateway:add_middleware(middleware_fn)
    table.insert(self.middleware, middleware_fn)
end

-- Test API Gateway
local gateway = APIGateway:new()

-- Register backends
gateway:register_backend("api_v1", {
    "http://api1.example.com",
    "http://api2.example.com",
    "http://api3.example.com"
}, "round_robin")

gateway:register_backend("api_v2", {
    "http://api-v2-1.example.com",
    "http://api-v2-2.example.com"
}, "random")

-- Register routes
gateway:register_route("/api/v1/.*", "api_v1", {
    auth_required = true,
    rate_limit = 100
})

gateway:register_route("/api/v2/.*", "api_v2", {
    auth_required = false,
    rate_limit = 200
})

-- Add middleware for auth checking
gateway:add_middleware(function(request, route)
    if route.auth_required and not request.headers then
        return {status = 401, error = "Authentication required"}
    end
    return nil
end)

-- Test routing
print("\n   Testing request routing:")

local requests = {
    {path = "/api/v1/users", method = "GET", headers = {auth = "token"}},
    {path = "/api/v1/orders", method = "POST", headers = {auth = "token"}},
    {path = "/api/v2/products", method = "GET"},
    {path = "/api/v1/secure", method = "GET"}  -- No auth header
}

for _, req in ipairs(requests) do
    local response = gateway:route_request(req)
    if response.status == 200 then
        print(string.format("   ✅ %s -> %s (%.3fms)", 
            req.path, response.backend, response.latency * 1000))
    else
        print(string.format("   ❌ %s -> %d: %s", 
            req.path, response.status, response.error))
    end
end

print()

-- ============================================================
-- Pattern 2: Service Discovery and Health Checking
-- ============================================================

print("2. Service Discovery and Health Checking")
print("-" .. string.rep("-", 40))

local ServiceDiscovery = {}
ServiceDiscovery.__index = ServiceDiscovery

function ServiceDiscovery:new()
    return setmetatable({
        services = {},
        health_checks = {},
        check_interval = 30,  -- seconds
        last_check = {}
    }, self)
end

function ServiceDiscovery:register_service(name, config)
    self.services[name] = {
        name = name,
        instances = {},
        version = config.version or "1.0.0",
        metadata = config.metadata or {},
        health_check = config.health_check
    }
    
    print(string.format("   Registered service: %s (v%s)", name, config.version or "1.0.0"))
end

function ServiceDiscovery:register_instance(service_name, instance)
    local service = self.services[service_name]
    if not service then
        return false, "Service not found"
    end
    
    instance.id = instance.id or (service_name .. "_" .. os.time())
    instance.registered_at = os.time()
    instance.status = "healthy"
    instance.health_score = 100
    
    table.insert(service.instances, instance)
    
    print(string.format("   Registered instance %s for service %s", 
        instance.id, service_name))
    
    return instance.id
end

function ServiceDiscovery:health_check(service_name)
    local service = self.services[service_name]
    if not service then
        return
    end
    
    print(string.format("   Checking health for service: %s", service_name))
    
    for _, instance in ipairs(service.instances) do
        -- Simulate health check
        local healthy = math.random() > 0.1  -- 90% healthy
        
        if healthy then
            instance.status = "healthy"
            instance.health_score = math.min(100, instance.health_score + 10)
        else
            instance.status = "unhealthy"
            instance.health_score = math.max(0, instance.health_score - 20)
        end
        
        instance.last_check = os.time()
        
        print(string.format("     Instance %s: %s (score: %d)", 
            instance.id, instance.status, instance.health_score))
    end
    
    self.last_check[service_name] = os.time()
end

function ServiceDiscovery:discover(service_name, requirements)
    local service = self.services[service_name]
    if not service then
        return nil, "Service not found"
    end
    
    requirements = requirements or {}
    local candidates = {}
    
    for _, instance in ipairs(service.instances) do
        local suitable = true
        
        -- Check health
        if requirements.min_health_score then
            if instance.health_score < requirements.min_health_score then
                suitable = false
            end
        end
        
        -- Check metadata requirements
        if requirements.metadata then
            for key, value in pairs(requirements.metadata) do
                if instance.metadata[key] ~= value then
                    suitable = false
                    break
                end
            end
        end
        
        if suitable and instance.status == "healthy" then
            table.insert(candidates, instance)
        end
    end
    
    if #candidates == 0 then
        return nil, "No suitable instances found"
    end
    
    -- Return best candidate (highest health score)
    table.sort(candidates, function(a, b)
        return a.health_score > b.health_score
    end)
    
    return candidates[1]
end

-- Test service discovery
local discovery = ServiceDiscovery:new()

-- Register services
discovery:register_service("user-service", {
    version = "2.1.0",
    health_check = "/health"
})

discovery:register_service("order-service", {
    version = "1.5.0",
    health_check = "/status"
})

-- Register instances
discovery:register_instance("user-service", {
    host = "user1.example.com",
    port = 8080,
    metadata = {region = "us-east"}
})

discovery:register_instance("user-service", {
    host = "user2.example.com",
    port = 8080,
    metadata = {region = "us-west"}
})

discovery:register_instance("order-service", {
    host = "order1.example.com",
    port = 9090,
    metadata = {region = "us-east"}
})

-- Perform health checks
discovery:health_check("user-service")
discovery:health_check("order-service")

-- Discover services
print("\n   Service discovery:")
local instance, err = discovery:discover("user-service", {
    min_health_score = 50
})

if instance then
    print(string.format("   Found user-service: %s:%d (health: %d)", 
        instance.host, instance.port, instance.health_score))
else
    print("   Error: " .. err)
end

print()

-- ============================================================
-- Pattern 3: Request/Response Transformation
-- ============================================================

print("3. Request/Response Transformation")
print("-" .. string.rep("-", 40))

local Transformer = {}
Transformer.__index = Transformer

function Transformer:new()
    return setmetatable({
        request_transforms = {},
        response_transforms = {},
        mappings = {}
    }, self)
end

function Transformer:add_request_transform(name, transform_fn)
    self.request_transforms[name] = transform_fn
end

function Transformer:add_response_transform(name, transform_fn)
    self.response_transforms[name] = transform_fn
end

function Transformer:transform_request(request, transforms)
    local transformed = request
    
    for _, transform_name in ipairs(transforms) do
        local transform_fn = self.request_transforms[transform_name]
        if transform_fn then
            transformed = transform_fn(transformed)
            print(string.format("   Applied request transform: %s", transform_name))
        end
    end
    
    return transformed
end

function Transformer:transform_response(response, transforms)
    local transformed = response
    
    for _, transform_name in ipairs(transforms) do
        local transform_fn = self.response_transforms[transform_name]
        if transform_fn then
            transformed = transform_fn(transformed)
            print(string.format("   Applied response transform: %s", transform_name))
        end
    end
    
    return transformed
end

-- Set up transformer
local transformer = Transformer:new()

-- Add request transforms
transformer:add_request_transform("add_timestamp", function(req)
    req.timestamp = os.time()
    return req
end)

transformer:add_request_transform("normalize_headers", function(req)
    if req.headers then
        local normalized = {}
        for k, v in pairs(req.headers) do
            normalized[string.lower(k)] = v
        end
        req.headers = normalized
    end
    return req
end)

transformer:add_request_transform("validate_json", function(req)
    if req.body and type(req.body) == "string" then
        -- Simulate JSON validation
        req.body_valid = true
    end
    return req
end)

-- Add response transforms
transformer:add_response_transform("add_cors", function(res)
    res.headers = res.headers or {}
    res.headers["Access-Control-Allow-Origin"] = "*"
    return res
end)

transformer:add_response_transform("compress", function(res)
    if res.body and #res.body > 1000 then
        res.headers = res.headers or {}
        res.headers["Content-Encoding"] = "gzip"
        res.body = "compressed_" .. string.sub(res.body, 1, 100)  -- Simulate
    end
    return res
end)

-- Test transformations
print("\n   Testing transformations:")

local request = {
    path = "/api/data",
    headers = {
        ["Content-Type"] = "application/json",
        ["X-API-Key"] = "secret"
    },
    body = '{"data": "test"}'
}

local transformed_req = transformer:transform_request(request, {
    "add_timestamp",
    "normalize_headers",
    "validate_json"
})

print(string.format("   Request timestamp added: %s", 
    transformed_req.timestamp and "yes" or "no"))
print(string.format("   Headers normalized: %s", 
    transformed_req.headers["content-type"] and "yes" or "no"))

local response = {
    status = 200,
    body = string.rep("data", 300)  -- Large response
}

local transformed_res = transformer:transform_response(response, {
    "add_cors",
    "compress"
})

print(string.format("   CORS headers added: %s", 
    transformed_res.headers["Access-Control-Allow-Origin"] and "yes" or "no"))
print(string.format("   Response compressed: %s", 
    transformed_res.headers["Content-Encoding"] and "yes" or "no"))

print()

-- ============================================================
-- Pattern 4: API Versioning and Migration
-- ============================================================

print("4. API Versioning and Migration")
print("-" .. string.rep("-", 40))

local APIVersioning = {}
APIVersioning.__index = APIVersioning

function APIVersioning:new()
    return setmetatable({
        versions = {},
        migrations = {},
        default_version = nil,
        deprecation_warnings = {}
    }, self)
end

function APIVersioning:register_version(version, endpoints)
    self.versions[version] = {
        version = version,
        endpoints = endpoints,
        deprecated = false,
        sunset_date = nil
    }
    
    if not self.default_version then
        self.default_version = version
    end
    
    print(string.format("   Registered API version: %s", version))
end

function APIVersioning:deprecate_version(version, sunset_date)
    if self.versions[version] then
        self.versions[version].deprecated = true
        self.versions[version].sunset_date = sunset_date
        
        print(string.format("   Deprecated version %s (sunset: %s)", 
            version, sunset_date))
    end
end

function APIVersioning:register_migration(from_version, to_version, migration_fn)
    local key = from_version .. "_to_" .. to_version
    self.migrations[key] = migration_fn
    
    print(string.format("   Registered migration: %s -> %s", 
        from_version, to_version))
end

function APIVersioning:route(request)
    -- Extract version from request
    local version = nil
    
    -- Check header
    if request.headers and request.headers["api-version"] then
        version = request.headers["api-version"]
    end
    
    -- Check URL path
    if not version then
        local v = string.match(request.path, "/v(%d+)/")
        if v then
            version = "v" .. v
        end
    end
    
    -- Use default if not specified
    version = version or self.default_version
    
    -- Check if version exists
    local version_info = self.versions[version]
    if not version_info then
        return {
            status = 400,
            error = "Unknown API version: " .. version
        }
    end
    
    -- Warn if deprecated
    if version_info.deprecated then
        print(string.format("   ⚠️  Warning: Using deprecated version %s", version))
        
        -- Try to migrate to latest
        local latest = self:get_latest_version()
        if latest and latest ~= version then
            local migrated = self:migrate_request(request, version, latest)
            if migrated then
                print(string.format("   Migrated request from %s to %s", 
                    version, latest))
                request = migrated
                version = latest
                version_info = self.versions[version]
            end
        end
    end
    
    return {
        status = 200,
        version = version,
        endpoints = version_info.endpoints
    }
end

function APIVersioning:migrate_request(request, from_version, to_version)
    local key = from_version .. "_to_" .. to_version
    local migration_fn = self.migrations[key]
    
    if migration_fn then
        return migration_fn(request)
    end
    
    return nil
end

function APIVersioning:get_latest_version()
    local latest = nil
    for version, info in pairs(self.versions) do
        if not info.deprecated then
            if not latest or version > latest then
                latest = version
            end
        end
    end
    return latest
end

-- Test API versioning
local versioning = APIVersioning:new()

-- Register versions
versioning:register_version("v1", {
    "/users", "/orders", "/products"
})

versioning:register_version("v2", {
    "/users", "/orders", "/products", "/analytics"
})

versioning:register_version("v3", {
    "/users", "/orders", "/products", "/analytics", "/recommendations"
})

-- Deprecate old version
versioning:deprecate_version("v1", "2024-12-31")

-- Register migration
versioning:register_migration("v1", "v2", function(request)
    -- Transform v1 request to v2 format
    if request.body and request.body.user_name then
        request.body.username = request.body.user_name
        request.body.user_name = nil
    end
    return request
end)

-- Test routing with different versions
print("\n   Testing API versioning:")

local test_requests = {
    {path = "/v1/users", headers = {}},
    {path = "/v2/analytics", headers = {}},
    {path = "/users", headers = {["api-version"] = "v3"}},
    {path = "/users", headers = {}}  -- Will use default
}

for _, req in ipairs(test_requests) do
    local result = versioning:route(req)
    if result.status == 200 then
        print(string.format("   %s -> Version %s", 
            req.path, result.version))
    else
        print(string.format("   %s -> Error: %s", 
            req.path, result.error))
    end
end

print()
print("🎯 Key Takeaways:")
print("   • API gateways centralize routing and load balancing")
print("   • Service discovery enables dynamic backend selection")
print("   • Transform requests/responses for compatibility")
print("   • Version APIs to support gradual migration")
print("   • Health checks ensure reliable service routing")