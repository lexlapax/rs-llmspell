-- Profile: memory (recommended)
-- Run with: llmspell -p memory run memory-session-isolation.lua
-- Adaptive memory system

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: cookbook
-- Recipe: Memory Session Isolation
-- Complexity: INTERMEDIATE
-- Real-World Use Case: Multi-user systems, parallel conversations
--
-- Purpose: Demonstrates how to manage multiple independent conversation
--          sessions with proper isolation. Essential for multi-user apps,
--          parallel workflows, or A/B testing scenarios.
-- Pattern: Session-based data isolation with cross-session querying
-- Crates Showcased: llmspell-memory, llmspell-bridge
-- Key Concepts:
--   • Creating isolated conversation sessions
--   • Session-specific memory queries
--   • Cross-session search capabilities
--   • Metadata-based filtering
--   • Session lifecycle management
--
-- Prerequisites:
--   • Completed getting-started/06-episodic-memory-basic.lua
--   • Understanding of session IDs and memory scoping
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/cookbook/memory-session-isolation.lua
--
-- EXPECTED OUTPUT:
-- Two independent sessions created
-- Each session maintains separate conversation history
-- Cross-session queries demonstrate data isolation
-- Session statistics showing independent memory stores
--
-- Time to Complete: <10 seconds
-- ============================================================

print("=== Memory Session Isolation Pattern ===")
print("Recipe: Managing multiple independent conversations\n")

-- ============================================================
-- Setup: Verify Memory Availability
-- ============================================================

if not Memory or not Memory.episodic then
    print("❌ Episodic memory not available")
    return {success = false, error = "Memory not configured"}
end

print("✅ Memory system ready\n")

-- ============================================================
-- Step 1: Create Two Independent Sessions
-- ============================================================

print("1. Creating two independent sessions...")

-- Session A: Project planning conversation
local session_alpha = "project-alpha-" .. os.time()
print("   Session Alpha (Project Planning): " .. session_alpha)

-- Session B: Technical support conversation
local session_beta = "project-beta-" .. os.time() + 1  -- +1 to ensure unique
print("   Session Beta (Technical Support): " .. session_beta)

print()

-- ============================================================
-- Step 2: Add Data to Session Alpha
-- ============================================================

print("2. Populating Session Alpha with project planning data...")

local alpha_exchanges = {
    {role = "user", content = "Let's plan the new dashboard feature", metadata = {project = "alpha", phase = "planning"}},
    {role = "assistant", content = "Great! Let's start with the user requirements. What are the key metrics?", metadata = {project = "alpha", phase = "planning"}},
    {role = "user", content = "We need real-time analytics and export capabilities", metadata = {project = "alpha", phase = "requirements"}},
    {role = "assistant", content = "Real-time analytics will require WebSocket connections. For exports, we should support CSV and JSON formats.", metadata = {project = "alpha", phase = "requirements"}},
}

local alpha_added = 0
for i, exchange in ipairs(alpha_exchanges) do
    local result = Memory.episodic.add(
        session_alpha,
        exchange.role,
        exchange.content,
        exchange.metadata
    )
    if result then
        alpha_added = alpha_added + 1
        print(string.format("   ✓ Added exchange %d to Alpha", i))
    end
end

print(string.format("   📝 Alpha: %d exchanges added\n", alpha_added))

-- ============================================================
-- Step 3: Add Data to Session Beta
-- ============================================================

print("3. Populating Session Beta with technical support data...")

local beta_exchanges = {
    {role = "user", content = "The authentication service is failing", metadata = {project = "beta", category = "bug"}},
    {role = "assistant", content = "Let's troubleshoot. Are you seeing timeout errors or authentication failures?", metadata = {project = "beta", category = "bug"}},
    {role = "user", content = "Timeout errors after 30 seconds", metadata = {project = "beta", category = "bug"}},
    {role = "assistant", content = "That suggests database connection issues. Check the connection pool settings and DB health.", metadata = {project = "beta", category = "bug"}},
}

local beta_added = 0
for i, exchange in ipairs(beta_exchanges) do
    local result = Memory.episodic.add(
        session_beta,
        exchange.role,
        exchange.content,
        exchange.metadata
    )
    if result then
        beta_added = beta_added + 1
        print(string.format("   ✓ Added exchange %d to Beta", i))
    end
end

print(string.format("   📝 Beta: %d exchanges added\n", beta_added))

-- ============================================================
-- Step 4: Query Session Alpha (Isolated)
-- ============================================================

print("4. Querying Session Alpha only...")

local alpha_query = "analytics and exports"
print(string.format("   Query: '%s'", alpha_query))

local alpha_results = Memory.episodic.search(
    session_alpha,  -- Session filter for Alpha only
    alpha_query,
    10
)

if alpha_results and type(alpha_results) == "table" and #alpha_results > 0 then
    print(string.format("   Found %d results in Alpha:", #alpha_results))
    for i, entry in ipairs(alpha_results) do
        local snippet = string.sub(entry.content, 1, 60)
        if #entry.content > 60 then snippet = snippet .. "..." end
        print(string.format("   %d. [%s] %s", i, entry.role, snippet))
    end

    -- Verify no Beta data leaked in
    local has_beta_data = false
    for _, entry in ipairs(alpha_results) do
        if entry.metadata and entry.metadata.project == "beta" then
            has_beta_data = true
            break
        end
    end

    if has_beta_data then
        print("   ⚠️ WARNING: Beta data found in Alpha results!")
    else
        print("   ✓ Verified: Only Alpha data returned")
    end
else
    print("   No results found")
end

print()

-- ============================================================
-- Step 5: Query Session Beta (Isolated)
-- ============================================================

print("5. Querying Session Beta only...")

local beta_query = "authentication timeout"
print(string.format("   Query: '%s'", beta_query))

local beta_results = Memory.episodic.search(
    session_beta,  -- Session filter for Beta only
    beta_query,
    10
)

if beta_results and type(beta_results) == "table" and #beta_results > 0 then
    print(string.format("   Found %d results in Beta:", #beta_results))
    for i, entry in ipairs(beta_results) do
        local snippet = string.sub(entry.content, 1, 60)
        if #entry.content > 60 then snippet = snippet .. "..." end
        print(string.format("   %d. [%s] %s", i, entry.role, snippet))
    end

    -- Verify no Alpha data leaked in
    local has_alpha_data = false
    for _, entry in ipairs(beta_results) do
        if entry.metadata and entry.metadata.project == "alpha" then
            has_alpha_data = true
            break
        end
    end

    if has_alpha_data then
        print("   ⚠️ WARNING: Alpha data found in Beta results!")
    else
        print("   ✓ Verified: Only Beta data returned")
    end
else
    print("   No results found")
end

print()

-- ============================================================
-- Step 6: Cross-Session Query (Global Search)
-- ============================================================

print("6. Performing cross-session search...")

local global_query = "project"
print(string.format("   Query: '%s' (no session filter)", global_query))

local global_results = Memory.episodic.search(
    "",  -- Empty string = search all sessions
    global_query,
    20
)

if global_results and type(global_results) == "table" and #global_results > 0 then
    print(string.format("   Found %d results across all sessions:", #global_results))

    -- Count results by session
    local alpha_count = 0
    local beta_count = 0
    local other_count = 0

    for _, entry in ipairs(global_results) do
        if entry.metadata and entry.metadata.project == "alpha" then
            alpha_count = alpha_count + 1
        elseif entry.metadata and entry.metadata.project == "beta" then
            beta_count = beta_count + 1
        else
            other_count = other_count + 1
        end
    end

    print(string.format("   Alpha results: %d", alpha_count))
    print(string.format("   Beta results: %d", beta_count))
    if other_count > 0 then
        print(string.format("   Other results: %d", other_count))
    end
else
    print("   No results found")
end

print()

-- ============================================================
-- Step 7: Verify Isolation with Stats
-- ============================================================

print("7. Memory statistics...")

local stats = Memory.stats()
if stats then
    print("📊 Overall memory state:")
    print(string.format("   Total episodic entries: %d", stats.episodic_count or 0))
    print(string.format("   Expected from this script: %d", alpha_added + beta_added))
    print(string.format("   (Note: May include entries from previous runs)"))

    if stats.sessions_with_unprocessed then
        print(string.format("   Sessions tracked: %d", stats.sessions_with_unprocessed))
    end
end

print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Session Isolation Pattern Demonstrated!")
print("\n✓ Key Learnings:")
print("   • Each session maintains independent memory")
print("   • Session IDs provide automatic data isolation")
print("   • Session-specific queries only return relevant data")
print("   • Cross-session queries can access all memories")
print("   • Metadata provides additional filtering capabilities")
print("\n📚 Use Cases:")
print("   • Multi-user chat applications")
print("   • Parallel workflow execution")
print("   • A/B testing with isolated contexts")
print("   • Multi-tenant systems with data privacy")
print("   • Conversation branching and forking")
print("\n🚀 Next Steps:")
print("   • Try features/memory-stats.lua for monitoring")
print("   • Explore consolidation patterns for long-term memory")
print("   • Integrate with Context global for hybrid retrieval")

-- Return summary
return {
    success = true,
    message = "Session isolation pattern completed",
    stats = {
        session_alpha = {
            id = session_alpha,
            exchanges = alpha_added
        },
        session_beta = {
            id = session_beta,
            exchanges = beta_added
        },
        isolation_verified = true
    }
}
