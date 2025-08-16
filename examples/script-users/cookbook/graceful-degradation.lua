-- Cookbook: Graceful Degradation - Fallback Strategies for Service Failures
-- Purpose: Implement patterns for graceful degradation when services fail
-- Prerequisites: Tools optional for enhanced features, fallback data sources
-- Expected Output: Demonstration of graceful degradation patterns
-- Version: 0.7.0
-- Tags: cookbook, graceful-degradation, resilience, fallback, production

print("=== Graceful Degradation Patterns ===\n")

-- ============================================================
-- Pattern 1: Service-Level Degradation
-- ============================================================

print("1. Service-Level Degradation")
print("-" .. string.rep("-", 40))

local ServiceDegradation = {}
ServiceDegradation.__index = ServiceDegradation

function ServiceDegradation:new()
    return setmetatable({
        services = {},
        service_health = {},
        degradation_levels = {
            FULL = 0,      -- All features available
            REDUCED = 1,   -- Some features disabled
            MINIMAL = 2,   -- Only core features
            OFFLINE = 3    -- Service unavailable
        },
        current_level = 0
    }, self)
end

function ServiceDegradation:register_service(name, config)
    self.services[name] = {
        name = name,
        priority = config.priority or 1,
        fallback = config.fallback,
        health_endpoint = config.health_endpoint,
        required_for_minimal = config.required_for_minimal or false
    }
    self.service_health[name] = true
    print(string.format("   Registered service: %s (priority: %d)", 
        name, config.priority))
end

function ServiceDegradation:check_service_health(service_name)
    -- Simulate health check (in production, make actual HTTP calls)
    local service = self.services[service_name]
    if not service then return false end
    
    -- Simulate some services failing
    if service_name == "recommendation_engine" then
        self.service_health[service_name] = false
        return false
    elseif service_name == "analytics" then
        self.service_health[service_name] = false 
        return false
    end
    
    self.service_health[service_name] = true
    return true
end

function ServiceDegradation:calculate_degradation_level()
    local failed_services = {}
    local failed_critical = false
    
    for name, service in pairs(self.services) do
        if not self:check_service_health(name) then
            table.insert(failed_services, name)
            if service.required_for_minimal then
                failed_critical = true
            end
        end
    end
    
    if failed_critical then
        return self.degradation_levels.OFFLINE
    elseif #failed_services >= 3 then
        return self.degradation_levels.MINIMAL
    elseif #failed_services >= 1 then
        return self.degradation_levels.REDUCED
    else
        return self.degradation_levels.FULL
    end
end

function ServiceDegradation:get_feature_availability(feature)
    local level = self:calculate_degradation_level()
    
    local feature_requirements = {
        user_recommendations = self.degradation_levels.REDUCED,
        advanced_analytics = self.degradation_levels.FULL,
        real_time_chat = self.degradation_levels.MINIMAL,
        user_profiles = self.degradation_levels.MINIMAL,
        premium_features = self.degradation_levels.FULL
    }
    
    return level <= (feature_requirements[feature] or self.degradation_levels.FULL)
end

-- Test service degradation
local degradation = ServiceDegradation:new()

degradation:register_service("user_service", {
    priority = 1,
    required_for_minimal = true,
    fallback = "cached_user_data"
})

degradation:register_service("recommendation_engine", {
    priority = 2,
    fallback = "popular_items"
})

degradation:register_service("analytics", {
    priority = 3,
    fallback = "basic_tracking"
})

degradation:register_service("notification_service", {
    priority = 2,
    fallback = "email_queue"
})

print("\n   Testing feature availability:")
local features = {"user_recommendations", "advanced_analytics", "real_time_chat", "premium_features"}

for _, feature in ipairs(features) do
    local available = degradation:get_feature_availability(feature)
    print(string.format("   %s: %s", feature, available and "✅ Available" or "❌ Degraded"))
end

print()

-- ============================================================
-- Pattern 2: Content-Based Degradation
-- ============================================================

print("2. Content-Based Degradation")
print("-" .. string.rep("-", 40))

local ContentDegradation = {}
ContentDegradation.__index = ContentDegradation

function ContentDegradation:new()
    return setmetatable({
        content_sources = {},
        fallback_content = {
            news = {"Local news unavailable", "Weather: Partly cloudy"},
            recommendations = {"Popular this week", "Trending now"},
            social_feed = {"Feed temporarily unavailable"},
            advertisements = {"Thank you for your patience"}
        }
    }, self)
end

function ContentDegradation:get_content(content_type, params)
    params = params or {}
    
    -- Try primary source
    local primary_result = self:try_primary_source(content_type, params)
    if primary_result then
        return {
            source = "primary",
            content = primary_result,
            quality = "high"
        }
    end
    
    -- Try secondary source
    local secondary_result = self:try_secondary_source(content_type, params)
    if secondary_result then
        return {
            source = "secondary", 
            content = secondary_result,
            quality = "medium"
        }
    end
    
    -- Fall back to static content
    local fallback = self.fallback_content[content_type] or {"Content unavailable"}
    return {
        source = "fallback",
        content = fallback,
        quality = "low"
    }
end

function ContentDegradation:try_primary_source(content_type, params)
    -- Simulate primary source failures
    if content_type == "news" then
        return nil -- Simulate API failure
    elseif content_type == "recommendations" then
        return {"Personalized recommendation 1", "Personalized recommendation 2"}
    end
    return nil
end

function ContentDegradation:try_secondary_source(content_type, params)
    -- Simulate secondary source (cached/simplified)
    if content_type == "news" then
        return {"Cached news item 1", "Cached news item 2"}
    elseif content_type == "recommendations" then
        return nil -- Simulate cache miss
    end
    return nil
end

-- Test content degradation
local content_degradation = ContentDegradation:new()

local content_types = {"news", "recommendations", "social_feed"}

for _, content_type in ipairs(content_types) do
    local result = content_degradation:get_content(content_type)
    print(string.format("   %s (%s quality, %s source):", 
        content_type, result.quality, result.source))
    for _, item in ipairs(result.content) do
        print(string.format("     - %s", item))
    end
end

print()

-- ============================================================
-- Pattern 3: Performance-Based Degradation
-- ============================================================

print("3. Performance-Based Degradation")
print("-" .. string.rep("-", 40))

local PerformanceDegradation = {}
PerformanceDegradation.__index = PerformanceDegradation

function PerformanceDegradation:new()
    return setmetatable({
        latency_thresholds = {
            low = 100,    -- < 100ms
            medium = 500, -- 100-500ms  
            high = 1000,  -- 500ms-1s
            critical = 2000 -- > 1s
        },
        current_latency = 0,
        feature_configs = {}
    }, self)
end

function PerformanceDegradation:register_feature(name, config)
    self.feature_configs[name] = {
        name = name,
        max_latency = config.max_latency or 1000,
        simplified_version = config.simplified_version,
        cache_fallback = config.cache_fallback
    }
end

function PerformanceDegradation:measure_latency()
    -- Simulate varying latency
    local latencies = {200, 800, 1500, 300, 600}
    self.current_latency = latencies[math.random(#latencies)]
    return self.current_latency
end

function PerformanceDegradation:get_feature_config(feature_name)
    local latency = self:measure_latency()
    local config = self.feature_configs[feature_name]
    
    if not config then
        return {enabled = false, reason = "feature not found"}
    end
    
    if latency > self.latency_thresholds.critical then
        return {
            enabled = false,
            reason = "system overloaded",
            alternative = "cached_response"
        }
    elseif latency > config.max_latency then
        return {
            enabled = true,
            simplified = true,
            reason = "high latency detected",
            version = config.simplified_version or "basic"
        }
    else
        return {
            enabled = true,
            simplified = false,
            version = "full"
        }
    end
end

-- Test performance degradation
local perf_degradation = PerformanceDegradation:new()

perf_degradation:register_feature("search", {
    max_latency = 500,
    simplified_version = "basic_search"
})

perf_degradation:register_feature("dashboard", {
    max_latency = 1000,
    simplified_version = "summary_only"
})

perf_degradation:register_feature("reports", {
    max_latency = 300,
    simplified_version = "cached_report"
})

print("   Testing performance-based degradation:")
local features = {"search", "dashboard", "reports"}

for i = 1, 3 do
    print(string.format("\n   Test %d:", i))
    for _, feature in ipairs(features) do
        local config = perf_degradation:get_feature_config(feature)
        local status = config.enabled and 
            (config.simplified and "🟡 Simplified" or "✅ Full") or "❌ Disabled"
        print(string.format("     %s: %s (latency: %dms)", 
            feature, status, perf_degradation.current_latency))
    end
end

print()

-- ============================================================
-- Pattern 4: User Experience Degradation
-- ============================================================

print("4. User Experience Degradation")
print("-" .. string.rep("-", 40))

local UXDegradation = {}
UXDegradation.__index = UXDegradation

function UXDegradation:new()
    return setmetatable({
        user_tiers = {
            premium = 3,
            standard = 2,
            basic = 1,
            anonymous = 0
        },
        feature_matrix = {},
        current_load = 0
    }, self)
end

function UXDegradation:define_feature_tiers(feature_name, tiers)
    self.feature_matrix[feature_name] = tiers
end

function UXDegradation:get_user_experience(user_tier, feature_name)
    local load = math.random(100) -- Simulate system load 0-100%
    local base_tier = self.user_tiers[user_tier] or 0
    
    -- Adjust tier based on system load
    local effective_tier = base_tier
    if load > 80 then
        effective_tier = math.max(0, base_tier - 2) -- Heavy degradation
    elseif load > 60 then
        effective_tier = math.max(0, base_tier - 1) -- Light degradation
    end
    
    local feature_tiers = self.feature_matrix[feature_name]
    if not feature_tiers then
        return {available = false, reason = "feature not defined"}
    end
    
    -- Find the best available tier for this user
    for tier_level = effective_tier, 0, -1 do
        if feature_tiers[tier_level] then
            return {
                available = true,
                tier = tier_level,
                feature_set = feature_tiers[tier_level],
                load_impact = load > 60,
                original_tier = base_tier
            }
        end
    end
    
    return {available = false, reason = "no suitable tier"}
end

-- Test UX degradation
local ux_degradation = UXDegradation:new()

-- Define video streaming tiers
ux_degradation:define_feature_tiers("video_streaming", {
    [3] = {quality = "4K", bitrate = "25Mbps", features = {"HDR", "surround_sound"}},
    [2] = {quality = "1080p", bitrate = "8Mbps", features = {"stereo_sound"}},
    [1] = {quality = "720p", bitrate = "3Mbps", features = {"basic_audio"}},
    [0] = {quality = "480p", bitrate = "1Mbps", features = {}}
})

-- Define API access tiers
ux_degradation:define_feature_tiers("api_access", {
    [3] = {rate_limit = "10000/hour", features = {"real_time", "webhooks", "priority_support"}},
    [2] = {rate_limit = "1000/hour", features = {"real_time"}},
    [1] = {rate_limit = "100/hour", features = {}},
    [0] = {rate_limit = "10/hour", features = {}}
})

print("   Testing user experience degradation:")
local users = {"premium", "standard", "basic", "anonymous"}
local features = {"video_streaming", "api_access"}

for _, user in ipairs(users) do
    print(string.format("\n   User: %s", user))
    for _, feature in ipairs(features) do
        local experience = ux_degradation:get_user_experience(user, feature)
        if experience.available then
            local degraded = experience.load_impact and " (degraded due to load)" or ""
            print(string.format("     %s: Tier %d%s", feature, experience.tier, degraded))
            for k, v in pairs(experience.feature_set) do
                if type(v) == "table" then
                    print(string.format("       %s: %s", k, table.concat(v, ", ")))
                else
                    print(string.format("       %s: %s", k, v))
                end
            end
        else
            print(string.format("     %s: ❌ %s", feature, experience.reason))
        end
    end
end

print()
print("🎯 Key Takeaways:")
print("   • Implement multiple fallback layers")
print("   • Degrade gracefully based on system health")
print("   • Prioritize core functionality over features")
print("   • Provide appropriate alternatives")
print("   • Monitor and adjust degradation thresholds")
print("   • Consider user impact when choosing degradation strategy")