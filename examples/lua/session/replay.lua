-- ABOUTME: Example demonstrating session replay and recovery capabilities
-- ABOUTME: Shows how to replay sessions, recover from failures, and audit history

-- CONFIG: Requires runtime integration (see README.md for current status)
-- WHY: Session replay enables debugging, auditing, and recovery from failures
-- STATUS: Session/Artifact globals implemented but not yet integrated into CLI runtime
-- TODO: Runtime needs to initialize SessionManager - see llmspell-bridge/src/runtime.rs

print("🔄 Session Replay and Recovery Example")
print("======================================")

-- This example demonstrates:
-- 1. Creating a session with multiple operations
-- 2. Saving session state at checkpoints
-- 3. Simulating a failure scenario
-- 4. Loading and replaying from a checkpoint
-- 5. Inspecting session history
-- 6. Recovery strategies

-- Helper to simulate a processing step
local function process_step(session_id, step_name, data)
    -- Store the input as an artifact
    local input_id = Artifact.store(
        session_id,
        "user_input",
        step_name .. "_input.json",
        JSON.stringify(data),
        {
            mime_type = "application/json",
            step = step_name,
            timestamp = os.time()
        }
    )
    
    -- Simulate processing
    local result = {
        step = step_name,
        input_data = data,
        processed_at = os.date("!%Y-%m-%dT%H:%M:%SZ"),
        success = true
    }
    
    -- Store the result
    local result_id = Artifact.store(
        session_id,
        "tool_result",
        step_name .. "_result.json",
        JSON.stringify(result),
        {
            mime_type = "application/json",
            step = step_name
        }
    )
    
    return result
end

-- Step 1: Create a multi-step processing session
print("\n1. Creating Multi-Step Processing Session")
print(string.rep("-", 40))

local session_id = Session.create({
    name = "Data Processing Pipeline",
    description = "Multi-step data transformation with checkpoints",
    tags = {"pipeline", "replay-demo"},
    metadata = {
        pipeline_version = "1.0",
        total_steps = 5
    }
})
print("✅ Created session:", session_id)

-- Step 2: Execute initial processing steps
print("\n2. Executing Processing Steps")
print(string.rep("-", 40))

-- Process step 1
print("🔧 Processing step 1: Data validation")
local step1_result = process_step(session_id, "validation", {
    records = 1000,
    source = "raw_data.csv"
})
print("  ✓ Validation complete:", step1_result.success)

-- Process step 2
print("🔧 Processing step 2: Data cleaning")
local step2_result = process_step(session_id, "cleaning", {
    records = 950,
    removed = 50,
    reason = "invalid_format"
})
print("  ✓ Cleaning complete:", step2_result.success)

-- Step 3: Create checkpoint
print("\n3. Creating Checkpoint")
print(string.rep("-", 40))
print("💾 Saving session state...")
Session.save(session_id)
print("✅ Checkpoint created")

-- Get current artifact count
local artifacts_before = Artifact.list(session_id)
print("  Artifacts saved:", #artifacts_before)

-- Step 4: Continue processing
print("\n4. Continuing Processing")
print(string.rep("-", 40))

-- Process step 3
print("🔧 Processing step 3: Data enrichment")
local step3_result = process_step(session_id, "enrichment", {
    records = 950,
    enriched_fields = {"category", "region"}
})
print("  ✓ Enrichment complete:", step3_result.success)

-- Step 5: Simulate failure
print("\n5. Simulating System Failure")
print(string.rep("-", 40))
print("💥 Critical error in step 4: transformation")
print("  Error: Out of memory during large dataset join")

-- Suspend the session due to failure
Session.suspend(session_id)
print("⏸️  Session suspended due to error")

-- Step 6: Recovery from checkpoint
print("\n6. Recovery from Checkpoint")
print(string.rep("-", 40))
print("🔄 Loading last known good state...")

-- In a real scenario, we might have lost the session_id
-- We can find it by listing sessions with our tags
local sessions = Session.list({
    tags = {"pipeline", "replay-demo"},
    status = "suspended"
})

if #sessions > 0 then
    local recovered_session = sessions[1]
    print("✅ Found suspended session:", recovered_session.id)
    print("  Name:", recovered_session.name)
    print("  Status:", recovered_session.status)
    
    -- Resume the session
    Session.resume(recovered_session.id)
    print("▶️  Session resumed")
    
    -- Check what artifacts we have
    local artifacts = Artifact.list(recovered_session.id)
    print("\n📋 Recovered artifacts:", #artifacts)
    for i, artifact in ipairs(artifacts) do
        if artifact.metadata and artifact.metadata.custom and artifact.metadata.custom.step then
            print(string.format("  - Step: %s, Type: %s", 
                artifact.metadata.custom.step, 
                artifact.artifact_type))
        end
    end
end

-- Step 7: Inspect session history
print("\n7. Session History Analysis")
print(string.rep("-", 40))

-- Get all artifacts and analyze the pipeline progress
local all_artifacts = Artifact.list(session_id)
local steps_completed = {}
for _, artifact in ipairs(all_artifacts) do
    if artifact.metadata and artifact.metadata.custom and artifact.metadata.custom.step then
        steps_completed[artifact.metadata.custom.step] = true
    end
end

print("📊 Pipeline progress:")
local pipeline_steps = {"validation", "cleaning", "enrichment", "transformation", "output"}
for i, step in ipairs(pipeline_steps) do
    local status = steps_completed[step] and "✓ Complete" or "⏳ Pending"
    print(string.format("  %d. %s: %s", i, step, status))
end

-- Step 8: Recovery strategies
print("\n8. Recovery Strategies")
print(string.rep("-", 40))

print("🛠️  Available recovery options:")
print("  1. Retry failed step with reduced batch size")
print("  2. Skip failed step and continue")
print("  3. Rollback to previous checkpoint")
print("  4. Create new branch from checkpoint")

-- Demonstrate option 1: Retry with smaller batch
print("\n📌 Implementing recovery option 1...")
print("🔧 Retrying step 4 with smaller batch size")

-- Process step 4 with smaller batch
local step4_result = process_step(session_id, "transformation_batch1", {
    records = 475,  -- Half the records
    batch = 1,
    total_batches = 2
})
print("  ✓ Batch 1 complete:", step4_result.success)

local step4_result2 = process_step(session_id, "transformation_batch2", {
    records = 475,
    batch = 2,
    total_batches = 2
})
print("  ✓ Batch 2 complete:", step4_result2.success)

-- Step 9: Complete the pipeline
print("\n9. Completing Pipeline")
print(string.rep("-", 40))

-- Process final step
print("🔧 Processing step 5: Output generation")
local step5_result = process_step(session_id, "output", {
    total_records = 950,
    output_format = "parquet",
    compression = "snappy"
})
print("  ✓ Output complete:", step5_result.success)

-- Save final state
Session.save(session_id)
print("💾 Final state saved")

-- Complete the session
Session.complete(session_id)
print("✅ Pipeline completed successfully")

-- Summary
print("\n\n🎉 Session Replay Example Completed!")
print("====================================")
print("\nDemonstrated capabilities:")
print("  ✓ Multi-step session with checkpoints")
print("  ✓ Session save/load for persistence")
print("  ✓ Failure simulation and recovery")
print("  ✓ Session querying and discovery")
print("  ✓ History inspection and analysis")
print("  ✓ Multiple recovery strategies")
print("\nKey takeaways:")
print("  • Save sessions at critical checkpoints")
print("  • Use tags and metadata for session discovery")
print("  • Artifacts provide audit trail of operations")
print("  • Suspended sessions can be resumed later")
print("  • Recovery strategies depend on failure type")