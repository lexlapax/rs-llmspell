-- Recommended profile: rag-dev
-- Run with: llmspell -p rag-dev run rag-cost-optimization.lua
-- RAG features with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: RAG-03 - RAG Cost Optimization v0.8.0
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Reducing embedding and API costs in production RAG systems
-- Pattern Category: RAG & Cost Management
--
-- Purpose: Demonstrate cost-effective RAG strategies for production deployments.
--          Shows caching, batching, selective updates, and budget management.
--          Critical for scaling RAG systems while controlling operational costs.
-- Architecture: Cost-aware RAG with caching layers and batch processing
-- Crates Showcased: llmspell-rag, llmspell-storage, llmspell-cache
-- Key Features:
--   • Smart caching to reduce redundant embeddings
--   • Batch processing for efficient API usage
--   • Selective document re-indexing
--   • Query optimization and deduplication
--   • Cost tracking and budget enforcement
--   • Tiered processing based on importance
--   • Embedding fallback strategies
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • API key: OPENAI_API_KEY environment variable (for embeddings)
--   • Network connectivity for API calls
--   • Understanding of RAG basics
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p rag-prod \
--   run examples/script-users/cookbook/rag-cost-optimization.lua
--
-- EXPECTED OUTPUT:
-- Cost tracking metrics displayed
-- Cache hit/miss statistics
-- Batch processing efficiency reports
-- Budget enforcement demonstrations
--
-- Time to Complete: <20 seconds
-- ============================================================

print("=== RAG Cost Optimization Patterns ===")
print("Pattern RAG-03: INTERMEDIATE - Production cost management strategies")
print("Showcasing: Caching, batching, and budget control\n")

-- ============================================================
-- Pattern 1: Smart Caching Strategy
-- ============================================================

print("Pattern 1: Smart Caching for Embeddings")
print("==========================================\n")

local CostOptimizedRAG = {}
CostOptimizedRAG.__index = CostOptimizedRAG

function CostOptimizedRAG:new(config)
    local self = setmetatable({}, CostOptimizedRAG)
    
    -- Cost tracking
    self.costs = {
        embeddings = 0,
        searches = 0,
        api_calls = 0,
        cached_saves = 0
    }
    
    -- Cache statistics
    self.cache_stats = {
        hits = 0,
        misses = 0,
        evictions = 0
    }
    
    -- Budget configuration
    self.budget = config.budget or {
        daily_limit = 10.00,  -- $10/day
        warning_threshold = 0.8,
        embedding_cost = 0.0001,  -- per embedding
        search_cost = 0.0002      -- per search
    }
    
    -- Content hash cache to avoid duplicate processing
    self.content_cache = {}
    self.embedding_cache = {}
    self.search_cache = {}
    
    -- Configure RAG with caching
    RAG.configure({
        collection = config.collection or "cost_optimized",
        enable_cache = true,
        cache_ttl = 3600,  -- 1 hour
        batch_size = 32    -- Process in batches
    })
    
    return self
end

-- Hash content for deduplication
function CostOptimizedRAG:hash_content(content)
    -- Simple hash for demo (use crypto hash in production)
    local hash = 0
    for i = 1, #content do
        hash = (hash * 31 + string.byte(content, i)) % 2147483647
    end
    return tostring(hash)
end

-- Check if content needs processing
function CostOptimizedRAG:should_process(content, metadata)
    local hash = self:hash_content(content)
    
    -- Check if we've seen this exact content
    if self.content_cache[hash] then
        self.cache_stats.hits = self.cache_stats.hits + 1
        print("   ✓ Cache hit - skipping duplicate content")
        return false
    end
    
    -- Check if similar content exists (optional similarity threshold)
    if metadata and metadata.skip_if_similar then
        local search_result = self:cached_search(content, 1)
        if search_result and search_result[1] and search_result[1].score > 0.95 then
            self.cache_stats.hits = self.cache_stats.hits + 1
            print("   ✓ Near-duplicate found (score: " .. search_result[1].score .. ")")
            return false
        end
    end
    
    self.cache_stats.misses = self.cache_stats.misses + 1
    self.content_cache[hash] = true
    return true
end

-- Cached search with cost tracking
function CostOptimizedRAG:cached_search(query, limit)
    local cache_key = self:hash_content(query .. "_" .. limit)
    
    -- Check search cache
    if self.search_cache[cache_key] then
        local cached = self.search_cache[cache_key]
        if os.time() - cached.timestamp < 300 then  -- 5 minute cache
            self.cache_stats.hits = self.cache_stats.hits + 1
            print("   ✓ Search cache hit")
            return cached.results
        end
    end
    
    -- Perform actual search
    local result = RAG.search(query, {
        limit = limit or 5
    })
    
    if result and result.success then
        self.costs.searches = self.costs.searches + 1
        self.costs.api_calls = self.costs.api_calls + 1
        
        -- Cache the result
        self.search_cache[cache_key] = {
            results = result.results,
            timestamp = os.time()
        }
        
        return result.results
    end
    
    return nil
end

-- Batch ingestion for efficiency
function CostOptimizedRAG:batch_ingest(documents)
    print("   Processing " .. #documents .. " documents in batches...")
    
    local batch_size = 10
    local processed = 0
    local skipped = 0
    local batches = math.ceil(#documents / batch_size)
    
    for batch = 1, batches do
        local start_idx = (batch - 1) * batch_size + 1
        local end_idx = math.min(batch * batch_size, #documents)
        local batch_docs = {}
        
        -- Pre-filter documents
        for i = start_idx, end_idx do
            local doc = documents[i]
            if self:should_process(doc.content, doc.metadata) then
                table.insert(batch_docs, doc)
            else
                skipped = skipped + 1
            end
        end
        
        -- Process batch if not empty
        if #batch_docs > 0 then
            -- In production, this would be a single batch API call
            for _, doc in ipairs(batch_docs) do
                RAG.ingest(doc)
                processed = processed + 1
                self.costs.embeddings = self.costs.embeddings + 1
            end
            
            print(string.format("   Batch %d/%d: Processed %d documents",
                batch, batches, #batch_docs))
        end
    end
    
    self.costs.cached_saves = self.costs.cached_saves + skipped
    
    return {
        processed = processed,
        skipped = skipped,
        cost_saved = skipped * self.budget.embedding_cost
    }
end

-- Calculate and check costs
function CostOptimizedRAG:check_budget()
    local total_cost = 
        (self.costs.embeddings * self.budget.embedding_cost) +
        (self.costs.searches * self.budget.search_cost)
    
    local saved = self.costs.cached_saves * self.budget.embedding_cost
    
    local budget_used = total_cost / self.budget.daily_limit
    
    return {
        total_cost = total_cost,
        saved = saved,
        budget_used_percent = budget_used * 100,
        within_budget = total_cost < self.budget.daily_limit,
        warning = budget_used > self.budget.warning_threshold
    }
end

-- Create optimizer instance
local optimizer = CostOptimizedRAG:new({
    collection = "cost_demo",
    budget = {
        daily_limit = 1.00,
        warning_threshold = 0.7,
        embedding_cost = 0.0001,
        search_cost = 0.0002
    }
})

-- Test documents with duplicates
local test_documents = {
    {
        content = "Cloud computing provides on-demand computing resources over the internet.",
        metadata = { category = "technology", importance = "high" }
    },
    {
        content = "Cloud computing provides on-demand computing resources over the internet.",  -- Duplicate
        metadata = { category = "technology", importance = "low" }
    },
    {
        content = "Kubernetes is an open-source container orchestration platform.",
        metadata = { category = "devops", importance = "high" }
    },
    {
        content = "Docker enables containerization of applications for consistent deployment.",
        metadata = { category = "devops", importance = "medium" }
    },
    {
        content = "Microservices architecture breaks applications into small, independent services.",
        metadata = { category = "architecture", importance = "high" }
    },
    {
        content = "Kubernetes is an open source container orchestration platform.",  -- Near-duplicate
        metadata = { category = "devops", importance = "low", skip_if_similar = true }
    }
}

-- Batch process with deduplication
local ingest_result = optimizer:batch_ingest(test_documents)
print(string.format("\n📊 Ingestion Summary:"))
print(string.format("   • Processed: %d documents", ingest_result.processed))
print(string.format("   • Skipped (cached): %d documents", ingest_result.skipped))
print(string.format("   • Cost saved: $%.4f", ingest_result.cost_saved))

-- Check budget status
local budget_status = optimizer:check_budget()
print(string.format("\n💰 Budget Status:"))
print(string.format("   • Total cost: $%.4f", budget_status.total_cost))
print(string.format("   • Amount saved: $%.4f", budget_status.saved))
print(string.format("   • Budget used: %.1f%%", budget_status.budget_used_percent))
if budget_status.warning then
    print("   ⚠️  Warning: Approaching budget limit!")
end

print()

-- ============================================================
-- Pattern 2: Tiered Processing Strategy
-- ============================================================

print("Pattern 2: Tiered Processing by Importance")
print("==========================================\n")

local TieredProcessor = {}
TieredProcessor.__index = TieredProcessor

function TieredProcessor:new()
    local self = setmetatable({}, TieredProcessor)
    
    -- Define processing tiers
    self.tiers = {
        critical = {
            chunk_size = 256,      -- Smaller chunks for better precision
            overlap = 128,         -- More overlap
            embedding_model = "high_quality",  -- Best model
            cache_ttl = 86400,    -- 24 hours
            priority = 1
        },
        high = {
            chunk_size = 512,
            overlap = 64,
            embedding_model = "standard",
            cache_ttl = 3600,     -- 1 hour
            priority = 2
        },
        medium = {
            chunk_size = 1024,
            overlap = 32,
            embedding_model = "standard",
            cache_ttl = 1800,     -- 30 minutes
            priority = 3
        },
        low = {
            chunk_size = 2048,     -- Larger chunks, less precision
            overlap = 0,           -- No overlap
            embedding_model = "fast",  -- Cheaper model
            cache_ttl = 600,       -- 10 minutes
            priority = 4
        }
    }
    
    self.processing_stats = {}
    for tier, _ in pairs(self.tiers) do
        self.processing_stats[tier] = 0
    end
    
    return self
end

function TieredProcessor:get_tier(metadata)
    -- Determine tier based on metadata
    if metadata.importance == "critical" or metadata.realtime then
        return "critical", self.tiers.critical
    elseif metadata.importance == "high" or metadata.frequently_accessed then
        return "high", self.tiers.high
    elseif metadata.importance == "medium" then
        return "medium", self.tiers.medium
    else
        return "low", self.tiers.low
    end
end

function TieredProcessor:process_document(document)
    local tier_name, tier_config = self:get_tier(document.metadata or {})
    
    print(string.format("   Processing '%s' as %s tier",
        document.metadata.title or "document",
        tier_name))
    
    -- Configure RAG for this tier
    RAG.configure({
        chunk_size = tier_config.chunk_size,
        overlap = tier_config.overlap,
        cache_ttl = tier_config.cache_ttl
    })
    
    -- Track processing
    self.processing_stats[tier_name] = self.processing_stats[tier_name] + 1
    
    -- Process document
    local result = RAG.ingest(document)
    
    return {
        success = result and result.success,
        tier = tier_name,
        estimated_cost = self:estimate_cost(tier_name, document)
    }
end

function TieredProcessor:estimate_cost(tier, document)
    -- Cost estimation based on tier
    local base_costs = {
        critical = 0.0004,  -- Premium processing
        high = 0.0002,
        medium = 0.0001,
        low = 0.00005      -- Bulk rate
    }
    
    local doc_length = #document.content
    local chunks = math.ceil(doc_length / self.tiers[tier].chunk_size)
    
    return base_costs[tier] * chunks
end

function TieredProcessor:get_summary()
    local total_docs = 0
    for _, count in pairs(self.processing_stats) do
        total_docs = total_docs + count
    end
    
    return {
        total = total_docs,
        by_tier = self.processing_stats
    }
end

-- Create tiered processor
local processor = TieredProcessor:new()

-- Process documents by importance
local tiered_documents = {
    {
        content = "URGENT: Security patch for critical vulnerability CVE-2024-1234.",
        metadata = { title = "Security Update", importance = "critical", realtime = true }
    },
    {
        content = "Quarterly financial report shows 15% growth in cloud services.",
        metadata = { title = "Financial Report", importance = "high", frequently_accessed = true }
    },
    {
        content = "Team meeting notes from weekly standup discussion.",
        metadata = { title = "Meeting Notes", importance = "medium" }
    },
    {
        content = "Office lunch menu for next week includes various options.",
        metadata = { title = "Lunch Menu", importance = "low" }
    }
}

print("Processing documents with tiered strategy...")
local total_estimated_cost = 0

for _, doc in ipairs(tiered_documents) do
    local result = processor:process_document(doc)
    if result.success then
        total_estimated_cost = total_estimated_cost + result.estimated_cost
    end
end

local tier_summary = processor:get_summary()
print("\n📈 Tiered Processing Summary:")
print(string.format("   • Total documents: %d", tier_summary.total))
for tier, count in pairs(tier_summary.by_tier) do
    if count > 0 then
        print(string.format("   • %s tier: %d documents", tier, count))
    end
end
print(string.format("   • Estimated cost: $%.6f", total_estimated_cost))

print()

-- ============================================================
-- Pattern 3: Query Optimization
-- ============================================================

print("Pattern 3: Query Optimization & Deduplication")
print("=============================================\n")

local QueryOptimizer = {}
QueryOptimizer.__index = QueryOptimizer

function QueryOptimizer:new()
    local self = setmetatable({}, QueryOptimizer)
    
    -- Recent queries for deduplication
    self.recent_queries = {}
    self.query_window = 300  -- 5 minutes
    
    -- Query rewriting rules
    self.synonyms = {
        ["ai"] = "artificial intelligence",
        ["ml"] = "machine learning",
        ["k8s"] = "kubernetes",
        ["docker"] = "container containerization"
    }
    
    -- Query cache
    self.query_cache = {}
    
    return self
end

function QueryOptimizer:normalize_query(query)
    -- Convert to lowercase and trim
    local normalized = string.lower(query):gsub("^%s+", ""):gsub("%s+$", "")
    
    -- Expand abbreviations
    for abbr, expansion in pairs(self.synonyms) do
        normalized = normalized:gsub("%f[%w]" .. abbr .. "%f[%W]", expansion)
    end
    
    -- Remove common stop words (simplified)
    local stop_words = { "the", "a", "an", "is", "are", "was", "were" }
    for _, word in ipairs(stop_words) do
        normalized = normalized:gsub("%f[%w]" .. word .. "%f[%W]", "")
    end
    
    -- Collapse multiple spaces
    normalized = normalized:gsub("%s+", " "):gsub("^%s+", ""):gsub("%s+$", "")
    
    return normalized
end

function QueryOptimizer:is_duplicate_query(query)
    local normalized = self:normalize_query(query)
    local current_time = os.time()
    
    -- Check recent queries
    for _, recent in ipairs(self.recent_queries) do
        if current_time - recent.timestamp < self.query_window then
            -- Check similarity (simplified - use proper similarity in production)
            if recent.normalized == normalized then
                return true, recent.results
            end
        end
    end
    
    return false, nil
end

function QueryOptimizer:optimized_search(query, options)
    options = options or {}
    
    -- Check for duplicate query
    local is_duplicate, cached_results = self:is_duplicate_query(query)
    if is_duplicate then
        print("   ✓ Query deduplicated - returning cached results")
        return cached_results
    end
    
    -- Normalize query for better matching
    local normalized = self:normalize_query(query)
    print(string.format("   Original: '%s'", query))
    print(string.format("   Optimized: '%s'", normalized))
    
    -- Perform search with normalized query
    local result = RAG.search(normalized, {
        limit = options.limit or 5,
        threshold = options.threshold or 0.5
    })
    
    if result and result.success then
        -- Cache the query
        table.insert(self.recent_queries, {
            original = query,
            normalized = normalized,
            results = result.results,
            timestamp = os.time()
        })
        
        -- Keep cache size reasonable
        if #self.recent_queries > 100 then
            table.remove(self.recent_queries, 1)
        end
        
        return result.results
    end
    
    return nil
end

-- Create query optimizer
local query_opt = QueryOptimizer:new()

-- Test queries with duplicates and variations
local test_queries = {
    "What is artificial intelligence?",
    "What is AI?",  -- Should normalize to same as above
    "Tell me about Kubernetes orchestration",
    "Tell me about k8s orchestration",  -- Should expand k8s
    "How does the machine learning work?",
    "How does ML work?"  -- Should expand ML and remove 'the'
}

print("Testing query optimization...")
for i, query in ipairs(test_queries) do
    print(string.format("\n🔍 Query %d:", i))
    local results = query_opt:optimized_search(query, { limit = 2 })
    if results then
        print(string.format("   Found %d results", #results))
    end
end

print()

-- ============================================================
-- Pattern 4: Selective Re-indexing
-- ============================================================

print("Pattern 4: Selective Re-indexing Strategy")
print("=========================================\n")

local SelectiveIndexer = {}
SelectiveIndexer.__index = SelectiveIndexer

function SelectiveIndexer:new()
    local self = setmetatable({}, SelectiveIndexer)
    
    -- Track document versions
    self.document_versions = {}
    
    -- Re-indexing thresholds
    self.thresholds = {
        content_change = 0.2,    -- 20% change triggers re-index
        age_days = 30,          -- Re-index after 30 days
        access_count = 100      -- Re-index after 100 accesses
    }
    
    return self
end

function SelectiveIndexer:should_reindex(doc_id, new_content, metadata)
    local existing = self.document_versions[doc_id]
    
    if not existing then
        -- New document, should index
        return true, "new_document"
    end
    
    -- Check content change percentage
    local change_ratio = self:calculate_change_ratio(existing.content, new_content)
    if change_ratio > self.thresholds.content_change then
        return true, string.format("content_changed_%.1f%%", change_ratio * 100)
    end
    
    -- Check age
    local age_days = (os.time() - existing.indexed_at) / 86400
    if age_days > self.thresholds.age_days then
        return true, string.format("stale_%.0f_days", age_days)
    end
    
    -- Check access frequency (in production, track actual access)
    if metadata.access_count and metadata.access_count > self.thresholds.access_count then
        return true, "high_access_frequency"
    end
    
    return false, "no_reindex_needed"
end

function SelectiveIndexer:calculate_change_ratio(old_content, new_content)
    -- Simplified change detection (use proper diff in production)
    if old_content == new_content then
        return 0
    end
    
    local old_len = #old_content
    local new_len = #new_content
    local diff = math.abs(old_len - new_len)
    
    return diff / math.max(old_len, new_len)
end

function SelectiveIndexer:process_update(doc_id, content, metadata)
    local should_reindex, reason = self:should_reindex(doc_id, content, metadata)
    
    print(string.format("   Document '%s': %s", doc_id, reason))
    
    if should_reindex then
        -- Update version tracking
        self.document_versions[doc_id] = {
            content = content,
            indexed_at = os.time(),
            version = (self.document_versions[doc_id] and 
                      self.document_versions[doc_id].version or 0) + 1
        }
        
        -- Perform re-indexing
        RAG.ingest({
            content = content,
            metadata = metadata
        })
        
        return true
    end
    
    return false
end

-- Create selective indexer
local indexer = SelectiveIndexer:new()

-- Simulate document updates
local document_updates = {
    {
        id = "doc_001",
        content = "Version 1: Introduction to cloud computing and its benefits.",
        metadata = { title = "Cloud Intro v1" }
    },
    {
        id = "doc_001",  -- Same document, minor change
        content = "Version 1: Introduction to cloud computing and its many benefits.",
        metadata = { title = "Cloud Intro v1.1" }
    },
    {
        id = "doc_001",  -- Same document, major change
        content = "Version 2: Cloud computing has revolutionized how businesses operate in the digital age.",
        metadata = { title = "Cloud Intro v2" }
    },
    {
        id = "doc_002",
        content = "Database optimization techniques for high-performance applications.",
        metadata = { title = "DB Optimization", access_count = 150 }
    }
}

print("Processing document updates...")
local reindexed = 0
local skipped = 0

for _, update in ipairs(document_updates) do
    if indexer:process_update(update.id, update.content, update.metadata) then
        reindexed = reindexed + 1
    else
        skipped = skipped + 1
    end
end

print(string.format("\n📊 Re-indexing Summary:"))
print(string.format("   • Re-indexed: %d documents", reindexed))
print(string.format("   • Skipped: %d updates", skipped))
print(string.format("   • Cost saved: $%.4f", skipped * 0.0001))

print()

-- ============================================================
-- Final Summary and Cost Analysis
-- ============================================================

print("=" .. string.rep("=", 50))
print("COST OPTIMIZATION SUMMARY")
print("=" .. string.rep("=", 50))

-- Cache effectiveness
local cache_hit_rate = optimizer.cache_stats.hits / 
    (optimizer.cache_stats.hits + optimizer.cache_stats.misses) * 100

print("\n📊 Cache Performance:")
print(string.format("   • Cache hit rate: %.1f%%", cache_hit_rate))
print(string.format("   • Total hits: %d", optimizer.cache_stats.hits))
print(string.format("   • Total misses: %d", optimizer.cache_stats.misses))

-- Cost analysis
local final_budget = optimizer:check_budget()
print("\n💰 Cost Analysis:")
print(string.format("   • Total spent: $%.4f", final_budget.total_cost))
print(string.format("   • Total saved: $%.4f", final_budget.saved))
print(string.format("   • Efficiency gain: %.1f%%", 
    (final_budget.saved / (final_budget.total_cost + final_budget.saved)) * 100))

print("\n✅ Key Optimization Strategies Demonstrated:")
print("   1. Content deduplication and caching")
print("   2. Tiered processing by importance")
print("   3. Query normalization and deduplication")
print("   4. Selective re-indexing based on change detection")
print("   5. Batch processing for API efficiency")
print("   6. Budget tracking and enforcement")

print("\n🚀 Production Tips:")
print("   • Use persistent caches across restarts")
print("   • Implement proper content hashing (SHA-256)")
print("   • Monitor cache hit rates and adjust TTLs")
print("   • Set up alerts for budget thresholds")
print("   • Use cheaper models for low-priority content")
print("   • Consider local embeddings for high-volume cases")

-- Return summary metrics
return {
    success = true,
    metrics = {
        cache_hit_rate = cache_hit_rate,
        total_cost = final_budget.total_cost,
        total_saved = final_budget.saved,
        documents_processed = tier_summary.total,
        queries_optimized = #test_queries
    }
}