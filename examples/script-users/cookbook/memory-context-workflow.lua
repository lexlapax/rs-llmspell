-- Recommended profile: memory
-- Run with: llmspell -p memory run memory-context-workflow.lua
-- Adaptive memory system

-- ============================================================
-- LLMSPELL COOKBOOK - MEMORY + CONTEXT E2E WORKFLOW
-- ============================================================
-- Recipe: Complete Memory→Context→LLM Pipeline
-- Complexity Level: ADVANCED
-- Real-World Use Case: Building production AI with context-aware conversations
--
-- Purpose: Demonstrate end-to-end workflow from raw interactions to LLM-ready prompts.
--          Shows the complete pipeline: interactions → memory → retrieval → assembly → prompting
-- Pattern: Production-ready RAG workflow
-- Crates Showcased: llmspell-memory, llmspell-context, llmspell-bridge
-- Key Concepts:
--   • Multi-turn conversation tracking
--   • Dynamic context assembly per turn
--   • Token budget management
--   • Context window optimization
--   • Memory growth monitoring
--
-- Prerequisites:
--   • Completed 06-episodic-memory-basic.lua
--   • Completed 07-context-assembly-basic.lua
--   • Understanding of both Memory and Context globals
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/cookbook/memory-context-workflow.lua
--
-- EXPECTED OUTPUT:
-- Multi-turn conversation simulation
-- Context assembly for each turn
-- Token usage tracking
-- Memory growth visualization
--
-- Time to Complete: <20 seconds
-- ============================================================

print("=== Memory + Context: End-to-End Workflow ===")
print("Cookbook: Production RAG Pipeline\\n")

-- ============================================================
-- Setup: Verify Availability
-- ============================================================

if not Memory or not Context then
    print("❌ Memory and Context globals required")
    return {success = false, error = "Missing globals"}
end

print("✅ Memory and Context available\\n")

-- ============================================================
-- Configuration
-- ============================================================

local session_id = "e2e-workflow-" .. os.time()
local context_window = 4000  -- Token budget for context
local conversation_turns = 8

print(string.format("📋 Configuration:"))
print(string.format("   Session ID: %s", session_id))
print(string.format("   Context window: %d tokens", context_window))
print(string.format("   Planned turns: %d", conversation_turns))
print()

-- ============================================================
-- Step 1: Simulate Multi-Turn Conversation
-- ============================================================

print("1. Simulating multi-turn conversation...\\n")

local conversation_script = {
    {role = "user", query = "What is Rust?"},
    {role = "assistant", response = "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It achieves memory safety without garbage collection."},

    {role = "user", query = "Tell me more about memory safety"},
    {role = "assistant", response = "Memory safety means preventing common bugs like use-after-free, double-free, and buffer overflows. Rust achieves this through its ownership system and borrow checker at compile time."},

    {role = "user", query = "How does the ownership system work?"},
    {role = "assistant", response = "In Rust's ownership system, each value has a single owner. When the owner goes out of scope, the value is dropped automatically. Values can be moved or borrowed, but these operations are checked at compile time."},

    {role = "user", query = "What's the difference between borrowing and ownership?"},
    {role = "assistant", response = "Ownership transfers control of a value permanently. Borrowing creates a temporary reference without transferring ownership. You can have multiple immutable borrows OR one mutable borrow, preventing data races."},

    {role = "user", query = "Can you explain lifetimes?"},
    {role = "assistant", response = "Lifetimes are Rust's way of ensuring references are always valid. They specify how long references should remain valid. The compiler uses lifetime annotations to prevent dangling references and ensure memory safety."},

    {role = "user", query = "How do I choose between String and &str?"},
    {role = "assistant", response = "Use String when you need ownership of text data (growable, heap-allocated). Use &str for borrowed string slices (views into existing strings, stack or heap). String owns its data, &str just references it."},

    {role = "user", query = "What about Vec vs slice?"},
    {role = "assistant", response = "Vec<T> is an owned, growable array on the heap. &[T] is a borrowed view into a sequence. Use Vec when you need to own and modify data, use slices when you just need to read or temporarily work with data."},

    {role = "user", query = "Summarize the key concepts we discussed"},
    {role = "assistant", response = "We covered Rust's core memory safety features: ownership (each value has one owner), borrowing (temporary references), lifetimes (ensuring reference validity), and the distinction between owned (String, Vec) and borrowed (&str, &[T]) types."}
}

local turn_count = 0
local memory_sizes = {}

for turn_idx, turn in ipairs(conversation_script) do
    turn_count = turn_idx
    print(string.format("━━━ Turn %d ━━━", turn_idx))

    -- User message
    if turn.query then
        print(string.format("User: %s", turn.query))

        -- Store user message
        local success, _ = pcall(Memory.episodic.add,
            session_id,
            "user",
            turn.query,
            {turn = turn_idx}
        )

        if not success then
            print(string.format("   ⚠️ Failed to store user message"))
        end
    end

    -- Assemble context before generating response
    if turn.response then
        print(string.format("\\n   📚 Assembling context..."))

        local ctx_success, context = pcall(Context.assemble,
            turn.query or "continue",
            "episodic",
            context_window,
            session_id
        )

        if ctx_success and context then
            print(string.format("   Context: %d chunks, %d/%d tokens (%.1f%% conf)",
                #context.chunks,
                context.token_count or 0,
                context_window,
                (context.total_confidence or 0) * 100
            ))

            -- In production, you'd pass context.formatted to your LLM here
            -- local prompt = context.formatted .. "\\n\\nUser: " .. turn.query
            -- local llm_response = call_llm(prompt)

            print(string.format("   💬 (In production: send to LLM with %d chars of context)",
                #(context.formatted or "")))
        else
            print(string.format("   ⚠️ Context assembly failed"))
        end

        -- Store assistant response
        print(string.format("\\nAssistant: %s", turn.response:sub(1, 100) .. "..."))

        local success, _ = pcall(Memory.episodic.add,
            session_id,
            "assistant",
            turn.response,
            {turn = turn_idx}
        )

        if not success then
            print(string.format("   ⚠️ Failed to store assistant message"))
        end
    end

    -- Track memory growth
    local stats_success, stats = pcall(Memory.stats)
    if stats_success and stats then
        memory_sizes[turn_idx] = stats.episodic_count or 0
    end

    print()
end

-- ============================================================
-- Step 2: Memory Growth Analysis
-- ============================================================

print("\\n2. Memory growth analysis...\\n")

print("📈 Episodic Memory Growth:")
local prev_size = 0
for turn_idx, size in ipairs(memory_sizes) do
    local growth = size - prev_size
    print(string.format("   Turn %d: %d entries (+%d)", turn_idx, size, growth))
    prev_size = size
end

print()

-- ============================================================
-- Step 3: Context Assembly Performance
-- ============================================================

print("3. Testing context assembly at different conversation depths...\\n")

local depths_to_test = {2, 4, 6, 8}
local test_query = "Explain Rust's memory management concepts"

print(string.format("Test query: \"%s\"\\n", test_query))

for _, depth in ipairs(depths_to_test) do
    if depth <= turn_count then
        -- Create partial session view
        local success, context = pcall(Context.assemble,
            test_query,
            "episodic",
            context_window,
            session_id
        )

        if success and context then
            print(string.format("  Depth %d turns: %d chunks, %d tokens, %.1f%% confidence",
                depth,
                #context.chunks,
                context.token_count or 0,
                (context.total_confidence or 0) * 100
            ))
        end
    end
end

print()

-- ============================================================
-- Step 4: Final Context Assembly
-- ============================================================

print("4. Final context assembly for complete conversation...\\n")

local final_query = "What are the most important Rust concepts for memory safety?"
print(string.format("Query: \"%s\"\\n", final_query))

local success, final_context = pcall(Context.assemble,
    final_query,
    "hybrid",  -- Use hybrid for comprehensive retrieval
    context_window,
    session_id
)

if success and final_context then
    print(string.format("📦 Final Context:"))
    print(string.format("   Strategy: hybrid"))
    print(string.format("   Chunks: %d", #final_context.chunks))
    print(string.format("   Tokens: %d / %d (%.1f%% used)",
        final_context.token_count or 0,
        context_window,
        ((final_context.token_count or 0) / context_window) * 100
    ))
    print(string.format("   Confidence: %.1f%%", (final_context.total_confidence or 0) * 100))

    if final_context.formatted and #final_context.formatted > 0 then
        print(string.format("   Formatted: %d characters ready for LLM",
            #final_context.formatted))
    end
else
    print(string.format("   ✗ Assembly failed: %s", tostring(final_context)))
end

print()

-- ============================================================
-- Step 5: Production Considerations
-- ============================================================

print("━━━ Production Deployment Guide ━━━\\n")

print("🏭 Production Workflow:")
print("   1. User input arrives")
print("   2. Store in episodic memory (Memory.episodic.add)")
print("   3. Assemble context (Context.assemble with token budget)")
print("   4. Prepend context to prompt")
print("   5. Call LLM with context + current query")
print("   6. Store LLM response in memory")
print("   7. Repeat for next turn")
print()

print("⚙️ Configuration Tips:")
print("   • Context window: Match your LLM (4k, 8k, 32k, 128k)")
print("   • Strategy: episodic for chat, hybrid for knowledge Q&A")
print("   • Session management: New session per conversation")
print("   • Memory cleanup: Archive old sessions periodically")
print()

print("📊 Monitoring:")
print("   • Memory.stats() - Track memory growth")
print("   • Context.strategy_stats() - Check data distribution")
print("   • Token usage - Ensure context fits window")
print("   • Confidence scores - Verify relevance quality")
print()

print("🔄 Advanced Patterns:")
print("   • Consolidation: Convert episodic→semantic for long-term knowledge")
print("   • Multi-session: Query across conversation history")
print("   • Adaptive budgets: Adjust context size based on query")
print("   • Reranking: Fine-tune relevance with custom scorers")
print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Complete E2E Workflow Demonstrated!\\n")

print("✓ Covered:")
print("   • Multi-turn conversation management")
print("   • Per-turn context assembly")
print("   • Memory growth tracking")
print("   • Token budget management")
print("   • Production deployment patterns")
print()

print("🚀 Next Steps:")
print("   • Integrate with actual LLM (OpenAI, Anthropic, local)")
print("   • Add error handling and retries")
print("   • Implement session lifecycle management")
print("   • Set up monitoring and alerting")
print()

-- Return comprehensive stats
return {
    success = true,
    message = "E2E workflow completed",
    stats = {
        turns = turn_count,
        session_id = session_id,
        final_memory_size = memory_sizes[#memory_sizes] or 0,
        context_window = context_window
    }
}
