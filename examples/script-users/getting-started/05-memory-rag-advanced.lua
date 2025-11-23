-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: getting-started
-- Profile: memory-development (recommended)
-- Example ID: 05 - Memory, RAG & Context Assembly Advanced v0.14.0
-- Complexity: INTERMEDIATE
-- Real-World Use Case: Building AI with document understanding and conversation memory
--
-- Purpose: Comprehensive introduction to LLMSpell's memory and retrieval systems.
--          Demonstrates RAG document ingestion, episodic memory for conversations,
--          context assembly with token budgeting, and integrated workflows that
--          combine all three systems for production-ready AI applications.
--
-- Architecture: Three-layer system: RAG (documents) → Memory (conversations) → Context (LLM prompts)
-- Crates Showcased: llmspell-rag, llmspell-memory, llmspell-context, llmspell-bridge
--
-- Key Features:
--   • RAG: Document ingestion, vector embeddings, semantic search
--   • Memory: Episodic conversation tracking with session isolation
--   • Context: Intelligent context assembly with reranking and token budgets
--   • Integration: End-to-end workflow combining RAG + Memory + Agent
--
-- Prerequisites:
--   • LLMSpell built with memory, RAG, and context features
--   • Environment: OPENAI_API_KEY for embeddings (or compatible provider)
--   • Network connectivity for API calls
--   • Understanding of vector search and semantic similarity concepts
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p memory-development \
--   run examples/script-users/getting-started/05-memory-rag-advanced.lua
--
-- EXPECTED OUTPUT:
-- - RAG documents ingested and searchable
-- - Conversation tracked in episodic memory
-- - Context assembled with relevance ranking
-- - Integrated workflow combining all systems
-- - Complete statistics and performance metrics
--
-- Runtime: ~10 minutes (includes API calls for embeddings)
-- Learning Time: 15-20 minutes
-- ============================================================

print("=== LLMSpell: Memory, RAG & Context Assembly ===")
print("Example 05: INTERMEDIATE - Comprehensive Memory & Retrieval")
print("Showcasing: RAG → Memory → Context → Agent integration\n")

-- ============================================================
-- SECTION 1: RAG (Retrieval-Augmented Generation)
-- ============================================================
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("SECTION 1: RAG - Document Ingestion & Search")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

-- Step 1.1: Check RAG Availability
print("1.1. Checking RAG availability...")
if not RAG then
    print("❌ RAG is not available. Please check your configuration.")
    print("   Build with: cargo build --all-features")
    return {
        success = false,
        error = "RAG not configured"
    }
end

print("✅ RAG is available\n")

-- Step 1.2: Configure RAG
print("1.2. Configuring RAG backend...")

local rag_config = {
    backend = "sqlite",           -- SQLite for local storage
    embedding_provider = "openai", -- OpenAI for embeddings
    embedding_model = "text-embedding-3-small", -- Fast, cost-effective model
    chunk_size = 512,             -- Characters per chunk
    chunk_overlap = 50            -- Overlap for context continuity
}

local config_success, config_error = pcall(RAG.configure, rag_config)

if not config_success then
    print("❌ RAG configuration failed: " .. tostring(config_error))
    return {
        success = false,
        error = "RAG configuration failed"
    }
end

print("✅ RAG configured with OpenAI embeddings\n")

-- Step 1.3: Ingest Documents
print("1.3. Ingesting programming language documentation...")

local documents = {
    {
        id = "rust-intro",
        content = "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It achieves memory safety without garbage collection through its ownership system.",
        metadata = {
            category = "programming",
            language = "rust",
            topic = "introduction"
        }
    },
    {
        id = "rust-ownership",
        content = "Ownership is Rust's most unique feature. Each value in Rust has a variable that's called its owner. There can only be one owner at a time. When the owner goes out of scope, the value will be dropped.",
        metadata = {
            category = "programming",
            language = "rust",
            topic = "ownership"
        }
    },
    {
        id = "python-intro",
        content = "Python is an interpreted, high-level programming language known for its simplicity and readability. It emphasizes code readability with significant whitespace and supports multiple programming paradigms.",
        metadata = {
            category = "programming",
            language = "python",
            topic = "introduction"
        }
    },
    {
        id = "javascript-intro",
        content = "JavaScript is a high-level, interpreted scripting language that conforms to the ECMAScript specification. It is a language that is also characterized as dynamic, weakly typed, prototype-based and multi-paradigm.",
        metadata = {
            category = "programming",
            language = "javascript",
            topic = "introduction"
        }
    },
    {
        id = "go-concurrency",
        content = "Go makes it easy to build concurrent programs with goroutines and channels. Goroutines are lightweight threads managed by the Go runtime. Channels are the pipes that connect concurrent goroutines.",
        metadata = {
            category = "programming",
            language = "go",
            topic = "concurrency"
        }
    }
}

local ingested_count = 0
for i, doc in ipairs(documents) do
    local success, result = pcall(RAG.ingest, doc.id, doc.content, doc.metadata)

    if success then
        ingested_count = ingested_count + 1
        print(string.format("   ✓ Document %d: %s", i, doc.id))
    else
        print(string.format("   ✗ Document %d failed: %s", i, tostring(result)))
    end
end

print(string.format("\n📚 Ingested %d/%d documents\n", ingested_count, #documents))

if ingested_count == 0 then
    print("❌ No documents were ingested. Cannot continue.")
    return {
        success = false,
        error = "Document ingestion failed"
    }
end

-- Step 1.4: Search Documents
print("1.4. Testing semantic search...")

local search_queries = {
    "memory management",
    "concurrent programming",
    "easy to read language",
    "systems programming"
}

for i, query in ipairs(search_queries) do
    print(string.format("\n🔍 Query %d: '%s'", i, query))

    local success, results = pcall(RAG.search, query, 3)

    if success and results then
        if #results > 0 then
            print(string.format("   Found %d relevant documents:", #results))

            for j, result in ipairs(results) do
                local snippet = string.sub(result.content, 1, 60)
                if #result.content > 60 then
                    snippet = snippet .. "..."
                end

                print(string.format("   %d. [%s] %s (score: %.3f)",
                    j,
                    result.metadata.language or "unknown",
                    snippet,
                    result.score or 0
                ))
            end
        else
            print("   No results found")
        end
    else
        print("   ✗ Search error: " .. tostring(results))
    end
end

print()

-- ============================================================
-- SECTION 2: Episodic Memory - Conversation Tracking
-- ============================================================
print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("SECTION 2: Episodic Memory - Conversation Tracking")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

-- Step 2.1: Check Memory Availability
print("2.1. Checking Memory availability...")
if not Memory then
    print("❌ Memory is not available. Please check your configuration.")
    return {
        success = false,
        error = "Memory not configured"
    }
end

print("✅ Memory is available")

-- Check available subsystems
local memory_subsystems = {}
if Memory.episodic then table.insert(memory_subsystems, "episodic") end
if Memory.semantic then table.insert(memory_subsystems, "semantic") end
if Memory.stats then table.insert(memory_subsystems, "stats") end

print("   Available subsystems: " .. table.concat(memory_subsystems, ", "))
print()

-- Step 2.2: Create Conversation Session
print("2.2. Creating conversation session...")

local session_id = "memory-rag-demo-" .. os.time()
print("   Session ID: " .. session_id)
print("   (Session IDs isolate different conversations)")
print()

-- Step 2.3: Add Conversation to Memory
print("2.3. Adding conversation exchanges to memory...")

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

print(string.format("\n💾 Added %d/%d exchanges to memory\n", added_count, #exchanges))

if added_count == 0 then
    print("❌ No exchanges were added. Cannot continue.")
    return {
        success = false,
        error = "Memory addition failed"
    }
end

-- Step 2.4: Search Memory
print("2.4. Searching memory by relevance...")

local memory_queries = {
    "ownership",
    "How does Rust prevent memory leaks?"
}

for i, query in ipairs(memory_queries) do
    print(string.format("\n🔍 Memory Query %d: '%s'", i, query))

    local success, entries = pcall(Memory.episodic.search,
        session_id,
        query,
        5
    )

    if success and entries then
        if #entries > 0 then
            print(string.format("   Found %d relevant entries:", #entries))

            for j, entry in ipairs(entries) do
                local snippet = string.sub(entry.content, 1, 80)
                if #entry.content > 80 then
                    snippet = snippet .. "..."
                end

                print(string.format("   %d. [%s] %s",
                    j,
                    entry.role,
                    snippet
                ))

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
-- SECTION 3: Context Assembly - Token-Budgeted Retrieval
-- ============================================================
print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("SECTION 3: Context Assembly - Token-Budgeted Retrieval")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

-- Step 3.1: Check Context Availability
print("3.1. Checking Context availability...")
if not Context then
    print("❌ Context is not available. Please check your configuration.")
    return {
        success = false,
        error = "Context not configured"
    }
end

print("✅ Context is available\n")

-- Step 3.2: Assemble Context
print("3.2. Assembling context for query: 'How does ownership work in Rust?'")

local success, result = pcall(Context.assemble,
    "How does ownership work in Rust?",
    "episodic",
    2000,
    session_id
)

if not success then
    print("   ✗ Context assembly failed: " .. tostring(result))
    return {
        success = false,
        error = "Context assembly failed"
    }
end

print("   ✅ Context assembled successfully\n")

-- Step 3.3: Inspect Context Metrics
print("3.3. Context metrics...")

print("📊 Context Metrics:")
print(string.format("   Chunks retrieved: %d", #result.chunks))
print(string.format("   Token count: %d / 2000", result.token_count))
print(string.format("   Confidence: %.2f%%", result.total_confidence * 100))

if result.temporal_span then
    print(string.format("   Temporal span: %s to %s",
        tostring(result.temporal_span[1] or "unknown"),
        tostring(result.temporal_span[2] or "unknown")))
end

print()

-- Step 3.4: Display Chunk Details
print("3.4. Chunk details (ranked by relevance)...")

if #result.chunks > 0 then
    for i, ranked_chunk in ipairs(result.chunks) do
        print(string.format("\n📝 Chunk %d:", i))
        print(string.format("   Role: %s", ranked_chunk.chunk.role))
        print(string.format("   Score: %.3f", ranked_chunk.score))

        local content = ranked_chunk.chunk.content
        local snippet = string.sub(content, 1, 100)
        if #content > 100 then
            snippet = snippet .. "..."
        end
        print(string.format("   Content: %s", snippet))

        if ranked_chunk.chunk.metadata and ranked_chunk.chunk.metadata.subtopic then
            print(string.format("   Topic: %s", ranked_chunk.chunk.metadata.subtopic))
        end
    end
else
    print("   No chunks retrieved")
end

print()

-- Step 3.5: Show Formatted Context
print("3.5. Formatted context preview...")

if result.formatted and #result.formatted > 0 then
    local formatted_snippet = string.sub(result.formatted, 1, 300)
    if #result.formatted > 300 then
        formatted_snippet = formatted_snippet .. "\n... [truncated]"
    end

    print("\n--- BEGIN FORMATTED CONTEXT ---")
    print(formatted_snippet)
    print("--- END FORMATTED CONTEXT ---\n")

    print(string.format("💡 Formatted context length: %d characters", #result.formatted))
    print("   This context is ready to be prepended to your LLM prompt!")
else
    print("   ⚠️ No formatted context available")
end

print()

-- ============================================================
-- SECTION 4: Integrated Workflow - RAG + Memory + Agent
-- ============================================================
print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("SECTION 4: Integrated Workflow - RAG + Memory + Agent")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

-- Step 4.1: Create Agent with RAG
print("4.1. Creating agent with RAG augmentation...")

if not Agent then
    print("⚠️ Agent not available, skipping integration workflow")
else
    local model = os.getenv("MODEL") or "gpt-3.5-turbo"

    local agent_success, agent = pcall(Agent.create, {
        model = model,
        system_prompt = "You are a helpful programming assistant with access to documentation. Use the provided context to answer questions accurately.",
        use_rag = true
    })

    if not agent_success then
        print("   ⚠️ Agent creation failed: " .. tostring(agent))
        print("   Continuing without agent integration...")
    else
        print("   ✅ Agent created with RAG integration\n")

        -- Step 4.2: Query Agent with Context
        print("4.2. Querying agent with context from memory...")

        local user_query = "Based on our conversation, explain the relationship between ownership and borrowing in Rust"

        print("   User query: " .. user_query)
        print("   Retrieving relevant context from memory...")

        -- Assemble context for this query
        local ctx_success, ctx_result = pcall(Context.assemble,
            user_query,
            "episodic",
            1500,
            session_id
        )

        if ctx_success and ctx_result.formatted then
            print(string.format("   Retrieved %d chunks (%d tokens)",
                #ctx_result.chunks, ctx_result.token_count))

            -- Combine context with query
            local augmented_query = ctx_result.formatted .. "\n\nQuestion: " .. user_query

            print("\n   Sending augmented query to agent...")
            local response_success, response = pcall(agent.complete, agent, augmented_query)

            if response_success then
                print("\n📨 Agent Response:")
                print("   " .. string.gsub(response, "\n", "\n   "))
            else
                print("   ⚠️ Agent query failed: " .. tostring(response))
            end
        else
            print("   ⚠️ Context assembly failed for agent query")
        end

        print()

        -- Step 4.3: Add Response to Memory
        print("4.3. Adding agent response to episodic memory...")

        if response_success then
            local mem_success, mem_result = pcall(Memory.episodic.add,
                session_id,
                "user",
                user_query,
                {topic = "programming", subtopic = "rust-integration"}
            )

            if mem_success then
                print("   ✓ User query added to memory")
            end

            local mem_success2, mem_result2 = pcall(Memory.episodic.add,
                session_id,
                "assistant",
                response,
                {topic = "programming", subtopic = "rust-integration"}
            )

            if mem_success2 then
                print("   ✓ Agent response added to memory")
                print("   💡 This creates a feedback loop: conversation → memory → context → agent → memory")
            end
        end
    end
end

print()

-- ============================================================
-- SECTION 5: Statistics & Summary
-- ============================================================
print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("SECTION 5: Statistics & Summary")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

-- Step 5.1: RAG Statistics
print("5.1. RAG Statistics...")
local rag_stats = RAG.stats()
if rag_stats then
    print("📊 RAG Status:")
    for key, value in pairs(rag_stats) do
        print(string.format("   %s: %s", key, tostring(value)))
    end
else
    print("   ⚠️ RAG statistics not available")
end

print()

-- Step 5.2: Memory Statistics
print("5.2. Memory Statistics...")
local mem_stats = Memory.stats()
if mem_stats then
    print("📊 Memory Status:")
    print("   Episodic entries: " .. (mem_stats.episodic_count or 0))
    print("   Semantic entries: " .. (mem_stats.semantic_count or 0))

    if mem_stats.consolidation_status then
        print("   Consolidation: " .. mem_stats.consolidation_status)
    end

    if mem_stats.sessions_with_unprocessed then
        print("   Sessions pending consolidation: " .. mem_stats.sessions_with_unprocessed)
    end
else
    print("   ⚠️ Memory statistics not available")
end

print()

-- ============================================================
-- Final Summary
-- ============================================================

print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("🎉 Congratulations! You've successfully:")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

print("✅ RAG (Retrieval-Augmented Generation):")
print("   • Configured RAG backend with OpenAI embeddings")
print("   • Ingested " .. ingested_count .. " programming documents")
print("   • Performed semantic document search")
print()

print("✅ Episodic Memory:")
print("   • Created isolated conversation session")
print("   • Added " .. added_count .. " exchanges to memory")
print("   • Searched memory with semantic queries")
print()

print("✅ Context Assembly:")
print("   • Assembled context with token budgets")
print("   • Retrieved and ranked relevant chunks")
print("   • Generated formatted context for LLM prompts")
print()

print("✅ Integrated Workflow:")
print("   • Combined RAG + Memory + Context + Agent")
print("   • Created feedback loop: conversation → memory → agent")
print("   • Demonstrated production-ready AI architecture")
print()

print("🔍 Key Concepts Learned:")
print("   • RAG enables AI to query external knowledge bases")
print("   • Episodic memory tracks conversation history")
print("   • Context assembly selects relevant information within token budgets")
print("   • Integration creates powerful, context-aware AI systems")
print()

print("🚀 Next Steps:")
print("   • Explore cookbook/memory-session-isolation.lua for multi-session patterns")
print("   • Try cookbook/context-strategy-comparison.lua for retrieval strategies")
print("   • Learn cookbook/memory-context-workflow.lua for advanced E2E patterns")
print("   • Integrate semantic memory for knowledge graph storage")
print()

print("📚 Related Examples:")
print("   • cookbook/memory-session-isolation.lua - Multi-session memory management")
print("   • cookbook/context-strategy-comparison.lua - Episodic vs Semantic vs Hybrid")
print("   • cookbook/rag-agent-integration.lua - Production RAG patterns")
print()

print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

-- Return comprehensive stats
return {
    success = true,
    message = "Memory, RAG & Context Assembly example completed",
    stats = {
        rag = {
            documents_ingested = ingested_count,
            searches_performed = #search_queries
        },
        memory = {
            exchanges_added = added_count,
            searches_performed = #memory_queries,
            session_id = session_id
        },
        context = {
            chunks_retrieved = #result.chunks,
            token_count = result.token_count,
            confidence = result.total_confidence
        }
    }
}
