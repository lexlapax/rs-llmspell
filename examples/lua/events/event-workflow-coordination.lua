-- ABOUTME: Events for workflow coordination and distributed process orchestration
-- ABOUTME: Demonstrates workflow state management, step coordination, and distributed process control

print("=== Event Workflow Coordination Example ===")
print("Demonstrates: Workflow orchestration, step coordination, distributed process control, and state management")
print()

local subscriptions = {}
local workflow_state = {
    active_workflows = {},
    completed_workflows = {},
    failed_workflows = {},
    workflow_stats = {
        total_created = 0,
        total_completed = 0,
        total_failed = 0,
        avg_duration = 0,
        step_executions = 0
    },
    coordination_patterns = {}
}

-- Helper function to generate workflow ID
local function generate_workflow_id()
    return "wf_" .. os.time() .. "_" .. math.random(1000, 9999)
end

-- Helper function to track workflow event
local function track_workflow_event(workflow_id, event_type, step_name, details)
    if not workflow_state.active_workflows[workflow_id] then
        workflow_state.active_workflows[workflow_id] = {
            id = workflow_id,
            created_at = os.time(),
            steps = {},
            current_step = nil,
            status = "created",
            metadata = {}
        }
        workflow_state.workflow_stats.total_created = workflow_state.workflow_stats.total_created + 1
    end
    
    local workflow = workflow_state.active_workflows[workflow_id]
    
    table.insert(workflow.steps, {
        timestamp = os.time(),
        event_type = event_type,
        step_name = step_name,
        details = details or {}
    })
    
    print(string.format("   🔄 [%s] %s: %s", workflow_id, event_type, step_name or "N/A"))
end

-- Helper function to complete workflow
local function complete_workflow(workflow_id, result)
    if workflow_state.active_workflows[workflow_id] then
        local workflow = workflow_state.active_workflows[workflow_id]
        workflow.completed_at = os.time()
        workflow.duration = workflow.completed_at - workflow.created_at
        workflow.result = result
        workflow.status = "completed"
        
        workflow_state.completed_workflows[workflow_id] = workflow
        workflow_state.active_workflows[workflow_id] = nil
        workflow_state.workflow_stats.total_completed = workflow_state.workflow_stats.total_completed + 1
        
        print(string.format("   ✅ Workflow %s completed in %ds", workflow_id, workflow.duration))
    end
end

-- Helper function to fail workflow
local function fail_workflow(workflow_id, error_reason)
    if workflow_state.active_workflows[workflow_id] then
        local workflow = workflow_state.active_workflows[workflow_id]
        workflow.failed_at = os.time()
        workflow.duration = workflow.failed_at - workflow.created_at
        workflow.error_reason = error_reason
        workflow.status = "failed"
        
        workflow_state.failed_workflows[workflow_id] = workflow
        workflow_state.active_workflows[workflow_id] = nil
        workflow_state.workflow_stats.total_failed = workflow_state.workflow_stats.total_failed + 1
        
        print(string.format("   ❌ Workflow %s failed in %ds: %s", workflow_id, workflow.duration, error_reason))
    end
end

print("1. Setting up workflow coordination subscriptions:")

print("   📡 Creating workflow coordination channels:")

-- Create subscriptions for different workflow coordination patterns
local workflow_patterns = {
    workflow_control = "workflow.control.*",
    step_coordination = "workflow.step.*",
    state_management = "workflow.state.*",
    error_handling = "workflow.error.*",
    completion_events = "workflow.complete.*",
    orchestration = "workflow.orchestration.*",
    distributed_coordination = "workflow.distributed.*"
}

for pattern_name, pattern in pairs(workflow_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Workflow coordination channels established")

print()
print("2. Simple sequential workflow:")

print("   🔄 Implementing simple sequential workflow:")

-- Define a simple data processing workflow
local simple_workflow_id = generate_workflow_id()
track_workflow_event(simple_workflow_id, "WORKFLOW_STARTED", "initialize")

local simple_workflow_steps = {
    {name = "validate_input", duration = 100, success_rate = 0.95},
    {name = "transform_data", duration = 200, success_rate = 0.98},
    {name = "validate_output", duration = 50, success_rate = 0.99},
    {name = "store_result", duration = 150, success_rate = 0.97}
}

print(string.format("   🎯 Starting workflow: %s", simple_workflow_id))

-- Execute workflow steps sequentially
local workflow_success = true
for i, step in ipairs(simple_workflow_steps) do
    track_workflow_event(simple_workflow_id, "STEP_STARTED", step.name)
    
    -- Publish step start event
    Event.publish("workflow.step.start", {
        workflow_id = simple_workflow_id,
        step_name = step.name,
        step_index = i,
        total_steps = #simple_workflow_steps,
        estimated_duration = step.duration
    })
    
    -- Simulate step execution
    local step_success = math.random() < step.success_rate
    local actual_duration = step.duration + math.random(-20, 20) -- Add some variance
    
    -- Simulate processing time (scaled down for demo)
    os.execute("sleep " .. (actual_duration / 1000))
    
    if step_success then
        track_workflow_event(simple_workflow_id, "STEP_COMPLETED", step.name, {
            duration = actual_duration,
            success = true
        })
        
        Event.publish("workflow.step.complete", {
            workflow_id = simple_workflow_id,
            step_name = step.name,
            step_index = i,
            duration_ms = actual_duration,
            success = true,
            output_data = {
                records_processed = math.random(100, 1000),
                quality_score = 0.8 + math.random() * 0.2
            }
        })
        
        workflow_state.workflow_stats.step_executions = workflow_state.workflow_stats.step_executions + 1
        print(string.format("   ✅ Step %d/%d: %s completed (%dms)", 
              i, #simple_workflow_steps, step.name, actual_duration))
    else
        track_workflow_event(simple_workflow_id, "STEP_FAILED", step.name, {
            duration = actual_duration,
            success = false
        })
        
        Event.publish("workflow.error.step_failed", {
            workflow_id = simple_workflow_id,
            step_name = step.name,
            step_index = i,
            error_type = "execution_failure",
            retry_possible = true
        })
        
        workflow_success = false
        print(string.format("   ❌ Step %d/%d: %s failed (%dms)", 
              i, #simple_workflow_steps, step.name, actual_duration))
        break
    end
end

-- Complete or fail the workflow
if workflow_success then
    Event.publish("workflow.complete.success", {
        workflow_id = simple_workflow_id,
        total_steps = #simple_workflow_steps,
        completion_time = os.time()
    })
    complete_workflow(simple_workflow_id, "success")
else
    Event.publish("workflow.complete.failure", {
        workflow_id = simple_workflow_id,
        failed_step = "transform_data",
        completion_time = os.time()
    })
    fail_workflow(simple_workflow_id, "Step execution failed")
end

print()
print("3. Parallel workflow execution:")

print("   🔀 Implementing parallel workflow execution:")

-- Define a parallel processing workflow
local parallel_workflow_id = generate_workflow_id()
track_workflow_event(parallel_workflow_id, "WORKFLOW_STARTED", "parallel_processing")

local parallel_branches = {
    {name = "process_branch_a", tasks = {"analyze_data", "generate_report"}, duration = 300},
    {name = "process_branch_b", tasks = {"validate_schema", "optimize_data"}, duration = 250},
    {name = "process_branch_c", tasks = {"compress_files", "upload_results"}, duration = 400}
}

print(string.format("   🎯 Starting parallel workflow: %s", parallel_workflow_id))

-- Start all parallel branches
local branch_results = {}
for i, branch in ipairs(parallel_branches) do
    track_workflow_event(parallel_workflow_id, "BRANCH_STARTED", branch.name)
    
    Event.publish("workflow.orchestration.branch_start", {
        workflow_id = parallel_workflow_id,
        branch_name = branch.name,
        branch_index = i,
        total_branches = #parallel_branches,
        tasks = branch.tasks,
        estimated_duration = branch.duration
    })
    
    print(string.format("   🚀 Branch %d: %s started with %d tasks", 
          i, branch.name, #branch.tasks))
end

-- Simulate parallel execution and completion
for i, branch in ipairs(parallel_branches) do
    -- Simulate processing time (branches run concurrently in real scenario)
    local actual_duration = branch.duration + math.random(-50, 50)
    local branch_success = math.random() < 0.92 -- 92% success rate
    
    if branch_success then
        track_workflow_event(parallel_workflow_id, "BRANCH_COMPLETED", branch.name, {
            duration = actual_duration,
            tasks_completed = #branch.tasks
        })
        
        Event.publish("workflow.orchestration.branch_complete", {
            workflow_id = parallel_workflow_id,
            branch_name = branch.name,
            branch_index = i,
            duration_ms = actual_duration,
            tasks_completed = #branch.tasks,
            success = true,
            results = {
                items_processed = math.random(500, 2000),
                success_rate = 0.9 + math.random() * 0.1
            }
        })
        
        branch_results[branch.name] = {success = true, duration = actual_duration}
        print(string.format("   ✅ Branch %d: %s completed (%dms)", 
              i, branch.name, actual_duration))
    else
        track_workflow_event(parallel_workflow_id, "BRANCH_FAILED", branch.name)
        
        Event.publish("workflow.error.branch_failed", {
            workflow_id = parallel_workflow_id,
            branch_name = branch.name,
            branch_index = i,
            error_type = "processing_error"
        })
        
        branch_results[branch.name] = {success = false, duration = actual_duration}
        print(string.format("   ❌ Branch %d: %s failed (%dms)", 
              i, branch.name, actual_duration))
    end
end

-- Check if all branches completed successfully
local all_branches_success = true
for _, result in pairs(branch_results) do
    if not result.success then
        all_branches_success = false
        break
    end
end

-- Complete parallel workflow
if all_branches_success then
    Event.publish("workflow.complete.parallel_success", {
        workflow_id = parallel_workflow_id,
        branches_completed = #parallel_branches,
        total_duration = math.max(table.unpack((function()
            local durations = {}
            for _, result in pairs(branch_results) do
                table.insert(durations, result.duration)
            end
            return durations
        end)()))
    })
    complete_workflow(parallel_workflow_id, "parallel_success")
else
    Event.publish("workflow.complete.partial_failure", {
        workflow_id = parallel_workflow_id,
        branches_completed = (function()
            local count = 0
            for _, result in pairs(branch_results) do
                if result.success then count = count + 1 end
            end
            return count
        end)(),
        total_branches = #parallel_branches
    })
    fail_workflow(parallel_workflow_id, "One or more parallel branches failed")
end

print()
print("4. Conditional workflow with decision points:")

print("   🎯 Implementing conditional workflow:")

-- Define a conditional workflow with decision points
local conditional_workflow_id = generate_workflow_id()
track_workflow_event(conditional_workflow_id, "WORKFLOW_STARTED", "conditional_processing")

print(string.format("   🎯 Starting conditional workflow: %s", conditional_workflow_id))

-- Initial data processing step
track_workflow_event(conditional_workflow_id, "STEP_STARTED", "initial_processing")

Event.publish("workflow.step.start", {
    workflow_id = conditional_workflow_id,
    step_name = "initial_processing",
    step_type = "data_processing"
})

-- Simulate initial processing
local data_quality = math.random()
local data_size = math.random(1000, 10000)

track_workflow_event(conditional_workflow_id, "STEP_COMPLETED", "initial_processing", {
    data_quality = data_quality,
    data_size = data_size
})

Event.publish("workflow.step.complete", {
    workflow_id = conditional_workflow_id,
    step_name = "initial_processing",
    output_data = {
        quality_score = data_quality,
        record_count = data_size
    }
})

print(string.format("   ✅ Initial processing: quality=%.2f, size=%d", data_quality, data_size))

-- Decision point 1: Data quality check
track_workflow_event(conditional_workflow_id, "DECISION_POINT", "quality_check")

Event.publish("workflow.control.decision", {
    workflow_id = conditional_workflow_id,
    decision_point = "quality_check",
    criteria = {
        quality_threshold = 0.7,
        actual_quality = data_quality
    }
})

if data_quality >= 0.7 then
    print("   🎯 Decision: High quality data - proceeding with advanced processing")
    
    -- High quality path
    track_workflow_event(conditional_workflow_id, "PATH_SELECTED", "high_quality_path")
    
    Event.publish("workflow.orchestration.path_selected", {
        workflow_id = conditional_workflow_id,
        path = "high_quality_processing",
        reason = "data_quality_sufficient"
    })
    
    -- Advanced processing steps
    local advanced_steps = {"feature_extraction", "model_training", "optimization"}
    for i, step in ipairs(advanced_steps) do
        track_workflow_event(conditional_workflow_id, "STEP_STARTED", step)
        
        Event.publish("workflow.step.start", {
            workflow_id = conditional_workflow_id,
            step_name = step,
            step_type = "advanced_processing",
            path = "high_quality"
        })
        
        -- Simulate step execution
        local step_duration = math.random(100, 300)
        os.execute("sleep " .. (step_duration / 1000))
        
        track_workflow_event(conditional_workflow_id, "STEP_COMPLETED", step)
        
        Event.publish("workflow.step.complete", {
            workflow_id = conditional_workflow_id,
            step_name = step,
            duration_ms = step_duration,
            path = "high_quality"
        })
        
        print(string.format("   ✅ Advanced step: %s completed (%dms)", step, step_duration))
    end
    
else
    print("   🎯 Decision: Low quality data - proceeding with basic processing")
    
    -- Low quality path
    track_workflow_event(conditional_workflow_id, "PATH_SELECTED", "basic_path")
    
    Event.publish("workflow.orchestration.path_selected", {
        workflow_id = conditional_workflow_id,
        path = "basic_processing",
        reason = "data_quality_insufficient"
    })
    
    -- Data cleaning and basic processing
    local basic_steps = {"data_cleaning", "basic_validation", "simple_output"}
    for i, step in ipairs(basic_steps) do
        track_workflow_event(conditional_workflow_id, "STEP_STARTED", step)
        
        Event.publish("workflow.step.start", {
            workflow_id = conditional_workflow_id,
            step_name = step,
            step_type = "basic_processing",
            path = "basic"
        })
        
        -- Simulate step execution
        local step_duration = math.random(50, 150)
        os.execute("sleep " .. (step_duration / 1000))
        
        track_workflow_event(conditional_workflow_id, "STEP_COMPLETED", step)
        
        Event.publish("workflow.step.complete", {
            workflow_id = conditional_workflow_id,
            step_name = step,
            duration_ms = step_duration,
            path = "basic"
        })
        
        print(string.format("   ✅ Basic step: %s completed (%dms)", step, step_duration))
    end
end

-- Decision point 2: Size check for additional processing
track_workflow_event(conditional_workflow_id, "DECISION_POINT", "size_check")

Event.publish("workflow.control.decision", {
    workflow_id = conditional_workflow_id,
    decision_point = "size_check",
    criteria = {
        size_threshold = 5000,
        actual_size = data_size
    }
})

if data_size > 5000 then
    print("   🎯 Decision: Large dataset - adding compression step")
    
    track_workflow_event(conditional_workflow_id, "STEP_STARTED", "compression")
    
    Event.publish("workflow.step.start", {
        workflow_id = conditional_workflow_id,
        step_name = "compression",
        step_type = "optimization",
        trigger = "large_dataset"
    })
    
    local compression_duration = math.random(200, 400)
    os.execute("sleep " .. (compression_duration / 1000))
    
    track_workflow_event(conditional_workflow_id, "STEP_COMPLETED", "compression")
    
    Event.publish("workflow.step.complete", {
        workflow_id = conditional_workflow_id,
        step_name = "compression",
        duration_ms = compression_duration,
        compression_ratio = 0.3 + math.random() * 0.4
    })
    
    print(string.format("   ✅ Compression step completed (%dms)", compression_duration))
end

-- Complete conditional workflow
Event.publish("workflow.complete.conditional", {
    workflow_id = conditional_workflow_id,
    path_taken = data_quality >= 0.7 and "high_quality" or "basic",
    compression_applied = data_size > 5000,
    final_quality = data_quality,
    final_size = data_size
})

complete_workflow(conditional_workflow_id, "conditional_success")

print()
print("5. Distributed workflow coordination:")

print("   🌐 Implementing distributed workflow coordination:")

-- Define a distributed workflow across multiple services
local distributed_workflow_id = generate_workflow_id()
track_workflow_event(distributed_workflow_id, "WORKFLOW_STARTED", "distributed_processing")

local distributed_services = {
    {name = "data_service", location = "us-east-1", steps = {"fetch_data", "validate_data"}},
    {name = "compute_service", location = "us-west-2", steps = {"process_data", "analyze_results"}},
    {name = "storage_service", location = "eu-west-1", steps = {"store_results", "backup_data"}},
    {name = "notification_service", location = "ap-southeast-1", steps = {"send_notifications"}}
}

print(string.format("   🎯 Starting distributed workflow: %s", distributed_workflow_id))
print(string.format("   🌍 Coordinating across %d services in different regions", #distributed_services))

-- Service coordination phase
for i, service in ipairs(distributed_services) do
    track_workflow_event(distributed_workflow_id, "SERVICE_REGISTERED", service.name)
    
    Event.publish("workflow.distributed.service_register", {
        workflow_id = distributed_workflow_id,
        service_name = service.name,
        service_location = service.location,
        service_index = i,
        total_services = #distributed_services,
        steps = service.steps,
        registration_time = os.time()
    })
    
    print(string.format("   📍 Service %d: %s registered (%s)", i, service.name, service.location))
end

-- Execute distributed steps with cross-service coordination
local service_states = {}
for i, service in ipairs(distributed_services) do
    service_states[service.name] = {
        status = "ready",
        completed_steps = 0,
        total_steps = #service.steps
    }
    
    -- Execute steps for this service
    for j, step in ipairs(service.steps) do
        track_workflow_event(distributed_workflow_id, "DISTRIBUTED_STEP_STARTED", step, {
            service = service.name,
            location = service.location
        })
        
        Event.publish("workflow.distributed.step_start", {
            workflow_id = distributed_workflow_id,
            service_name = service.name,
            service_location = service.location,
            step_name = step,
            step_index = j,
            service_step_count = #service.steps
        })
        
        -- Simulate network latency and processing time
        local step_duration = math.random(150, 400) -- Longer for distributed
        local network_latency = math.random(50, 200) -- Network overhead
        local total_duration = step_duration + network_latency
        
        os.execute("sleep " .. (total_duration / 1000))
        
        -- Simulate occasional network/service issues
        local step_success = math.random() < 0.94 -- 94% success rate (distributed is less reliable)
        
        if step_success then
            track_workflow_event(distributed_workflow_id, "DISTRIBUTED_STEP_COMPLETED", step, {
                service = service.name,
                duration = total_duration,
                network_latency = network_latency
            })
            
            Event.publish("workflow.distributed.step_complete", {
                workflow_id = distributed_workflow_id,
                service_name = service.name,
                service_location = service.location,
                step_name = step,
                duration_ms = step_duration,
                network_latency_ms = network_latency,
                success = true
            })
            
            service_states[service.name].completed_steps = service_states[service.name].completed_steps + 1
            
            print(string.format("   ✅ %s[%s]: %s completed (%dms + %dms network)", 
                  service.name, service.location, step, step_duration, network_latency))
        else
            track_workflow_event(distributed_workflow_id, "DISTRIBUTED_STEP_FAILED", step, {
                service = service.name,
                error_type = "network_timeout"
            })
            
            Event.publish("workflow.error.distributed_failure", {
                workflow_id = distributed_workflow_id,
                service_name = service.name,
                service_location = service.location,
                step_name = step,
                error_type = "network_timeout",
                retry_recommended = true
            })
            
            service_states[service.name].status = "failed"
            
            print(string.format("   ❌ %s[%s]: %s failed (network timeout)", 
                  service.name, service.location, step))
            break -- Stop processing this service
        end
    end
    
    -- Update service completion status
    if service_states[service.name].completed_steps == #service.steps then
        service_states[service.name].status = "completed"
        
        Event.publish("workflow.distributed.service_complete", {
            workflow_id = distributed_workflow_id,
            service_name = service.name,
            service_location = service.location,
            steps_completed = service_states[service.name].completed_steps
        })
        
        print(string.format("   🎉 Service %s completed all steps", service.name))
    end
end

-- Check overall distributed workflow completion
local completed_services = 0
local failed_services = 0

for service_name, state in pairs(service_states) do
    if state.status == "completed" then
        completed_services = completed_services + 1
    elseif state.status == "failed" then
        failed_services = failed_services + 1
    end
end

if completed_services == #distributed_services then
    Event.publish("workflow.complete.distributed_success", {
        workflow_id = distributed_workflow_id,
        services_completed = completed_services,
        total_services = #distributed_services,
        coordination_successful = true
    })
    complete_workflow(distributed_workflow_id, "distributed_success")
else
    Event.publish("workflow.complete.distributed_partial", {
        workflow_id = distributed_workflow_id,
        services_completed = completed_services,
        services_failed = failed_services,
        total_services = #distributed_services
    })
    fail_workflow(distributed_workflow_id, string.format("%d services failed", failed_services))
end

print()
print("6. Workflow state monitoring and recovery:")

print("   📊 Workflow state monitoring:")

-- Monitor all active workflows
print("   🔍 Active workflow status:")
for workflow_id, workflow in pairs(workflow_state.active_workflows) do
    local runtime = os.time() - workflow.created_at
    print(string.format("   • %s: %s (%d steps, %ds runtime)", 
          workflow_id, workflow.status, #workflow.steps, runtime))
end

print("   📈 Workflow statistics:")
print(string.format("   • Total workflows created: %d", workflow_state.workflow_stats.total_created))
print(string.format("   • Completed workflows: %d", workflow_state.workflow_stats.total_completed))
print(string.format("   • Failed workflows: %d", workflow_state.workflow_stats.total_failed))
print(string.format("   • Step executions: %d", workflow_state.workflow_stats.step_executions))

-- Calculate success rate
local total_finished = workflow_state.workflow_stats.total_completed + workflow_state.workflow_stats.total_failed
local success_rate = total_finished > 0 and 
                    (workflow_state.workflow_stats.total_completed / total_finished) * 100 or 0

print(string.format("   • Success rate: %.1f%%", success_rate))

-- Calculate average duration for completed workflows
local total_duration = 0
local completed_count = 0

for _, workflow in pairs(workflow_state.completed_workflows) do
    if workflow.duration then
        total_duration = total_duration + workflow.duration
        completed_count = completed_count + 1
    end
end

if completed_count > 0 then
    local avg_duration = total_duration / completed_count
    print(string.format("   • Average workflow duration: %.1fs", avg_duration))
end

print()
print("7. Event-driven workflow recovery:")

print("   🚑 Implementing workflow recovery patterns:")

-- Simulate a workflow that needs recovery
local recovery_workflow_id = generate_workflow_id()
track_workflow_event(recovery_workflow_id, "WORKFLOW_STARTED", "recovery_test")

-- Simulate workflow failure
track_workflow_event(recovery_workflow_id, "STEP_STARTED", "critical_step")
track_workflow_event(recovery_workflow_id, "STEP_FAILED", "critical_step", {
    error_type = "timeout",
    recoverable = true
})

Event.publish("workflow.error.recoverable_failure", {
    workflow_id = recovery_workflow_id,
    failed_step = "critical_step",
    error_type = "timeout",
    recovery_strategy = "retry_with_backoff"
})

print(string.format("   ⚠️  Workflow %s encountered recoverable failure", recovery_workflow_id))

-- Implement recovery strategy
print("   🔄 Initiating recovery strategy: retry with backoff")

for retry_attempt = 1, 3 do
    track_workflow_event(recovery_workflow_id, "RECOVERY_ATTEMPT", "critical_step", {
        attempt = retry_attempt,
        strategy = "retry_with_backoff"
    })
    
    Event.publish("workflow.control.recovery_attempt", {
        workflow_id = recovery_workflow_id,
        retry_attempt = retry_attempt,
        max_attempts = 3,
        step_name = "critical_step",
        strategy = "retry_with_backoff"
    })
    
    -- Simulate retry with increasing success probability
    local retry_success = math.random() < (0.3 + (retry_attempt * 0.25))
    local retry_duration = 200 * retry_attempt -- Increasing backoff
    
    os.execute("sleep " .. (retry_duration / 1000))
    
    if retry_success then
        track_workflow_event(recovery_workflow_id, "RECOVERY_SUCCESSFUL", "critical_step", {
            attempt = retry_attempt,
            duration = retry_duration
        })
        
        Event.publish("workflow.control.recovery_success", {
            workflow_id = recovery_workflow_id,
            recovered_step = "critical_step",
            successful_attempt = retry_attempt,
            recovery_duration = retry_duration
        })
        
        Event.publish("workflow.complete.recovered", {
            workflow_id = recovery_workflow_id,
            recovery_attempts = retry_attempt,
            recovery_successful = true
        })
        
        complete_workflow(recovery_workflow_id, "recovered_success")
        print(string.format("   ✅ Recovery successful on attempt %d (%dms)", retry_attempt, retry_duration))
        break
    else
        print(string.format("   ❌ Recovery attempt %d failed", retry_attempt))
        
        if retry_attempt == 3 then
            track_workflow_event(recovery_workflow_id, "RECOVERY_EXHAUSTED", "critical_step")
            
            Event.publish("workflow.error.recovery_exhausted", {
                workflow_id = recovery_workflow_id,
                failed_step = "critical_step",
                total_attempts = 3
            })
            
            fail_workflow(recovery_workflow_id, "Recovery attempts exhausted")
            print("   ❌ All recovery attempts exhausted - workflow failed")
        end
    end
end

print()
print("8. Workflow coordination best practices:")

print("   💡 Workflow Coordination Best Practices:")
print("   • Use event-driven communication for loose coupling")
print("   • Implement proper state management and persistence")
print("   • Design workflows with recovery and retry mechanisms")
print("   • Monitor workflow progress and performance metrics")
print("   • Use correlation IDs for distributed workflow tracking")
print("   • Implement timeouts and deadlock prevention")
print("   • Provide clear error handling and failure modes")
print("   • Use conditional logic for flexible workflow paths")
print("   • Implement proper coordination for parallel execution")
print("   • Log all workflow events for debugging and audit")

print()
print("9. Workflow coordination cleanup:")

-- Process any remaining workflow events
print("   📥 Processing remaining workflow events:")

local events_processed = 0
for subscription_name, sub_id in pairs(subscriptions) do
    local received_count = 0
    
    for attempt = 1, 3 do
        local received = Event.receive(sub_id, 100)
        if received then
            received_count = received_count + 1
            events_processed = events_processed + 1
            
            print(string.format("   📨 %s: %s", subscription_name, received.event_type or "unknown"))
        else
            break
        end
    end
    
    if received_count > 0 then
        print(string.format("   📊 %s: processed %d events", subscription_name, received_count))
    end
end

print(string.format("   ✅ Processed %d workflow coordination events", events_processed))

print()
print("10. Final workflow statistics:")

print("   📊 Final Workflow Statistics:")
print(string.format("   • Workflows created: %d", workflow_state.workflow_stats.total_created))
print(string.format("   • Workflows completed: %d", workflow_state.workflow_stats.total_completed))
print(string.format("   • Workflows failed: %d", workflow_state.workflow_stats.total_failed))
print(string.format("   • Total step executions: %d", workflow_state.workflow_stats.step_executions))
print(string.format("   • Overall success rate: %.1f%%", success_rate))

-- Cleanup subscriptions
local cleanup_count = 0
for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name)
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "workflow coordination subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event workflow coordination example complete!")
print("   Key concepts demonstrated:")
print("   • Sequential workflow execution with step tracking")
print("   • Parallel workflow branches with synchronization")
print("   • Conditional workflows with decision points")
print("   • Distributed workflow coordination across services")
print("   • Workflow state monitoring and progress tracking")
print("   • Event-driven recovery and retry mechanisms")
print("   • Comprehensive workflow statistics and analytics")
print("   • Best practices for robust workflow orchestration")
print("   • Cross-service coordination and error handling")
print("   • Workflow lifecycle management and cleanup")