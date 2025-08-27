-- RAG End-to-End Performance Benchmark
-- Measures performance of various RAG operations

print("=== RAG Performance Benchmark ===")
print("")

-- Configuration
local INGEST_COUNT = 500      -- Number of documents to ingest
local SEARCH_COUNT = 100      -- Number of searches to perform
local BATCH_SIZE = 50          -- Batch size for operations
local TOP_K = 10               -- Results per search

-- Helper functions
local function format_time(seconds)
    if seconds < 0.001 then
        return string.format("%.3f μs", seconds * 1000000)
    elseif seconds < 1 then
        return string.format("%.3f ms", seconds * 1000)
    else
        return string.format("%.3f s", seconds)
    end
end

local function benchmark_operation(name, fn, count)
    collectgarbage("collect")
    local start = os.clock()
    
    for i = 1, count do
        fn(i)
    end
    
    local total_time = os.clock() - start
    local avg_time = total_time / count
    
    print(string.format("%s:", name))
    print(string.format("  Total: %s", format_time(total_time)))
    print(string.format("  Average: %s", format_time(avg_time)))
    print(string.format("  Ops/sec: %.0f", 1/avg_time))
    
    return {
        total = total_time,
        average = avg_time,
        ops_per_sec = 1/avg_time
    }
end

-- Check RAG availability
if not RAG then
    error("RAG not available - ensure config has rag.enabled=true")
end

print("Starting benchmark with:")
print(string.format("  Documents: %d", INGEST_COUNT))
print(string.format("  Searches: %d", SEARCH_COUNT))
print(string.format("  Batch size: %d", BATCH_SIZE))
print("")

-- Benchmark 1: Document Ingestion
print("1. Document Ingestion Benchmark")
print("-" .. string.rep("-", 40))

local doc_ids = {}
local ingest_stats = benchmark_operation(
    "Single document ingestion",
    function(i)
        local id = RAG.ingest({
            content = string.format(
                "Document %d: This is a test document with various keywords like " ..
                "machine learning, artificial intelligence, natural language processing, " ..
                "computer vision, deep learning, neural networks, and data science.",
                i
            ),
            metadata = {
                index = i,
                batch = math.floor(i / BATCH_SIZE),
                timestamp = os.time(),
                category = (i % 5 == 0) and "AI" or "Tech"
            }
        })
        table.insert(doc_ids, id)
    end,
    INGEST_COUNT
)

print("")

-- Benchmark 2: Batch Ingestion (if supported)
print("2. Batch Ingestion Benchmark")
print("-" .. string.rep("-", 40))

if RAG.ingest_batch then
    local batch_docs = {}
    for i = 1, BATCH_SIZE do
        table.insert(batch_docs, {
            content = string.format("Batch document %d with content", i),
            metadata = { batch = true, index = i }
        })
    end
    
    local batch_stats = benchmark_operation(
        "Batch ingestion",
        function(i)
            RAG.ingest_batch(batch_docs)
        end,
        5  -- Do 5 batches
    )
    
    print(string.format("  Documents per batch: %d", BATCH_SIZE))
else
    print("  Batch ingestion not supported")
end

print("")

-- Benchmark 3: Search Performance
print("3. Search Performance Benchmark")
print("-" .. string.rep("-", 40))

local search_queries = {
    "machine learning",
    "artificial intelligence",
    "neural networks",
    "data science",
    "deep learning",
    "computer vision",
    "natural language",
    "processing",
    "algorithms",
    "technology"
}

local search_stats = benchmark_operation(
    "Vector search",
    function(i)
        local query = search_queries[(i % #search_queries) + 1]
        local results = RAG.search({
            query = query,
            top_k = TOP_K
        })
    end,
    SEARCH_COUNT
)

print("")

-- Benchmark 4: Filtered Search (if supported)
print("4. Filtered Search Benchmark")
print("-" .. string.rep("-", 40))

local filtered_stats = benchmark_operation(
    "Filtered search",
    function(i)
        local results = RAG.search({
            query = "technology",
            top_k = TOP_K,
            metadata_filter = { category = "AI" }
        })
    end,
    SEARCH_COUNT / 2  -- Fewer filtered searches
)

print("")

-- Benchmark 5: Memory Usage
print("5. Memory and Resource Usage")
print("-" .. string.rep("-", 40))

collectgarbage("collect")
local mem_kb = collectgarbage("count")
print(string.format("  Lua memory usage: %.2f MB", mem_kb / 1024))
print(string.format("  Documents in index: %d", #doc_ids))
print(string.format("  Estimated bytes per doc: %.0f", (mem_kb * 1024) / #doc_ids))

print("")

-- Benchmark 6: Concurrent-like Operations
print("6. Concurrent-like Operations")
print("-" .. string.rep("-", 40))

local concurrent_start = os.clock()
local concurrent_ops = 0

-- Mix of operations
for i = 1, 100 do
    if i % 3 == 0 then
        -- Search
        RAG.search({
            query = "concurrent test",
            top_k = 5
        })
    else
        -- Ingest
        RAG.ingest({
            content = "Concurrent document " .. i,
            metadata = { concurrent = true }
        })
    end
    concurrent_ops = concurrent_ops + 1
end

local concurrent_time = os.clock() - concurrent_start
print(string.format("  Total operations: %d", concurrent_ops))
print(string.format("  Total time: %s", format_time(concurrent_time)))
print(string.format("  Ops/sec: %.0f", concurrent_ops / concurrent_time))

print("")

-- Summary
print("=" .. string.rep("=", 50))
print("BENCHMARK SUMMARY")
print("=" .. string.rep("=", 50))

print(string.format("Ingestion:"))
print(string.format("  - Rate: %.0f docs/sec", ingest_stats.ops_per_sec))
print(string.format("  - Latency: %s per doc", format_time(ingest_stats.average)))

print(string.format("\nSearch:"))
print(string.format("  - Rate: %.0f queries/sec", search_stats.ops_per_sec))
print(string.format("  - Latency: %s per query", format_time(search_stats.average)))

print(string.format("\nFiltered Search:"))
print(string.format("  - Rate: %.0f queries/sec", filtered_stats.ops_per_sec))
print(string.format("  - Latency: %s per query", format_time(filtered_stats.average)))

-- Performance thresholds check
print("\nPerformance Check:")
local all_pass = true

if ingest_stats.average > 0.1 then
    print("  ⚠ Ingestion slower than 100ms threshold")
    all_pass = false
else
    print("  ✓ Ingestion within performance target")
end

if search_stats.average > 0.1 then
    print("  ⚠ Search slower than 100ms threshold")
    all_pass = false
else
    print("  ✓ Search within performance target")
end

if mem_kb / 1024 > 500 then
    print("  ⚠ Memory usage exceeds 500MB")
    all_pass = false
else
    print("  ✓ Memory usage acceptable")
end

print("")
if all_pass then
    print("✅ All performance targets met!")
else
    print("⚠ Some performance targets not met - optimization needed")
end

print("\nBenchmark complete!")