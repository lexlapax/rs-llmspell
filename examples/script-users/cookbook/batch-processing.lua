-- Cookbook: Batch Processing - Efficient Bulk Operations
-- Purpose: Implement batch processing patterns for handling large datasets efficiently
-- Prerequisites: None
-- Expected Output: Demonstration of batch processing strategies
-- Version: 0.7.0
-- Tags: cookbook, batch-processing, performance, bulk-operations, production

print("=== Batch Processing Patterns ===\n")

-- ============================================================
-- Pattern 1: Simple Batch Processor
-- ============================================================

print("1. Simple Batch Processing")
print("-" .. string.rep("-", 40))

local BatchProcessor = {}
BatchProcessor.__index = BatchProcessor

function BatchProcessor:new(options)
    options = options or {}
    return setmetatable({
        batch_size = options.batch_size or 10,
        processor = options.processor,
        on_batch_complete = options.on_batch_complete,
        on_error = options.on_error,
        stats = {
            total_processed = 0,
            total_failed = 0,
            batches_processed = 0,
            processing_time = 0
        }
    }, self)
end

function BatchProcessor:process(items)
    local results = {
        successful = {},
        failed = {},
        batches = {}
    }
    
    local start_time = os.time()
    
    -- Process in batches
    for i = 1, #items, self.batch_size do
        local batch_end = math.min(i + self.batch_size - 1, #items)
        local batch = {}
        
        -- Extract batch
        for j = i, batch_end do
            table.insert(batch, items[j])
        end
        
        print(string.format("   Processing batch %d (%d items)...", 
            self.stats.batches_processed + 1, #batch))
        
        -- Process batch
        local batch_results = self:process_batch(batch)
        table.insert(results.batches, batch_results)
        
        -- Update stats
        self.stats.batches_processed = self.stats.batches_processed + 1
        self.stats.total_processed = self.stats.total_processed + batch_results.success_count
        self.stats.total_failed = self.stats.total_failed + batch_results.failure_count
        
        -- Callback
        if self.on_batch_complete then
            self.on_batch_complete(batch_results, self.stats.batches_processed)
        end
        
        -- Aggregate results
        for _, item in ipairs(batch_results.successful) do
            table.insert(results.successful, item)
        end
        for _, item in ipairs(batch_results.failed) do
            table.insert(results.failed, item)
        end
    end
    
    self.stats.processing_time = os.time() - start_time
    
    return results
end

function BatchProcessor:process_batch(batch)
    local results = {
        successful = {},
        failed = {},
        success_count = 0,
        failure_count = 0
    }
    
    for _, item in ipairs(batch) do
        if self.processor then
            local success, result = pcall(self.processor, item)
            
            if success then
                table.insert(results.successful, {
                    input = item,
                    output = result
                })
                results.success_count = results.success_count + 1
            else
                table.insert(results.failed, {
                    input = item,
                    error = result
                })
                results.failure_count = results.failure_count + 1
                
                if self.on_error then
                    self.on_error(item, result)
                end
            end
        end
    end
    
    return results
end

function BatchProcessor:get_stats()
    return {
        total_processed = self.stats.total_processed,
        total_failed = self.stats.total_failed,
        batches_processed = self.stats.batches_processed,
        success_rate = self.stats.total_processed > 0 and 
            (self.stats.total_processed / (self.stats.total_processed + self.stats.total_failed) * 100) or 0,
        processing_time = self.stats.processing_time,
        items_per_second = self.stats.processing_time > 0 and 
            (self.stats.total_processed / self.stats.processing_time) or 0
    }
end

-- Test batch processor
local processor = BatchProcessor:new({
    batch_size = 3,
    processor = function(item)
        -- Simulate processing
        if item % 4 == 0 then
            error("Item " .. item .. " failed")
        end
        return item * 2
    end,
    on_batch_complete = function(results, batch_num)
        print(string.format("     Batch %d complete: %d success, %d failed", 
            batch_num, results.success_count, results.failure_count))
    end
})

local test_items = {}
for i = 1, 10 do
    table.insert(test_items, i)
end

local results = processor:process(test_items)
local stats = processor:get_stats()

print(string.format("\n   Final stats: %d processed, %.1f%% success rate", 
    stats.total_processed, stats.success_rate))

print()

-- ============================================================
-- Pattern 2: Parallel Batch Processing
-- ============================================================

print("2. Parallel Batch Processing")
print("-" .. string.rep("-", 40))

local ParallelBatchProcessor = {}
ParallelBatchProcessor.__index = ParallelBatchProcessor

function ParallelBatchProcessor:new(options)
    options = options or {}
    return setmetatable({
        batch_size = options.batch_size or 10,
        max_parallel = options.max_parallel or 3,
        processor = options.processor,
        active_batches = {},
        queue = {},
        results = {
            successful = {},
            failed = {}
        }
    }, self)
end

function ParallelBatchProcessor:create_batches(items)
    local batches = {}
    
    for i = 1, #items, self.batch_size do
        local batch_end = math.min(i + self.batch_size - 1, #items)
        local batch = {}
        
        for j = i, batch_end do
            table.insert(batch, items[j])
        end
        
        table.insert(batches, {
            id = #batches + 1,
            items = batch,
            status = "pending"
        })
    end
    
    return batches
end

function ParallelBatchProcessor:process(items)
    local batches = self:create_batches(items)
    self.queue = batches
    
    print(string.format("   Created %d batches (max %d parallel)", 
        #batches, self.max_parallel))
    
    -- Simulate parallel processing
    while #self.queue > 0 or next(self.active_batches) do
        -- Start new batches if under limit
        while #self.queue > 0 and self:count_active() < self.max_parallel do
            local batch = table.remove(self.queue, 1)
            self:start_batch(batch)
        end
        
        -- Process active batches
        self:process_active()
        
        -- Simulate time passing
        -- os.execute("sleep 0.1")
    end
    
    return self.results
end

function ParallelBatchProcessor:count_active()
    local count = 0
    for _ in pairs(self.active_batches) do
        count = count + 1
    end
    return count
end

function ParallelBatchProcessor:start_batch(batch)
    batch.status = "processing"
    batch.start_time = os.time()
    self.active_batches[batch.id] = batch
    
    print(string.format("   Starting batch %d (%d items)", 
        batch.id, #batch.items))
end

function ParallelBatchProcessor:process_active()
    for id, batch in pairs(self.active_batches) do
        -- Process batch items
        for _, item in ipairs(batch.items) do
            if self.processor then
                local success, result = pcall(self.processor, item)
                
                if success then
                    table.insert(self.results.successful, {
                        batch_id = id,
                        item = item,
                        result = result
                    })
                else
                    table.insert(self.results.failed, {
                        batch_id = id,
                        item = item,
                        error = result
                    })
                end
            end
        end
        
        -- Mark as complete
        batch.status = "complete"
        batch.end_time = os.time()
        
        print(string.format("   Completed batch %d", id))
        
        -- Remove from active
        self.active_batches[id] = nil
    end
end

-- Test parallel processor
local parallel = ParallelBatchProcessor:new({
    batch_size = 4,
    max_parallel = 2,
    processor = function(item)
        return "Processed: " .. item
    end
})

local items_to_process = {"A", "B", "C", "D", "E", "F", "G", "H", "I", "J"}
local parallel_results = parallel:process(items_to_process)

print(string.format("   Processed %d items successfully", 
    #parallel_results.successful))

print()

-- ============================================================
-- Pattern 3: Streaming Batch Processor
-- ============================================================

print("3. Streaming Batch Processor")
print("-" .. string.rep("-", 40))

local StreamingBatchProcessor = {}
StreamingBatchProcessor.__index = StreamingBatchProcessor

function StreamingBatchProcessor:new(options)
    options = options or {}
    return setmetatable({
        batch_size = options.batch_size or 10,
        batch_timeout = options.batch_timeout or 5, -- seconds
        processor = options.processor,
        buffer = {},
        last_flush = os.time(),
        auto_flush = options.auto_flush ~= false,
        stats = {
            batches_flushed = 0,
            items_processed = 0
        }
    }, self)
end

function StreamingBatchProcessor:add(item)
    table.insert(self.buffer, {
        item = item,
        timestamp = os.time()
    })
    
    -- Check if we should flush
    if self.auto_flush then
        if #self.buffer >= self.batch_size then
            print("   Buffer full, flushing batch...")
            self:flush()
        elseif self:should_flush_by_time() then
            print("   Timeout reached, flushing batch...")
            self:flush()
        end
    end
end

function StreamingBatchProcessor:should_flush_by_time()
    return (os.time() - self.last_flush) >= self.batch_timeout and #self.buffer > 0
end

function StreamingBatchProcessor:flush()
    if #self.buffer == 0 then
        return nil
    end
    
    local batch = self.buffer
    self.buffer = {}
    self.last_flush = os.time()
    
    -- Process the batch
    local results = {}
    if self.processor then
        for _, entry in ipairs(batch) do
            local success, result = pcall(self.processor, entry.item)
            table.insert(results, {
                item = entry.item,
                success = success,
                result = result,
                latency = os.time() - entry.timestamp
            })
        end
    end
    
    self.stats.batches_flushed = self.stats.batches_flushed + 1
    self.stats.items_processed = self.stats.items_processed + #batch
    
    print(string.format("   Flushed batch %d with %d items", 
        self.stats.batches_flushed, #batch))
    
    return results
end

function StreamingBatchProcessor:get_buffer_status()
    return {
        size = #self.buffer,
        oldest_item_age = #self.buffer > 0 and 
            (os.time() - self.buffer[1].timestamp) or 0,
        time_until_flush = self.batch_timeout - (os.time() - self.last_flush)
    }
end

-- Test streaming processor
local streamer = StreamingBatchProcessor:new({
    batch_size = 5,
    batch_timeout = 3,
    processor = function(item)
        return string.upper(item)
    end
})

print("   Adding items to streaming processor...")
local stream_items = {"a", "b", "c", "d", "e", "f", "g"}

for i, item in ipairs(stream_items) do
    streamer:add(item)
    local status = streamer:get_buffer_status()
    print(string.format("     Added '%s', buffer: %d/%d", 
        item, status.size, streamer.batch_size))
end

-- Final flush
if #streamer.buffer > 0 then
    print("   Final flush...")
    streamer:flush()
end

print(string.format("   Total: %d items in %d batches", 
    streamer.stats.items_processed, streamer.stats.batches_flushed))

print()

-- ============================================================
-- Pattern 4: Adaptive Batch Sizing
-- ============================================================

print("4. Adaptive Batch Sizing")
print("-" .. string.rep("-", 40))

local AdaptiveBatchProcessor = {}
AdaptiveBatchProcessor.__index = AdaptiveBatchProcessor

function AdaptiveBatchProcessor:new(options)
    options = options or {}
    return setmetatable({
        min_batch_size = options.min_batch_size or 5,
        max_batch_size = options.max_batch_size or 100,
        current_batch_size = options.initial_batch_size or 10,
        target_time = options.target_time or 1, -- Target seconds per batch
        processor = options.processor,
        performance_history = {}
    }, self)
end

function AdaptiveBatchProcessor:adjust_batch_size(actual_time)
    -- Calculate adjustment factor
    local factor = self.target_time / actual_time
    
    -- Apply adjustment with damping
    local new_size = math.floor(self.current_batch_size * ((factor + 1) / 2))
    
    -- Clamp to limits
    new_size = math.max(self.min_batch_size, 
                math.min(self.max_batch_size, new_size))
    
    if new_size ~= self.current_batch_size then
        print(string.format("   Adjusting batch size: %d -> %d (time was %.2fs, target %.2fs)", 
            self.current_batch_size, new_size, actual_time, self.target_time))
        self.current_batch_size = new_size
    end
end

function AdaptiveBatchProcessor:process(items)
    local results = {
        batches = {},
        total_time = 0
    }
    
    local i = 1
    local batch_num = 0
    
    while i <= #items do
        batch_num = batch_num + 1
        local batch_end = math.min(i + self.current_batch_size - 1, #items)
        local batch = {}
        
        for j = i, batch_end do
            table.insert(batch, items[j])
        end
        
        -- Process batch with timing
        local start_time = os.clock()
        
        local batch_results = {}
        if self.processor then
            for _, item in ipairs(batch) do
                local success, result = pcall(self.processor, item)
                table.insert(batch_results, {
                    item = item,
                    success = success,
                    result = result
                })
            end
        end
        
        local batch_time = os.clock() - start_time
        
        print(string.format("   Batch %d: %d items in %.3fs (%.1f items/sec)", 
            batch_num, #batch, batch_time, #batch / batch_time))
        
        -- Record performance
        table.insert(self.performance_history, {
            size = #batch,
            time = batch_time,
            items_per_second = #batch / batch_time
        })
        
        -- Adjust batch size based on performance
        self:adjust_batch_size(batch_time)
        
        table.insert(results.batches, {
            number = batch_num,
            size = #batch,
            time = batch_time,
            results = batch_results
        })
        
        results.total_time = results.total_time + batch_time
        i = batch_end + 1
    end
    
    return results
end

-- Test adaptive batch processor
local adaptive = AdaptiveBatchProcessor:new({
    min_batch_size = 2,
    max_batch_size = 20,
    initial_batch_size = 5,
    target_time = 0.1,
    processor = function(item)
        -- Simulate variable processing time
        local complexity = item % 10
        for i = 1, complexity * 1000 do
            -- Busy work
            local x = math.sqrt(i)
        end
        return item * 2
    end
})

local adaptive_items = {}
for i = 1, 30 do
    table.insert(adaptive_items, i)
end

print("   Processing with adaptive batch sizing:")
local adaptive_results = adaptive:process(adaptive_items)

print(string.format("\n   Processed %d batches in %.3fs total", 
    #adaptive_results.batches, adaptive_results.total_time))

print()

-- ============================================================
-- Pattern 5: Batch Aggregation and Deduplication
-- ============================================================

print("5. Batch Aggregation with Deduplication")
print("-" .. string.rep("-", 40))

local AggregatingBatchProcessor = {}
AggregatingBatchProcessor.__index = AggregatingBatchProcessor

function AggregatingBatchProcessor:new(options)
    options = options or {}
    return setmetatable({
        batch_size = options.batch_size or 10,
        dedup_key = options.dedup_key or function(item) return item end,
        aggregator = options.aggregator,
        processor = options.processor,
        pending = {},
        seen_keys = {}
    }, self)
end

function AggregatingBatchProcessor:add(item)
    local key = self.dedup_key(item)
    
    -- Deduplication
    if self.seen_keys[key] then
        print("   Duplicate detected: " .. tostring(key))
        return false
    end
    
    self.seen_keys[key] = true
    table.insert(self.pending, item)
    
    -- Process if batch is full
    if #self.pending >= self.batch_size then
        return self:process_batch()
    end
    
    return true
end

function AggregatingBatchProcessor:process_batch()
    if #self.pending == 0 then
        return nil
    end
    
    local batch = self.pending
    self.pending = {}
    
    -- Aggregate if aggregator provided
    local to_process = batch
    if self.aggregator then
        to_process = self.aggregator(batch)
        print(string.format("   Aggregated %d items into %d", 
            #batch, type(to_process) == "table" and #to_process or 1))
    end
    
    -- Process
    local results = nil
    if self.processor then
        results = self.processor(to_process)
    end
    
    -- Clear seen keys for this batch
    for _, item in ipairs(batch) do
        local key = self.dedup_key(item)
        self.seen_keys[key] = nil
    end
    
    return results
end

-- Test aggregating processor
local aggregator = AggregatingBatchProcessor:new({
    batch_size = 5,
    dedup_key = function(item)
        return item.id
    end,
    aggregator = function(items)
        -- Aggregate by type
        local aggregated = {}
        for _, item in ipairs(items) do
            local type = item.type
            if not aggregated[type] then
                aggregated[type] = {
                    type = type,
                    count = 0,
                    total_value = 0
                }
            end
            aggregated[type].count = aggregated[type].count + 1
            aggregated[type].total_value = aggregated[type].total_value + item.value
        end
        
        -- Convert to array
        local result = {}
        for _, agg in pairs(aggregated) do
            table.insert(result, agg)
        end
        return result
    end,
    processor = function(aggregated)
        print("   Processing aggregated batch:")
        for _, agg in ipairs(aggregated) do
            print(string.format("     Type %s: %d items, total value: %d", 
                agg.type, agg.count, agg.total_value))
        end
        return aggregated
    end
})

-- Add items with some duplicates
local test_agg_items = {
    {id = 1, type = "A", value = 10},
    {id = 2, type = "B", value = 20},
    {id = 1, type = "A", value = 10}, -- Duplicate
    {id = 3, type = "A", value = 15},
    {id = 4, type = "B", value = 25},
    {id = 5, type = "C", value = 30}
}

print("   Adding items with deduplication:")
for _, item in ipairs(test_agg_items) do
    local added = aggregator:add(item)
    if added then
        print(string.format("     Added: id=%d, type=%s", item.id, item.type))
    end
end

-- Process remaining
aggregator:process_batch()

print()
print("🎯 Key Takeaways:")
print("   • Batch processing improves throughput")
print("   • Parallel batching increases efficiency")
print("   • Streaming batches handle continuous data")
print("   • Adaptive sizing optimizes performance")
print("   • Deduplication prevents duplicate processing")
print("   • Aggregation reduces processing overhead")