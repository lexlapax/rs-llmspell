-- Recommended profile: research
-- Run with: llmspell -p research run context-strategy-comparison.lua
-- Context engineering

-- ============================================================
-- LLMSPELL COOKBOOK - CONTEXT STRATEGY COMPARISON
-- ============================================================
-- Recipe: Comparing Episodic, Semantic, and Hybrid Retrieval
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Choosing optimal retrieval strategy for different query types
--
-- Purpose: Compare three context assembly strategies to understand their strengths:
--          - Episodic: Recent conversation history (temporal)
--          - Semantic: Knowledge graph entities (conceptual)
--          - Hybrid: Combined approach (best of both)
-- Pattern: Comparative analysis with metrics
-- Crates Showcased: llmspell-context, llmspell-memory
-- Key Concepts:
--   • Strategy selection based on query type
--   • Performance characteristics (speed, relevance)
--   • Token efficiency across strategies
--   • When to use each approach
--
-- Prerequisites:
--   • Completed 06-episodic-memory-basic.lua
--   • Completed 07-context-assembly-basic.lua
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/cookbook/context-strategy-comparison.lua
--
-- EXPECTED OUTPUT:
-- Side-by-side comparison of three strategies
-- Performance metrics (chunks, tokens, confidence)
-- Recommendations for strategy selection
--
-- Time to Complete: <15 seconds
-- ============================================================

print("=== Context Strategy Comparison ===")
print("Cookbook: Episodic vs Semantic vs Hybrid Retrieval\\n")

-- ============================================================
-- Setup: Verify Availability
-- ============================================================

if not Memory or not Context then
    print("❌ Memory and Context globals required")
    return {success = false, error = "Missing globals"}
end

print("✅ Memory and Context available\\n")

-- ============================================================
-- Step 1: Populate Memory with Rich Dataset
-- ============================================================

print("1. Creating rich dataset for comparison...\\n")

local session_id = "strategy-comp-" .. os.time()

-- Add diverse conversation covering multiple topics
local conversation = {
    -- Rust programming
    {role = "user", content = "What is Rust?", metadata = {topic = "rust-intro"}},
    {role = "assistant", content = "Rust is a systems programming language focused on safety, concurrency, and performance. It prevents memory bugs through ownership.", metadata = {topic = "rust-intro"}},
    {role = "user", content = "How does ownership work?", metadata = {topic = "rust-ownership"}},
    {role = "assistant", content = "Ownership ensures each value has one owner. When the owner goes out of scope, the value is automatically dropped. This prevents memory leaks.", metadata = {topic = "rust-ownership"}},

    -- Go programming
    {role = "user", content = "Tell me about Go", metadata = {topic = "go-intro"}},
    {role = "assistant", content = "Go is a statically typed, compiled language designed for simplicity and efficiency. It features garbage collection and excellent concurrency support via goroutines.", metadata = {topic = "go-intro"}},
    {role = "user", content = "What are goroutines?", metadata = {topic = "go-concurrency"}},
    {role = "assistant", content = "Goroutines are lightweight threads managed by the Go runtime. They're incredibly cheap to create, allowing thousands to run concurrently.", metadata = {topic = "go-concurrency"}},

    -- Comparison
    {role = "user", content = "Which is better for systems programming?", metadata = {topic = "comparison"}},
    {role = "assistant", content = "Rust offers stronger safety guarantees with zero-cost abstractions, while Go provides faster compilation and simpler syntax. The choice depends on your priorities.", metadata = {topic = "comparison"}},
}

local added = 0
for _, exchange in ipairs(conversation) do
    local success, _ = pcall(Memory.episodic.add, session_id, exchange.role, exchange.content, exchange.metadata)
    if success then
        added = added + 1
    end
end

print(string.format("   ✓ Added %d exchanges to episodic memory", added))
print()

-- ============================================================
-- Step 2: Define Test Queries
-- ============================================================

print("2. Testing with different query types...\\n")

local queries = {
    {
        query = "How does Rust handle memory management?",
        type = "Recent conversation",
        expected_best = "episodic"
    },
    {
        query = "Explain concurrency in programming languages",
        type = "Conceptual knowledge",
        expected_best = "semantic or hybrid"
    },
    {
        query = "goroutines vs ownership",
        type = "Cross-topic comparison",
        expected_best = "hybrid"
    }
}

-- ============================================================
-- Step 3: Compare Strategies for Each Query
-- ============================================================

for q_idx, query_info in ipairs(queries) do
    print(string.format("\\n━━━ Query %d: %s ━━━", q_idx, query_info.type))
    print(string.format("Query: \"%s\"", query_info.query))
    print(string.format("Expected best: %s\\n", query_info.expected_best))

    local results = {}

    -- Test each strategy
    local strategies = {"episodic", "semantic", "hybrid"}
    for _, strategy in ipairs(strategies) do
        print(string.format("  Testing %s strategy...", strategy))

        local success, result = pcall(Context.assemble,
            query_info.query,
            strategy,
            2000,  -- token budget
            session_id
        )

        if success and result then
            results[strategy] = result
            print(string.format("    ✓ Chunks: %d | Tokens: %d | Confidence: %.1f%%",
                #result.chunks,
                result.token_count or 0,
                (result.total_confidence or 0) * 100
            ))
        else
            print(string.format("    ✗ Failed: %s", tostring(result)))
            results[strategy] = nil
        end
    end

    -- Determine winner
    print("\\n  📊 Analysis:")
    local best_strategy = nil
    local best_score = -1

    for strategy, result in pairs(results) do
        local score = (result.total_confidence or 0) * #result.chunks
        if score > best_score then
            best_score = score
            best_strategy = strategy
        end
    end

    if best_strategy then
        print(string.format("  🏆 Best strategy: %s", best_strategy))
        print(string.format("  Expected: %s", query_info.expected_best))

        if string.find(query_info.expected_best, best_strategy) then
            print("  ✓ Matches expectation!")
        else
            print("  ℹ️ Different from expectation (data-dependent)")
        end
    end
end

print()

-- ============================================================
-- Step 4: Strategy Characteristics Summary
-- ============================================================

print("\\n━━━ Strategy Characteristics ━━━\\n")

print("📝 Episodic Strategy:")
print("   • Source: Recent conversation history")
print("   • Best for: Contextual follow-ups, conversation continuity")
print("   • Speed: Fast (no graph traversal)")
print("   • Scope: Session-specific or cross-session")
print()

print("🧠 Semantic Strategy:")
print("   • Source: Knowledge graph entities")
print("   • Best for: Conceptual queries, fact retrieval")
print("   • Speed: Moderate (graph query + vector search)")
print("   • Scope: Global knowledge base")
print()

print("🔀 Hybrid Strategy:")
print("   • Source: Combined episodic + semantic")
print("   • Best for: Complex queries needing both context and knowledge")
print("   • Speed: Slower (combines both)")
print("   • Scope: Comprehensive retrieval")
print()

-- ============================================================
-- Step 5: Performance Stats
-- ============================================================

print("━━━ Strategy Performance Stats ━━━\\n")

local success, stats = pcall(Context.strategy_stats)
if success and stats then
    print(string.format("📊 Memory State:"))
    print(string.format("   Episodic entries: %d", stats.episodic_count or 0))
    print(string.format("   Semantic entries: %d", stats.semantic_count or 0))
    print(string.format("   Available strategies: %s", table.concat(stats.strategies or {}, ", ")))
else
    print("   ⚠️ Stats not available")
end

print()

-- ============================================================
-- Summary & Recommendations
-- ============================================================

print("━━━ Decision Guide ━━━\\n")
print("🎯 When to use each strategy:\\n")

print("Use EPISODIC when:")
print("  • Query references recent conversation")
print("  • Need temporal context (\"what did we discuss?\")")
print("  • Building conversational agents")
print("  • Session isolation is important")
print()

print("Use SEMANTIC when:")
print("  • Query is conceptual (\"what is X?\")")
print("  • Need factual knowledge retrieval")
print("  • Working with knowledge bases")
print("  • Cross-session knowledge needed")
print()

print("Use HYBRID when:")
print("  • Query complexity is high")
print("  • Need both recent context AND facts")
print("  • Comparative queries (\"X vs Y\")")
print("  • Uncertain which source is best")
print()

print("💡 Pro Tip: Start with hybrid for unknown query types, then optimize!")
print()

-- Return success
return {
    success = true,
    message = "Strategy comparison completed",
    stats = {
        queries_tested = #queries,
        session_id = session_id,
        exchanges_added = added
    }
}
