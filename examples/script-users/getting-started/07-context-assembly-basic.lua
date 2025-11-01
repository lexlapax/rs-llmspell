-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Example ID: 07 - Context Assembly Basics v0.13.0
-- Complexity Level: BEGINNER
-- Real-World Use Case: Intelligent context retrieval for LLM prompts
--
-- Purpose: Learn how to assemble relevant context from memory for LLM queries.
--          Demonstrates the Memory → Context workflow that transforms raw
--          conversations into ranked, token-budgeted context chunks.
-- Architecture: Context assembly pipeline with episodic retrieval + BM25 reranking
-- Crates Showcased: llmspell-context, llmspell-memory, llmspell-bridge
-- Key Features:
--   • Adding conversation to episodic memory
--   • Assembling context using episodic strategy
--   • Token budget management
--   • Inspecting ranked chunks and metadata
--   • Understanding reranking scores
--
-- Prerequisites:
--   • Completed 06-episodic-memory-basic.lua
--   • Understanding of episodic memory operations
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/getting-started/07-context-assembly-basic.lua
--
-- EXPECTED OUTPUT:
-- Conversation added to memory
-- Context assembled with ranked chunks
-- Token usage and confidence metrics
-- Chunk details with scores
--
-- Time to Complete: <10 seconds
-- ============================================================

print("=== LLMSpell: Your First Context-Aware Query ===")
print("Example 07: BEGINNER - Context Assembly Operations")
print("Showcasing: Memory→Context workflow for LLM prompting\\n")

-- ============================================================
-- Step 1: Check Memory and Context Availability
-- ============================================================

print("1. Checking Memory and Context availability...")
if not Memory then
    print("❌ Memory is not available. Please check your configuration.")
    return {
        success = false,
        error = "Memory not configured"
    }
end

if not Context then
    print("❌ Context is not available. Please check your configuration.")
    return {
        success = false,
        error = "Context not configured"
    }
end

print("✅ Memory and Context are available\\n")

-- ============================================================
-- Step 2: Populate Memory with Conversation
-- ============================================================

print("2. Adding conversation to episodic memory...")

-- Create unique session
local session_id = "context-demo-" .. os.time()
print("   Session ID: " .. session_id)

-- Add a conversation about Rust programming
local exchanges = {
    {
        role = "user",
        content = "What is Rust?",
        metadata = {topic = "programming", subtopic = "rust-intro"}
    },
    {
        role = "assistant",
        content = "Rust is a systems programming language focused on safety, concurrency, and performance. It guarantees memory safety without garbage collection through its unique ownership system.",
        metadata = {topic = "programming", subtopic = "rust-intro"}
    },
    {
        role = "user",
        content = "Tell me about ownership in Rust",
        metadata = {topic = "programming", subtopic = "rust-ownership"}
    },
    {
        role = "assistant",
        content = "Ownership is Rust's memory management approach. Each value has a single owner, and when the owner goes out of scope, the value is dropped. This prevents memory leaks and data races at compile time.",
        metadata = {topic = "programming", subtopic = "rust-ownership"}
    },
    {
        role = "user",
        content = "What about borrowing?",
        metadata = {topic = "programming", subtopic = "rust-borrowing"}
    },
    {
        role = "assistant",
        content = "Borrowing allows temporary access to owned data without transferring ownership. The borrow checker enforces rules: you can have multiple immutable borrows OR one mutable borrow, but not both simultaneously.",
        metadata = {topic = "programming", subtopic = "rust-borrowing"}
    },
    {
        role = "user",
        content = "Can you explain lifetimes?",
        metadata = {topic = "programming", subtopic = "rust-lifetimes"}
    },
    {
        role = "assistant",
        content = "Lifetimes are Rust's way of tracking how long references are valid. They prevent dangling references and use-after-free bugs. The compiler infers most lifetimes, but sometimes you need explicit lifetime annotations.",
        metadata = {topic = "programming", subtopic = "rust-lifetimes"}
    }
}

-- Add exchanges to memory
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
        print(string.format("   ✓ Exchange %d: %s", i, exchange.role))
    else
        print(string.format("   ✗ Exchange %d failed: %s", i, tostring(result)))
    end
end

print(string.format("\\n💾 Added %d/%d exchanges to memory\\n", added_count, #exchanges))

if added_count == 0 then
    print("❌ No exchanges were added. Cannot continue.")
    return {
        success = false,
        error = "Memory addition failed"
    }
end

-- ============================================================
-- Step 3: Assemble Context for a Query
-- ============================================================

print("3. Assembling context for query: 'How does ownership work in Rust?'")

-- Assemble context using episodic strategy
local success, result = pcall(Context.assemble,
    "How does ownership work in Rust?",  -- query
    "episodic",                           -- strategy
    2000,                                 -- token budget
    session_id                            -- session filter
)

if not success then
    print("   ✗ Context assembly failed: " .. tostring(result))
    return {
        success = false,
        error = "Context assembly failed"
    }
end

print("   ✅ Context assembled successfully\\n")

-- ============================================================
-- Step 4: Inspect Assembled Context
-- ============================================================

print("4. Inspecting assembled context...")

print(string.format("📊 Context Metrics:"))
print(string.format("   Chunks retrieved: %d", #result.chunks))
print(string.format("   Token count: %d / 2000", result.token_count))
print(string.format("   Confidence: %.2f%%", result.total_confidence * 100))

-- Show temporal span
if result.temporal_span then
    print(string.format("   Temporal span: %s to %s",
        tostring(result.temporal_span[1] or "unknown"),
        tostring(result.temporal_span[2] or "unknown")))
end

print()

-- ============================================================
-- Step 5: Display Chunk Details
-- ============================================================

print("5. Chunk details (ranked by relevance)...")

if #result.chunks > 0 then
    for i, ranked_chunk in ipairs(result.chunks) do
        print(string.format("\\n📝 Chunk %d:", i))
        print(string.format("   Role: %s", ranked_chunk.chunk.role))
        print(string.format("   Score: %.3f", ranked_chunk.score))

        -- Show content snippet (first 100 chars)
        local content = ranked_chunk.chunk.content
        local snippet = string.sub(content, 1, 100)
        if #content > 100 then
            snippet = snippet .. "..."
        end
        print(string.format("   Content: %s", snippet))

        -- Show metadata if present
        if ranked_chunk.chunk.metadata and ranked_chunk.chunk.metadata.subtopic then
            print(string.format("   Topic: %s", ranked_chunk.chunk.metadata.subtopic))
        end
    end
else
    print("   No chunks retrieved (empty memory or query mismatch)")
end

print()

-- ============================================================
-- Step 6: Use Formatted Context
-- ============================================================

print("6. Formatted context (ready for LLM)...")

if result.formatted and #result.formatted > 0 then
    print("\\n--- BEGIN FORMATTED CONTEXT ---")
    -- Show first 500 chars of formatted context
    local formatted_snippet = string.sub(result.formatted, 1, 500)
    if #result.formatted > 500 then
        formatted_snippet = formatted_snippet .. "\\n... [truncated]"
    end
    print(formatted_snippet)
    print("--- END FORMATTED CONTEXT ---\\n")

    print(string.format("💡 Formatted context length: %d characters", #result.formatted))
    print("   This context is ready to be prepended to your LLM prompt!")
else
    print("   ⚠️ No formatted context available")
end

print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Congratulations! You've successfully:")
print("   ✓ Added conversation to episodic memory")
print("   ✓ Assembled context using episodic strategy")
print("   ✓ Retrieved " .. #result.chunks .. " relevant chunks")
print("   ✓ Inspected ranking scores and metadata")
print("   ✓ Generated formatted context for LLM prompting")
print()
print("🔍 Key Concepts:")
print("   • Context assembly transforms memory into LLM-ready input")
print("   • Token budgets control context size for LLM windows")
print("   • Reranking scores indicate relevance to query")
print("   • Session filtering isolates context to specific conversations")
print()
print("🚀 Next Steps:")
print("   • Try context-strategy-comparison.lua for strategy comparison")
print("   • Explore memory-context-workflow.lua for E2E patterns")
print("   • Learn about hybrid retrieval combining episodic + semantic")
print()
print("Next: Check out cookbook/context-strategy-comparison.lua!")

-- Return success with stats
return {
    success = true,
    message = "Context assembly example completed",
    stats = {
        exchanges_added = added_count,
        chunks_retrieved = #result.chunks,
        token_count = result.token_count,
        confidence = result.total_confidence,
        session_id = session_id
    }
}
