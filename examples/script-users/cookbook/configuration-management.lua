-- Cookbook: Configuration Management - Environment-based Config
-- Purpose: Implement configuration management patterns for different environments
-- Prerequisites: None
-- Expected Output: Configuration management demonstration
-- Version: 0.7.0
-- Tags: cookbook, configuration, environment, settings, production

print("=== Configuration Management Patterns ===\n")

-- ============================================================
-- Pattern 1: Environment-Based Configuration
-- ============================================================

print("1. Environment-Based Configuration")
print("-" .. string.rep("-", 40))

local ConfigManager = {}
ConfigManager.__index = ConfigManager

function ConfigManager:new()
    return setmetatable({
        configs = {},
        current_env = nil,
        defaults = {},
        overrides = {}
    }, self)
end

function ConfigManager:set_environment(env)
    self.current_env = env
    print("   Environment set to: " .. env)
end

function ConfigManager:add_config(env, config)
    self.configs[env] = config
end

function ConfigManager:set_defaults(defaults)
    self.defaults = defaults
end

function ConfigManager:set_override(key, value)
    self.overrides[key] = value
end

function ConfigManager:get(key, env)
    env = env or self.current_env or "development"
    
    -- Check overrides first
    if self.overrides[key] ~= nil then
        return self.overrides[key]
    end
    
    -- Check environment config
    local env_config = self.configs[env] or {}
    if env_config[key] ~= nil then
        return env_config[key]
    end
    
    -- Fall back to defaults
    return self.defaults[key]
end

function ConfigManager:get_all(env)
    env = env or self.current_env or "development"
    
    -- Merge configs: defaults -> environment -> overrides
    local merged = {}
    
    -- Start with defaults
    for k, v in pairs(self.defaults) do
        merged[k] = v
    end
    
    -- Apply environment config
    local env_config = self.configs[env] or {}
    for k, v in pairs(env_config) do
        merged[k] = v
    end
    
    -- Apply overrides
    for k, v in pairs(self.overrides) do
        merged[k] = v
    end
    
    return merged
end

-- Create config manager
local config = ConfigManager:new()

-- Set defaults
config:set_defaults({
    api_url = "http://localhost:3000",
    timeout = 30,
    retry_count = 3,
    log_level = "info",
    cache_enabled = true
})

-- Add environment configs
config:add_config("development", {
    api_url = "http://localhost:3000",
    log_level = "debug",
    cache_enabled = false
})

config:add_config("staging", {
    api_url = "https://staging.api.example.com",
    timeout = 60,
    log_level = "info"
})

config:add_config("production", {
    api_url = "https://api.example.com",
    timeout = 120,
    retry_count = 5,
    log_level = "error",
    cache_enabled = true
})

-- Test different environments
local envs = {"development", "staging", "production"}
for _, env in ipairs(envs) do
    config:set_environment(env)
    print("\n   Config for " .. env .. ":")
    local all_config = config:get_all()
    for k, v in pairs(all_config) do
        print("     " .. k .. ": " .. tostring(v))
    end
end

print()

-- ============================================================
-- Pattern 2: Feature Flags
-- ============================================================

print("2. Feature Flags Pattern")
print("-" .. string.rep("-", 40))

local FeatureFlags = {}
FeatureFlags.__index = FeatureFlags

function FeatureFlags:new()
    return setmetatable({
        flags = {},
        user_overrides = {},
        percentage_rollouts = {}
    }, self)
end

function FeatureFlags:define(name, options)
    options = options or {}
    self.flags[name] = {
        enabled = options.enabled or false,
        description = options.description,
        rollout_percentage = options.rollout_percentage,
        allowed_users = options.allowed_users or {},
        conditions = options.conditions
    }
end

function FeatureFlags:is_enabled(name, user_id)
    local flag = self.flags[name]
    if not flag then
        return false
    end
    
    -- Check user override
    if self.user_overrides[user_id] and 
       self.user_overrides[user_id][name] ~= nil then
        return self.user_overrides[user_id][name]
    end
    
    -- Check if user is in allowed list
    if flag.allowed_users and #flag.allowed_users > 0 then
        for _, allowed in ipairs(flag.allowed_users) do
            if allowed == user_id then
                return true
            end
        end
    end
    
    -- Check percentage rollout
    if flag.rollout_percentage then
        -- Simple hash-based rollout
        local hash = 0
        for i = 1, #user_id do
            hash = hash + string.byte(user_id, i)
        end
        local user_percent = (hash % 100) + 1
        if user_percent <= flag.rollout_percentage then
            return true
        end
    end
    
    -- Check conditions
    if flag.conditions then
        return flag.conditions(user_id)
    end
    
    return flag.enabled
end

function FeatureFlags:set_user_override(user_id, flag_name, enabled)
    if not self.user_overrides[user_id] then
        self.user_overrides[user_id] = {}
    end
    self.user_overrides[user_id][flag_name] = enabled
end

-- Create feature flags manager
local features = FeatureFlags:new()

-- Define features
features:define("new_ui", {
    enabled = false,
    description = "New UI design",
    rollout_percentage = 20  -- 20% of users
})

features:define("advanced_search", {
    enabled = true,
    description = "Advanced search functionality",
    allowed_users = {"user123", "user456"}
})

features:define("beta_api", {
    enabled = false,
    description = "Beta API endpoints",
    conditions = function(user_id)
        -- Enable for users with "beta" in their ID
        return string.find(user_id, "beta") ~= nil
    end
})

-- Test feature flags
print("   Testing feature flags:")
local test_users = {"user001", "user123", "beta_user", "user999"}

for _, user in ipairs(test_users) do
    print("\n   User: " .. user)
    print("     new_ui: " .. tostring(features:is_enabled("new_ui", user)))
    print("     advanced_search: " .. tostring(features:is_enabled("advanced_search", user)))
    print("     beta_api: " .. tostring(features:is_enabled("beta_api", user)))
end

print()

-- ============================================================
-- Pattern 3: Secret Management
-- ============================================================

print("3. Secret Management Pattern")
print("-" .. string.rep("-", 40))

local SecretManager = {}
SecretManager.__index = SecretManager

function SecretManager:new(options)
    options = options or {}
    return setmetatable({
        secrets = {},
        encrypted = options.encrypted or false,
        key_prefix = options.key_prefix or "SECRET_",
        redact_logs = options.redact_logs ~= false
    }, self)
end

function SecretManager:load_from_env()
    -- Simulate loading from environment variables
    local env_secrets = {
        API_KEY = "sk-1234567890abcdef",
        DATABASE_PASSWORD = "super_secret_password",
        JWT_SECRET = "jwt_secret_key_here"
    }
    
    for key, value in pairs(env_secrets) do
        if string.sub(key, 1, #self.key_prefix) == self.key_prefix or
           string.find(key, "PASSWORD") or
           string.find(key, "SECRET") or
           string.find(key, "KEY") then
            self.secrets[key] = {
                value = value,
                loaded_at = os.time(),
                accessed_count = 0
            }
        end
    end
    
    print("   Loaded " .. self:count() .. " secrets from environment")
end

function SecretManager:get(key)
    local secret = self.secrets[key]
    if not secret then
        return nil
    end
    
    secret.accessed_count = secret.accessed_count + 1
    secret.last_accessed = os.time()
    
    return secret.value
end

function SecretManager:get_redacted(key)
    local value = self:get(key)
    if not value then
        return nil
    end
    
    -- Redact for logging
    if #value <= 8 then
        return string.rep("*", #value)
    else
        return string.sub(value, 1, 3) .. string.rep("*", #value - 6) .. string.sub(value, -3)
    end
end

function SecretManager:count()
    local count = 0
    for _ in pairs(self.secrets) do
        count = count + 1
    end
    return count
end

function SecretManager:rotate(key, new_value)
    local secret = self.secrets[key]
    if not secret then
        return false
    end
    
    -- Archive old value
    secret.previous_value = secret.value
    secret.rotated_at = os.time()
    secret.value = new_value
    
    print("   Secret '" .. key .. "' rotated successfully")
    return true
end

-- Test secret manager
local secrets = SecretManager:new({
    encrypted = false,  -- Would be true in production
    redact_logs = true
})

secrets:load_from_env()

print("\n   Testing secret access:")
print("   API_KEY (redacted): " .. tostring(secrets:get_redacted("API_KEY")))
print("   Full API_KEY length: " .. #secrets:get("API_KEY"))

-- Rotate a secret
secrets:rotate("API_KEY", "sk-new1234567890xyz")
print("   After rotation (redacted): " .. tostring(secrets:get_redacted("API_KEY")))

print()

-- ============================================================
-- Pattern 4: Dynamic Configuration Reloading
-- ============================================================

print("4. Dynamic Configuration Reloading")
print("-" .. string.rep("-", 40))

local DynamicConfig = {}
DynamicConfig.__index = DynamicConfig

function DynamicConfig:new(options)
    options = options or {}
    return setmetatable({
        config = {},
        watchers = {},
        version = 0,
        last_reload = nil,
        reload_interval = options.reload_interval or 60  -- seconds
    }, self)
end

function DynamicConfig:load()
    -- Simulate loading from file/API
    self.config = {
        api = {
            timeout = 30,
            retry = 3,
            endpoints = {
                users = "/api/v1/users",
                posts = "/api/v1/posts"
            }
        },
        features = {
            cache = true,
            compression = true,
            rate_limit = 100
        },
        logging = {
            level = "info",
            format = "json"
        }
    }
    
    self.version = self.version + 1
    self.last_reload = os.time()
    
    -- Notify watchers
    self:notify_watchers()
    
    print("   Config loaded (version " .. self.version .. ")")
end

function DynamicConfig:watch(name, callback)
    table.insert(self.watchers, {
        name = name,
        callback = callback
    })
end

function DynamicConfig:notify_watchers()
    for _, watcher in ipairs(self.watchers) do
        watcher.callback(self.config, self.version)
    end
end

function DynamicConfig:should_reload()
    if not self.last_reload then
        return true
    end
    
    return (os.time() - self.last_reload) >= self.reload_interval
end

function DynamicConfig:get(path)
    -- Navigate config tree with dot notation
    local parts = {}
    for part in string.gmatch(path, "[^%.]+") do
        table.insert(parts, part)
    end
    
    local current = self.config
    for _, part in ipairs(parts) do
        if type(current) ~= "table" then
            return nil
        end
        current = current[part]
    end
    
    return current
end

-- Test dynamic config
local dynamic = DynamicConfig:new({
    reload_interval = 30
})

-- Add watchers
dynamic:watch("logger", function(config, version)
    print("   [Logger] Config updated to v" .. version)
end)

dynamic:watch("api_client", function(config, version)
    print("   [API Client] Config updated to v" .. version)
end)

-- Load config
dynamic:load()

-- Access config values
print("\n   Accessing config values:")
print("   API timeout: " .. tostring(dynamic:get("api.timeout")))
print("   Users endpoint: " .. tostring(dynamic:get("api.endpoints.users")))
print("   Cache enabled: " .. tostring(dynamic:get("features.cache")))
print("   Log level: " .. tostring(dynamic:get("logging.level")))

print()

-- ============================================================
-- Pattern 5: Configuration Validation
-- ============================================================

print("5. Configuration Validation")
print("-" .. string.rep("-", 40))

local ConfigValidator = {}
ConfigValidator.__index = ConfigValidator

function ConfigValidator:new()
    return setmetatable({
        rules = {},
        errors = {}
    }, self)
end

function ConfigValidator:add_rule(path, rule)
    self.rules[path] = rule
end

function ConfigValidator:validate(config)
    self.errors = {}
    local valid = true
    
    for path, rule in pairs(self.rules) do
        local value = self:get_value(config, path)
        local rule_valid, error_msg = rule(value, config)
        
        if not rule_valid then
            valid = false
            table.insert(self.errors, {
                path = path,
                error = error_msg,
                value = value
            })
        end
    end
    
    return valid, self.errors
end

function ConfigValidator:get_value(config, path)
    local parts = {}
    for part in string.gmatch(path, "[^%.]+") do
        table.insert(parts, part)
    end
    
    local current = config
    for _, part in ipairs(parts) do
        if type(current) ~= "table" then
            return nil
        end
        current = current[part]
    end
    
    return current
end

-- Create validator
local validator = ConfigValidator:new()

-- Add validation rules
validator:add_rule("api.timeout", function(value)
    if type(value) ~= "number" then
        return false, "Must be a number"
    end
    if value < 1 or value > 300 then
        return false, "Must be between 1 and 300 seconds"
    end
    return true
end)

validator:add_rule("api.retry", function(value)
    if type(value) ~= "number" then
        return false, "Must be a number"
    end
    if value < 0 or value > 10 then
        return false, "Must be between 0 and 10"
    end
    return true
end)

validator:add_rule("logging.level", function(value)
    local valid_levels = {debug = true, info = true, warn = true, error = true}
    if not valid_levels[value] then
        return false, "Must be one of: debug, info, warn, error"
    end
    return true
end)

-- Test validation
local test_config = {
    api = {
        timeout = 500,  -- Invalid: too high
        retry = 3
    },
    logging = {
        level = "verbose"  -- Invalid: not a valid level
    }
}

print("   Validating configuration:")
local is_valid, errors = validator:validate(test_config)

if is_valid then
    print("   ✅ Configuration is valid")
else
    print("   ❌ Configuration has errors:")
    for _, err in ipairs(errors) do
        print(string.format("     %s: %s (value: %s)", 
            err.path, err.error, tostring(err.value)))
    end
end

print()
print("🎯 Key Takeaways:")
print("   • Use environment-specific configurations")
print("   • Implement feature flags for gradual rollouts")
print("   • Protect secrets and rotate them regularly")
print("   • Support dynamic config reloading")
print("   • Always validate configuration values")