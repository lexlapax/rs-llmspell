-- Application: Production Data Pipeline
-- Purpose: Complete data processing system with monitoring, recovery, and scaling
-- Prerequisites: Config file, API keys for monitoring services (optional)
-- Expected Output: Fully operational data pipeline with real-time monitoring
-- Version: 0.7.0
-- Tags: application, data-pipeline, production, monitoring, etl

-- ABOUTME: Production-ready data pipeline application with monitoring and recovery
-- ABOUTME: Builds upon cookbook patterns with enterprise features

print("=== Production Data Pipeline Application ===\n")

-- Load configuration
local config = {
    pipeline_name = "ProductionDataPipeline",
    batch_size = 100,
    retry_attempts = 3,
    monitoring = {
        enabled = true,
        metrics_interval = 60,  -- seconds
        alert_thresholds = {
            error_rate = 0.05,  -- 5% error rate
            latency_ms = 5000,  -- 5 second latency
            throughput_min = 10  -- 10 records/minute minimum
        }
    },
    recovery = {
        dead_letter_queue = true,
        checkpoint_interval = 100,  -- records
        rollback_enabled = true
    },
    scaling = {
        auto_scale = true,
        min_workers = 1,
        max_workers = 10,
        scale_up_threshold = 0.8,  -- 80% capacity
        scale_down_threshold = 0.2  -- 20% capacity
    }
}

-- ============================================================
-- Core Pipeline Components
-- ============================================================

local Pipeline = {}
Pipeline.__index = Pipeline

function Pipeline:new(config)
    local self = setmetatable({}, Pipeline)
    self.config = config
    self.name = config.pipeline_name
    self.state = "initialized"
    self.metrics = {
        total_processed = 0,
        total_errors = 0,
        current_throughput = 0,
        average_latency = 0,
        last_checkpoint = 0
    }
    self.workers = {}
    self.dead_letter_queue = {}
    self.monitoring_agent = nil
    
    -- Initialize components
    self:init_monitoring()
    self:init_recovery()
    self:init_scaling()
    
    return self
end

-- ============================================================
-- Monitoring System
-- ============================================================

function Pipeline:init_monitoring()
    if not self.config.monitoring.enabled then
        return
    end
    
    print("Initializing monitoring system...")
    
    -- Create monitoring agent
    local monitor = Agent.builder()
        :name(self.name .. "_monitor")
        :description("Monitors pipeline health and performance")
        :type("system")
        :build()
    
    if monitor then
        self.monitoring_agent = monitor
        print("  ✅ Monitoring agent created")
    else
        print("  ⚠️ Monitoring agent creation failed (no API key?)")
    end
    
    -- Initialize metrics collection
    self.metrics_collector = {
        interval = self.config.monitoring.metrics_interval,
        last_collection = os.time(),
        history = {}
    }
end

function Pipeline:collect_metrics()
    local current_time = os.time()
    local metrics_snapshot = {
        timestamp = current_time,
        processed = self.metrics.total_processed,
        errors = self.metrics.total_errors,
        error_rate = self.metrics.total_errors / math.max(1, self.metrics.total_processed),
        throughput = self.metrics.current_throughput,
        latency = self.metrics.average_latency,
        workers = #self.workers,
        dlq_size = #self.dead_letter_queue
    }
    
    table.insert(self.metrics_collector.history, metrics_snapshot)
    
    -- Check alert thresholds
    self:check_alerts(metrics_snapshot)
    
    -- Keep only last hour of metrics
    local one_hour_ago = current_time - 3600
    local filtered_history = {}
    for _, metric in ipairs(self.metrics_collector.history) do
        if metric.timestamp > one_hour_ago then
            table.insert(filtered_history, metric)
        end
    end
    self.metrics_collector.history = filtered_history
    
    return metrics_snapshot
end

function Pipeline:check_alerts(metrics)
    local thresholds = self.config.monitoring.alert_thresholds
    local alerts = {}
    
    if metrics.error_rate > thresholds.error_rate then
        table.insert(alerts, {
            severity = "HIGH",
            type = "ERROR_RATE",
            message = string.format("Error rate %.2f%% exceeds threshold %.2f%%", 
                metrics.error_rate * 100, thresholds.error_rate * 100)
        })
    end
    
    if metrics.latency > thresholds.latency_ms then
        table.insert(alerts, {
            severity = "MEDIUM",
            type = "HIGH_LATENCY",
            message = string.format("Latency %dms exceeds threshold %dms", 
                metrics.latency, thresholds.latency_ms)
        })
    end
    
    if metrics.throughput < thresholds.throughput_min and self.state == "running" then
        table.insert(alerts, {
            severity = "LOW",
            type = "LOW_THROUGHPUT",
            message = string.format("Throughput %d/min below minimum %d/min", 
                metrics.throughput, thresholds.throughput_min)
        })
    end
    
    -- Send alerts
    for _, alert in ipairs(alerts) do
        self:send_alert(alert)
    end
    
    return alerts
end

function Pipeline:send_alert(alert)
    print(string.format("\n🚨 ALERT [%s]: %s", alert.severity, alert.message))
    
    -- In production, integrate with alerting services
    if self.monitoring_agent then
        -- Could send to PagerDuty, Slack, etc.
        -- For demo, just log it
    end
end

-- ============================================================
-- Recovery System
-- ============================================================

function Pipeline:init_recovery()
    print("Initializing recovery system...")
    
    if self.config.recovery.dead_letter_queue then
        self.dead_letter_queue = {}
        print("  ✅ Dead letter queue enabled")
    end
    
    if self.config.recovery.checkpoint_interval > 0 then
        self.checkpoint = {
            interval = self.config.recovery.checkpoint_interval,
            last_id = 0,
            state = {}
        }
        print("  ✅ Checkpointing enabled (every " .. self.checkpoint.interval .. " records)")
    end
    
    if self.config.recovery.rollback_enabled then
        self.rollback_points = {}
        print("  ✅ Rollback capability enabled")
    end
end

function Pipeline:handle_failure(record, error_msg)
    self.metrics.total_errors = self.metrics.total_errors + 1
    
    local retry_count = record._retry_count or 0
    
    if retry_count < self.config.retry_attempts then
        -- Retry with exponential backoff
        record._retry_count = retry_count + 1
        local backoff = math.pow(2, retry_count) -- 1, 2, 4 seconds
        
        print(string.format("  ⚠️ Retry %d/%d for record after %ds", 
            retry_count + 1, self.config.retry_attempts, backoff))
        
        -- In production, would use proper timer
        os.execute("sleep " .. backoff)
        
        return self:process_record(record)
    else
        -- Send to dead letter queue
        if self.config.recovery.dead_letter_queue then
            table.insert(self.dead_letter_queue, {
                record = record,
                error = error_msg,
                timestamp = os.time(),
                retries = retry_count
            })
            print(string.format("  ❌ Record sent to DLQ after %d retries", retry_count))
        end
        return false
    end
end

function Pipeline:save_checkpoint()
    if not self.checkpoint then
        return
    end
    
    self.checkpoint.last_id = self.metrics.total_processed
    self.checkpoint.state = {
        metrics = self.metrics,
        dlq_size = #self.dead_letter_queue,
        timestamp = os.time()
    }
    
    -- In production, persist to durable storage
    Tool.invoke("file_operations", {
        operation = "write",
        path = "/tmp/pipeline_checkpoint.json",
        input = Tool.invoke("json_processor", {
            operation = "parse",
            input = self.checkpoint.state
        }).result
    })
    
    print(string.format("  💾 Checkpoint saved at record %d", self.checkpoint.last_id))
end

function Pipeline:restore_from_checkpoint()
    local result = Tool.invoke("file_operations", {
        operation = "read",
        path = "/tmp/pipeline_checkpoint.json"
    })
    
    if result.success then
        local checkpoint_data = Tool.invoke("json_processor", {
            operation = "parse",
            input = result.result
        }).result
        
        if checkpoint_data then
            self.metrics = checkpoint_data.metrics
            self.checkpoint.last_id = checkpoint_data.metrics.total_processed
            print(string.format("  ✅ Restored from checkpoint (record %d)", 
                self.checkpoint.last_id))
            return true
        end
    end
    
    return false
end

-- ============================================================
-- Scaling System
-- ============================================================

function Pipeline:init_scaling()
    if not self.config.scaling.auto_scale then
        return
    end
    
    print("Initializing auto-scaling system...")
    
    self.scaling = {
        current_workers = self.config.scaling.min_workers,
        last_scale_time = os.time(),
        scale_cooldown = 60  -- seconds
    }
    
    -- Initialize minimum workers
    for i = 1, self.config.scaling.min_workers do
        self:add_worker()
    end
    
    print(string.format("  ✅ Auto-scaling enabled (%d-%d workers)", 
        self.config.scaling.min_workers, self.config.scaling.max_workers))
end

function Pipeline:add_worker()
    local worker_id = #self.workers + 1
    local worker = {
        id = worker_id,
        status = "idle",
        processed = 0,
        errors = 0
    }
    
    table.insert(self.workers, worker)
    print(string.format("  ➕ Added worker %d (total: %d)", worker_id, #self.workers))
    
    return worker
end

function Pipeline:remove_worker()
    if #self.workers > self.config.scaling.min_workers then
        local removed = table.remove(self.workers)
        print(string.format("  ➖ Removed worker %d (total: %d)", removed.id, #self.workers))
        return true
    end
    return false
end

function Pipeline:check_scaling()
    if not self.config.scaling.auto_scale then
        return
    end
    
    local current_time = os.time()
    if current_time - self.scaling.last_scale_time < self.scaling.scale_cooldown then
        return -- In cooldown period
    end
    
    -- Calculate current load
    local active_workers = 0
    for _, worker in ipairs(self.workers) do
        if worker.status == "processing" then
            active_workers = active_workers + 1
        end
    end
    
    local load = active_workers / #self.workers
    
    -- Scale up if needed
    if load > self.config.scaling.scale_up_threshold then
        if #self.workers < self.config.scaling.max_workers then
            self:add_worker()
            self.scaling.last_scale_time = current_time
        end
    -- Scale down if needed
    elseif load < self.config.scaling.scale_down_threshold then
        if #self.workers > self.config.scaling.min_workers then
            self:remove_worker()
            self.scaling.last_scale_time = current_time
        end
    end
end

-- ============================================================
-- Core Processing
-- ============================================================

function Pipeline:process_batch(batch)
    print(string.format("\n📦 Processing batch of %d records", #batch))
    
    local start_time = os.time() * 1000  -- milliseconds
    local success_count = 0
    local error_count = 0
    
    for _, record in ipairs(batch) do
        -- Assign to worker
        local worker = self:get_available_worker()
        if worker then
            worker.status = "processing"
            
            local success = self:process_record(record)
            
            if success then
                success_count = success_count + 1
                worker.processed = worker.processed + 1
            else
                error_count = error_count + 1
                worker.errors = worker.errors + 1
            end
            
            worker.status = "idle"
        else
            -- No workers available, queue for later
            print("  ⏳ No workers available, queuing record")
        end
        
        -- Update metrics
        self.metrics.total_processed = self.metrics.total_processed + 1
        
        -- Checkpoint if needed
        if self.checkpoint and 
           self.metrics.total_processed % self.checkpoint.interval == 0 then
            self:save_checkpoint()
        end
    end
    
    -- Update performance metrics
    local end_time = os.time() * 1000
    local batch_time = end_time - start_time
    self.metrics.average_latency = batch_time / #batch
    self.metrics.current_throughput = (#batch / batch_time) * 60000  -- per minute
    
    print(string.format("  ✅ Batch complete: %d success, %d errors (%.2fms avg latency)",
        success_count, error_count, self.metrics.average_latency))
    
    -- Check scaling needs
    self:check_scaling()
    
    -- Collect metrics
    if os.time() - self.metrics_collector.last_collection > self.metrics_collector.interval then
        self:collect_metrics()
        self.metrics_collector.last_collection = os.time()
    end
    
    return success_count, error_count
end

function Pipeline:process_record(record)
    -- Simulate processing with potential failure
    local processing_time = math.random(10, 100) / 1000  -- 10-100ms
    os.execute("sleep " .. processing_time)
    
    -- Simulate occasional failures (10% failure rate for demo)
    if math.random() < 0.1 then
        return self:handle_failure(record, "Simulated processing error")
    end
    
    -- Transform record (example)
    record.processed_at = os.time()
    record.pipeline = self.name
    
    return true
end

function Pipeline:get_available_worker()
    for _, worker in ipairs(self.workers) do
        if worker.status == "idle" then
            return worker
        end
    end
    return nil
end

-- ============================================================
-- Pipeline Lifecycle
-- ============================================================

function Pipeline:start()
    print("\n🚀 Starting pipeline: " .. self.name)
    
    self.state = "running"
    
    -- Try to restore from checkpoint
    self:restore_from_checkpoint()
    
    -- Start monitoring
    if self.monitoring_agent then
        print("  ✅ Monitoring active")
    end
    
    print(string.format("  ✅ Pipeline started with %d workers", #self.workers))
end

function Pipeline:stop()
    print("\n🛑 Stopping pipeline: " .. self.name)
    
    self.state = "stopped"
    
    -- Save final checkpoint
    self:save_checkpoint()
    
    -- Process dead letter queue
    if #self.dead_letter_queue > 0 then
        print(string.format("  ⚠️ %d records in dead letter queue", #self.dead_letter_queue))
        -- In production, would persist DLQ for manual review
    end
    
    -- Final metrics
    print("\n📊 Final Metrics:")
    print(string.format("  • Total Processed: %d", self.metrics.total_processed))
    print(string.format("  • Total Errors: %d", self.metrics.total_errors))
    print(string.format("  • Error Rate: %.2f%%", 
        (self.metrics.total_errors / math.max(1, self.metrics.total_processed)) * 100))
    print(string.format("  • Average Latency: %.2fms", self.metrics.average_latency))
    print(string.format("  • DLQ Size: %d", #self.dead_letter_queue))
end

function Pipeline:get_status()
    return {
        state = self.state,
        metrics = self.metrics,
        workers = #self.workers,
        dlq_size = #self.dead_letter_queue,
        last_checkpoint = self.checkpoint and self.checkpoint.last_id or 0
    }
end

-- ============================================================
-- Main Application
-- ============================================================

-- Create pipeline instance
local pipeline = Pipeline:new(config)

-- Start the pipeline
pipeline:start()

-- Simulate data processing
print("\n📥 Simulating data ingestion...")

for batch_num = 1, 3 do
    -- Generate batch of records
    local batch = {}
    for i = 1, config.batch_size do
        table.insert(batch, {
            id = (batch_num - 1) * config.batch_size + i,
            data = "Record_" .. i,
            timestamp = os.time()
        })
    end
    
    -- Process batch
    pipeline:process_batch(batch)
    
    -- Brief pause between batches
    os.execute("sleep 1")
end

-- Check for DLQ processing
if #pipeline.dead_letter_queue > 0 then
    print(string.format("\n♻️ Processing %d records from DLQ...", #pipeline.dead_letter_queue))
    
    -- Attempt to reprocess DLQ records
    local dlq_batch = {}
    for _, item in ipairs(pipeline.dead_letter_queue) do
        table.insert(dlq_batch, item.record)
    end
    
    -- Clear DLQ before reprocessing
    pipeline.dead_letter_queue = {}
    
    -- Reprocess with increased retry limit
    local original_retries = config.retry_attempts
    config.retry_attempts = 1  -- One more try
    pipeline:process_batch(dlq_batch)
    config.retry_attempts = original_retries
end

-- Stop the pipeline
pipeline:stop()

-- Display final status
print("\n=== Pipeline Status ===")
local status = pipeline:get_status()
for key, value in pairs(status) do
    if type(value) == "table" then
        print(string.format("%s:", key))
        for k, v in pairs(value) do
            print(string.format("  • %s: %s", k, tostring(v)))
        end
    else
        print(string.format("%s: %s", key, tostring(value)))
    end
end

print("\n=== Production Data Pipeline Complete ===")