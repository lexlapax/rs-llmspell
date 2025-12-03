-- Profile: minimal
-- Run with: llmspell -p development run webhook-integration.lua
-- Development environment with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: cookbook
-- Pattern ID: 05 - Webhook Integration v0.7.0
-- Complexity: PRODUCTION
-- Real-World Use Case: External system integration via webhooks
-- Pattern Category: Integration & Event Processing
--
-- Purpose: Production webhook integration patterns for connecting LLMSpell
--          to external systems. Implements webhook sending, retry logic,
--          signature verification, and event processing for enterprise integrations.
-- Architecture: Event-driven webhook handling with reliability patterns
-- Crates Showcased: llmspell-tools (webhook_caller), llmspell-bridge
-- Key Features:
--   • Webhook sending with retry logic
--   • Signature verification for security
--   • Event batching for efficiency
--   • Circuit breaker for failing endpoints
--   • Webhook response handling
--   • Dead letter queue patterns
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Network access for webhook testing
--   • Optional: webhook_caller tool for production use
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/cookbook/webhook-integration.lua
--
-- EXPECTED OUTPUT:
-- 4 webhook patterns demonstrated:
-- 1. Basic webhook sending with retry
-- 2. Batched webhook events
-- 3. Webhook with signature verification
-- 4. Circuit breaker for failing webhooks
--
-- Time to Complete: <5 seconds
-- Production Notes: Implement webhook signing for security, use queues for
--                   reliability, monitor webhook latency and success rates,
--                   implement idempotency keys to prevent duplicate processing.
-- ============================================================

print("=== Webhook Integration Patterns ===")
print("Pattern 05: PRODUCTION - External system integration\n")

-- ============================================================
-- Pattern 1: Webhook Handler with Validation
-- ============================================================

print("1. Webhook Handler with Validation")
print("-" .. string.rep("-", 40))

local WebhookHandler = {}
WebhookHandler.__index = WebhookHandler

function WebhookHandler:new(options)
    options = options or {}
    return setmetatable({
        secret = options.secret,
        handlers = {},
        validation_rules = {},
        event_log = {},
        max_retries = options.max_retries or 3
    }, self)
end

function WebhookHandler:register(event_type, handler, validation)
    self.handlers[event_type] = handler
    if validation then
        self.validation_rules[event_type] = validation
    end
    print(string.format("   Registered handler for '%s' events", event_type))
end

function WebhookHandler:validate_signature(payload, signature)
    if not self.secret then
        return true  -- No secret configured
    end
    
    -- Simple HMAC validation (simulated)
    local expected = "hmac_" .. self.secret .. "_" .. tostring(#payload)
    return signature == expected
end

function WebhookHandler:validate_payload(event_type, payload)
    local validation = self.validation_rules[event_type]
    if not validation then
        return true, nil
    end
    
    -- Run validation rules
    for field, rule in pairs(validation) do
        local value = payload[field]
        
        if rule.required and value == nil then
            return false, "Missing required field: " .. field
        end
        
        if value ~= nil and rule.type and type(value) ~= rule.type then
            return false, string.format("Invalid type for %s: expected %s, got %s",
                field, rule.type, type(value))
        end
        
        if value ~= nil and rule.pattern then
            if not string.match(tostring(value), rule.pattern) then
                return false, "Field " .. field .. " doesn't match pattern"
            end
        end
    end
    
    return true, nil
end

function WebhookHandler:process(request)
    local event_id = "evt_" .. os.time() .. "_" .. math.random(1000)
    
    -- Validate signature
    if request.signature and not self:validate_signature(request.payload, request.signature) then
        return {
            success = false,
            error = "Invalid signature",
            event_id = event_id
        }
    end
    
    -- Extract event type
    local event_type = request.payload.event or request.payload.type
    if not event_type then
        return {
            success = false,
            error = "Missing event type",
            event_id = event_id
        }
    end
    
    -- Validate payload
    local valid, err = self:validate_payload(event_type, request.payload)
    if not valid then
        return {
            success = false,
            error = err,
            event_id = event_id
        }
    end
    
    -- Find handler
    local handler = self.handlers[event_type]
    if not handler then
        return {
            success = false,
            error = "No handler for event type: " .. event_type,
            event_id = event_id
        }
    end
    
    -- Process with retry logic
    local attempts = 0
    local last_error = nil
    
    while attempts < self.max_retries do
        attempts = attempts + 1
        
        local success, result = pcall(handler, request.payload)
        
        if success then
            -- Log successful processing
            table.insert(self.event_log, {
                event_id = event_id,
                event_type = event_type,
                timestamp = os.time(),
                attempts = attempts,
                status = "success"
            })
            
            return {
                success = true,
                result = result,
                event_id = event_id,
                attempts = attempts
            }
        else
            last_error = result
            print(string.format("   Attempt %d failed: %s", attempts, result))
        end
    end
    
    -- Log failure
    table.insert(self.event_log, {
        event_id = event_id,
        event_type = event_type,
        timestamp = os.time(),
        attempts = attempts,
        status = "failed",
        error = last_error
    })
    
    return {
        success = false,
        error = last_error,
        event_id = event_id,
        attempts = attempts
    }
end

-- Test webhook handler
local webhook = WebhookHandler:new({
    secret = "webhook_secret",
    max_retries = 2
})

-- Register handlers
webhook:register("order.created", function(payload)
    print(string.format("   Processing order %s: $%.2f", 
        payload.order_id, payload.amount))
    return {processed = true}
end, {
    order_id = {required = true, type = "string"},
    amount = {required = true, type = "number"}
})

webhook:register("user.signup", function(payload)
    print(string.format("   New user signup: %s", payload.email))
    return {welcome_sent = true}
end, {
    email = {required = true, pattern = "^[^@]+@[^@]+$"}
})

-- Test processing
print("\n   Testing webhook processing:")

local result = webhook:process({
    payload = {
        event = "order.created",
        order_id = "ORD-123",
        amount = 99.99
    }
})
print(string.format("   Result: %s (event: %s)", 
    result.success and "Success" or "Failed", result.event_id))

result = webhook:process({
    payload = {
        event = "user.signup",
        email = "invalid-email"
    }
})
print(string.format("   Result: %s - %s", 
    result.success and "Success" or "Failed", result.error or "OK"))

print()

-- ============================================================
-- Pattern 2: Webhook Queue with Deduplication
-- ============================================================

print("2. Webhook Queue with Deduplication")
print("-" .. string.rep("-", 40))

local WebhookQueue = {}
WebhookQueue.__index = WebhookQueue

function WebhookQueue:new(options)
    options = options or {}
    return setmetatable({
        queue = {},
        processing = {},
        processed = {},
        dedup_window = options.dedup_window or 3600,  -- 1 hour
        max_queue_size = options.max_queue_size or 1000
    }, self)
end

function WebhookQueue:generate_dedup_key(webhook)
    -- Generate deduplication key from webhook data
    return string.format("%s_%s_%s", 
        webhook.event_type or "",
        webhook.id or "",
        webhook.timestamp or "")
end

function WebhookQueue:is_duplicate(webhook)
    local dedup_key = self:generate_dedup_key(webhook)
    local previous = self.processed[dedup_key]
    
    if not previous then
        return false
    end
    
    -- Check if within dedup window
    local age = os.time() - previous.processed_at
    if age > self.dedup_window then
        -- Expired, not a duplicate
        self.processed[dedup_key] = nil
        return false
    end
    
    return true
end

function WebhookQueue:enqueue(webhook)
    -- Check for duplicates
    if self:is_duplicate(webhook) then
        print(string.format("   Duplicate webhook ignored: %s", 
            self:generate_dedup_key(webhook)))
        return false, "duplicate"
    end
    
    -- Check queue size
    if #self.queue >= self.max_queue_size then
        return false, "queue_full"
    end
    
    -- Add to queue
    webhook.queued_at = os.time()
    webhook.retry_count = 0
    table.insert(self.queue, webhook)
    
    print(string.format("   Webhook queued: %s (queue size: %d)", 
        webhook.event_type, #self.queue))
    
    return true
end

function WebhookQueue:process_next()
    if #self.queue == 0 then
        return nil
    end
    
    -- Get next webhook
    local webhook = table.remove(self.queue, 1)
    local dedup_key = self:generate_dedup_key(webhook)
    
    -- Mark as processing
    self.processing[dedup_key] = webhook
    
    -- Simulate processing
    print(string.format("   Processing: %s", webhook.event_type))
    
    -- Mark as processed
    self.processing[dedup_key] = nil
    self.processed[dedup_key] = {
        webhook = webhook,
        processed_at = os.time()
    }
    
    -- Cleanup old processed entries
    self:cleanup_processed()
    
    return webhook
end

function WebhookQueue:cleanup_processed()
    local now = os.time()
    local to_remove = {}
    
    for key, entry in pairs(self.processed) do
        if now - entry.processed_at > self.dedup_window then
            table.insert(to_remove, key)
        end
    end
    
    for _, key in ipairs(to_remove) do
        self.processed[key] = nil
    end
    
    if #to_remove > 0 then
        print(string.format("   Cleaned up %d expired entries", #to_remove))
    end
end

-- Test webhook queue
local queue = WebhookQueue:new({
    dedup_window = 60,
    max_queue_size = 10
})

-- Test deduplication
print("\n   Testing webhook queue:")

queue:enqueue({
    event_type = "payment.received",
    id = "PAY-001",
    timestamp = os.time()
})

queue:enqueue({
    event_type = "payment.received",
    id = "PAY-001",
    timestamp = os.time()  -- Duplicate
})

queue:enqueue({
    event_type = "payment.received",
    id = "PAY-002",
    timestamp = os.time()
})

-- Process queue
while true do
    local webhook = queue:process_next()
    if not webhook then
        break
    end
end

print()

-- ============================================================
-- Pattern 3: Webhook Relay and Fan-out
-- ============================================================

print("3. Webhook Relay and Fan-out")
print("-" .. string.rep("-", 40))

local WebhookRelay = {}
WebhookRelay.__index = WebhookRelay

function WebhookRelay:new()
    return setmetatable({
        subscribers = {},
        transformers = {},
        filters = {},
        stats = {
            received = 0,
            relayed = 0,
            filtered = 0
        }
    }, self)
end

function WebhookRelay:subscribe(name, config)
    self.subscribers[name] = {
        name = name,
        endpoint = config.endpoint,
        events = config.events or {"*"},  -- Subscribe to specific events or all
        transform = config.transform,
        filter = config.filter,
        active = true
    }
    
    print(string.format("   Subscriber '%s' registered for events: %s", 
        name, table.concat(config.events or {"*"}, ", ")))
end

function WebhookRelay:receive(webhook)
    self.stats.received = self.stats.received + 1
    
    print(string.format("   Received webhook: %s", webhook.event_type))
    
    -- Fan out to subscribers
    for name, subscriber in pairs(self.subscribers) do
        if subscriber.active then
            self:relay_to_subscriber(webhook, subscriber)
        end
    end
end

function WebhookRelay:relay_to_subscriber(webhook, subscriber)
    -- Check if subscriber wants this event
    if not self:matches_events(webhook.event_type, subscriber.events) then
        return
    end
    
    -- Apply filter if defined
    if subscriber.filter then
        local should_relay = subscriber.filter(webhook)
        if not should_relay then
            self.stats.filtered = self.stats.filtered + 1
            return
        end
    end
    
    -- Transform webhook if needed
    local payload = webhook
    if subscriber.transform then
        payload = subscriber.transform(webhook)
    end
    
    -- Relay to subscriber
    print(string.format("     Relaying to %s: %s", 
        subscriber.name, subscriber.endpoint))
    
    self.stats.relayed = self.stats.relayed + 1
    
    -- Simulate sending to endpoint
    -- In real implementation, would make HTTP request
    return {
        subscriber = subscriber.name,
        endpoint = subscriber.endpoint,
        payload = payload
    }
end

function WebhookRelay:matches_events(event_type, subscribed_events)
    for _, pattern in ipairs(subscribed_events) do
        if pattern == "*" then
            return true
        end
        
        -- Support wildcards like "order.*"
        local regex_pattern = "^" .. pattern:gsub("%.", "%%."):gsub("%*", ".*") .. "$"
        if string.match(event_type, regex_pattern) then
            return true
        end
    end
    
    return false
end

-- Test webhook relay
local relay = WebhookRelay:new()

-- Subscribe different services
relay:subscribe("analytics", {
    endpoint = "https://analytics.example.com/events",
    events = {"*"}  -- All events
})

relay:subscribe("billing", {
    endpoint = "https://billing.example.com/webhooks",
    events = {"order.*", "payment.*"},
    filter = function(webhook)
        -- Only relay high-value transactions
        return webhook.amount and webhook.amount > 100
    end
})

relay:subscribe("notifications", {
    endpoint = "https://notify.example.com/events",
    events = {"user.*", "order.completed"},
    transform = function(webhook)
        -- Transform to notification format
        return {
            type = "notification",
            event = webhook.event_type,
            user_id = webhook.user_id,
            message = "Event: " .. webhook.event_type
        }
    end
})

-- Test relaying
print("\n   Testing webhook relay:")

relay:receive({
    event_type = "order.created",
    order_id = "ORD-456",
    amount = 250.00,
    user_id = "USER-123"
})

relay:receive({
    event_type = "user.login",
    user_id = "USER-123",
    timestamp = os.time()
})

print(string.format("\n   Stats: Received=%d, Relayed=%d, Filtered=%d",
    relay.stats.received, relay.stats.relayed, relay.stats.filtered))

print()

-- ============================================================
-- Pattern 4: Webhook Security and Rate Limiting
-- ============================================================

print("4. Webhook Security and Rate Limiting")
print("-" .. string.rep("-", 40))

local SecureWebhookHandler = {}
SecureWebhookHandler.__index = SecureWebhookHandler

function SecureWebhookHandler:new(options)
    options = options or {}
    return setmetatable({
        allowed_ips = options.allowed_ips or {},
        rate_limits = {},
        blocked_ips = {},
        secret_keys = options.secret_keys or {},
        request_log = {}
    }, self)
end

function SecureWebhookHandler:check_ip_whitelist(ip)
    if #self.allowed_ips == 0 then
        return true  -- No whitelist configured
    end
    
    for _, allowed in ipairs(self.allowed_ips) do
        if ip == allowed or string.match(ip, allowed) then
            return true
        end
    end
    
    return false
end

function SecureWebhookHandler:check_rate_limit(ip, limit)
    limit = limit or 100  -- Default 100 requests per window
    
    local key = "rate_" .. ip
    local now = os.time()
    
    if not self.rate_limits[key] then
        self.rate_limits[key] = {
            count = 0,
            window_start = now
        }
    end
    
    local rate_data = self.rate_limits[key]
    
    -- Reset window if expired (1 minute window)
    if now - rate_data.window_start > 60 then
        rate_data.count = 0
        rate_data.window_start = now
    end
    
    rate_data.count = rate_data.count + 1
    
    if rate_data.count > limit then
        -- Block IP temporarily
        self.blocked_ips[ip] = now + 300  -- Block for 5 minutes
        return false
    end
    
    return true
end

function SecureWebhookHandler:verify_hmac(payload, signature, secret)
    -- Simulate HMAC verification
    local payload_str = ""
    for k, v in pairs(payload) do
        payload_str = payload_str .. k .. "=" .. tostring(v) .. "&"
    end
    
    local expected = "sha256=" .. secret .. "_" .. string.sub(payload_str, 1, 10)
    return signature == expected
end

function SecureWebhookHandler:process_secure(request)
    local ip = request.ip or "0.0.0.0"
    
    -- Log request
    table.insert(self.request_log, {
        ip = ip,
        timestamp = os.time(),
        path = request.path
    })
    
    -- Check if IP is blocked
    if self.blocked_ips[ip] then
        if os.time() < self.blocked_ips[ip] then
            return {
                status = 403,
                error = "IP temporarily blocked"
            }
        else
            self.blocked_ips[ip] = nil  -- Unblock
        end
    end
    
    -- Check IP whitelist
    if not self:check_ip_whitelist(ip) then
        return {
            status = 403,
            error = "IP not whitelisted"
        }
    end
    
    -- Check rate limit
    if not self:check_rate_limit(ip, 10) then  -- 10 requests per minute
        return {
            status = 429,
            error = "Rate limit exceeded"
        }
    end
    
    -- Verify signature
    local signature = request.headers and request.headers["X-Webhook-Signature"]
    if signature then
        local valid = false
        for _, secret in ipairs(self.secret_keys) do
            if self:verify_hmac(request.payload, signature, secret) then
                valid = true
                break
            end
        end
        
        if not valid then
            return {
                status = 401,
                error = "Invalid signature"
            }
        end
    end
    
    print(string.format("   ✅ Request from %s passed all security checks", ip))
    
    return {
        status = 200,
        message = "Webhook processed successfully"
    }
end

-- Test secure webhook handler
local secure = SecureWebhookHandler:new({
    allowed_ips = {"192.168.1.*", "10.0.0.1"},
    secret_keys = {"secret1", "secret2"}
})

print("\n   Testing secure webhook handler:")

-- Test from allowed IP
local response = secure:process_secure({
    ip = "192.168.1.100",
    path = "/webhook",
    payload = {event = "test"}
})
print(string.format("   From 192.168.1.100: %d - %s", 
    response.status, response.error or response.message))

-- Test from blocked IP
response = secure:process_secure({
    ip = "1.2.3.4",
    path = "/webhook",
    payload = {event = "test"}
})
print(string.format("   From 1.2.3.4: %d - %s", 
    response.status, response.error or response.message))

-- Test rate limiting
print("\n   Testing rate limiting:")
for i = 1, 12 do
    response = secure:process_secure({
        ip = "10.0.0.1",
        path = "/webhook",
        payload = {event = "test" .. i}
    })
    
    if response.status ~= 200 then
        print(string.format("   Request %d: %d - %s", 
            i, response.status, response.error))
        break
    end
end

print()
print("🎯 Key Takeaways:")
print("   • Validate webhook signatures for security")
print("   • Implement deduplication to prevent duplicate processing")
print("   • Use fan-out patterns for multiple consumers")
print("   • Apply rate limiting to prevent abuse")
print("   • Log and monitor webhook activity")