-- RAG Error Handling Validation Test
-- Tests error conditions and recovery

print("=== RAG Error Handling Validation ===")
print("")

local test_count = 0
local pass_count = 0
local fail_count = 0

-- Helper function to run a test
local function test(name, fn)
    test_count = test_count + 1
    print(string.format("Test %d: %s", test_count, name))
    
    local ok, err = pcall(fn)
    if ok then
        pass_count = pass_count + 1
        print("  ✓ PASS")
    else
        fail_count = fail_count + 1
        print("  ✗ FAIL: " .. tostring(err))
    end
    print("")
end

-- Helper to test that something fails
local function should_fail(name, fn, expected_error)
    test_count = test_count + 1
    print(string.format("Test %d: %s (should fail)", test_count, name))
    
    local ok, err = pcall(fn)
    if ok then
        fail_count = fail_count + 1
        print("  ✗ FAIL: Expected error but succeeded")
    else
        pass_count = pass_count + 1
        if expected_error and not string.find(tostring(err), expected_error) then
            print("  ✓ PASS (failed as expected, but different error)")
            print("    Expected: " .. expected_error)
            print("    Got: " .. tostring(err))
        else
            print("  ✓ PASS (failed as expected)")
        end
    end
    print("")
end

-- Check RAG availability
if not RAG then
    error("RAG not available - ensure config has rag.enabled=true")
end

print("Testing error handling scenarios...")
print("")

-- Test 1: Missing required fields
should_fail("Ingest with missing content", function()
    RAG.ingest({
        metadata = { test = true }
        -- Missing content field
    })
end)

should_fail("Ingest with nil content", function()
    RAG.ingest({
        content = nil,
        metadata = { test = true }
    })
end)

should_fail("Ingest with empty content", function()
    RAG.ingest({
        content = "",
        metadata = { test = true }
    })
end)

-- Test 2: Search parameter validation
should_fail("Search with missing query", function()
    RAG.search({
        top_k = 5
        -- Missing query field
    })
end)

should_fail("Search with negative top_k", function()
    RAG.search({
        query = "test",
        top_k = -1
    })
end)

should_fail("Search with zero top_k", function()
    RAG.search({
        query = "test", 
        top_k = 0
    })
end)

should_fail("Search with huge top_k", function()
    RAG.search({
        query = "test",
        top_k = 999999999
    })
end)

-- Test 3: Invalid data types
should_fail("Ingest with number content", function()
    RAG.ingest({
        content = 12345,
        metadata = { test = true }
    })
end)

should_fail("Ingest with table content", function()
    RAG.ingest({
        content = { "this", "is", "wrong" },
        metadata = { test = true }
    })
end)

should_fail("Search with number query", function()
    RAG.search({
        query = 42,
        top_k = 5
    })
end)

should_fail("Search with string top_k", function()
    RAG.search({
        query = "test",
        top_k = "five"
    })
end)

-- Test 4: Multi-tenant errors (if enabled)
test("Multi-tenant validation", function()
    if not RAG.config or not RAG.config.multi_tenant then
        print("    Skipping - multi-tenant not enabled")
        return
    end
    
    -- Try to access another tenant's data
    RAG.ingest({
        content = "Secret tenant data",
        tenant_id = "tenant_a"
    })
    
    local results = RAG.search({
        query = "secret",
        tenant_id = "tenant_b"
    })
    
    -- Should not find the other tenant's data
    assert(#results == 0, "Tenant isolation violated!")
end)

-- Test 5: Recovery after errors
test("Recovery after error", function()
    -- Cause an error
    pcall(function()
        RAG.ingest({ content = nil })
    end)
    
    -- System should still work
    local id = RAG.ingest({
        content = "Recovery test document",
        metadata = { recovery = true }
    })
    
    assert(id ~= nil, "Failed to ingest after error")
    
    local results = RAG.search({
        query = "recovery",
        top_k = 5
    })
    
    assert(results ~= nil, "Search failed after error")
end)

-- Test 6: Boundary conditions
test("Boundary conditions", function()
    -- Very short content
    local id1 = RAG.ingest({
        content = "x",
        metadata = { boundary = "short" }
    })
    assert(id1 ~= nil, "Failed on very short content")
    
    -- Very long content
    local long_content = string.rep("This is a test. ", 10000)
    local id2 = RAG.ingest({
        content = long_content,
        metadata = { boundary = "long" }
    })
    assert(id2 ~= nil, "Failed on very long content")
    
    -- Unicode content
    local id3 = RAG.ingest({
        content = "Testing unicode: 你好世界 🚀 émojis",
        metadata = { boundary = "unicode" }
    })
    assert(id3 ~= nil, "Failed on unicode content")
    
    -- Special characters
    local id4 = RAG.ingest({
        content = "Special chars: !@#$%^&*()_+-=[]{}|;:',.<>?/`~",
        metadata = { boundary = "special" }
    })
    assert(id4 ~= nil, "Failed on special characters")
end)

-- Test 7: Metadata validation
test("Metadata handling", function()
    -- No metadata (should work)
    local id1 = RAG.ingest({
        content = "Document without metadata"
    })
    assert(id1 ~= nil, "Failed with no metadata")
    
    -- Empty metadata (should work)
    local id2 = RAG.ingest({
        content = "Document with empty metadata",
        metadata = {}
    })
    assert(id2 ~= nil, "Failed with empty metadata")
    
    -- Complex metadata
    local id3 = RAG.ingest({
        content = "Document with complex metadata",
        metadata = {
            string_field = "value",
            number_field = 42,
            boolean_field = true,
            nested = {
                field = "nested value"
            },
            array = {1, 2, 3}
        }
    })
    assert(id3 ~= nil, "Failed with complex metadata")
end)

-- Test 8: Concurrent error handling
test("Concurrent operations with errors", function()
    local success_count = 0
    local error_count = 0
    
    for i = 1, 20 do
        if i % 3 == 0 then
            -- Intentionally cause an error
            local ok = pcall(function()
                RAG.ingest({ content = nil })
            end)
            if not ok then
                error_count = error_count + 1
            end
        else
            -- Normal operation
            local ok = pcall(function()
                RAG.ingest({
                    content = "Concurrent doc " .. i
                })
            end)
            if ok then
                success_count = success_count + 1
            else
                error_count = error_count + 1
            end
        end
    end
    
    print(string.format("    Success: %d, Errors: %d", success_count, error_count))
    assert(success_count > 0, "No operations succeeded")
end)

-- Test 9: Invalid filter conditions
should_fail("Search with invalid filter type", function()
    RAG.search({
        query = "test",
        top_k = 5,
        metadata_filter = "not a table"  -- Should be a table
    })
end)

-- Test 10: Resource exhaustion simulation
test("Graceful degradation under load", function()
    -- Try to ingest many documents rapidly
    local ingested = 0
    local failed = 0
    
    for i = 1, 1000 do
        local ok = pcall(function()
            RAG.ingest({
                content = string.rep("Load test document ", 100),
                metadata = { load_test = i }
            })
        end)
        
        if ok then
            ingested = ingested + 1
        else
            failed = failed + 1
            -- System might reject due to rate limiting or resources
            -- This is acceptable behavior
        end
        
        -- Break if we're getting too many failures
        if failed > 100 then
            break
        end
    end
    
    print(string.format("    Ingested: %d, Failed: %d", ingested, failed))
    
    -- System should still be responsive
    local results = RAG.search({
        query = "load test",
        top_k = 5
    })
    assert(results ~= nil, "System unresponsive after load")
end)

-- Summary
print(string.rep("=", 50))
print("ERROR HANDLING VALIDATION RESULTS")
print(string.rep("=", 50))
print(string.format("Total tests: %d", test_count))
print(string.format("Passed: %d", pass_count))
print(string.format("Failed: %d", fail_count))
print("")

if fail_count == 0 then
    print("✅ All error handling tests passed!")
else
    print(string.format("⚠ %d test(s) failed", fail_count))
    os.exit(1)
end