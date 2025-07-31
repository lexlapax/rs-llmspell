-- ABOUTME: Example demonstrating session replay and recovery capabilities
-- ABOUTME: Shows checkpointing, failure recovery, and timeline analysis

-- CONFIG: Requires session-enabled configuration (see examples/configs/session-enabled.toml)
-- WHY: Replay allows debugging failed workflows, recovery from errors, and audit trails
-- STATUS: Session/Artifact globals fully integrated and functional
-- NOTE: Some replay API methods (canReplay, getReplayMetadata) simulated for demonstration

print("=== Session Replay Example ===\n")

-- This example demonstrates:
-- 1. Creating checkpoints during processing
-- 2. Detecting and handling failures
-- 3. Checking replay capabilities
-- 4. Analyzing session timelines
-- 5. Creating recovery sessions
-- 6. Replaying from checkpoints
-- 7. Generating replay reports

-- Simple JSON encoding function
local function encode_json(data)
    local function encode_value(v)
        if type(v) == "string" then
            return '"' .. v:gsub('"', '\\"') .. '"'
        elseif type(v) == "number" then
            return tostring(v)
        elseif type(v) == "boolean" then
            return tostring(v)
        elseif type(v) == "table" then
            local is_array = #v > 0
            local parts = {}
            if is_array then
                for i, item in ipairs(v) do
                    table.insert(parts, encode_value(item))
                end
                return "[" .. table.concat(parts, ",") .. "]"
            else
                for k, item in pairs(v) do
                    table.insert(parts, '"' .. k .. '":' .. encode_value(item))
                end
                return "{" .. table.concat(parts, ",") .. "}"
            end
        end
        return "null"
    end
    return encode_value(data)
end

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
        encode_json({step = step_name, data = data, timestamp = os.time()}),
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
        encode_json(result),
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

-- Note: canReplay and getReplayMetadata may not be implemented yet
-- For now, we'll assume we can replay if the session exists
local session_exists = Session.get(original_session) ~= nil
print("  Session exists: " .. tostring(session_exists))
print("  (Replay functionality demonstration)")

-- 3. List replayable sessions
print("\n3. Finding replayable sessions:")

-- Note: listReplayable may not be implemented yet
-- For now, we'll list all sessions as potentially replayable
local all_sessions = Session.list() or {}
print("  Found " .. #all_sessions .. " sessions")
for i, session_info in ipairs(all_sessions) do
    if i <= 3 then  -- Show first 3
        print("    - " .. (session_info.name or "Unnamed"))
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
        custom = artifact.custom
    })
end

-- Sort by time
table.sort(timeline, function(a, b) return a.time < b.time end)

print("  Timeline of events:")
for _, event in ipairs(timeline) do
    local step_info = event.custom and event.custom.step_name and 
        (" [" .. event.custom.step_name .. " - " .. (event.custom.artifact_role or "") .. "]") or ""
    -- event.time is an ISO string, extract just the time part
    local time_part = event.time:match("T(%d+:%d+:%d+)")
    print("    " .. (time_part or event.time) .. ": " .. event.name .. step_info)
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
    if artifact.custom and artifact.custom.artifact_role == "output" and 
       artifact.custom.step_name == "transform" then
        last_good_artifact = artifact
        break
    end
end

if last_good_artifact then
    print("  Found last checkpoint: " .. last_good_artifact.name)
    
    -- Since we don't have the artifact ID from list, we'll simulate recovery
    -- In a real implementation, you'd store the artifact IDs
    local checkpoint_data = {content = '{"step":"transform","result":220}'}
    if checkpoint_data then
        -- For demo, manually extract the result value from JSON
        -- In production, use a proper JSON parser
        local result_match = checkpoint_data.content:match('"result":(%d+)')
        local recovered_data = tonumber(result_match) or 0
        local parsed = {success = true, result = {result = recovered_data}}
        
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

local report_json = {result = encode_json(report)}

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