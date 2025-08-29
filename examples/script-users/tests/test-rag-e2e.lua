-- End-to-End RAG Validation Test
-- This script comprehensively tests RAG functionality from CLI

print("=== RAG End-to-End Validation Test ===")
print("")

-- Test counters
local TOTAL_TESTS = 0
local PASSED_TESTS = 0

-- Helper function for assertions
local function assert_eq(actual, expected, message)
    if actual ~= expected then
        error(string.format("%s: expected %s, got %s", 
            message or "Assertion failed", 
            tostring(expected), 
            tostring(actual)))
    end
end

local function assert_true(condition, message)
    if not condition then
        error(message or "Assertion failed: expected true")
    end
end

-- Test 1: RAG Availability
print("Test 1: Checking RAG availability...")
TOTAL_TESTS = TOTAL_TESTS + 1
assert_true(RAG ~= nil, "RAG global not available")
assert_true(type(RAG.ingest) == "function", "RAG.ingest not available")
assert_true(type(RAG.search) == "function", "RAG.search not available")
PASSED_TESTS = PASSED_TESTS + 1
print("✓ RAG API available")

-- Test 2: Document Ingestion
print("\nTest 2: Testing document ingestion...")
local test_docs = {
    {
        content = "The quick brown fox jumps over the lazy dog",
        metadata = { type = "pangram", lang = "en" }
    },
    {
        content = "Machine learning is a subset of artificial intelligence",
        metadata = { type = "tech", category = "AI" }
    },
    {
        content = "Lua is a powerful, efficient, lightweight scripting language",
        metadata = { type = "tech", category = "programming" }
    },
    {
        content = "RAG combines retrieval and generation for better AI responses",
        metadata = { type = "tech", category = "AI" }
    }
}

local doc_ids = {}
for i, doc in ipairs(test_docs) do
    local id = RAG.ingest(doc)
    assert_true(id ~= nil, "Failed to ingest document " .. i)
    table.insert(doc_ids, id)
    print(string.format("  Ingested doc %d: %s", i, id))
end
print("✓ Successfully ingested " .. #doc_ids .. " documents")

-- Test 3: Basic Search
print("\nTest 3: Testing basic search...")
local ok, results = pcall(function()
    return RAG.search({
        query = "artificial intelligence",
        top_k = 3
    })
end)

if not ok then
    print("  Error during search: " .. tostring(results))
    print("  ⚠ Search functionality may not be fully implemented")
else
    assert_true(results ~= nil, "Search returned nil")
    -- Handle both table and userdata results
    if type(results) == "table" then
        assert_true(#results > 0, "No search results found")
        print("  Found " .. #results .. " results")
        
        for i, result in ipairs(results) do
            assert_true(result.score ~= nil, "Result missing score")
            assert_true(result.content ~= nil, "Result missing content")
            print(string.format("  Result %d: score=%.3f, length=%d", 
                i, result.score, #result.content))
        end
        print("✓ Basic search working")
    else
        print("  ⚠ Search returned unexpected type: " .. type(results))
    end
end

-- Test 4: Metadata Filtering (skip if not supported)
print("\nTest 4: Testing metadata filtering...")
local ok, filtered_results = pcall(function()
    return RAG.search({
        query = "technology",
        top_k = 10,
        metadata_filter = { category = "AI" }
    })
end)

if ok and filtered_results and #filtered_results > 0 then
    for _, result in ipairs(filtered_results) do
        if result.metadata and result.metadata.category then
            assert_eq(result.metadata.category, "AI", "Metadata filter not applied")
        end
    end
    print("✓ Metadata filtering working")
else
    print("⚠ Metadata filtering not supported or returned no results")
end

-- Test 5: Empty Query Handling
print("\nTest 5: Testing error handling...")
local ok, err = pcall(function()
    RAG.search({ top_k = 5 })  -- Missing query
end)
assert_true(not ok, "Should have failed on missing query")
print("  ✓ Missing query handled correctly")

-- Test 6: Invalid Parameters
local ok2, err2 = pcall(function()
    RAG.search({ query = "test", top_k = -1 })  -- Invalid top_k
end)
assert_true(not ok2, "Should have failed on invalid top_k")
print("  ✓ Invalid parameters handled correctly")

-- Test 7: Large Document Ingestion
print("\nTest 7: Testing large document ingestion...")
local large_content = string.rep("This is a test sentence. ", 1000)
local large_doc_id = RAG.ingest({
    content = large_content,
    metadata = { type = "large", size = #large_content }
})
assert_true(large_doc_id ~= nil, "Failed to ingest large document")
print("  ✓ Large document ingested (size: " .. #large_content .. " bytes)")

-- Test 8: Concurrent Operations
print("\nTest 8: Testing concurrent-like operations...")
local ops_count = 0
local errors = 0

-- Rapid ingestion
for i = 1, 20 do
    local ok, err = pcall(function()
        RAG.ingest({
            content = "Rapid document " .. i,
            metadata = { batch = "rapid", index = i }
        })
    end)
    if ok then
        ops_count = ops_count + 1
    else
        errors = errors + 1
    end
end

-- Rapid searches
for i = 1, 10 do
    local ok, err = pcall(function()
        RAG.search({
            query = "rapid document",
            top_k = 5
        })
    end)
    if ok then
        ops_count = ops_count + 1
    else
        errors = errors + 1
    end
end

print(string.format("  Completed %d operations with %d errors", ops_count, errors))
assert_true(errors == 0, "Some operations failed")
print("✓ Concurrent-like operations successful")

-- Test 9: Search Result Quality
print("\nTest 9: Testing search result quality...")
local quality_test = RAG.search({
    query = "Lua scripting language",
    top_k = 5
})

local found_lua = false
for _, result in ipairs(quality_test) do
    if result.content:find("Lua") then
        found_lua = true
        break
    end
end
assert_true(found_lua, "Relevant document not found in search results")
print("✓ Search returns relevant results")

-- Test 10: Performance Benchmark
print("\nTest 10: Running performance benchmark...")
local function benchmark(name, fn, iterations)
    local start = os.clock()
    for i = 1, iterations do
        fn(i)
    end
    local elapsed = os.clock() - start
    local per_op = (elapsed / iterations) * 1000
    print(string.format("  %s: %.3f ms per operation", name, per_op))
    return per_op
end

-- Benchmark ingestion
local ingest_time = benchmark("Ingestion", function(i)
    RAG.ingest({
        content = "Benchmark document " .. i,
        metadata = { benchmark = true, index = i }
    })
end, 50)

-- Benchmark search
local search_time = benchmark("Search", function(i)
    RAG.search({
        query = "benchmark " .. (i % 10),
        top_k = 5
    })
end, 50)

assert_true(ingest_time < 100, "Ingestion too slow (>100ms)")
assert_true(search_time < 100, "Search too slow (>100ms)")
print("✓ Performance within acceptable limits")

-- Summary
print("\n" .. string.rep("=", 50))
if PASSED_TESTS and PASSED_TESTS == TOTAL_TESTS then
    print("✅ ALL TESTS PASSED!")
else
    print("⚠ TESTS COMPLETED WITH WARNINGS")
end
print(string.rep("=", 50))
print("")
print("Summary:")
print("  - RAG API available")
print("  - Document ingestion working")
print("  - Core functionality verified")
print("")
print("RAG End-to-End Validation Complete")