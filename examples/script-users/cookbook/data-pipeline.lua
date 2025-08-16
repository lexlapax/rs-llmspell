-- Cookbook: Data Pipeline Patterns - Building Data Processing Pipelines
-- Purpose: Implement patterns for data ingestion, transformation, and processing
-- Prerequisites: Tools: database_connector, file_operations (for production use), data sources as needed
-- Expected Output: Demonstration of data pipeline patterns
-- Version: 0.7.0
-- Tags: cookbook, data, pipeline, etl, streaming, production

print("=== Data Pipeline Patterns ===\n")

-- ============================================================
-- Pattern 1: ETL Pipeline (Extract, Transform, Load)
-- ============================================================

print("1. ETL Pipeline Pattern")
print("-" .. string.rep("-", 40))

local ETLPipeline = {}
ETLPipeline.__index = ETLPipeline

function ETLPipeline:new(name)
    return setmetatable({
        name = name,
        extractors = {},
        transformers = {},
        loaders = {},
        stages = {},
        stats = {
            extracted = 0,
            transformed = 0,
            loaded = 0,
            errors = 0
        }
    }, self)
end

function ETLPipeline:add_extractor(name, extractor_fn)
    self.extractors[name] = extractor_fn
    print(string.format("   Added extractor: %s", name))
end

function ETLPipeline:add_transformer(name, transformer_fn)
    self.transformers[name] = transformer_fn
    print(string.format("   Added transformer: %s", name))
end

function ETLPipeline:add_loader(name, loader_fn)
    self.loaders[name] = loader_fn
    print(string.format("   Added loader: %s", name))
end

function ETLPipeline:define_flow(stages)
    self.stages = stages
    print(string.format("   Defined pipeline flow with %d stages", #stages))
end

function ETLPipeline:run(input)
    print(string.format("\n   Running ETL pipeline: %s", self.name))
    local data = input
    
    for i, stage in ipairs(self.stages) do
        print(string.format("   Stage %d: %s (%s)", i, stage.name, stage.type))
        
        local processor = nil
        if stage.type == "extract" then
            processor = self.extractors[stage.processor]
        elseif stage.type == "transform" then
            processor = self.transformers[stage.processor]
        elseif stage.type == "load" then
            processor = self.loaders[stage.processor]
        end
        
        if not processor then
            print(string.format("     ❌ Processor not found: %s", stage.processor))
            self.stats.errors = self.stats.errors + 1
            return nil, "Processor not found"
        end
        
        local success, result = pcall(processor, data)
        
        if success then
            data = result
            
            if stage.type == "extract" then
                self.stats.extracted = self.stats.extracted + 
                    (type(result) == "table" and #result or 1)
            elseif stage.type == "transform" then
                self.stats.transformed = self.stats.transformed + 1
            elseif stage.type == "load" then
                self.stats.loaded = self.stats.loaded + 1
            end
            
            print(string.format("     ✅ Processed successfully"))
        else
            print(string.format("     ❌ Error: %s", result))
            self.stats.errors = self.stats.errors + 1
            
            if stage.error_handler then
                data = stage.error_handler(data, result)
            else
                return nil, result
            end
        end
    end
    
    return data
end

function ETLPipeline:get_stats()
    return {
        extracted = self.stats.extracted,
        transformed = self.stats.transformed,
        loaded = self.stats.loaded,
        errors = self.stats.errors,
        success_rate = self.stats.errors == 0 and 100 or 
            ((self.stats.loaded / (self.stats.loaded + self.stats.errors)) * 100)
    }
end

-- Test ETL Pipeline
local etl = ETLPipeline:new("UserDataPipeline")

-- Add extractors
etl:add_extractor("csv_reader", function(config)
    -- Simulate CSV extraction
    return {
        {name = "Alice", age = "30", email = "alice@example.com"},
        {name = "Bob", age = "25", email = "bob@example.com"},
        {name = "Charlie", age = "invalid", email = "charlie"}
    }
end)

-- Add transformers
etl:add_transformer("validate", function(records)
    local valid_records = {}
    for _, record in ipairs(records) do
        -- Validate email
        if record.email and string.find(record.email, "@") then
            -- Validate age
            local age = tonumber(record.age)
            if age then
                record.age = age
                table.insert(valid_records, record)
            else
                print(string.format("       Skipped invalid record: %s", record.name))
            end
        end
    end
    return valid_records
end)

etl:add_transformer("enrich", function(records)
    for _, record in ipairs(records) do
        -- Add computed fields
        record.id = "USER_" .. os.time() .. "_" .. math.random(1000)
        record.created_at = os.time()
        record.age_group = record.age < 30 and "young" or "adult"
    end
    return records
end)

-- Add loaders
etl:add_loader("database", function(records)
    print(string.format("       Loading %d records to database", #records))
    for _, record in ipairs(records) do
        print(string.format("         - %s (%s)", record.name, record.id))
    end
    return {success = true, count = #records}
end)

-- Define pipeline flow
etl:define_flow({
    {name = "Extract CSV", type = "extract", processor = "csv_reader"},
    {name = "Validate Data", type = "transform", processor = "validate"},
    {name = "Enrich Data", type = "transform", processor = "enrich"},
    {name = "Load to DB", type = "load", processor = "database"}
})

-- Run pipeline
local result = etl:run({source = "users.csv"})

-- Show stats
local stats = etl:get_stats()
print(string.format("\n   Pipeline Stats: Extracted=%d, Transformed=%d, Loaded=%d, Errors=%d",
    stats.extracted, stats.transformed, stats.loaded, stats.errors))

print()

-- ============================================================
-- Pattern 2: Stream Processing Pipeline
-- ============================================================

print("2. Stream Processing Pipeline")
print("-" .. string.rep("-", 40))

local StreamPipeline = {}
StreamPipeline.__index = StreamPipeline

function StreamPipeline:new()
    return setmetatable({
        processors = {},
        buffer = {},
        window_size = 5,
        watermark = 0,
        state = {}
    }, self)
end

function StreamPipeline:add_processor(name, processor)
    table.insert(self.processors, {
        name = name,
        fn = processor.fn,
        type = processor.type or "map",  -- map, filter, aggregate, window
        window_size = processor.window_size
    })
    print(string.format("   Added processor: %s (%s)", name, processor.type or "map"))
end

function StreamPipeline:process_event(event)
    local current = event
    
    for _, processor in ipairs(self.processors) do
        if processor.type == "map" then
            current = processor.fn(current)
            
        elseif processor.type == "filter" then
            if not processor.fn(current) then
                return nil  -- Filtered out
            end
            
        elseif processor.type == "aggregate" then
            -- Update state
            local key = current.key or "default"
            self.state[key] = processor.fn(self.state[key], current)
            current = self.state[key]
            
        elseif processor.type == "window" then
            -- Windowed aggregation
            table.insert(self.buffer, current)
            
            if #self.buffer >= (processor.window_size or self.window_size) then
                local window_data = {}
                for i = 1, processor.window_size or self.window_size do
                    table.insert(window_data, table.remove(self.buffer, 1))
                end
                
                current = processor.fn(window_data)
            else
                return nil  -- Wait for more events
            end
        end
        
        if current == nil then
            return nil
        end
    end
    
    return current
end

function StreamPipeline:process_batch(events)
    local results = {}
    
    for _, event in ipairs(events) do
        local result = self:process_event(event)
        if result then
            table.insert(results, result)
        end
    end
    
    return results
end

-- Test stream pipeline
local stream = StreamPipeline:new()

-- Add processors
stream:add_processor("parse", {
    type = "map",
    fn = function(event)
        -- Parse raw event
        return {
            timestamp = event.ts or os.time(),
            user_id = event.user,
            action = event.action,
            value = event.value or 1
        }
    end
})

stream:add_processor("filter_valid", {
    type = "filter",
    fn = function(event)
        return event.user_id ~= nil and event.action ~= nil
    end
})

stream:add_processor("count_by_action", {
    type = "aggregate",
    fn = function(state, event)
        state = state or {count = 0, total_value = 0}
        state.count = state.count + 1
        state.total_value = state.total_value + event.value
        state.action = event.action
        return state
    end
})

stream:add_processor("window_average", {
    type = "window",
    window_size = 3,
    fn = function(window)
        local sum = 0
        for _, item in ipairs(window) do
            sum = sum + (item.total_value or 0)
        end
        return {
            window_avg = sum / #window,
            window_count = #window
        }
    end
})

-- Process stream of events
print("\n   Processing event stream:")

local events = {
    {ts = 1, user = "u1", action = "click", value = 1},
    {ts = 2, user = "u2", action = "view", value = 2},
    {ts = 3, user = nil, action = "error"},  -- Will be filtered
    {ts = 4, user = "u3", action = "click", value = 3},
    {ts = 5, user = "u1", action = "purchase", value = 100},
    {ts = 6, user = "u2", action = "click", value = 1}
}

for i, event in ipairs(events) do
    print(string.format("   Event %d: user=%s, action=%s", 
        i, event.user or "nil", event.action))
    
    local result = stream:process_event(event)
    if result then
        if result.window_avg then
            print(string.format("     Window result: avg=%.2f", result.window_avg))
        elseif result.count then
            print(string.format("     Aggregate: count=%d, total=%.2f", 
                result.count, result.total_value))
        end
    end
end

print()

-- ============================================================
-- Pattern 3: Data Quality Pipeline
-- ============================================================

print("3. Data Quality Pipeline")
print("-" .. string.rep("-", 40))

local DataQualityPipeline = {}
DataQualityPipeline.__index = DataQualityPipeline

function DataQualityPipeline:new()
    return setmetatable({
        rules = {},
        metrics = {},
        quarantine = {},
        stats = {
            total = 0,
            passed = 0,
            failed = 0,
            quarantined = 0
        }
    }, self)
end

function DataQualityPipeline:add_rule(name, rule)
    self.rules[name] = {
        name = name,
        check = rule.check,
        severity = rule.severity or "warning",  -- warning, error, critical
        action = rule.action or "log"  -- log, quarantine, reject, fix
    }
    print(string.format("   Added quality rule: %s (severity: %s)", 
        name, rule.severity or "warning"))
end

function DataQualityPipeline:check_quality(data)
    self.stats.total = self.stats.total + 1
    local issues = {}
    local quality_score = 100
    
    for rule_name, rule in pairs(self.rules) do
        local passed, message = rule.check(data)
        
        if not passed then
            table.insert(issues, {
                rule = rule_name,
                severity = rule.severity,
                message = message or "Check failed",
                action = rule.action
            })
            
            -- Deduct from quality score
            if rule.severity == "critical" then
                quality_score = quality_score - 50
            elseif rule.severity == "error" then
                quality_score = quality_score - 20
            else
                quality_score = quality_score - 5
            end
        end
    end
    
    quality_score = math.max(0, quality_score)
    
    -- Take actions based on issues
    local processed_data = data
    local status = "passed"
    
    for _, issue in ipairs(issues) do
        if issue.action == "reject" then
            self.stats.failed = self.stats.failed + 1
            return nil, "Rejected: " .. issue.message
            
        elseif issue.action == "quarantine" then
            table.insert(self.quarantine, {
                data = data,
                issue = issue,
                timestamp = os.time()
            })
            self.stats.quarantined = self.stats.quarantined + 1
            status = "quarantined"
            
        elseif issue.action == "fix" then
            -- Attempt to fix the data
            processed_data = self:attempt_fix(processed_data, issue)
        end
    end
    
    if status == "passed" then
        self.stats.passed = self.stats.passed + 1
    end
    
    return processed_data, {
        status = status,
        quality_score = quality_score,
        issues = issues
    }
end

function DataQualityPipeline:attempt_fix(data, issue)
    -- Simple fix attempts
    if issue.rule == "missing_field" then
        -- Add default value
        for key, value in pairs(data) do
            if value == nil or value == "" then
                data[key] = "N/A"
            end
        end
    elseif issue.rule == "format" then
        -- Try to normalize format
        if data.email then
            data.email = string.lower(data.email)
        end
    end
    
    return data
end

function DataQualityPipeline:get_metrics()
    return {
        total_processed = self.stats.total,
        passed = self.stats.passed,
        failed = self.stats.failed,
        quarantined = self.stats.quarantined,
        pass_rate = self.stats.total > 0 and 
            (self.stats.passed / self.stats.total * 100) or 0,
        quarantine_size = #self.quarantine
    }
end

-- Test data quality pipeline
local quality = DataQualityPipeline:new()

-- Add quality rules
quality:add_rule("completeness", {
    check = function(data)
        local required = {"id", "name", "email"}
        for _, field in ipairs(required) do
            if not data[field] or data[field] == "" then
                return false, "Missing required field: " .. field
            end
        end
        return true
    end,
    severity = "error",
    action = "reject"
})

quality:add_rule("email_format", {
    check = function(data)
        if data.email and not string.find(data.email, "^[^@]+@[^@]+%.[^@]+$") then
            return false, "Invalid email format"
        end
        return true
    end,
    severity = "warning",
    action = "fix"
})

quality:add_rule("age_range", {
    check = function(data)
        if data.age then
            local age = tonumber(data.age)
            if not age or age < 0 or age > 150 then
                return false, "Invalid age value"
            end
        end
        return true
    end,
    severity = "error",
    action = "quarantine"
})

-- Test data quality checks
print("\n   Testing data quality:")

local test_data = {
    {id = "1", name = "Alice", email = "alice@example.com", age = 30},
    {id = "2", name = "Bob", email = "invalid-email", age = 25},
    {id = "3", name = "", email = "charlie@example.com", age = 200},
    {id = "4", name = "David", email = "david@example.com", age = 35}
}

for i, record in ipairs(test_data) do
    local result, quality_info = quality:check_quality(record)
    
    if result then
        print(string.format("   Record %d: %s (score: %d)", 
            i, quality_info.status, quality_info.quality_score))
        
        if #quality_info.issues > 0 then
            for _, issue in ipairs(quality_info.issues) do
                print(string.format("     Issue: %s - %s", 
                    issue.rule, issue.message))
            end
        end
    else
        print(string.format("   Record %d: Rejected - %s", i, quality_info))
    end
end

-- Show metrics
local metrics = quality:get_metrics()
print(string.format("\n   Quality Metrics: Pass rate: %.1f%%, Quarantined: %d",
    metrics.pass_rate, metrics.quarantined))

print()

-- ============================================================
-- Pattern 4: Change Data Capture (CDC) Pipeline
-- ============================================================

print("4. Change Data Capture Pipeline")
print("-" .. string.rep("-", 40))

local CDCPipeline = {}
CDCPipeline.__index = CDCPipeline

function CDCPipeline:new()
    return setmetatable({
        snapshot = {},
        change_log = {},
        subscribers = {},
        sequence = 0
    }, self)
end

function CDCPipeline:take_snapshot(data)
    self.snapshot = {}
    for key, value in pairs(data) do
        self.snapshot[key] = self:deep_copy(value)
    end
    
    print(string.format("   Snapshot taken with %d items", self:count_items(data)))
end

function CDCPipeline:capture_changes(new_data)
    local changes = {
        added = {},
        modified = {},
        deleted = {},
        timestamp = os.time(),
        sequence = self.sequence + 1
    }
    
    -- Find added and modified
    for key, new_value in pairs(new_data) do
        local old_value = self.snapshot[key]
        
        if old_value == nil then
            changes.added[key] = new_value
        elseif not self:deep_equal(old_value, new_value) then
            changes.modified[key] = {
                old = old_value,
                new = new_value
            }
        end
    end
    
    -- Find deleted
    for key, old_value in pairs(self.snapshot) do
        if new_data[key] == nil then
            changes.deleted[key] = old_value
        end
    end
    
    -- Store changes if any
    local has_changes = next(changes.added) or next(changes.modified) or next(changes.deleted)
    
    if has_changes then
        self.sequence = changes.sequence
        table.insert(self.change_log, changes)
        
        -- Notify subscribers
        self:notify_subscribers(changes)
        
        -- Update snapshot
        self:take_snapshot(new_data)
        
        return changes
    end
    
    return nil
end

function CDCPipeline:subscribe(name, callback)
    self.subscribers[name] = callback
    print(string.format("   Subscribed: %s", name))
end

function CDCPipeline:notify_subscribers(changes)
    for name, callback in pairs(self.subscribers) do
        callback(changes)
    end
end

function CDCPipeline:replay_changes(from_sequence, to_sequence)
    to_sequence = to_sequence or self.sequence
    
    print(string.format("   Replaying changes from sequence %d to %d", 
        from_sequence, to_sequence))
    
    local replayed = {}
    
    for _, change in ipairs(self.change_log) do
        if change.sequence >= from_sequence and change.sequence <= to_sequence then
            table.insert(replayed, change)
        end
    end
    
    return replayed
end

function CDCPipeline:count_items(data)
    local count = 0
    for _ in pairs(data) do
        count = count + 1
    end
    return count
end

function CDCPipeline:deep_copy(obj)
    if type(obj) ~= "table" then
        return obj
    end
    local copy = {}
    for k, v in pairs(obj) do
        copy[k] = self:deep_copy(v)
    end
    return copy
end

function CDCPipeline:deep_equal(a, b)
    if type(a) ~= type(b) then
        return false
    end
    if type(a) ~= "table" then
        return a == b
    end
    for k, v in pairs(a) do
        if not self:deep_equal(v, b[k]) then
            return false
        end
    end
    for k in pairs(b) do
        if a[k] == nil then
            return false
        end
    end
    return true
end

-- Test CDC pipeline
local cdc = CDCPipeline:new()

-- Subscribe to changes
cdc:subscribe("audit_log", function(changes)
    print("   [Audit] Changes detected:")
    
    local count = 0
    for _ in pairs(changes.added) do count = count + 1 end
    if count > 0 then
        print(string.format("     Added: %d items", count))
    end
    
    count = 0
    for _ in pairs(changes.modified) do count = count + 1 end
    if count > 0 then
        print(string.format("     Modified: %d items", count))
    end
    
    count = 0
    for _ in pairs(changes.deleted) do count = count + 1 end
    if count > 0 then
        print(string.format("     Deleted: %d items", count))
    end
end)

-- Initial data
local data_v1 = {
    user1 = {name = "Alice", status = "active"},
    user2 = {name = "Bob", status = "active"},
    user3 = {name = "Charlie", status = "inactive"}
}

print("\n   Testing CDC:")
cdc:take_snapshot(data_v1)

-- Make changes
local data_v2 = {
    user1 = {name = "Alice", status = "inactive"},  -- Modified
    user2 = {name = "Bob", status = "active"},      -- Unchanged
    -- user3 deleted
    user4 = {name = "David", status = "active"}     -- Added
}

cdc:capture_changes(data_v2)

-- More changes
local data_v3 = {
    user1 = {name = "Alice Johnson", status = "active"},  -- Modified
    user2 = {name = "Bob", status = "active"},
    user4 = {name = "David", status = "active"},
    user5 = {name = "Eve", status = "pending"}            -- Added
}

cdc:capture_changes(data_v3)

-- Replay changes
print("\n   Change history:")
local history = cdc:replay_changes(1, cdc.sequence)
for i, change in ipairs(history) do
    print(string.format("   Sequence %d: %d adds, %d mods, %d dels",
        change.sequence,
        cdc:count_items(change.added),
        cdc:count_items(change.modified),
        cdc:count_items(change.deleted)))
end

print()
print("🎯 Key Takeaways:")
print("   • ETL pipelines structure data processing")
print("   • Stream processing handles real-time data")
print("   • Data quality ensures reliable pipelines")
print("   • CDC captures incremental changes")
print("   • Pipeline patterns enable scalable data processing")