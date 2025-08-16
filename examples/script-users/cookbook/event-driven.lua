-- Cookbook: Event-Driven Architecture - Building Reactive Systems
-- Purpose: Implement patterns for event-driven architectures and reactive systems
-- Prerequisites: None
-- Expected Output: Demonstration of event-driven patterns
-- Version: 0.7.0
-- Tags: cookbook, events, reactive, messaging, production

print("=== Event-Driven Architecture Patterns ===\n")

-- ============================================================
-- Pattern 1: Event Bus with Topics
-- ============================================================

print("1. Event Bus with Topics")
print("-" .. string.rep("-", 40))

local EventBus = {}
EventBus.__index = EventBus

function EventBus:new()
    return setmetatable({
        subscribers = {},
        topics = {},
        event_queue = {},
        event_history = {},
        max_history = 100
    }, self)
end

function EventBus:create_topic(name, config)
    config = config or {}
    self.topics[name] = {
        name = name,
        subscribers = {},
        persistent = config.persistent or false,
        retention = config.retention or 3600,
        message_count = 0
    }
    print(string.format("   Created topic: %s%s", 
        name, config.persistent and " (persistent)" or ""))
end

function EventBus:subscribe(topic, subscriber_id, handler, filter)
    if not self.topics[topic] then
        self:create_topic(topic)
    end
    
    local subscription = {
        id = subscriber_id,
        handler = handler,
        filter = filter,
        received = 0,
        subscribed_at = os.time()
    }
    
    self.topics[topic].subscribers[subscriber_id] = subscription
    
    -- Global subscriber registry
    if not self.subscribers[subscriber_id] then
        self.subscribers[subscriber_id] = {}
    end
    table.insert(self.subscribers[subscriber_id], topic)
    
    print(string.format("   %s subscribed to topic '%s'", subscriber_id, topic))
    
    return function()
        -- Unsubscribe function
        self.topics[topic].subscribers[subscriber_id] = nil
    end
end

function EventBus:publish(topic, event)
    if not self.topics[topic] then
        self:create_topic(topic)
    end
    
    local topic_data = self.topics[topic]
    topic_data.message_count = topic_data.message_count + 1
    
    -- Add metadata
    event._topic = topic
    event._timestamp = os.time()
    event._id = topic .. "_" .. topic_data.message_count
    
    -- Store in history
    table.insert(self.event_history, event)
    if #self.event_history > self.max_history then
        table.remove(self.event_history, 1)
    end
    
    -- Deliver to subscribers
    local delivered = 0
    for sub_id, subscription in pairs(topic_data.subscribers) do
        if not subscription.filter or subscription.filter(event) then
            subscription.handler(event)
            subscription.received = subscription.received + 1
            delivered = delivered + 1
        end
    end
    
    print(string.format("   Published to '%s': delivered to %d subscribers", 
        topic, delivered))
    
    return event._id
end

function EventBus:get_topic_stats(topic)
    local topic_data = self.topics[topic]
    if not topic_data then
        return nil
    end
    
    local subscriber_count = 0
    local total_received = 0
    
    for _, sub in pairs(topic_data.subscribers) do
        subscriber_count = subscriber_count + 1
        total_received = total_received + sub.received
    end
    
    return {
        name = topic,
        subscribers = subscriber_count,
        messages_published = topic_data.message_count,
        messages_delivered = total_received
    }
end

-- Test event bus
local bus = EventBus:new()

-- Create topics
bus:create_topic("orders", {persistent = true})
bus:create_topic("users", {persistent = false})

-- Subscribe handlers
bus:subscribe("orders", "billing", function(event)
    print(string.format("     [Billing] Order %s: $%.2f", 
        event.order_id, event.amount))
end)

bus:subscribe("orders", "inventory", function(event)
    print(string.format("     [Inventory] Update stock for order %s", 
        event.order_id))
end, function(event)
    -- Only care about completed orders
    return event.status == "completed"
end)

bus:subscribe("users", "notifications", function(event)
    print(string.format("     [Notifications] User event: %s", event.action))
end)

-- Publish events
print("\n   Publishing events:")
bus:publish("orders", {
    order_id = "ORD-789",
    amount = 150.00,
    status = "pending"
})

bus:publish("orders", {
    order_id = "ORD-790",
    amount = 200.00,
    status = "completed"
})

bus:publish("users", {
    user_id = "USER-456",
    action = "login"
})

-- Show stats
print("\n   Topic stats:")
local stats = bus:get_topic_stats("orders")
print(string.format("   Orders: %d subscribers, %d messages, %d delivered",
    stats.subscribers, stats.messages_published, stats.messages_delivered))

print()

-- ============================================================
-- Pattern 2: Event Sourcing with Projections
-- ============================================================

print("2. Event Sourcing with Projections")
print("-" .. string.rep("-", 40))

local EventStore = {}
EventStore.__index = EventStore

function EventStore:new()
    return setmetatable({
        streams = {},
        projections = {},
        global_position = 0
    }, self)
end

function EventStore:append(stream_id, event_type, data)
    if not self.streams[stream_id] then
        self.streams[stream_id] = {
            id = stream_id,
            events = {},
            version = 0
        }
    end
    
    local stream = self.streams[stream_id]
    stream.version = stream.version + 1
    self.global_position = self.global_position + 1
    
    local event = {
        stream_id = stream_id,
        event_type = event_type,
        data = data,
        stream_version = stream.version,
        global_position = self.global_position,
        timestamp = os.time()
    }
    
    table.insert(stream.events, event)
    
    -- Update projections
    self:update_projections(event)
    
    return event
end

function EventStore:read_stream(stream_id, from_version)
    from_version = from_version or 1
    
    local stream = self.streams[stream_id]
    if not stream then
        return {}
    end
    
    local events = {}
    for i = from_version, #stream.events do
        table.insert(events, stream.events[i])
    end
    
    return events
end

function EventStore:create_projection(name, handlers)
    self.projections[name] = {
        name = name,
        handlers = handlers,
        state = {},
        last_position = 0
    }
    
    -- Replay all events to build projection
    self:rebuild_projection(name)
    
    print(string.format("   Created projection: %s", name))
end

function EventStore:update_projections(event)
    for name, projection in pairs(self.projections) do
        local handler = projection.handlers[event.event_type]
        if handler then
            projection.state = handler(projection.state, event)
            projection.last_position = event.global_position
        end
    end
end

function EventStore:rebuild_projection(name)
    local projection = self.projections[name]
    if not projection then
        return
    end
    
    projection.state = {}
    projection.last_position = 0
    
    -- Replay all events
    for _, stream in pairs(self.streams) do
        for _, event in ipairs(stream.events) do
            local handler = projection.handlers[event.event_type]
            if handler then
                projection.state = handler(projection.state, event)
                projection.last_position = event.global_position
            end
        end
    end
    
    print(string.format("   Rebuilt projection '%s' up to position %d", 
        name, projection.last_position))
end

function EventStore:get_projection(name)
    local projection = self.projections[name]
    return projection and projection.state or nil
end

-- Test event sourcing
local store = EventStore:new()

-- Define projection handlers
local order_summary_handlers = {
    ["order.created"] = function(state, event)
        state.total_orders = (state.total_orders or 0) + 1
        state.total_value = (state.total_value or 0) + event.data.amount
        state.orders = state.orders or {}
        state.orders[event.data.order_id] = {
            status = "created",
            amount = event.data.amount
        }
        return state
    end,
    ["order.completed"] = function(state, event)
        state.completed_orders = (state.completed_orders or 0) + 1
        if state.orders and state.orders[event.data.order_id] then
            state.orders[event.data.order_id].status = "completed"
        end
        return state
    end,
    ["order.cancelled"] = function(state, event)
        state.cancelled_orders = (state.cancelled_orders or 0) + 1
        if state.orders and state.orders[event.data.order_id] then
            local order = state.orders[event.data.order_id]
            state.total_value = state.total_value - order.amount
            state.orders[event.data.order_id] = nil
        end
        return state
    end
}

-- Create projection
store:create_projection("order_summary", order_summary_handlers)

-- Append events
print("\n   Appending events:")
store:append("order-001", "order.created", {
    order_id = "order-001",
    amount = 100.00
})

store:append("order-002", "order.created", {
    order_id = "order-002",
    amount = 250.00
})

store:append("order-001", "order.completed", {
    order_id = "order-001"
})

store:append("order-002", "order.cancelled", {
    order_id = "order-002",
    reason = "Out of stock"
})

-- Get projection state
local summary = store:get_projection("order_summary")
print("\n   Order Summary Projection:")
print(string.format("   Total orders: %d", summary.total_orders or 0))
print(string.format("   Completed: %d", summary.completed_orders or 0))
print(string.format("   Cancelled: %d", summary.cancelled_orders or 0))
print(string.format("   Total value: $%.2f", summary.total_value or 0))

print()

-- ============================================================
-- Pattern 3: Command and Query Separation (CQRS)
-- ============================================================

print("3. Command and Query Separation (CQRS)")
print("-" .. string.rep("-", 40))

local CQRS = {}
CQRS.__index = CQRS

function CQRS:new()
    return setmetatable({
        command_handlers = {},
        query_handlers = {},
        event_store = {},
        read_models = {},
        command_log = {}
    }, self)
end

function CQRS:register_command(name, handler, validator)
    self.command_handlers[name] = {
        handler = handler,
        validator = validator
    }
    print(string.format("   Registered command: %s", name))
end

function CQRS:register_query(name, handler)
    self.query_handlers[name] = handler
    print(string.format("   Registered query: %s", name))
end

function CQRS:execute_command(command)
    local handler_info = self.command_handlers[command.type]
    if not handler_info then
        return {
            success = false,
            error = "Unknown command: " .. command.type
        }
    end
    
    -- Validate command
    if handler_info.validator then
        local valid, err = handler_info.validator(command.data)
        if not valid then
            return {
                success = false,
                error = "Validation failed: " .. err
            }
        end
    end
    
    -- Execute command
    local success, result = pcall(handler_info.handler, command.data)
    
    -- Log command
    table.insert(self.command_log, {
        command = command,
        success = success,
        result = result,
        timestamp = os.time()
    })
    
    if success then
        -- Store events
        if result.events then
            for _, event in ipairs(result.events) do
                table.insert(self.event_store, event)
                self:update_read_models(event)
            end
        end
        
        return {
            success = true,
            result = result.data,
            events = result.events
        }
    else
        return {
            success = false,
            error = result
        }
    end
end

function CQRS:execute_query(query)
    local handler = self.query_handlers[query.type]
    if not handler then
        return {
            success = false,
            error = "Unknown query: " .. query.type
        }
    end
    
    local success, result = pcall(handler, query.params, self.read_models)
    
    if success then
        return {
            success = true,
            data = result
        }
    else
        return {
            success = false,
            error = result
        }
    end
end

function CQRS:update_read_models(event)
    -- Update denormalized read models based on events
    if event.type == "ProductCreated" then
        self.read_models.products = self.read_models.products or {}
        self.read_models.products[event.data.id] = {
            id = event.data.id,
            name = event.data.name,
            price = event.data.price,
            stock = event.data.initial_stock
        }
    elseif event.type == "StockUpdated" then
        if self.read_models.products and self.read_models.products[event.data.product_id] then
            self.read_models.products[event.data.product_id].stock = event.data.new_stock
        end
    end
end

-- Test CQRS
local cqrs = CQRS:new()

-- Register commands
cqrs:register_command("CreateProduct", function(data)
    -- Business logic for creating product
    return {
        data = {product_id = "PROD-" .. os.time()},
        events = {{
            type = "ProductCreated",
            data = {
                id = "PROD-" .. os.time(),
                name = data.name,
                price = data.price,
                initial_stock = data.stock
            }
        }}
    }
end, function(data)
    -- Validation
    if not data.name or data.name == "" then
        return false, "Product name required"
    end
    if not data.price or data.price <= 0 then
        return false, "Valid price required"
    end
    return true
end)

cqrs:register_command("UpdateStock", function(data)
    return {
        data = {updated = true},
        events = {{
            type = "StockUpdated",
            data = {
                product_id = data.product_id,
                new_stock = data.stock
            }
        }}
    }
end)

-- Register queries
cqrs:register_query("GetProduct", function(params, read_models)
    if read_models.products then
        return read_models.products[params.id]
    end
    return nil
end)

cqrs:register_query("ListProducts", function(params, read_models)
    local products = {}
    if read_models.products then
        for _, product in pairs(read_models.products) do
            table.insert(products, product)
        end
    end
    return products
end)

-- Execute commands
print("\n   Executing commands:")

local result = cqrs:execute_command({
    type = "CreateProduct",
    data = {
        name = "Widget",
        price = 29.99,
        stock = 100
    }
})
print(string.format("   CreateProduct: %s", 
    result.success and "Success" or result.error))

result = cqrs:execute_command({
    type = "UpdateStock",
    data = {
        product_id = "PROD-" .. os.time(),
        stock = 75
    }
})
print(string.format("   UpdateStock: %s", 
    result.success and "Success" or result.error))

-- Execute queries
print("\n   Executing queries:")

result = cqrs:execute_query({
    type = "ListProducts",
    params = {}
})

if result.success and result.data then
    print(string.format("   Products found: %d", #result.data))
    for _, product in ipairs(result.data) do
        print(string.format("     - %s: $%.2f (stock: %d)", 
            product.name, product.price, product.stock))
    end
end

print()

-- ============================================================
-- Pattern 4: Saga Pattern for Distributed Transactions
-- ============================================================

print("4. Saga Pattern for Distributed Transactions")
print("-" .. string.rep("-", 40))

local Saga = {}
Saga.__index = Saga

function Saga:new(name)
    return setmetatable({
        name = name,
        steps = {},
        compensations = {},
        state = "pending",
        completed_steps = {},
        context = {}
    }, self)
end

function Saga:add_step(name, action, compensation)
    table.insert(self.steps, {
        name = name,
        action = action,
        compensation = compensation
    })
    print(string.format("   Added step '%s' to saga '%s'", name, self.name))
end

function Saga:execute()
    print(string.format("\n   Executing saga: %s", self.name))
    self.state = "running"
    
    for i, step in ipairs(self.steps) do
        print(string.format("   Step %d: %s", i, step.name))
        
        local success, result = pcall(step.action, self.context)
        
        if success then
            print(string.format("     ✅ %s completed", step.name))
            table.insert(self.completed_steps, {
                step = step,
                result = result
            })
            
            -- Update context with result
            if type(result) == "table" then
                for k, v in pairs(result) do
                    self.context[k] = v
                end
            end
        else
            print(string.format("     ❌ %s failed: %s", step.name, result))
            self.state = "failed"
            
            -- Compensate completed steps
            self:compensate()
            
            return false, result
        end
    end
    
    self.state = "completed"
    print(string.format("   ✅ Saga '%s' completed successfully", self.name))
    return true, self.context
end

function Saga:compensate()
    print(string.format("   ⚠️  Compensating saga '%s'", self.name))
    
    -- Compensate in reverse order
    for i = #self.completed_steps, 1, -1 do
        local completed = self.completed_steps[i]
        local step = completed.step
        
        if step.compensation then
            print(string.format("     Compensating: %s", step.name))
            local success, err = pcall(step.compensation, self.context)
            
            if not success then
                print(string.format("     ⚠️  Compensation failed for %s: %s", 
                    step.name, err))
            end
        end
    end
    
    self.state = "compensated"
end

-- Test saga pattern
local order_saga = Saga:new("OrderProcessing")

-- Add saga steps
order_saga:add_step("ValidateOrder", 
    function(ctx)
        print("     Validating order...")
        if not ctx.order_id then
            error("Order ID required")
        end
        return {validated = true}
    end,
    function(ctx)
        print("     Rolling back order validation")
    end
)

order_saga:add_step("ReserveInventory",
    function(ctx)
        print("     Reserving inventory...")
        -- Simulate inventory reservation
        return {reservation_id = "RES-" .. os.time()}
    end,
    function(ctx)
        print("     Releasing inventory reservation: " .. (ctx.reservation_id or "none"))
    end
)

order_saga:add_step("ProcessPayment",
    function(ctx)
        print("     Processing payment...")
        -- Simulate payment failure
        if ctx.fail_payment then
            error("Payment declined")
        end
        return {payment_id = "PAY-" .. os.time()}
    end,
    function(ctx)
        print("     Refunding payment: " .. (ctx.payment_id or "none"))
    end
)

order_saga:add_step("ShipOrder",
    function(ctx)
        print("     Creating shipment...")
        return {tracking_number = "TRACK-" .. os.time()}
    end,
    function(ctx)
        print("     Cancelling shipment: " .. (ctx.tracking_number or "none"))
    end
)

-- Execute successful saga
print("\n   Test 1: Successful saga")
order_saga.context = {order_id = "ORD-123"}
local success, result = order_saga:execute()

-- Reset and test failed saga
print("\n   Test 2: Failed saga (payment fails)")
order_saga = Saga:new("OrderProcessing")

-- Re-add steps (simplified for demo)
order_saga:add_step("ValidateOrder", 
    function(ctx) return {validated = true} end,
    function(ctx) print("     Rolling back validation") end
)

order_saga:add_step("ReserveInventory",
    function(ctx) return {reservation_id = "RES-" .. os.time()} end,
    function(ctx) print("     Releasing reservation") end
)

order_saga:add_step("ProcessPayment",
    function(ctx) error("Payment declined") end,
    function(ctx) print("     Refunding payment") end
)

order_saga.context = {order_id = "ORD-456", fail_payment = true}
success, result = order_saga:execute()

print()
print("🎯 Key Takeaways:")
print("   • Event bus enables decoupled communication")
print("   • Event sourcing provides complete audit trail")
print("   • CQRS optimizes for different read/write patterns")
print("   • Sagas manage distributed transactions")
print("   • Compensations handle failure scenarios")