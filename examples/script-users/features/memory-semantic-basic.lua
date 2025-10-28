-- ============================================================
-- LLMSPELL FEATURES - SEMANTIC MEMORY
-- ============================================================
-- Feature: Semantic Memory and Knowledge Graph
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Knowledge management, entity relationship tracking
--
-- Purpose: Learn how to use semantic memory for storing structured
--          knowledge, entities, and relationships. Unlike episodic memory
--          (conversations), semantic memory stores facts and knowledge.
-- Pattern: Entity-relationship knowledge graph storage
-- Crates Showcased: llmspell-memory, llmspell-graph, llmspell-bridge
-- Key Features:
--   • Storing entities with embeddings
--   • Querying knowledge by similarity
--   • Entity relationships and metadata
--   • Long-term knowledge persistence
--   • Integration with consolidation
--
-- Prerequisites:
--   • Completed getting-started/06-episodic-memory-basic.lua
--   • Understanding of semantic vs episodic memory
--
-- HOW TO RUN:
-- ./target/debug/llmspell \
--   run examples/script-users/features/memory-semantic-basic.lua
--
-- EXPECTED OUTPUT:
-- Entities stored in knowledge graph
-- Semantic similarity queries
-- Knowledge retrieval by topic
-- Integration with episodic memory
--
-- Time to Complete: <10 seconds
-- ============================================================

print("=== Semantic Memory & Knowledge Graph ===")
print("Feature: Structured knowledge storage and retrieval\n")

-- ============================================================
-- Setup: Verify Memory Availability
-- ============================================================

if not Memory then
    print("❌ Memory system not available")
    return {success = false, error = "Memory not configured"}
end

if not Memory.semantic then
    print("❌ Semantic memory not available")
    print("   Note: Semantic memory requires graph backend")
    return {success = false, error = "Semantic memory not configured"}
end

print("✅ Semantic memory available\n")

-- ============================================================
-- Step 1: Understand Semantic vs Episodic Memory
-- ============================================================

print("1. Understanding memory types...")

print("📚 Memory Type Comparison:")
print("   Episodic Memory:")
print("     • Stores conversations and interactions")
print("     • Time-ordered sequence of exchanges")
print("     • Session-specific context")
print("     • Example: 'User asked about Rust at 2pm'")
print()
print("   Semantic Memory:")
print("     • Stores facts and knowledge")
print("     • Entities and relationships")
print("     • Timeless structured information")
print("     • Example: 'Rust is a programming language'")
print()

-- ============================================================
-- Step 2: Store Entities in Semantic Memory
-- ============================================================

print("2. Storing entities in semantic memory...")

-- Define knowledge entities
local entities = {
    {
        id = "entity:rust",
        type = "programming_language",
        content = "Rust is a systems programming language focused on safety, concurrency, and performance",
        metadata = {
            name = "Rust",
            year = 2010,
            paradigm = "multi-paradigm",
            typing = "static"
        }
    },
    {
        id = "entity:ownership",
        type = "concept",
        content = "Ownership is Rust's memory management system that ensures memory safety without garbage collection",
        metadata = {
            name = "Ownership",
            related_to = "rust",
            category = "memory_management"
        }
    },
    {
        id = "entity:borrowing",
        type = "concept",
        content = "Borrowing allows references to data without taking ownership, enforced by the borrow checker",
        metadata = {
            name = "Borrowing",
            related_to = "ownership",
            category = "memory_management"
        }
    },
    {
        id = "entity:cargo",
        type = "tool",
        content = "Cargo is Rust's build system and package manager, handling dependencies and compilation",
        metadata = {
            name = "Cargo",
            related_to = "rust",
            category = "tooling"
        }
    }
}

print(string.format("   Adding %d entities to knowledge graph...", #entities))

local added_entities = 0
for i, entity in ipairs(entities) do
    -- Note: Semantic memory API may vary based on implementation
    -- This demonstrates the conceptual usage pattern
    local result = Memory.semantic.add(entity)

    if result and result.success then
        added_entities = added_entities + 1
        print(string.format("   ✓ Entity %d: %s (%s)",
            i, entity.metadata.name, entity.type))
    else
        -- If direct add not available, document it
        print(string.format("   ℹ️ Entity %d: %s (requires consolidation)",
            i, entity.metadata.name))
    end
end

if added_entities > 0 then
    print(string.format("\n   📝 Added %d entities directly", added_entities))
else
    print("\n   ℹ️ Note: Entities typically added via episodic→semantic consolidation")
    print("   See consolidation examples for automated knowledge extraction")
end

print()

-- ============================================================
-- Step 3: Query Semantic Memory
-- ============================================================

print("3. Querying semantic memory...")

-- Test different knowledge queries
local queries = {
    "memory management in programming",
    "Rust language features",
    "build tools and package management"
}

for i, query in ipairs(queries) do
    print(string.format("\n🔍 Query %d: '%s'", i, query))

    local result = Memory.semantic.query(query, 5)

    if result and result.success and result.entities then
        if #result.entities > 0 then
            print(string.format("   Found %d entities:", #result.entities))

            for j, entity in ipairs(result.entities) do
                local name = "Unknown"
                local entity_type = "unknown"

                if entity.metadata and entity.metadata.name then
                    name = entity.metadata.name
                end
                if entity.type then
                    entity_type = entity.type
                end

                print(string.format("   %d. %s [%s]", j, name, entity_type))

                -- Show snippet if available
                if entity.content then
                    local snippet = string.sub(entity.content, 1, 60)
                    if #entity.content > 60 then snippet = snippet .. "..." end
                    print(string.format("      %s", snippet))
                end
            end
        else
            print("   No entities found")
            print("   Tip: Add data via consolidation or direct entity storage")
        end
    else
        print("   ℹ️ Semantic query not available or no results")
        print("   This is normal if no consolidation has run yet")
    end
end

print()

-- ============================================================
-- Step 4: Semantic Memory Statistics
-- ============================================================

print("4. Semantic memory statistics...")

local stats = Memory.stats()
if stats then
    print("📊 Knowledge graph state:")
    print(string.format("   Semantic entities: %d", stats.semantic_count or 0))
    print(string.format("   Episodic entries: %d", stats.episodic_count or 0))

    if stats.semantic_count and stats.episodic_count then
        local ratio = 0
        if stats.episodic_count > 0 then
            ratio = (stats.semantic_count / stats.episodic_count) * 100
        end
        print(string.format("   Consolidation ratio: %.1f%%", ratio))
        print("   (Semantic/Episodic - higher means more knowledge extracted)")
    end
else
    print("   ⚠️ Statistics not available")
end

print()

-- ============================================================
-- Step 5: Episodic→Semantic Workflow
-- ============================================================

print("5. Demonstrating episodic→semantic workflow...")

print("\n💡 Knowledge Extraction Workflow:")
print("   1. Conversations stored in episodic memory")
print("   2. Consolidation engine processes episodic entries")
print("   3. LLM extracts entities and relationships")
print("   4. Entities stored in semantic memory (knowledge graph)")
print("   5. Future queries leverage both memories")

-- Add sample episodic data to demonstrate
local session = "semantic-demo-" .. os.time()
print(string.format("\n   Creating sample conversation (session: %s)...", session:sub(1, 20) .. "..."))

local sample_conversation = {
    {role = "user", content = "What are Rust's key features?"},
    {role = "assistant", content = "Rust's key features include ownership for memory safety, zero-cost abstractions, and excellent concurrency support."},
    {role = "user", content = "How does ownership work?"},
    {role = "assistant", content = "Ownership ensures each value has exactly one owner. When the owner goes out of scope, the value is dropped automatically."}
}

local conv_added = 0
for _, exchange in ipairs(sample_conversation) do
    local success, result = pcall(Memory.episodic.add,
        session,
        exchange.role,
        exchange.content,
        {category = "rust-learning"}
    )
    if success then
        conv_added = conv_added + 1
    end
end

print(string.format("   ✓ Added %d exchanges to episodic memory", conv_added))
print("\n   ℹ️ Run consolidation to extract knowledge:")
print("   • Via CLI: `llmspell memory consolidate --session " .. session .. "`")
print("   • Via Lua: `Memory.consolidate(session, {force = true})`")

print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Semantic Memory Concepts Covered!")
print("\n✓ Key Learnings:")
print("   • Semantic memory stores structured knowledge")
print("   • Entities have types, embeddings, and metadata")
print("   • Knowledge extracted via consolidation from episodic memory")
print("   • Semantic queries find related knowledge")
print("   • Graph structure enables relationship traversal")
print("\n📚 Memory System Architecture:")
print("   Episodic → [Consolidation] → Semantic")
print("   (Events)     (LLM Extract)    (Knowledge)")
print("\n🔄 Typical Workflow:")
print("   1. User conversations → episodic memory")
print("   2. Periodic consolidation runs")
print("   3. LLM extracts entities/facts")
print("   4. Entities stored in knowledge graph")
print("   5. Hybrid queries use both memories")
print("\n🚀 Next Steps:")
print("   • Explore consolidation with cookbook examples")
print("   • Learn about hybrid memory queries")
print("   • Integrate with Context global for retrieval")

-- Return success
return {
    success = true,
    message = "Semantic memory exploration completed",
    stats = {
        entities_demonstrated = #entities,
        queries_performed = #queries,
        episodic_added = conv_added,
        session_id = session
    }
}
