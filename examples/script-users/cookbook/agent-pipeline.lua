-- Cookbook: Agent Pipeline - Sequential Agent Processing Patterns
-- Purpose: Implement patterns for sequential agent processing and pipeline coordination
-- Prerequisites: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable, cookbook config
-- Expected Output: Demonstration of agent pipeline patterns
-- Version: 0.7.0
-- Tags: cookbook, agent-pipeline, sequential-processing, workflow, coordination

print("=== Agent Pipeline Patterns ===\n")

-- ============================================================
-- Pattern 1: Linear Processing Pipeline
-- ============================================================

print("1. Linear Processing Pipeline")
print("-" .. string.rep("-", 40))

local LinearPipeline = {}
LinearPipeline.__index = LinearPipeline

function LinearPipeline:new(name)
    return setmetatable({
        name = name,
        stages = {},
        stage_order = {},
        pipeline_state = {},
        execution_history = {}
    }, self)
end

function LinearPipeline:add_stage(stage_name, processor_config)
    local stage = {
        name = stage_name,
        processor_type = processor_config.processor_type or "function",
        processor = processor_config.processor,
        agent_name = processor_config.agent_name,
        validation = processor_config.validation,
        timeout = processor_config.timeout or 30,
        retry_count = processor_config.retry_count or 0,
        required_inputs = processor_config.required_inputs or {},
        outputs = processor_config.outputs or {}
    }
    
    self.stages[stage_name] = stage
    table.insert(self.stage_order, stage_name)
    
    print(string.format("   ➕ Added stage: %s (%s)", 
        stage_name, stage.processor_type))
end

function LinearPipeline:execute_pipeline(initial_data)
    local pipeline_id = "pipeline_" .. os.time() .. "_" .. math.random(1000)
    
    self.pipeline_state[pipeline_id] = {
        id = pipeline_id,
        status = "running",
        current_stage = 1,
        data = initial_data,
        stage_results = {},
        errors = {},
        start_time = os.clock()
    }
    
    local state = self.pipeline_state[pipeline_id]
    
    print(string.format("   🚀 Starting pipeline: %s", self.name))
    print(string.format("     Pipeline ID: %s", pipeline_id))
    
    for i, stage_name in ipairs(self.stage_order) do
        state.current_stage = i
        
        print(string.format("\n   Stage %d: %s", i, stage_name))
        
        local success, result = self:execute_stage(pipeline_id, stage_name, state.data)
        
        if success then
            state.data = result.output_data or state.data
            state.stage_results[stage_name] = result
            print(string.format("     ✅ Stage completed (%.1fms)", result.execution_time))
        else
            state.status = "failed"
            state.errors[stage_name] = result.error
            print(string.format("     ❌ Stage failed: %s", result.error))
            break
        end
    end
    
    if state.status ~= "failed" then
        state.status = "completed"
    end
    
    state.end_time = os.clock()
    state.total_time = (state.end_time - state.start_time) * 1000
    
    print(string.format("\n   Pipeline %s: %s (%.1fms total)", 
        pipeline_id, state.status, state.total_time))
    
    return state
end

function LinearPipeline:execute_stage(pipeline_id, stage_name, input_data)
    local stage = self.stages[stage_name]
    local start_time = os.clock()
    
    -- Validate inputs if specified
    if stage.validation then
        local validation_result = stage.validation(input_data)
        if not validation_result.valid then
            return false, {
                error = "Validation failed: " .. (validation_result.error or "Unknown error"),
                execution_time = (os.clock() - start_time) * 1000
            }
        end
    end
    
    local success, result
    
    if stage.processor_type == "function" then
        success, result = pcall(stage.processor, input_data)
    elseif stage.processor_type == "agent" then
        success, result = self:execute_agent_stage(stage, input_data)
    else
        return false, {
            error = "Unknown processor type: " .. stage.processor_type,
            execution_time = (os.clock() - start_time) * 1000
        }
    end
    
    local execution_time = (os.clock() - start_time) * 1000
    
    if success then
        return true, {
            output_data = result,
            execution_time = execution_time,
            stage = stage_name
        }
    else
        return false, {
            error = tostring(result),
            execution_time = execution_time,
            stage = stage_name
        }
    end
end

function LinearPipeline:execute_agent_stage(stage, input_data)
    -- Simulate agent execution (in real implementation, call actual agent)
    print(string.format("       Executing agent: %s", stage.agent_name))
    
    -- Simulate processing time
    local processing_time = math.random(20, 100) / 1000 -- 20-100ms
    local end_time = os.clock() + processing_time
    while os.clock() < end_time do end
    
    -- Simulate agent processing result
    local processed_data = {
        original = input_data,
        processed_by = stage.agent_name,
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        stage_metadata = {
            stage_name = stage.name,
            processing_time = processing_time * 1000
        }
    }
    
    -- Add stage-specific transformations
    if stage.name == "data_extraction" then
        processed_data.extracted_fields = {"field1", "field2", "field3"}
    elseif stage.name == "analysis" then
        processed_data.analysis_results = {
            sentiment = "positive",
            confidence = 0.85,
            key_topics = {"topic1", "topic2"}
        }
    elseif stage.name == "formatting" then
        processed_data.formatted_output = "Formatted: " .. tostring(input_data.text or "data")
    end
    
    return true, processed_data
end

function LinearPipeline:get_pipeline_status(pipeline_id)
    return self.pipeline_state[pipeline_id]
end

-- Test linear pipeline
local text_pipeline = LinearPipeline:new("Text Processing Pipeline")

-- Add stages
text_pipeline:add_stage("data_extraction", {
    processor_type = "agent",
    agent_name = "extraction_agent",
    validation = function(data)
        if not data.text then
            return {valid = false, error = "Missing text field"}
        end
        return {valid = true}
    end
})

text_pipeline:add_stage("analysis", {
    processor_type = "agent", 
    agent_name = "analysis_agent",
    timeout = 45
})

text_pipeline:add_stage("formatting", {
    processor_type = "function",
    processor = function(data)
        -- Simple formatting function
        return {
            formatted_text = "FORMATTED: " .. (data.original.text or ""),
            metadata = data,
            final_stage = true
        }
    end
})

-- Execute pipeline
print("   Testing linear processing pipeline:")

local test_data = {
    text = "Sample text for processing through the agent pipeline",
    source = "user_input",
    priority = "normal"
}

local result = text_pipeline:execute_pipeline(test_data)

print(string.format("   Final result available: %s", 
    result.data.final_stage and "Yes" or "No"))

print()

-- ============================================================
-- Pattern 2: Conditional Branch Pipeline
-- ============================================================

print("2. Conditional Branch Pipeline")
print("-" .. string.rep("-", 40))

local BranchPipeline = {}
BranchPipeline.__index = BranchPipeline

function BranchPipeline:new(name)
    return setmetatable({
        name = name,
        stages = {},
        branches = {},
        execution_paths = {}
    }, self)
end

function BranchPipeline:add_stage(stage_name, config)
    self.stages[stage_name] = {
        name = stage_name,
        processor = config.processor,
        agent_name = config.agent_name,
        condition_check = config.condition_check,
        branches = config.branches or {}
    }
    
    print(string.format("   🔀 Added branch stage: %s", stage_name))
end

function BranchPipeline:execute_conditional_pipeline(initial_data, start_stage)
    local execution_id = "exec_" .. os.time()
    local execution_path = {}
    
    local current_data = initial_data
    local current_stage = start_stage or "entry"
    
    print(string.format("   🎯 Starting conditional pipeline from: %s", current_stage))
    
    while current_stage do
        table.insert(execution_path, current_stage)
        
        local stage = self.stages[current_stage]
        if not stage then
            print(string.format("   ❌ Stage not found: %s", current_stage))
            break
        end
        
        print(string.format("   Processing stage: %s", current_stage))
        
        -- Execute stage processor
        local success, result = pcall(stage.processor, current_data)
        
        if not success then
            print(string.format("   ❌ Stage failed: %s", result))
            break
        end
        
        current_data = result
        
        -- Determine next stage based on conditions
        local next_stage = nil
        
        if stage.condition_check then
            local condition_result = stage.condition_check(current_data)
            
            if condition_result.branch then
                next_stage = condition_result.next_stage
                print(string.format("     → Branch: %s → %s", 
                    condition_result.branch, next_stage or "END"))
            end
        end
        
        current_stage = next_stage
    end
    
    print(string.format("   ✅ Pipeline completed. Path: %s", 
        table.concat(execution_path, " → ")))
    
    return {
        execution_id = execution_id,
        path = execution_path,
        final_data = current_data
    }
end

-- Test conditional pipeline
local decision_pipeline = BranchPipeline:new("Decision Processing Pipeline")

decision_pipeline:add_stage("entry", {
    processor = function(data)
        print("     Analyzing input data...")
        data.analysis = {
            complexity = data.complexity or "medium",
            priority = data.priority or "normal",
            data_type = type(data.content) == "table" and "structured" or "unstructured"
        }
        return data
    end,
    condition_check = function(data)
        if data.analysis.priority == "urgent" then
            return {branch = "urgent_path", next_stage = "urgent_processing"}
        elseif data.analysis.complexity == "high" then
            return {branch = "complex_path", next_stage = "detailed_analysis"}
        else
            return {branch = "standard_path", next_stage = "standard_processing"}
        end
    end
})

decision_pipeline:add_stage("urgent_processing", {
    processor = function(data)
        print("     🚨 Urgent processing mode...")
        data.processed = true
        data.processing_mode = "urgent"
        return data
    end,
    condition_check = function(data)
        return {branch = "finalize", next_stage = "output_stage"}
    end
})

decision_pipeline:add_stage("detailed_analysis", {
    processor = function(data)
        print("     🔍 Detailed analysis mode...")
        data.detailed_analysis = {
            algorithms_applied = {"advanced_nlp", "sentiment_analysis"},
            processing_time = "extended"
        }
        return data
    end,
    condition_check = function(data)
        return {branch = "finalize", next_stage = "output_stage"}
    end
})

decision_pipeline:add_stage("standard_processing", {
    processor = function(data)
        print("     ⚙️  Standard processing mode...")
        data.processed = true
        data.processing_mode = "standard"
        return data
    end,
    condition_check = function(data)
        return {branch = "finalize", next_stage = "output_stage"}
    end
})

decision_pipeline:add_stage("output_stage", {
    processor = function(data)
        print("     📤 Generating output...")
        data.output_generated = true
        data.completion_time = os.date("%Y-%m-%d %H:%M:%S")
        return data
    end
})

-- Test different execution paths
print("   Testing conditional branch pipeline:")

local test_scenarios = {
    {content = "Simple task", priority = "normal", complexity = "low"},
    {content = "Complex analysis", priority = "normal", complexity = "high"},
    {content = "Emergency request", priority = "urgent", complexity = "medium"}
}

for i, scenario in ipairs(test_scenarios) do
    print(string.format("\n   Scenario %d: %s priority, %s complexity", 
        i, scenario.priority, scenario.complexity))
    
    local result = decision_pipeline:execute_conditional_pipeline(scenario, "entry")
end

print()

-- ============================================================
-- Pattern 3: Parallel Processing with Synchronization
-- ============================================================

print("3. Parallel Processing with Synchronization")
print("-" .. string.rep("-", 40))

local ParallelPipeline = {}
ParallelPipeline.__index = ParallelPipeline

function ParallelPipeline:new(name)
    return setmetatable({
        name = name,
        parallel_groups = {},
        sync_points = {},
        execution_state = {}
    }, self)
end

function ParallelPipeline:add_parallel_group(group_name, processors)
    self.parallel_groups[group_name] = {
        name = group_name,
        processors = processors,
        sync_strategy = "wait_all" -- wait_all, wait_any, wait_majority
    }
    
    print(string.format("   🔄 Added parallel group: %s (%d processors)", 
        group_name, #processors))
end

function ParallelPipeline:execute_parallel_pipeline(input_data)
    local execution_id = "parallel_" .. os.time()
    
    print(string.format("   🚀 Starting parallel pipeline: %s", self.name))
    
    local results = {}
    
    for group_name, group in pairs(self.parallel_groups) do
        print(string.format("\n   Executing parallel group: %s", group_name))
        
        local group_results = {}
        local group_start = os.clock()
        
        -- Simulate parallel execution (in real implementation, use actual parallel execution)
        for i, processor in ipairs(group.processors) do
            print(string.format("     Processor %d: %s", i, processor.name))
            
            local proc_start = os.clock()
            local success, result = pcall(processor.func, input_data)
            local proc_time = (os.clock() - proc_start) * 1000
            
            group_results[processor.name] = {
                success = success,
                result = success and result or nil,
                error = not success and result or nil,
                execution_time = proc_time
            }
            
            print(string.format("       %s (%.1fms)", 
                success and "✅ Completed" or "❌ Failed", proc_time))
        end
        
        local group_time = (os.clock() - group_start) * 1000
        
        -- Synchronization point
        local sync_result = self:synchronize_group(group, group_results)
        
        results[group_name] = {
            group_results = group_results,
            sync_result = sync_result,
            group_execution_time = group_time
        }
        
        print(string.format("   Group %s: %s", group_name, 
            sync_result.success and "✅ Synchronized" or "❌ Sync failed"))
    end
    
    return {
        execution_id = execution_id,
        results = results,
        overall_success = self:check_overall_success(results)
    }
end

function ParallelPipeline:synchronize_group(group, group_results)
    local successful_count = 0
    local total_count = 0
    local aggregated_results = {}
    
    for processor_name, result in pairs(group_results) do
        total_count = total_count + 1
        if result.success then
            successful_count = successful_count + 1
            if result.result then
                table.insert(aggregated_results, result.result)
            end
        end
    end
    
    local success_rate = successful_count / total_count
    
    if group.sync_strategy == "wait_all" then
        return {
            success = successful_count == total_count,
            aggregated_results = aggregated_results,
            success_rate = success_rate
        }
    elseif group.sync_strategy == "wait_any" then
        return {
            success = successful_count > 0,
            aggregated_results = aggregated_results,
            success_rate = success_rate
        }
    elseif group.sync_strategy == "wait_majority" then
        return {
            success = success_rate > 0.5,
            aggregated_results = aggregated_results,
            success_rate = success_rate
        }
    end
    
    return {success = false, aggregated_results = {}, success_rate = 0}
end

function ParallelPipeline:check_overall_success(results)
    for group_name, group_result in pairs(results) do
        if not group_result.sync_result.success then
            return false
        end
    end
    return true
end

-- Test parallel pipeline
local data_pipeline = ParallelPipeline:new("Data Processing Pipeline")

-- Add parallel processing groups
data_pipeline:add_parallel_group("data_validation", {
    {
        name = "schema_validator",
        func = function(data)
            -- Simulate schema validation
            local delay = math.random(10, 30) / 1000
            local end_time = os.clock() + delay
            while os.clock() < end_time do end
            
            return {
                valid = true,
                schema_version = "v1.0",
                validation_time = delay * 1000
            }
        end
    },
    {
        name = "data_quality_checker",
        func = function(data)
            -- Simulate data quality check
            local delay = math.random(15, 25) / 1000
            local end_time = os.clock() + delay
            while os.clock() < end_time do end
            
            return {
                quality_score = 0.92,
                issues_found = 2,
                quality_metrics = {"completeness", "accuracy"}
            }
        end
    },
    {
        name = "security_scanner",
        func = function(data)
            -- Simulate security scan
            local delay = math.random(20, 40) / 1000
            local end_time = os.clock() + delay
            while os.clock() < end_time do end
            
            return {
                threats_detected = 0,
                security_level = "safe",
                scan_duration = delay * 1000
            }
        end
    }
})

data_pipeline:add_parallel_group("data_enrichment", {
    {
        name = "geo_enricher",
        func = function(data)
            local delay = math.random(25, 35) / 1000
            local end_time = os.clock() + delay
            while os.clock() < end_time do end
            
            return {
                geo_data = {
                    country = "US",
                    region = "California",
                    timezone = "PST"
                }
            }
        end
    },
    {
        name = "metadata_extractor",
        func = function(data)
            local delay = math.random(10, 20) / 1000
            local end_time = os.clock() + delay
            while os.clock() < end_time do end
            
            return {
                metadata = {
                    created_at = os.date("%Y-%m-%d %H:%M:%S"),
                    file_size = 1024,
                    format = "json"
                }
            }
        end
    }
})

-- Execute parallel pipeline
print("   Testing parallel processing pipeline:")

local test_data = {
    dataset_id = "test_001",
    records_count = 1000,
    source = "api_import"
}

local parallel_result = data_pipeline:execute_parallel_pipeline(test_data)

print(string.format("\n   Parallel pipeline result: %s", 
    parallel_result.overall_success and "✅ Success" or "❌ Failed"))

for group_name, group_result in pairs(parallel_result.results) do
    print(string.format("   Group %s: %.1f%% success rate (%.1fms)", 
        group_name, 
        group_result.sync_result.success_rate * 100,
        group_result.group_execution_time))
end

print()
print("🎯 Key Takeaways:")
print("   • Design pipelines with clear stage responsibilities")
print("   • Implement proper error handling and recovery")
print("   • Use conditional branching for complex workflows")
print("   • Leverage parallel processing for independent tasks")
print("   • Implement synchronization points for coordination")
print("   • Monitor pipeline performance and bottlenecks")