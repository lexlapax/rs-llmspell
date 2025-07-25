-- ABOUTME: Workflow integration hooks for stage coordination and execution flow
-- ABOUTME: Demonstrates BeforeWorkflowStart, WorkflowStageTransition, BeforeWorkflowStage, AfterWorkflowStage, WorkflowCheckpoint, WorkflowRollback, AfterWorkflowComplete, WorkflowError

print("=== Workflow Integration Hooks Example ===")
print("Demonstrates: Complete workflow lifecycle with coordination and error handling")
print()

local handles = {}
local workflow_state = {
    current_workflow = nil,
    stages_completed = {},
    checkpoints = {},
    execution_log = {},
    performance_metrics = {}
}

-- Helper function to log workflow events
local function log_workflow_event(event_type, workflow_name, stage_name, details)
    local entry = {
        timestamp = os.time(),
        event_type = event_type,
        workflow = workflow_name,
        stage = stage_name or "N/A",
        details = details or {}
    }
    table.insert(workflow_state.execution_log, entry)
    
    print(string.format("   📋 [%s] %s - Workflow: %s, Stage: %s", 
          os.date("%H:%M:%S", entry.timestamp), event_type, workflow_name, stage_name or "N/A"))
end

print("1. Registering workflow lifecycle hooks:")

-- Before Workflow Start - Initialization and preparation
handles.before_start = Hook.register("BeforeWorkflowStart", function(context)
    local workflow_name = context.component_id.name
    workflow_state.current_workflow = workflow_name
    workflow_state.stages_completed = {}
    workflow_state.checkpoints = {}
    
    log_workflow_event("WORKFLOW_START", workflow_name, nil, {
        workflow_type = context.data and context.data.workflow_type or "unknown"
    })
    
    print("   🚀 Initializing workflow:", workflow_name)
    
    -- Could perform initialization like:
    -- - Validate workflow configuration
    -- - Allocate resources
    -- - Initialize shared state
    -- - Set up monitoring
    
    workflow_state.performance_metrics[workflow_name] = {
        start_time = os.clock(),
        stages = {}
    }
    
    return "continue"
end, "high")
print("   ✅ Registered BeforeWorkflowStart hook")

-- Workflow Stage Transition - Control flow between stages
handles.stage_transition = Hook.register("WorkflowStageTransition", function(context)
    local workflow_name = context.component_id.name
    local from_stage = context.data and context.data.from_stage or "unknown"
    local to_stage = context.data and context.data.to_stage or "unknown"
    
    log_workflow_event("STAGE_TRANSITION", workflow_name, to_stage, {
        from = from_stage,
        to = to_stage
    })
    
    print(string.format("   🔄 Transitioning: %s → %s", from_stage, to_stage))
    
    -- Could perform transition logic like:
    -- - Validate stage prerequisites
    -- - Transfer state between stages
    -- - Update workflow progress
    -- - Check resource availability
    
    -- Example: Block certain transitions based on conditions
    local blocked_transitions = {
        ["error_stage"] = "Cannot transition to error stage without proper handling",
        ["unauthorized_stage"] = "Insufficient permissions for this stage"
    }
    
    if blocked_transitions[to_stage] then
        print("   🚫 Transition blocked:", blocked_transitions[to_stage])
        return {
            type = "cancel",
            reason = blocked_transitions[to_stage]
        }
    end
    
    return "continue"
end, "high")
print("   ✅ Registered WorkflowStageTransition hook")

-- Before Workflow Stage - Pre-stage preparation
handles.before_stage = Hook.register("BeforeWorkflowStage", function(context)
    local workflow_name = context.component_id.name
    local stage_name = context.data and context.data.stage_name or "unknown"
    
    log_workflow_event("STAGE_START", workflow_name, stage_name)
    
    print("   ⚡ Starting stage:", stage_name)
    
    -- Record stage start time
    if not workflow_state.performance_metrics[workflow_name] then
        workflow_state.performance_metrics[workflow_name] = {stages = {}}
    end
    workflow_state.performance_metrics[workflow_name].stages[stage_name] = {
        start_time = os.clock()
    }
    
    -- Could perform pre-stage tasks like:
    -- - Validate stage inputs
    -- - Initialize stage resources
    -- - Set up stage monitoring
    -- - Check dependencies
    
    return "continue"
end, "normal")
print("   ✅ Registered BeforeWorkflowStage hook")

-- After Workflow Stage - Post-stage cleanup and validation
handles.after_stage = Hook.register("AfterWorkflowStage", function(context)
    local workflow_name = context.component_id.name
    local stage_name = context.data and context.data.stage_name or "unknown"
    
    log_workflow_event("STAGE_COMPLETE", workflow_name, stage_name)
    
    print("   ✅ Completed stage:", stage_name)
    
    -- Record stage completion time
    if workflow_state.performance_metrics[workflow_name] and 
       workflow_state.performance_metrics[workflow_name].stages[stage_name] then
        local stage_metrics = workflow_state.performance_metrics[workflow_name].stages[stage_name]
        stage_metrics.end_time = os.clock()
        stage_metrics.duration = (stage_metrics.end_time - stage_metrics.start_time) * 1000
        
        print(string.format("   ⏱️  Stage duration: %.2fms", stage_metrics.duration))
    end
    
    -- Track completed stages
    table.insert(workflow_state.stages_completed, {
        stage = stage_name,
        timestamp = os.time(),
        success = true
    })
    
    -- Could perform post-stage tasks like:
    -- - Save stage results
    -- - Update progress indicators
    -- - Validate stage outputs
    -- - Clean up stage resources
    
    return "continue"
end, "normal")
print("   ✅ Registered AfterWorkflowStage hook")

print()
print("2. Registering workflow control and recovery hooks:")

-- Workflow Checkpoint - Save workflow state
handles.checkpoint = Hook.register("WorkflowCheckpoint", function(context)
    local workflow_name = context.component_id.name
    local checkpoint_name = context.data and context.data.checkpoint_name or "auto_checkpoint"
    
    log_workflow_event("CHECKPOINT", workflow_name, nil, {
        checkpoint = checkpoint_name
    })
    
    print("   💾 Creating checkpoint:", checkpoint_name)
    
    -- Save workflow state
    local checkpoint_data = {
        timestamp = os.time(),
        workflow = workflow_name,
        stages_completed = #workflow_state.stages_completed,
        current_state = context.data or {},
        checkpoint_name = checkpoint_name
    }
    
    workflow_state.checkpoints[checkpoint_name] = checkpoint_data
    
    print(string.format("   📊 Checkpoint saved - %d stages completed", 
          checkpoint_data.stages_completed))
    
    -- Could perform checkpoint tasks like:
    -- - Persist workflow state to storage
    -- - Create recovery metadata
    -- - Backup intermediate results
    -- - Update checkpoint registry
    
    return "continue"
end, "normal")
print("   ✅ Registered WorkflowCheckpoint hook")

-- Workflow Rollback - Handle workflow rollback
handles.rollback = Hook.register("WorkflowRollback", function(context)
    local workflow_name = context.component_id.name
    local rollback_reason = context.data and context.data.reason or "unknown"
    local target_checkpoint = context.data and context.data.target_checkpoint or "latest"
    
    log_workflow_event("ROLLBACK", workflow_name, nil, {
        reason = rollback_reason,
        target = target_checkpoint
    })
    
    print("   ⏪ Rolling back workflow:", workflow_name)
    print("   🔧 Rollback reason:", rollback_reason)
    print("   🎯 Target checkpoint:", target_checkpoint)
    
    -- Find the target checkpoint
    local checkpoint = workflow_state.checkpoints[target_checkpoint]
    if checkpoint then
        print(string.format("   📍 Restoring to checkpoint from %s", 
              os.date("%H:%M:%S", checkpoint.timestamp)))
        
        -- Could perform rollback tasks like:
        -- - Restore workflow state from checkpoint
        -- - Undo completed stages
        -- - Clean up resources
        -- - Reset progress indicators
        
        return {
            type = "modified",
            data = {
                rollback_completed = true,
                restored_checkpoint = target_checkpoint,
                stages_rolled_back = #workflow_state.stages_completed - checkpoint.stages_completed
            }
        }
    else
        print("   ❌ Checkpoint not found:", target_checkpoint)
        return {
            type = "cancel",
            reason = "Target checkpoint not found: " .. target_checkpoint
        }
    end
end, "high")
print("   ✅ Registered WorkflowRollback hook")

-- After Workflow Complete - Final workflow cleanup
handles.after_complete = Hook.register("AfterWorkflowComplete", function(context)
    local workflow_name = context.component_id.name
    
    log_workflow_event("WORKFLOW_COMPLETE", workflow_name)
    
    print("   🎉 Workflow completed:", workflow_name)
    
    -- Calculate total workflow duration
    if workflow_state.performance_metrics[workflow_name] then
        local metrics = workflow_state.performance_metrics[workflow_name]
        metrics.end_time = os.clock()
        metrics.total_duration = (metrics.end_time - metrics.start_time) * 1000
        
        print(string.format("   ⏱️  Total workflow duration: %.2fms", metrics.total_duration))
        
        -- Show stage breakdown
        print("   📊 Stage performance breakdown:")
        for stage_name, stage_metrics in pairs(metrics.stages) do
            if stage_metrics.duration then
                print(string.format("     • %s: %.2fms", stage_name, stage_metrics.duration))
            end
        end
    end
    
    print("   📋 Stages completed:", #workflow_state.stages_completed)
    print("   💾 Checkpoints created:", table.concat(
        (function()
            local names = {}
            for name, _ in pairs(workflow_state.checkpoints) do
                table.insert(names, name)
            end
            return names
        end)(), ", "))
    
    -- Could perform completion tasks like:
    -- - Generate workflow report
    -- - Archive workflow data
    -- - Send completion notifications
    -- - Update workflow statistics
    -- - Clean up resources
    
    return "continue"
end, "low")
print("   ✅ Registered AfterWorkflowComplete hook")

-- Workflow Error - Handle workflow errors
handles.workflow_error = Hook.register("WorkflowError", function(context)
    local workflow_name = context.component_id.name
    local error_message = context.data and context.data.error_message or "Unknown workflow error"
    local error_stage = context.data and context.data.error_stage or "unknown"
    
    log_workflow_event("WORKFLOW_ERROR", workflow_name, error_stage, {
        error = error_message
    })
    
    print("   ❌ Workflow error in:", workflow_name)
    print("   🔧 Error stage:", error_stage)
    print("   💥 Error message:", error_message)
    
    -- Error recovery strategies
    local recovery_options = {
        ["stage_retry"] = "Retry the failed stage",
        ["rollback_checkpoint"] = "Rollback to last checkpoint",
        ["skip_stage"] = "Skip the failed stage and continue",
        ["abort_workflow"] = "Abort the entire workflow"
    }
    
    -- Determine recovery strategy based on error type
    local recovery_strategy = "rollback_checkpoint" -- Default
    if error_message:find("timeout") then
        recovery_strategy = "stage_retry"
    elseif error_message:find("validation") then
        recovery_strategy = "skip_stage"
    elseif error_message:find("critical") then
        recovery_strategy = "abort_workflow"
    end
    
    print("   🔧 Recovery strategy:", recovery_options[recovery_strategy])
    
    return {
        type = "modified",
        data = {
            error_handled = true,
            recovery_strategy = recovery_strategy,
            error_details = {
                message = error_message,
                stage = error_stage,
                timestamp = os.time()
            },
            workflow_state_preserved = true
        }
    }
end, "highest")
print("   ✅ Registered WorkflowError hook with recovery strategies")

print()
print("3. Workflow monitoring dashboard:")
print("   📊 Current Workflow State:")
print("   • Active workflow:", workflow_state.current_workflow or "none")
print("   • Stages completed:", #workflow_state.stages_completed)
print("   • Checkpoints available:", (function()
    local count = 0
    for _ in pairs(workflow_state.checkpoints) do count = count + 1 end
    return count
end)())
print("   • Execution events logged:", #workflow_state.execution_log)

print()
print("4. Workflow execution log summary:")
if #workflow_state.execution_log > 0 then
    print("   📋 Recent workflow events:")
    for i = math.max(1, #workflow_state.execution_log - 5), #workflow_state.execution_log do
        local entry = workflow_state.execution_log[i]
        print(string.format("   %d. %s: %s (%s)", 
              i, entry.event_type, entry.workflow, entry.stage))
    end
end

print()
print("5. Workflow hook coordination chain:")
local workflow_hooks = Hook.list()
local workflow_hook_count = 0

print("   🔗 Active workflow hooks:")
for _, hook in ipairs(workflow_hooks) do
    if hook.name:find("Workflow") then
        workflow_hook_count = workflow_hook_count + 1
        print(string.format("   %d. %s (%s priority)", 
              workflow_hook_count, hook.name, hook.priority))
    end
end

print("   📋 Total workflow hooks:", workflow_hook_count)

print()
print("6. Cleaning up workflow hooks:")
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "hook")
end

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)

print()
print("✨ Workflow integration hooks example complete!")
print("   Key concepts demonstrated:")
print("   • Complete workflow lifecycle coverage (8 hook points)")
print("   • Stage transition control and validation")
print("   • Checkpoint creation and rollback mechanisms")
print("   • Performance monitoring across workflow stages")
print("   • Error handling with multiple recovery strategies")
print("   • Workflow state management and coordination")
print("   • Cross-stage data flow and dependency management")