-- ============================================================
-- LLMSPELL FEATURES - MEMORY MONITORING
-- ============================================================
-- Feature: Memory Statistics and Monitoring
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Production monitoring, capacity planning
--
-- Purpose: Learn how to monitor memory system health, track growth,
--          and understand consolidation status. Essential for production
--          deployments and understanding memory usage patterns.
-- Pattern: Observability for memory subsystems
-- Crates Showcased: llmspell-memory, llmspell-bridge
-- Key Features:
--   • Real-time memory statistics
--   • Memory growth tracking
--   • Consolidation monitoring
--   • Session activity tracking
--   • Capacity planning metrics
--
-- Prerequisites:
--   • Completed getting-started/06-episodic-memory-basic.lua
--   • Basic understanding of episodic and semantic memory
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/features/memory-stats.lua
--
-- EXPECTED OUTPUT:
-- Initial memory state snapshot
-- Memory growth metrics after data additions
-- Consolidation status and pending work
-- Session activity summary
--
-- Time to Complete: <10 seconds
-- ============================================================

print("=== Memory Statistics & Monitoring ===")
print("Feature: Tracking memory system health and usage\n")

-- ============================================================
-- Setup: Verify Memory Availability
-- ============================================================

if not Memory then
    print("❌ Memory system not available")
    return {success = false, error = "Memory not configured"}
end

print("✅ Memory system available\n")

-- ============================================================
-- Step 1: Capture Initial State
-- ============================================================

print("1. Capturing initial memory state...")

local stats_before = Memory.stats()
if stats_before then
    print("📊 Initial state:")
    print(string.format("   Episodic entries: %d", stats_before.episodic_count or 0))
    print(string.format("   Semantic entries: %d", stats_before.semantic_count or 0))

    if stats_before.sessions_with_unprocessed then
        print(string.format("   Sessions with unprocessed data: %d",
            stats_before.sessions_with_unprocessed))
    end

    if stats_before.consolidation_status then
        print(string.format("   Consolidation status: %s",
            stats_before.consolidation_status))
    end

    -- Capture all available metrics
    print("\n   All available metrics:")
    for key, value in pairs(stats_before) do
        print(string.format("   • %s: %s", key, tostring(value)))
    end
else
    print("   ⚠️ Could not retrieve statistics")
    stats_before = {episodic_count = 0, semantic_count = 0}
end

print()

-- ============================================================
-- Step 2: Add Test Data to Track Growth
-- ============================================================

print("2. Adding test data to track memory growth...")

local session = "stats-demo-" .. os.time()
local exchanges_to_add = 20

print(string.format("   Creating session with %d exchanges...", exchanges_to_add))

local added_count = 0
for i = 1, exchanges_to_add do
    local user_result = Memory.episodic.add(
        session,
        "user",
        string.format("Test query number %d about memory statistics", i),
        {test = true, iteration = i}
    )

    local assistant_result = Memory.episodic.add(
        session,
        "assistant",
        string.format("Test response number %d explaining memory monitoring", i),
        {test = true, iteration = i}
    )

    if user_result and user_result.success and
       assistant_result and assistant_result.success then
        added_count = added_count + 2
    end
end

print(string.format("   ✓ Added %d exchanges", added_count))

print()

-- ============================================================
-- Step 3: Capture After State
-- ============================================================

print("3. Capturing memory state after additions...")

local stats_after = Memory.stats()
if stats_after then
    print("📊 After additions:")
    print(string.format("   Episodic entries: %d", stats_after.episodic_count or 0))
    print(string.format("   Semantic entries: %d", stats_after.semantic_count or 0))

    -- Calculate growth
    local episodic_growth = (stats_after.episodic_count or 0) - (stats_before.episodic_count or 0)
    local semantic_growth = (stats_after.semantic_count or 0) - (stats_before.semantic_count or 0)

    print("\n📈 Growth metrics:")
    print(string.format("   Episodic entries added: +%d", episodic_growth))
    print(string.format("   Semantic entries added: +%d", semantic_growth))
    print(string.format("   Expected additions: %d", added_count))

    if episodic_growth == added_count then
        print("   ✓ Growth matches expected additions")
    else
        print("   ⚠️ Growth differs from expected (may include other sessions)")
    end
else
    print("   ⚠️ Could not retrieve statistics")
    stats_after = {episodic_count = 0, semantic_count = 0}
end

print()

-- ============================================================
-- Step 4: Monitor Consolidation Status
-- ============================================================

print("4. Monitoring consolidation status...")

if stats_after.consolidation_status then
    print(string.format("   Status: %s", stats_after.consolidation_status))

    -- Check if consolidation is needed
    if stats_after.sessions_with_unprocessed and
       stats_after.sessions_with_unprocessed > 0 then
        print(string.format("   ⚠️ %d sessions have unprocessed data",
            stats_after.sessions_with_unprocessed))
        print("   Consider running consolidation to extract knowledge")
    else
        print("   ✓ No sessions pending consolidation")
    end

    -- Show last consolidation time if available
    if stats_after.last_consolidation then
        print(string.format("   Last consolidation: %s", stats_after.last_consolidation))
    end

    -- Show pending count if available
    if stats_after.pending_consolidation_count then
        print(string.format("   Entries pending consolidation: %d",
            stats_after.pending_consolidation_count))
    end
else
    print("   ℹ️ Consolidation status not available")
end

print()

-- ============================================================
-- Step 5: Session Activity Tracking
-- ============================================================

print("5. Session activity tracking...")

-- Search for our test session to verify it exists
local session_check = Memory.episodic.search(
    "memory statistics",
    5,
    session
)

if session_check and session_check.success and session_check.entries then
    local entry_count = #session_check.entries

    print(string.format("   Session '%s' found", session:sub(1, 30) .. "..."))
    print(string.format("   Entries in session: %d", entry_count))
    print("   ✓ Session is active and queryable")

    -- Show sample entry
    if entry_count > 0 then
        local sample = session_check.entries[1]
        local snippet = string.sub(sample.content, 1, 60)
        if #sample.content > 60 then snippet = snippet .. "..." end
        print(string.format("   Sample: [%s] %s", sample.role, snippet))
    end
else
    print("   ⚠️ Could not verify session activity")
end

print()

-- ============================================================
-- Step 6: Capacity Planning Insights
-- ============================================================

print("6. Capacity planning insights...")

-- Calculate memory per entry (rough estimate)
local total_entries = (stats_after.episodic_count or 0) + (stats_after.semantic_count or 0)

if total_entries > 0 then
    print("📉 Usage patterns:")
    print(string.format("   Total memory entries: %d", total_entries))

    -- Calculate ratio
    local episodic_pct = 0
    if total_entries > 0 then
        episodic_pct = ((stats_after.episodic_count or 0) / total_entries) * 100
    end
    print(string.format("   Episodic: %.1f%%", episodic_pct))
    print(string.format("   Semantic: %.1f%%", 100 - episodic_pct))

    -- Growth rate per session
    if episodic_growth > 0 then
        print(string.format("\n   Average entries per exchange: %.1f",
            episodic_growth / exchanges_to_add))
    end

    -- Rough capacity estimates
    print("\n📊 Estimated capacity (at current rate):")
    print("   1,000 exchanges → ~1,000 episodic entries")
    print("   10,000 exchanges → ~10,000 episodic entries")
    print("   (Actual usage depends on consolidation settings)")
else
    print("   Insufficient data for capacity planning")
end

print()

-- ============================================================
-- Step 7: Health Check Summary
-- ============================================================

print("7. Memory system health check...")

local health = {
    episodic_available = (stats_after.episodic_count ~= nil),
    semantic_available = (stats_after.semantic_count ~= nil),
    stats_available = (stats_after ~= nil),
    growth_verified = (episodic_growth > 0),
    consolidation_tracked = (stats_after.consolidation_status ~= nil)
}

print("🏥 Health indicators:")
for key, value in pairs(health) do
    local status = value and "✓" or "✗"
    print(string.format("   %s %s", status, key))
end

local health_score = 0
for _, value in pairs(health) do
    if value then health_score = health_score + 1 end
end

local total_checks = 5
local health_pct = (health_score / total_checks) * 100

print(string.format("\n   Overall health: %.0f%% (%d/%d checks passed)",
    health_pct, health_score, total_checks))

if health_score == total_checks then
    print("   ✓ Memory system is fully operational")
elseif health_score >= 3 then
    print("   ⚠️ Memory system has minor issues")
else
    print("   ❌ Memory system has significant issues")
end

print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Memory Monitoring Completed!")
print("\n✓ Key Metrics Tracked:")
print("   • Episodic memory growth")
print("   • Semantic memory status")
print("   • Consolidation progress")
print("   • Session activity")
print("   • System health indicators")
print("\n📚 Monitoring Best Practices:")
print("   • Track stats before/after operations")
print("   • Monitor consolidation regularly")
print("   • Watch for unprocessed sessions")
print("   • Plan capacity based on growth rates")
print("   • Set up alerts for health checks")
print("\n🚀 Production Recommendations:")
print("   • Log stats periodically (hourly/daily)")
print("   • Set alerts for consolidation backlog")
print("   • Monitor growth trends over time")
print("   • Track query performance metrics")

-- Return monitoring summary
return {
    success = true,
    message = "Memory monitoring completed",
    metrics = {
        episodic_before = stats_before.episodic_count or 0,
        episodic_after = stats_after.episodic_count or 0,
        growth = episodic_growth,
        health_score = health_pct,
        session_id = session
    }
}
