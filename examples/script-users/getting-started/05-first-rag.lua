-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Example ID: 05 - First RAG (Retrieval-Augmented Generation) v0.8.0
-- Complexity Level: BEGINNER
-- Real-World Use Case: Building knowledge bases and semantic search systems
--
-- Purpose: Learn how to use RAG for document ingestion and semantic search.
--          Demonstrates basic RAG operations: ingesting documents, creating
--          embeddings, searching by similarity, and using results with agents.
--          This is your gateway to building AI-powered knowledge systems.
-- Architecture: Vector storage with HNSW algorithm, embedding generation
-- Crates Showcased: llmspell-rag, llmspell-storage, llmspell-bridge
-- Key Features:
--   • Document ingestion with metadata
--   • Vector embedding generation
--   • Semantic similarity search
--   • RAG configuration basics
--   • Integration with agents for augmented responses
--   • Collection management
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • RAG configuration file (see configs/rag-basic.toml)
--   • Embedding provider configured (OpenAI recommended)
--   • Network connectivity for API calls
--
-- HOW TO RUN:
-- ./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
--   run examples/script-users/getting-started/05-first-rag.lua
--
-- EXPECTED OUTPUT:
-- RAG system initialized
-- Documents ingested successfully
-- Search results with relevance scores
-- Agent response augmented with retrieved context
--
-- Time to Complete: <15 seconds
-- ============================================================

print("=== LLMSpell: Your First RAG System ===")
print("Example 05: BEGINNER - Retrieval-Augmented Generation")
print("Showcasing: Document ingestion and semantic search\n")

-- ============================================================
-- Step 1: Check RAG Availability
-- ============================================================

print("1. Checking RAG availability...")
if not RAG then
    print("❌ RAG is not available. Please check your configuration.")
    print("   Ensure you're using a RAG-enabled config file.")
    print("   See examples/script-users/configs/rag-basic.toml")
    return {
        success = false,
        error = "RAG not configured"
    }
end

print("✅ RAG is available")

-- Check available methods
local rag_methods = {}
for name, value in pairs(RAG) do
    if type(value) == "function" then
        table.insert(rag_methods, name)
    end
end
table.sort(rag_methods)
print("   Available methods: " .. table.concat(rag_methods, ", "))

print()

-- ============================================================
-- Step 2: Configure RAG (Optional)
-- ============================================================

print("2. Configuring RAG settings...")

-- Configure RAG with custom settings (optional - configuration happens at initialization)
-- Note: RAG.configure() is currently a placeholder that validates parameters but doesn't change runtime config
RAG.configure({
    collection = "getting_started",  -- Name your collection  
    chunk_size = 512,                -- Size of document chunks
    overlap = 64,                    -- Overlap between chunks
    search_limit = 5                 -- Default number of results
})

print("✅ RAG configuration validated (runtime config set via TOML file)")

print()

-- ============================================================
-- Step 3: Ingest Documents
-- ============================================================

print("3. Ingesting sample documents...")

-- Sample knowledge base about programming languages
local documents = {
    {
        content = "Python is a high-level, interpreted programming language known for its simplicity and readability. It was created by Guido van Rossum and first released in 1991. Python emphasizes code readability with its use of significant indentation. It supports multiple programming paradigms including procedural, object-oriented, and functional programming.",
        metadata = {
            title = "Python Overview",
            category = "programming_language",
            year = 1991
        }
    },
    {
        content = "JavaScript is a dynamic programming language that conforms to the ECMAScript specification. Initially designed for client-side web development, JavaScript has evolved to become a versatile language used for server-side development with Node.js, mobile apps, and desktop applications. It features first-class functions and prototype-based object orientation.",
        metadata = {
            title = "JavaScript Overview",
            category = "programming_language",
            year = 1995
        }
    },
    {
        content = "Rust is a systems programming language focused on safety, speed, and concurrency. Created by Mozilla Research, Rust provides memory safety without garbage collection through its ownership system. It's designed for performance-critical applications and has been voted the most loved programming language in Stack Overflow surveys multiple years running.",
        metadata = {
            title = "Rust Overview",
            category = "programming_language",
            year = 2010
        }
    },
    {
        content = "Machine learning is a subset of artificial intelligence that enables systems to learn and improve from experience without being explicitly programmed. It focuses on developing algorithms that can access data and use it to learn patterns and make decisions. Common applications include recommendation systems, fraud detection, and natural language processing.",
        metadata = {
            title = "Machine Learning Basics",
            category = "ai_ml",
            topic = "fundamentals"
        }
    },
    {
        content = "Neural networks are computing systems inspired by biological neural networks in animal brains. They consist of interconnected nodes (neurons) organized in layers. Deep learning uses neural networks with multiple hidden layers to progressively extract higher-level features from raw input, enabling breakthroughs in computer vision and natural language processing.",
        metadata = {
            title = "Neural Networks and Deep Learning",
            category = "ai_ml",
            topic = "deep_learning"
        }
    }
}

-- Ingest each document
local ingested_count = 0
local failed_count = 0

for i, doc in ipairs(documents) do
    local result = RAG.ingest(doc)
    
    if result and result.success then
        ingested_count = ingested_count + 1
        print(string.format("   ✓ Document %d: %s", i, doc.metadata.title or "Untitled"))
    else
        failed_count = failed_count + 1
        local error_msg = result and result.error or "Unknown error"
        print(string.format("   ✗ Document %d failed: %s", i, error_msg))
    end
end

print(string.format("\n📚 Ingested %d/%d documents successfully", 
    ingested_count, #documents))

if ingested_count == 0 then
    print("❌ No documents were ingested. Cannot continue with search.")
    return {
        success = false,
        error = "Document ingestion failed"
    }
end

print()

-- ============================================================
-- Step 4: Search Documents
-- ============================================================

print("4. Searching the knowledge base...")

-- Test different search queries
local queries = {
    "What programming language emphasizes safety and memory management?",
    "Tell me about interpreted languages",
    "How do neural networks work?",
    "Which language was created in the 1990s?"
}

for i, query in ipairs(queries) do
    print(string.format("\n🔍 Query %d: '%s'", i, query))
    
    -- Perform semantic search
    local search_result = RAG.search(query, {
        limit = 3,  -- Top 3 results
        threshold = 0.5  -- Minimum similarity score
    })
    
    if search_result and search_result.success and search_result.results then
        local results = search_result.results
        
        if #results > 0 then
            print("   Found " .. #results .. " relevant documents:")
            
            for j, result in ipairs(results) do
                print(string.format("   %d. [Score: %.3f] %s",
                    j,
                    result.score or 0,
                    result.metadata and result.metadata.title or "Untitled"
                ))
                
                -- Show snippet of content (first 100 chars)
                if result.content then
                    local snippet = string.sub(result.content, 1, 100)
                    if #result.content > 100 then
                        snippet = snippet .. "..."
                    end
                    print("      " .. snippet)
                end
            end
        else
            print("   No relevant documents found (below threshold)")
        end
    else
        local error_msg = search_result and search_result.error or "Search failed"
        print("   ✗ Search error: " .. error_msg)
    end
end

print()

-- ============================================================
-- Step 5: Use RAG with an Agent (if available)
-- ============================================================

print("5. Combining RAG with Agent for augmented responses...")

-- Try to create an agent directly (requires API key to be set)
if Agent then
    print("   Agent API available, attempting to create agent...")
    
    -- Create an agent using the builder pattern
    local agent = Agent.builder()
        :name("rag_assistant")
        :type("llm")
        :model("openai/gpt-3.5-turbo")  -- Use explicit model
        :system_prompt("You are a helpful assistant. Use the provided context to answer questions accurately.")
        :temperature(0.7)
        :max_tokens(500)
        :build()
    
    if agent then
        print("✅ Agent created successfully")
        
        -- Search for context
        local question = "What makes Rust different from other programming languages?"
        print("\n📝 Question: " .. question)
        
        -- Get relevant context from RAG
        local context_result = RAG.search(question, {
            limit = 2
        })
        
        if context_result and context_result.success and context_result.results then
            -- Build context string
            local context_parts = {}
            for _, result in ipairs(context_result.results) do
                if result.content then
                    table.insert(context_parts, result.content)
                end
            end
            
            local context = table.concat(context_parts, "\n\n")
            
            -- Create augmented prompt
            local augmented_prompt = string.format(
                "Context:\n%s\n\nQuestion: %s\n\nAnswer based on the context provided:",
                context,
                question
            )
            
            print("\n🤖 Agent response with RAG context:")
            
            -- Get agent response using execute method with proper input table
            local response = agent:execute({
                prompt = augmented_prompt
            })
            
            if response then
                -- Extract content from response
                local content = nil
                if type(response) == "string" then
                    content = response
                elseif type(response) == "table" then
                    -- Check various possible fields where content might be
                    content = response.content or 
                             response.result or 
                             response.output or
                             response.text or
                             response.message or
                             "Response received but content format unknown"
                end
                print("   " .. tostring(content))
            else
                print("   ✗ Agent did not return a response")
            end
        else
            print("   ⚠️ Could not retrieve context for augmentation")
        end
    else
        print("   ⚠️ Could not create agent - check API key is set")
    end
else
    print("   ⚠️ No providers configured - skipping agent integration")
    print("   To enable: configure a provider in your config file")
end

print()

-- ============================================================
-- Step 6: Get RAG Statistics
-- ============================================================

print("6. RAG system statistics...")

-- Get stats for the default collection
local stats = RAG.get_stats("default", nil)
if stats then
    print("📊 Current statistics:")
    print("   Total vectors: " .. (stats.total_vectors or 0))
    print("   Storage size (bytes): " .. (stats.total_storage_bytes or 0))
    
    -- Display any additional stats that might be available
    for key, value in pairs(stats) do
        if key ~= "total_vectors" and key ~= "total_storage_bytes" then
            print(string.format("   %s: %s", key, tostring(value)))
        end
    end
else
    print("   ⚠️ Statistics not available")
end

print()

-- ============================================================
-- Summary
-- ============================================================

print("🎉 Congratulations! You've successfully:")
print("   ✓ Initialized the RAG system")
print("   ✓ Ingested " .. ingested_count .. " documents into the knowledge base")
print("   ✓ Performed semantic searches with similarity scoring")
print("   ✓ Retrieved relevant context for questions")
if Agent then
    print("   ✓ Integrated RAG with an agent for augmented responses")
end
print("   ✓ Viewed system statistics")
print()
print("🚀 Next Steps:")
print("   • Try the cookbook RAG examples for advanced patterns")
print("   • Experiment with different embedding models")
print("   • Build your own knowledge base with real documents")
print("   • Explore multi-tenant RAG for isolated collections")
print()
print("Next: Check out cookbook/rag-multi-tenant.lua for advanced RAG patterns!")

-- Return success
return {
    success = true,
    message = "First RAG example completed",
    stats = {
        documents_ingested = ingested_count,
        searches_performed = #queries
    }
}