#!/usr/bin/env llmspell

-- session_replay.lua - Demonstrates session replay and recovery capabilities
-- This example shows how to use replay functionality for debugging and recovery

print("=== Session Replay Example ===\n")

-- Create a session that we'll replay later
local original_session = Session.create({
    name = "Data Processing Pipeline",
    description = "Multi-step data processing with replay capability",
    tags = {"pipeline", "replay", "example"},
    metadata = {
        pipeline_version = "2.0",
        replay_enabled = true
    }
})

print("Created session: " .. original_session)
Session.setCurrent(original_session)

-- Helper function to simulate processing steps
local function processStep(step_name, data, should_fail)
    print("\n→ Processing: " .. step_name)
    
    -- Store step input as artifact
    local input_artifact = Artifact.store(
        original_session,
        "system_generated",
        step_name .. "_input.json",
        Tool.execute("json-processor", {
            operation = "stringify",
            input = {step = step_name, data = data, timestamp = os.time()}
        }).result,
        {
            step_name = step_name,
            artifact_role = "input",
            tags = {"pipeline", "step-data"}
        }
    )
    
    -- Simulate processing
    if should_fail then
        error("Simulated failure in step: " .. step_name)
    end
    
    -- Transform data (simple example)
    local result = {
        step = step_name,
        input_data = data,
        processed_at = os.time(),
        result = step_name == "transform" and data * 2 or data + 10
    }
    
    -- Store step output
    local output_artifact = Artifact.store(
        original_session,
        "system_generated",
        step_name .. "_output.json",
        Tool.execute("json-processor", {
            operation = "stringify",
            input = result
        }).result,
        {
            step_name = step_name,
            artifact_role = "output",
            tags = {"pipeline", "step-data"}
        }
    )
    
    print("  ✓ " .. step_name .. " completed")
    return result
end

-- Execute pipeline with checkpoints
print("\n1. Running original pipeline:")

local pipeline_data = 100
local results = {}

-- Step 1: Validation
local success, result = pcall(function()
    return processStep("validation", pipeline_data, false)
end)
if success then
    results.validation = result
    Session.save(original_session) -- Checkpoint
end

-- Step 2: Transform
success, result = pcall(function()
    return processStep("transform", results.validation.result, false)
end)
if success then
    results.transform = result
    Session.save(original_session) -- Checkpoint
end

-- Step 3: Analysis (this will fail)
print("\n⚠ Simulating failure...")
success, result = pcall(function()
    return processStep("analysis", results.transform.result, true)
end)
if not success then
    print("  ✗ Pipeline failed: " .. tostring(result))
    results.failure = {
        step = "analysis",
        error = tostring(result),
        timestamp = os.time()
    }
end

-- 2. Check replay capability
print("\n2. Checking replay capability:")

local can_replay = Session.canReplay(original_session)
print("  Can replay session: " .. tostring(can_replay))

local replay_metadata = Session.getReplayMetadata(original_session)
if replay_metadata then
    print("  Replay metadata available:")
    print("    Event count: " .. (replay_metadata.event_count or 0))
    print("    Checkpoints: " .. (replay_metadata.checkpoint_count or 0))
    print("    Time range: " .. (replay_metadata.time_range or "N/A"))
end

-- 3. List replayable sessions
print("\n3. Finding replayable sessions:")

local replayable = Session.listReplayable()
print("  Found " .. #replayable .. " replayable sessions")
for i, session_info in ipairs(replayable) do
    if i <= 3 then  -- Show first 3
        print("    - " .. session_info.id .. ": " .. (session_info.name or "Unnamed"))
    end
end

-- 4. Analyze session timeline
print("\n4. Analyzing session timeline:")

-- Get all artifacts to reconstruct timeline
local artifacts = Artifact.list(original_session)
local timeline = {}

for _, artifact in ipairs(artifacts) do
    table.insert(timeline, {
        time = artifact.created_at,
        type = artifact.artifact_type,
        name = artifact.name,
        metadata = artifact.metadata
    })
end

-- Sort by time
table.sort(timeline, function(a, b) return a.time < b.time end)

print("  Timeline of events:")
for _, event in ipairs(timeline) do
    local step_info = event.metadata.step_name and 
        (" [" .. event.metadata.step_name .. " - " .. event.metadata.artifact_role .. "]") or ""
    print("    " .. os.date("%H:%M:%S", event.time) .. ": " .. event.name .. step_info)
end

-- 5. Create recovery session
print("\n5. Creating recovery session:")

local recovery_session = Session.create({
    name = "Pipeline Recovery",
    description = "Recovery from failed pipeline",
    parent_session_id = original_session,
    tags = {"recovery", "pipeline"},
    metadata = {
        original_session = original_session,
        recovery_reason = "analysis_step_failure",
        recovery_started = os.time()
    }
})

print("  Created recovery session: " .. recovery_session)
Session.setCurrent(recovery_session)

-- 6. Replay from last checkpoint
print("\n6. Replaying from last checkpoint:")

-- Find the last successful step's output
local last_good_artifact = nil
for i = #artifacts, 1, -1 do
    local artifact = artifacts[i]
    if artifact.metadata.artifact_role == "output" and 
       artifact.metadata.step_name == "transform" then
        last_good_artifact = artifact
        break
    end
end

if last_good_artifact then
    print("  Found last checkpoint: " .. last_good_artifact.name)
    
    -- Retrieve the data
    local checkpoint_data = Artifact.get(original_session, last_good_artifact.id)
    if checkpoint_data then
        local parsed = Tool.execute("json-processor", {
            operation = "parse",
            input = checkpoint_data.content
        })
        
        if parsed.success then
            local recovered_data = parsed.result.result
            print("  Recovered data: " .. tostring(recovered_data))
            
            -- Retry the failed step
            print("\n7. Retrying failed step:")
            success, result = pcall(function()
                return processStep("analysis", recovered_data, false)  -- Don't fail this time
            end)
            
            if success then
                results.analysis = result
                print("  ✓ Recovery successful!")
                
                -- Complete the pipeline
                success, result = pcall(function()
                    return processStep("finalization", results.analysis.result, false)
                end)
                
                if success then
                    results.finalization = result
                    print("  ✓ Pipeline completed successfully!")
                end
            end
        end
    end
end

-- 8. Generate replay report
print("\n8. Generating replay report:")

local report = {
    original_session = original_session,
    recovery_session = recovery_session,
    failure_info = results.failure,
    recovery_time = os.time(),
    steps_completed = {},
    steps_replayed = {"analysis", "finalization"},
    final_result = results.finalization and results.finalization.result or nil
}

-- List completed steps
for step_name, _ in pairs(results) do
    if step_name ~= "failure" then
        table.insert(report.steps_completed, step_name)
    end
end

local report_json = Tool.execute("json-processor", {
    operation = "stringify",
    input = report,
    pretty = true
})

Artifact.store(
    recovery_session,
    "system_generated",
    "replay_report.json",
    report_json.result,
    {
        report_type = "replay_summary",
        tags = {"report", "replay", "recovery"}
    }
)

print("  Generated replay report")

-- 9. Compare original vs recovery sessions
print("\n9. Session comparison:")

local original_artifacts = #Artifact.list(original_session)
local recovery_artifacts = #Artifact.list(recovery_session)

print("  Original session artifacts: " .. original_artifacts)
print("  Recovery session artifacts: " .. recovery_artifacts)
print("  Recovery efficiency: " .. 
    string.format("%.1f%%", (recovery_artifacts / original_artifacts) * 100))

-- Complete both sessions
Session.complete(original_session)
Session.complete(recovery_session)

print("\n✓ Session replay example completed!")

-- Summary
print("\n=== Summary ===")
print("This example demonstrated:")
print("  • Creating checkpoints during processing")
print("  • Detecting and handling failures")
print("  • Checking replay capabilities")
print("  • Analyzing session timelines")
print("  • Creating recovery sessions")
print("  • Replaying from checkpoints")
print("  • Generating replay reports")
print("\nUse replay capabilities for:")
print("  • Debugging failed workflows")
print("  • Recovery from errors")
print("  • Audit trails")
print("  • Performance analysis")
print("  • Testing and validation")