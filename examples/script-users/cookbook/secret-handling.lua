-- Cookbook: Secret Handling - Secure Credential Management
-- Purpose: Implement patterns for secure handling of secrets, API keys, and sensitive data
-- Prerequisites: Secure environment, environment variables support
-- Expected Output: Demonstration of secure secret handling patterns
-- Version: 0.7.0
-- Tags: cookbook, secret-handling, security, credentials, encryption

print("=== Secret Handling Patterns ===\n")

-- ============================================================
-- Pattern 1: Environment Variable Management
-- ============================================================

print("1. Environment Variable Management")
print("-" .. string.rep("-", 40))

local EnvSecretManager = {}
EnvSecretManager.__index = EnvSecretManager

function EnvSecretManager:new()
    return setmetatable({
        required_secrets = {},
        loaded_secrets = {},
        validation_patterns = {
            api_key = "^[a-zA-Z0-9_-]{16,}$",
            bearer_token = "^[a-zA-Z0-9_.-]{20,}$",
            database_url = "^[a-zA-Z]+://",
            encryption_key = "^[a-fA-F0-9]{32,}$"
        }
    }, self)
end

function EnvSecretManager:register_required_secret(name, config)
    self.required_secrets[name] = {
        name = name,
        env_var = config.env_var or name:upper(),
        required = config.required ~= false,
        pattern = config.pattern,
        description = config.description or "",
        sensitive = config.sensitive ~= false
    }
    
    print(string.format("   🔐 Registered secret: %s (env: %s)", 
        name, config.env_var or name:upper()))
end

function EnvSecretManager:load_secrets()
    local missing_secrets = {}
    local invalid_secrets = {}
    
    for name, config in pairs(self.required_secrets) do
        local value = os.getenv(config.env_var)
        
        if not value then
            if config.required then
                table.insert(missing_secrets, config.env_var)
            end
        else
            -- Validate pattern if specified
            if config.pattern then
                local pattern = self.validation_patterns[config.pattern] or config.pattern
                if not string.match(value, pattern) then
                    table.insert(invalid_secrets, {
                        name = name,
                        env_var = config.env_var,
                        issue = "Invalid format"
                    })
                end
            end
            
            -- Store secret (never log the actual value)
            self.loaded_secrets[name] = {
                value = value,
                loaded_at = os.time(),
                env_var = config.env_var,
                sensitive = config.sensitive
            }
            
            print(string.format("   ✅ Loaded secret: %s", name))
        end
    end
    
    -- Report issues
    if #missing_secrets > 0 then
        print("   ❌ Missing required environment variables:")
        for _, var in ipairs(missing_secrets) do
            print(string.format("     • %s", var))
        end
    end
    
    if #invalid_secrets > 0 then
        print("   ⚠️  Invalid secret formats:")
        for _, invalid in ipairs(invalid_secrets) do
            print(string.format("     • %s (%s): %s", 
                invalid.env_var, invalid.name, invalid.issue))
        end
    end
    
    return {
        success = #missing_secrets == 0 and #invalid_secrets == 0,
        loaded_count = self:get_loaded_count(),
        missing_secrets = missing_secrets,
        invalid_secrets = invalid_secrets
    }
end

function EnvSecretManager:get_secret(name)
    local secret = self.loaded_secrets[name]
    if not secret then
        error("Secret not loaded: " .. name)
    end
    return secret.value
end

function EnvSecretManager:has_secret(name)
    return self.loaded_secrets[name] ~= nil
end

function EnvSecretManager:get_loaded_count()
    local count = 0
    for _ in pairs(self.loaded_secrets) do
        count = count + 1
    end
    return count
end

function EnvSecretManager:mask_secret(value)
    if not value or #value < 8 then
        return "***"
    end
    return string.sub(value, 1, 4) .. string.rep("*", #value - 8) .. string.sub(value, -4)
end

function EnvSecretManager:get_secrets_status()
    local status = {
        total_registered = 0,
        loaded = 0,
        missing = 0,
        secrets = {}
    }
    
    for name, config in pairs(self.required_secrets) do
        status.total_registered = status.total_registered + 1
        
        local secret_status = {
            name = name,
            env_var = config.env_var,
            required = config.required,
            loaded = self.loaded_secrets[name] ~= nil
        }
        
        if secret_status.loaded then
            status.loaded = status.loaded + 1
            if not config.sensitive then
                secret_status.masked_value = self:mask_secret(self.loaded_secrets[name].value)
            end
        else
            status.missing = status.missing + 1
        end
        
        table.insert(status.secrets, secret_status)
    end
    
    return status
end

-- Test environment secret management
local env_manager = EnvSecretManager:new()

-- Register required secrets (simulating real environment variables)
env_manager:register_required_secret("api_key", {
    env_var = "OPENAI_API_KEY",
    pattern = "api_key",
    description = "OpenAI API key for LLM access",
    required = true
})

env_manager:register_required_secret("database_url", {
    env_var = "DATABASE_URL", 
    pattern = "database_url",
    description = "Database connection string",
    required = true
})

env_manager:register_required_secret("encryption_key", {
    env_var = "ENCRYPTION_KEY",
    pattern = "encryption_key",
    description = "Key for encrypting sensitive data",
    required = false
})

env_manager:register_required_secret("webhook_secret", {
    env_var = "WEBHOOK_SECRET",
    description = "Secret for webhook validation",
    required = false,
    sensitive = false -- Can be masked in logs
})

print("   Testing environment secret loading:")

-- Simulate some environment variables being set
local simulated_env = {
    OPENAI_API_KEY = "sk-1234567890abcdef1234567890abcdef",
    -- DATABASE_URL missing (will show as missing)
    ENCRYPTION_KEY = "invalidkey", -- Invalid format
    WEBHOOK_SECRET = "webhook_secret_123"
}

-- Temporarily override os.getenv for testing
local original_getenv = os.getenv
os.getenv = function(var_name)
    return simulated_env[var_name]
end

local load_result = env_manager:load_secrets()

-- Restore original function
os.getenv = original_getenv

print(string.format("   Load result: %s", load_result.success and "✅ Success" or "❌ Failed"))
print(string.format("   Loaded: %d secrets", load_result.loaded_count))

if #load_result.missing_secrets > 0 then
    print(string.format("   Missing: %s", table.concat(load_result.missing_secrets, ", ")))
end

-- Show secrets status
local status = env_manager:get_secrets_status()
print(string.format("\n   Secrets status (%d/%d loaded):", status.loaded, status.total_registered))
for _, secret in ipairs(status.secrets) do
    local status_icon = secret.loaded and "✅" or "❌"
    print(string.format("     %s %s (%s)", status_icon, secret.name, secret.env_var))
    if secret.masked_value then
        print(string.format("       Value: %s", secret.masked_value))
    end
end

print()

-- ============================================================
-- Pattern 2: In-Memory Secret Storage with Rotation
-- ============================================================

print("2. In-Memory Secret Storage with Rotation")
print("-" .. string.rep("-", 40))

local SecretStore = {}
SecretStore.__index = SecretStore

function SecretStore:new()
    return setmetatable({
        secrets = {},
        rotation_schedule = {},
        access_log = {},
        encryption_enabled = false
    }, self)
end

function SecretStore:store_secret(name, value, config)
    config = config or {}
    
    local secret = {
        name = name,
        value = value,
        created_at = os.time(),
        expires_at = config.expires_at,
        ttl = config.ttl,
        access_count = 0,
        last_accessed = nil,
        auto_rotate = config.auto_rotate or false,
        rotation_interval = config.rotation_interval or 86400, -- 24 hours
        metadata = config.metadata or {}
    }
    
    self.secrets[name] = secret
    
    if secret.auto_rotate then
        self.rotation_schedule[name] = secret.created_at + secret.rotation_interval
    end
    
    print(string.format("   💾 Stored secret: %s", name))
    if secret.ttl then
        print(string.format("     TTL: %d seconds", secret.ttl))
    end
    if secret.auto_rotate then
        print(string.format("     Auto-rotation: every %d seconds", secret.rotation_interval))
    end
end

function SecretStore:get_secret(name, accessor)
    local secret = self.secrets[name]
    
    if not secret then
        self:log_access(name, accessor, "not_found")
        return nil, "Secret not found"
    end
    
    -- Check expiration
    local current_time = os.time()
    
    if secret.expires_at and current_time > secret.expires_at then
        self:log_access(name, accessor, "expired")
        self.secrets[name] = nil
        return nil, "Secret expired"
    end
    
    if secret.ttl and (current_time - secret.created_at) > secret.ttl then
        self:log_access(name, accessor, "ttl_expired")
        self.secrets[name] = nil
        return nil, "Secret TTL expired"
    end
    
    -- Update access tracking
    secret.access_count = secret.access_count + 1
    secret.last_accessed = current_time
    
    self:log_access(name, accessor, "success")
    
    return secret.value, nil
end

function SecretStore:log_access(secret_name, accessor, result)
    table.insert(self.access_log, {
        secret_name = secret_name,
        accessor = accessor or "unknown",
        result = result,
        timestamp = os.time()
    })
    
    -- Keep log size manageable
    if #self.access_log > 100 then
        table.remove(self.access_log, 1)
    end
end

function SecretStore:rotate_secret(name, new_value)
    local secret = self.secrets[name]
    
    if not secret then
        return false, "Secret not found"
    end
    
    local old_value = secret.value
    secret.value = new_value
    secret.created_at = os.time()
    secret.access_count = 0
    
    if secret.auto_rotate then
        self.rotation_schedule[name] = secret.created_at + secret.rotation_interval
    end
    
    print(string.format("   🔄 Rotated secret: %s", name))
    
    return true, nil
end

function SecretStore:check_rotation_schedule()
    local current_time = os.time()
    local rotations_needed = {}
    
    for secret_name, rotation_time in pairs(self.rotation_schedule) do
        if current_time >= rotation_time then
            table.insert(rotations_needed, secret_name)
        end
    end
    
    return rotations_needed
end

function SecretStore:get_secret_stats()
    local stats = {
        total_secrets = 0,
        expired_secrets = 0,
        auto_rotating_secrets = 0,
        total_accesses = #self.access_log,
        secrets = {}
    }
    
    local current_time = os.time()
    
    for name, secret in pairs(self.secrets) do
        stats.total_secrets = stats.total_secrets + 1
        
        local is_expired = false
        if secret.expires_at and current_time > secret.expires_at then
            is_expired = true
            stats.expired_secrets = stats.expired_secrets + 1
        end
        
        if secret.auto_rotate then
            stats.auto_rotating_secrets = stats.auto_rotating_secrets + 1
        end
        
        table.insert(stats.secrets, {
            name = name,
            access_count = secret.access_count,
            age_seconds = current_time - secret.created_at,
            is_expired = is_expired,
            auto_rotate = secret.auto_rotate,
            last_accessed = secret.last_accessed
        })
    end
    
    return stats
end

function SecretStore:clear_expired_secrets()
    local current_time = os.time()
    local removed = {}
    
    for name, secret in pairs(self.secrets) do
        local should_remove = false
        
        if secret.expires_at and current_time > secret.expires_at then
            should_remove = true
        elseif secret.ttl and (current_time - secret.created_at) > secret.ttl then
            should_remove = true
        end
        
        if should_remove then
            self.secrets[name] = nil
            self.rotation_schedule[name] = nil
            table.insert(removed, name)
        end
    end
    
    if #removed > 0 then
        print(string.format("   🧹 Cleared %d expired secrets: %s", 
            #removed, table.concat(removed, ", ")))
    end
    
    return removed
end

-- Test in-memory secret storage
local secret_store = SecretStore:new()

print("   Testing in-memory secret storage:")

-- Store secrets with different configurations
secret_store:store_secret("api_key", "sk-prod-key-12345", {
    ttl = 3600, -- 1 hour
    auto_rotate = true,
    rotation_interval = 1800, -- 30 minutes
    metadata = {environment = "production"}
})

secret_store:store_secret("temp_token", "temp-xyz-789", {
    ttl = 5, -- 5 seconds (for quick expiration test)
    metadata = {purpose = "testing"}
})

secret_store:store_secret("permanent_key", "permanent-key-abc", {
    auto_rotate = false,
    metadata = {type = "encryption"}
})

-- Test secret access
print("\n   Testing secret access:")

local api_key, err = secret_store:get_secret("api_key", "user_service")
if api_key then
    print(string.format("     ✅ Retrieved api_key: %s", 
        secret_store:mask_secret and secret_store:mask_secret(api_key) or "***"))
else
    print(string.format("     ❌ Failed to get api_key: %s", err))
end

-- Access same secret again
api_key, err = secret_store:get_secret("api_key", "auth_service")

-- Try to access non-existent secret
local missing, missing_err = secret_store:get_secret("missing_key", "test_service")
print(string.format("     Missing key result: %s", missing_err))

-- Wait and check expiration
print("\n   Waiting for temp_token to expire...")
local wait_end = os.time() + 6
while os.time() < wait_end do end

local expired_token, expired_err = secret_store:get_secret("temp_token", "test_service")
print(string.format("     Expired token result: %s", expired_err))

-- Check rotation schedule
local rotations_needed = secret_store:check_rotation_schedule()
if #rotations_needed > 0 then
    print(string.format("   ⏰ Secrets needing rotation: %s", 
        table.concat(rotations_needed, ", ")))
end

-- Show statistics
local stats = secret_store:get_secret_stats()
print(string.format("\n   Secret store stats:"))
print(string.format("     Total secrets: %d", stats.total_secrets))
print(string.format("     Auto-rotating: %d", stats.auto_rotating_secrets))
print(string.format("     Total accesses: %d", stats.total_accesses))

secret_store:clear_expired_secrets()

print()

-- ============================================================
-- Pattern 3: Secret Encryption and Masking
-- ============================================================

print("3. Secret Encryption and Masking")
print("-" .. string.rep("-", 40))

local SecretEncryption = {}
SecretEncryption.__index = SecretEncryption

function SecretEncryption:new(master_key)
    return setmetatable({
        master_key = master_key or "default_master_key_change_me!",
        encrypted_secrets = {},
        masking_rules = {
            default = {prefix = 2, suffix = 2, char = "*"},
            api_key = {prefix = 3, suffix = 4, char = "*"},
            token = {prefix = 4, suffix = 0, char = "•"},
            password = {prefix = 0, suffix = 0, char = "*"}
        }
    }, self)
end

function SecretEncryption:simple_encrypt(plaintext, key)
    -- Simple XOR encryption (NOT for production use!)
    key = key or self.master_key
    local encrypted = {}
    
    for i = 1, #plaintext do
        local char_code = string.byte(plaintext, i)
        local key_char = string.byte(key, ((i - 1) % #key) + 1)
        local encrypted_char = char_code ~ key_char -- XOR
        table.insert(encrypted, string.char(encrypted_char))
    end
    
    return table.concat(encrypted)
end

function SecretEncryption:simple_decrypt(ciphertext, key)
    -- Simple XOR decryption (same as encryption with XOR)
    return self:simple_encrypt(ciphertext, key)
end

function SecretEncryption:encode_base64(data)
    -- Simple base64-like encoding (not actual base64)
    local encoded = ""
    for i = 1, #data do
        encoded = encoded .. string.format("%02x", string.byte(data, i))
    end
    return encoded
end

function SecretEncryption:decode_base64(encoded)
    local decoded = ""
    for i = 1, #encoded, 2 do
        local hex_pair = string.sub(encoded, i, i + 1)
        decoded = decoded .. string.char(tonumber(hex_pair, 16))
    end
    return decoded
end

function SecretEncryption:store_encrypted_secret(name, plaintext, metadata)
    local encrypted = self:simple_encrypt(plaintext)
    local encoded = self:encode_base64(encrypted)
    
    self.encrypted_secrets[name] = {
        ciphertext = encoded,
        metadata = metadata or {},
        created_at = os.time(),
        algorithm = "simple_xor", -- Never use this in production!
        checksum = #plaintext -- Simple integrity check
    }
    
    print(string.format("   🔒 Encrypted and stored: %s", name))
end

function SecretEncryption:retrieve_encrypted_secret(name)
    local stored = self.encrypted_secrets[name]
    if not stored then
        return nil, "Encrypted secret not found"
    end
    
    local decoded = self:decode_base64(stored.ciphertext)
    local decrypted = self:simple_decrypt(decoded)
    
    -- Simple integrity check
    if #decrypted ~= stored.checksum then
        return nil, "Secret integrity check failed"
    end
    
    return decrypted, nil
end

function SecretEncryption:mask_secret(value, secret_type)
    secret_type = secret_type or "default"
    local rule = self.masking_rules[secret_type] or self.masking_rules.default
    
    if not value or #value <= (rule.prefix + rule.suffix) then
        return string.rep(rule.char, 8)
    end
    
    local prefix = rule.prefix > 0 and string.sub(value, 1, rule.prefix) or ""
    local suffix = rule.suffix > 0 and string.sub(value, -rule.suffix) or ""
    local middle_length = #value - rule.prefix - rule.suffix
    local middle = string.rep(rule.char, math.max(4, middle_length))
    
    return prefix .. middle .. suffix
end

function SecretEncryption:audit_log_safe(secret_name, action, user)
    -- Safe logging that never exposes secret values
    local log_entry = {
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        secret_name = secret_name,
        action = action,
        user = user or "system",
        success = true
    }
    
    print(string.format("   📋 AUDIT: %s - %s by %s at %s", 
        secret_name, action, log_entry.user, log_entry.timestamp))
    
    return log_entry
end

function SecretEncryption:secure_comparison(secret1, secret2)
    -- Constant-time string comparison to prevent timing attacks
    if #secret1 ~= #secret2 then
        return false
    end
    
    local result = 0
    for i = 1, #secret1 do
        result = result | (string.byte(secret1, i) ~ string.byte(secret2, i))
    end
    
    return result == 0
end

-- Test secret encryption and masking
local encryption = SecretEncryption:new("production_master_key_256bit")

print("   Testing secret encryption and masking:")

-- Store encrypted secrets
encryption:store_encrypted_secret("production_api_key", 
    "sk-prod-1234567890abcdef1234567890abcdef", 
    {environment = "production", service = "openai"})

encryption:store_encrypted_secret("database_password", 
    "super_secure_db_password_123!",
    {database = "main", role = "admin"})

encryption:store_encrypted_secret("jwt_secret",
    "jwt-secret-key-for-token-signing",
    {purpose = "authentication", algorithm = "HS256"})

-- Retrieve and test secrets
print("\n   Testing secret retrieval:")

local api_key, err = encryption:retrieve_encrypted_secret("production_api_key")
if api_key then
    print(string.format("     ✅ Retrieved API key: %s", 
        encryption:mask_secret(api_key, "api_key")))
    encryption:audit_log_safe("production_api_key", "ACCESSED", "user_service")
else
    print(string.format("     ❌ Failed to retrieve API key: %s", err))
end

-- Test different masking rules
print("\n   Testing secret masking patterns:")

local test_secrets = {
    {value = "sk-1234567890abcdef", type = "api_key"},
    {value = "bearer-token-xyz-789", type = "token"},
    {value = "secretpassword", type = "password"},
    {value = "short", type = "default"}
}

for _, test in ipairs(test_secrets) do
    local masked = encryption:mask_secret(test.value, test.type)
    print(string.format("     %s (%s): %s", test.type, test.value, masked))
end

-- Test secure comparison
print("\n   Testing secure comparison:")

local secret1 = "identical_secret"
local secret2 = "identical_secret"
local secret3 = "different_secret"

print(string.format("     Same secrets: %s", 
    encryption:secure_comparison(secret1, secret2) and "✅ Match" or "❌ No match"))
print(string.format("     Different secrets: %s", 
    encryption:secure_comparison(secret1, secret3) and "❌ Match" or "✅ No match"))

print()
print("🎯 Key Takeaways:")
print("   • Never store secrets in plain text or code")
print("   • Use environment variables for secret injection")
print("   • Implement secret rotation for long-lived services")
print("   • Always mask secrets in logs and outputs")
print("   • Use proper encryption for secrets at rest")
print("   • Implement audit logging for secret access")
print("   • Use constant-time comparison for secret validation")