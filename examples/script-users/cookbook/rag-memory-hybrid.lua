-- Profile: rag-dev (recommended)
-- Run with: llmspell -p rag-dev run rag-memory-hybrid.lua
-- RAG features with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: cookbook
-- Recipe: Combining Document Search (RAG) with Conversation Memory
-- Complexity: ADVANCED
-- Real-World Use Case: Building context-aware AI assistants with both document knowledge and conversation history
--
-- Purpose: Demonstrate hybrid retrieval that combines:
--          1. RAG vector search over ingested documents
--          2. Episodic memory of conversation history
--          3. Weighted merge for balanced results
-- Pattern: Production RAG+Memory workflow
-- Crates Showcased: llmspell-rag, llmspell-memory, llmspell-context
-- Key Concepts:
--   • RAG document ingestion and vector search
--   • Episodic memory tracking
--   • Hybrid retrieval strategy
--   • Weighted source merging (40% RAG + 60% Memory default)
--   • Session-aware context assembly
--
-- Prerequisites:
--   • Completed 06-episodic-memory-basic.lua
--   • Completed 07-context-assembly-basic.lua
--   • Understanding of RAG and Context globals
--   • RAG pipeline configured (see config)
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/cookbook/rag-memory-hybrid.lua
--
-- EXPECTED OUTPUT:
-- Document ingestion confirmation
-- Conversation tracking
-- Hybrid retrieval results with source attribution
-- Balanced mix of RAG documents and memory chunks
--
-- Time to Complete: <30 seconds
-- ============================================================

print("=== RAG + Memory: Hybrid Retrieval Workflow ===")
print("Cookbook: Production Context Assembly\\n")

-- ============================================================
-- Setup: Verify Availability
-- ============================================================

if not RAG or not Memory or not Context then
    print("❌ RAG, Memory, and Context globals required")
    print("   Ensure RAG pipeline is configured in config.toml")
    return {success = false, error = "Missing globals or RAG not configured"}
end

print("✅ RAG, Memory, and Context available\\n")

-- ============================================================
-- Configuration
-- ============================================================

local session_id = "rag-memory-demo-" .. os.time()
local context_budget = 2000  -- Token budget for hybrid retrieval

print(string.format("📋 Configuration:"))
print(string.format("   Session ID: %s", session_id))
print(string.format("   Context budget: %d tokens", context_budget))
print(string.format("   Strategy: rag (hybrid RAG+Memory)"))
print(string.format("   Default weighting: 40%% RAG + 60%% Memory"))
print()

-- ============================================================
-- Step 1: Ingest Documents (RAG Knowledge Base)
-- ============================================================

print("1. Ingesting documents into RAG pipeline...\\n")

local rust_docs = {
    {
        content = "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It achieves memory safety without garbage collection through its ownership system.",
        metadata = {source = "rust-book", topic = "overview"}
    },
    {
        content = "The Rust ownership system ensures memory safety. Each value has a single owner. When the owner goes out of scope, the value is dropped. Values can be moved or borrowed, but the borrow checker enforces rules at compile time.",
        metadata = {source = "rust-book", topic = "ownership"}
    },
    {
        content = "Rust borrowing rules: You can have either one mutable reference OR any number of immutable references at a time. References must always be valid. This prevents data races at compile time.",
        metadata = {source = "rust-docs", topic = "borrowing"}
    },
}

local ingested_count = 0
for i, doc in ipairs(rust_docs) do
    local result = RAG.ingest(doc, {})
    if result and result.success then
        ingested_count = ingested_count + 1
        print(string.format("   ✓ Document %d ingested (source: %s)", i, doc.metadata.source))
    else
        print(string.format("   ✗ Document %d failed", i))
    end
end

print(string.format("\\n   📚 Ingested %d/%d documents\\n", ingested_count, #rust_docs))

-- ============================================================
-- Step 2: Track Conversation in Episodic Memory
-- ============================================================

print("2. Adding conversation to episodic memory...\\n")

local conversation = {
    {role = "user", content = "I'm learning Rust. Can you help me understand the basics?"},
    {role = "assistant", content = "Of course! Rust is a modern systems programming language. What aspect would you like to explore first - syntax, memory management, or concurrency?"},
    {role = "user", content = "I keep hearing about ownership. What is that?"},
    {role = "assistant", content = "Ownership is Rust's most unique feature. It's how Rust manages memory without a garbage collector. Let me explain the three rules of ownership..."},
    {role = "user", content = "How does borrowing work in practice?"},
}

for _, turn in ipairs(conversation) do
    Memory.episodic.add(session_id, turn.role, turn.content, {timestamp = os.time()})
    print(string.format("   💬 %s: %s", turn.role, turn.content:sub(1, 60) .. "..."))
end

print(string.format("\\n   📝 Tracked %d conversation turns\\n", #conversation))

-- ============================================================
-- Step 3: Hybrid Retrieval (RAG + Memory)
-- ============================================================

print("3. Performing hybrid retrieval...\\n")

local query = "Explain Rust ownership and borrowing rules"
print(string.format("   🔍 Query: \"%s\"\\n", query))

-- Use "rag" strategy for hybrid RAG+Memory retrieval
local result = Context.assemble(query, "rag", context_budget, session_id)

if not result or not result.chunks then
    print("   ❌ Hybrid retrieval failed")
    return {success = false, error = "Retrieval failed"}
end

print(string.format("   ✅ Retrieved %d context chunks", #result.chunks))
print(string.format("   📊 Token usage: %d/%d tokens (%.1f%%)\\n",
    result.token_count, context_budget,
    (result.token_count / context_budget) * 100))

-- ============================================================
-- Step 4: Analyze Result Sources
-- ============================================================

print("4. Analyzing result sources...\\n")

local rag_chunks = 0
local memory_chunks = 0

for i, chunk in ipairs(result.chunks) do
    local source = chunk.chunk.source or "unknown"
    local content_preview = (chunk.chunk.content or ""):sub(1, 80)
    local score = chunk.score or 0.0

    -- Classify by source
    if source:match("^memory:") then
        memory_chunks = memory_chunks + 1
        print(string.format("   [%d] 💾 MEMORY (%.3f): %s...", i, score, content_preview))
    else
        rag_chunks = rag_chunks + 1
        print(string.format("   [%d] 📚 RAG:%s (%.3f): %s...", i, source, score, content_preview))
    end
end

print()
print(string.format("   📊 Source Distribution:"))
print(string.format("      RAG documents: %d chunks (%.1f%%)",
    rag_chunks, (rag_chunks / #result.chunks) * 100))
print(string.format("      Memory history: %d chunks (%.1f%%)",
    memory_chunks, (memory_chunks / #result.chunks) * 100))
print()

-- ============================================================
-- Step 5: Memory Statistics
-- ============================================================

print("5. Memory statistics...\\n")

local stats = Memory.stats()
print(string.format("   Episodic entries: %d", stats.episodic_count))
print(string.format("   Semantic entries: %d", stats.semantic_count))
print(string.format("   Consolidation status: %s", stats.consolidation_status))
print()

-- ============================================================
-- Summary
-- ============================================================

print("=== Hybrid Retrieval Summary ===")
print()
print("✅ Successfully demonstrated:")
print("   • Document ingestion into RAG pipeline")
print("   • Conversation tracking in episodic memory")
print("   • Hybrid retrieval combining both sources")
print("   • Balanced merging with configurable weights")
print("   • Session-aware context assembly")
print()
print("💡 Key Insights:")
print("   • RAG provides factual document content")
print("   • Memory provides conversation context")
print("   • Hybrid strategy balances both sources")
print("   • Default 40/60 weighting favors recent memory")
print("   • BM25 reranking ensures relevance")
print()
print("📈 Next Steps:")
print("   • Adjust token budget for your use case")
print("   • Experiment with different queries")
print("   • Try other strategies (episodic, semantic, hybrid)")
print("   • Integrate with LLM for full conversation flow")
print()

return {
    success = true,
    session_id = session_id,
    ingested_documents = ingested_count,
    conversation_turns = #conversation,
    retrieved_chunks = #result.chunks,
    token_usage = result.token_count,
    rag_chunks = rag_chunks,
    memory_chunks = memory_chunks,
}
