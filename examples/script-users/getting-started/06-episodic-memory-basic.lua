-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Example ID: 06 - Episodic Memory Basics v0.13.0
-- Complexity Level: BEGINNER
-- Real-World Use Case: Building conversational AI with memory
--
-- Purpose: Learn how to use episodic memory for conversation tracking.
--          Demonstrates basic memory operations: adding exchanges, searching
--          by relevance, and monitoring memory statistics.
--          This is your gateway to building AI with context awareness.
-- Architecture: Memory subsystem with vector search, session isolation
-- Crates Showcased: llmspell-memory, llmspell-bridge
-- Key Features:
--   • Adding conversation exchanges to memory
--   • Searching memory by semantic relevance
--   • Session-based memory isolation
--   • Memory statistics and monitoring
--   • Metadata tagging for better organization
--
-- Prerequisites:
--   • LLMSpell installed and built with memory features
--   • API key: OPENAI_API_KEY environment variable (for embeddings)
--   • Network connectivity for API calls
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/getting-started/06-episodic-memory-basic.lua
--
-- EXPECTED OUTPUT:
-- Memory system initialized
-- Conversation exchanges added to session
-- Search results with relevance ranking
-- Memory statistics summary
--
-- Time to Complete: <10 seconds
-- ============================================================

print("=== LLMSpell: Your First Memory-Enhanced Conversation ===")
print("Example 06: BEGINNER - Episodic Memory Operations")
print("Showcasing: Conversation tracking and memory search\n")

-- ============================================================
-- Step 1: Check Memory Availability
-- ============================================================

print("1. Checking Memory availability...")
if not Memory then
    print("❌ Memory is not available. Please check your configuration.")
    print("   Ensure you're using a memory-enabled build.")
    print("   Build with: cargo build --all-features")
    return {
        success = false,
        error = "Memory not configured"
    }
end

print("✅ Memory is available")

-- Check available subsystems
local memory_subsystems = {}
if Memory.episodic then
    table.insert(memory_subsystems, "episodic")
end
if Memory.semantic then
    table.insert(memory_subsystems, "semantic")
end
if Memory.stats then
    table.insert(memory_subsystems, "stats")
end

print("   Available subsystems: " .. table.concat(memory_subsystems, ", "))
print()

-- ============================================================
-- Step 2: Create a Conversation Session
-- ============================================================

print("2. Creating conversation session...")

-- Create unique session ID with timestamp
local session_id = "demo-session-" .. os.time()
print("   Session ID: " .. session_id)
print("   (Session IDs help isolate different conversations)")

print()

-- ============================================================
-- Step 3: Add Conversation to Memory
-- ============================================================

print("3. Adding conversation exchanges to memory...")

-- Add a series of exchanges about Rust programming
local exchanges = {
    {
        role = "user",
        content = "What is Rust?",
        metadata = {topic = "programming", subtopic = "rust-intro"}
    },
    {
        role = "assistant",
        content = "Rust is a systems programming language focused on safety and performance. It guarantees memory safety without garbage collection through its unique ownership system.",
        metadata = {topic = "programming", subtopic = "rust-intro"}
    },
    {
        role = "user",
        content = "Tell me about ownership in Rust",
        metadata = {topic = "programming", subtopic = "rust-ownership"}
    },
    {
        role = "assistant",
        content = "Ownership is Rust's unique approach to memory management. Each value has a single owner, and when the owner goes out of scope, the value is dropped. This prevents memory leaks and data races at compile time.",
        metadata = {topic = "programming", subtopic = "rust-ownership"}
    },
    {
        role = "user",
        content = "What about borrowing?",
        metadata = {topic = "programming", subtopic = "rust-borrowing"}
    },
    {
        role = "assistant",
        content = "Borrowing allows you to reference data without taking ownership. You can have multiple immutable borrows OR one mutable borrow, but not both. This prevents data races at compile time.",
        metadata = {topic = "programming", subtopic = "rust-borrowing"}
    }
}

-- Add each exchange to episodic memory
local added_count = 0
for i, exchange in ipairs(exchanges) do
    local success, result = pcall(Memory.episodic.add,
        session_id,
        exchange.role,
        exchange.content,
        exchange.metadata
    )

    if success then
        added_count = added_count + 1
        print(string.format("   ✓ Exchange %d: %s (id: %s)", i, exchange.role, result:sub(1, 8)))
    else
        print(string.format("   ✗ Exchange %d failed: %s", i, tostring(result)))
    end
end

print(string.format("\n💾 Added %d/%d exchanges to memory", added_count, #exchanges))

if added_count == 0 then
    print("❌ No exchanges were added. Cannot continue.")
    return {
        success = false,
        error = "Memory addition failed"
    }
end

print()

-- ============================================================
-- Step 4: Search Memory by Relevance
-- ============================================================

print("4. Searching memory...")

-- Test different search queries
local queries = {
    "ownership",
    "How does Rust prevent memory leaks?",
    "borrowing rules",
    "What is Rust good for?"
}

for i, query in ipairs(queries) do
    print(string.format("\n🔍 Query %d: '%s'", i, query))

    -- Search episodic memory (session-specific)
    local success, entries = pcall(Memory.episodic.search,
        session_id,  -- session filter
        query,       -- search query
        10           -- limit
    )

    if success and entries then
        if #entries > 0 then
            print(string.format("   Found %d relevant entries:", #entries))

            for j, entry in ipairs(entries) do
                -- Show first 80 chars of content
                local snippet = string.sub(entry.content, 1, 80)
                if #entry.content > 80 then
                    snippet = snippet .. "..."
                end

                print(string.format("   %d. [%s] %s",
                    j,
                    entry.role,
                    snippet
                ))

                -- Show metadata if present
                if entry.metadata and entry.metadata.subtopic then
                    print(string.format("      📌 %s", entry.metadata.subtopic))
                end
            end
        else
            print("   No relevant entries found")
        end
    else
        print("   ✗ Search error: " .. tostring(entries))
    end
end

print()

-- ============================================================
-- Step 5: Memory Statistics
-- ============================================================

print("5. Memory statistics...")

local stats = Memory.stats()
if stats then
    print("📊 Current memory state:")
    print("   Episodic entries: " .. (stats.episodic_count or 0))
    print("   Semantic entries: " .. (stats.semantic_count or 0))

    -- Show consolidation status if available
    if stats.consolidation_status then
        print("   Consolidation: " .. stats.consolidation_status)
    end

    -- Show session info if available
    if stats.sessions_with_unprocessed then
        print("   Sessions pending consolidation: " .. stats.sessions_with_unprocessed)
    end

    -- Show any additional stats
    for key, value in pairs(stats) do
        if key ~= "episodic_count" and
           key ~= "semantic_count" and
           key ~= "consolidation_status" and
           key ~= "sessions_with_unprocessed" then
            print(string.format("   %s: %s", key, tostring(value)))
        end
    end
else
    print("   ⚠️ Statistics not available")
end

print()

-- ============================================================
-- Step 6: Cross-Session Search (Optional)
-- ============================================================

print("6. Demonstrating cross-session search...")

-- Search WITHOUT session filter to show all memories
local success, global_entries = pcall(Memory.episodic.search,
    "",             -- Empty session = search all sessions
    "programming",  -- query
    5               -- limit
)

if success and global_entries then
    print(string.format("   Found %d entries across all sessions", #global_entries))
    print("   (This demonstrates that memory persists across sessions)")
else
    print("   ⚠️ Global search not available or returned no results")
end

print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Congratulations! You've successfully:")
print("   ✓ Initialized the Memory system")
print("   ✓ Created a conversation session with unique ID")
print("   ✓ Added " .. added_count .. " exchanges to episodic memory")
print("   ✓ Performed semantic searches with relevance ranking")
print("   ✓ Retrieved conversation context using natural language queries")
print("   ✓ Viewed memory statistics and monitoring data")
print()
print("🚀 Next Steps:")
print("   • Try cookbook/memory-session-isolation.lua for multi-session patterns")
print("   • Explore semantic memory for knowledge graph storage")
print("   • Learn about memory consolidation for long-term knowledge")
print("   • Integrate memory with agents for context-aware conversations")
print()
print("Next: Check out cookbook/memory-session-isolation.lua!")

-- Return success with stats
return {
    success = true,
    message = "Episodic memory example completed",
    stats = {
        exchanges_added = added_count,
        searches_performed = #queries,
        session_id = session_id
    }
}
