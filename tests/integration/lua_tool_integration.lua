-- ABOUTME: Integration tests for refactored tools accessible from Lua scripts
-- ABOUTME: Validates tool functionality, error handling, and response formats

local function test_hash_calculator()
    print("Testing HashCalculatorTool...")
    
    -- Test MD5 hash
    local tool = Tool.create("hash-calculator")
    local result = tool:execute({
        parameters = {
            algorithm = "md5",
            input = "Hello, World!"
        }
    })
    
    assert(result.success == true, "MD5 hash should succeed")
    assert(result.result.hash == "65a8e27d8879283831b664bd8b7f0ad4", "MD5 hash mismatch")
    
    -- Test SHA-256 hash
    result = tool:execute({
        parameters = {
            algorithm = "sha256",
            input = "Test data"
        }
    })
    
    assert(result.success == true, "SHA-256 hash should succeed")
    assert(result.result.algorithm == "sha256", "Algorithm should be sha256")
    
    -- Test error handling for invalid algorithm
    local ok, err = pcall(function()
        tool:execute({
            parameters = {
                algorithm = "invalid",
                input = "Test"
            }
        })
    end)
    assert(not ok, "Invalid algorithm should fail")
    
    print("✓ HashCalculatorTool tests passed")
end

local function test_base64_encoder()
    print("Testing Base64EncoderTool...")
    
    local tool = Tool.create("base64-encoder")
    
    -- Test encoding
    local result = tool:execute({
        parameters = {
            operation = "encode",
            input = "Hello, Base64!"
        }
    })
    
    assert(result.success == true, "Base64 encode should succeed")
    assert(result.result.output == "SGVsbG8sIEJhc2U2NCE=", "Base64 encoding mismatch")
    
    -- Test decoding
    result = tool:execute({
        parameters = {
            operation = "decode",
            input = "SGVsbG8sIEJhc2U2NCE="
        }
    })
    
    assert(result.success == true, "Base64 decode should succeed")
    assert(result.result.output == "Hello, Base64!", "Base64 decoding mismatch")
    
    -- Test URL-safe variant
    result = tool:execute({
        parameters = {
            operation = "encode",
            variant = "url-safe",
            input = "Test+/Data"
        }
    })
    
    assert(result.success == true, "URL-safe encoding should succeed")
    assert(not string.find(result.result.output, "+"), "URL-safe should not contain +")
    assert(not string.find(result.result.output, "/"), "URL-safe should not contain /")
    
    print("✓ Base64EncoderTool tests passed")
end

local function test_uuid_generator()
    print("Testing UuidGeneratorTool...")
    
    local tool = Tool.create("uuid-generator")
    
    -- Test UUID v4 generation
    local result = tool:execute({
        parameters = {
            version = "v4",
            count = 5
        }
    })
    
    assert(result.success == true, "UUID generation should succeed")
    assert(#result.result.uuids == 5, "Should generate 5 UUIDs")
    
    -- Validate UUID format
    for _, uuid in ipairs(result.result.uuids) do
        assert(string.match(uuid, "^%x%x%x%x%x%x%x%x%-%x%x%x%x%-%x%x%x%x%-%x%x%x%x%-%x%x%x%x%x%x%x%x%x%x%x%x$"), 
               "Invalid UUID format: " .. uuid)
    end
    
    -- Test nil UUID
    result = tool:execute({
        parameters = {
            version = "nil"
        }
    })
    
    assert(result.success == true, "Nil UUID generation should succeed")
    assert(result.result.uuids[1] == "00000000-0000-0000-0000-000000000000", "Nil UUID mismatch")
    
    print("✓ UuidGeneratorTool tests passed")
end

local function test_text_manipulator()
    print("Testing TextManipulatorTool...")
    
    local tool = Tool.create("text-manipulator")
    
    -- Test case conversion
    local result = tool:execute({
        parameters = {
            operation = "case",
            input = "Hello World",
            format = "upper"
        }
    })
    
    assert(result.success == true, "Case conversion should succeed")
    assert(result.result.output == "HELLO WORLD", "Uppercase conversion failed")
    
    -- Test trim operation
    result = tool:execute({
        parameters = {
            operation = "trim",
            input = "  spaces everywhere  "
        }
    })
    
    assert(result.success == true, "Trim operation should succeed")
    assert(result.result.output == "spaces everywhere", "Trim failed")
    
    -- Test regex replace
    result = tool:execute({
        parameters = {
            operation = "regex",
            input = "foo123bar456",
            pattern = "\\d+",
            replacement = "X"
        }
    })
    
    assert(result.success == true, "Regex replace should succeed")
    assert(result.result.output == "fooXbarX", "Regex replace failed")
    
    print("✓ TextManipulatorTool tests passed")
end

local function test_calculator()
    print("Testing CalculatorTool...")
    
    local tool = Tool.create("calculator")
    
    -- Test arithmetic expression
    local result = tool:execute({
        parameters = {
            operation = "evaluate",
            expression = "2 + 3 * 4"
        }
    })
    
    assert(result.success == true, "Calculation should succeed")
    assert(result.result.result == 14, "Arithmetic evaluation failed: " .. tostring(result.result.result))
    
    -- Test with variables
    result = tool:execute({
        parameters = {
            operation = "evaluate",
            expression = "x^2 + y",
            variables = {
                x = 5,
                y = 10
            }
        }
    })
    
    assert(result.success == true, "Variable calculation should succeed")
    assert(result.result.result == 35, "Variable evaluation failed")
    
    -- Test comparison
    result = tool:execute({
        parameters = {
            operation = "evaluate",
            expression = "10 > 5"
        }
    })
    
    assert(result.success == true, "Comparison should succeed")
    assert(result.result.result == true, "Comparison evaluation failed")
    assert(result.result.result_type == "boolean", "Result type should be boolean")
    
    print("✓ CalculatorTool tests passed")
end

local function test_datetime_handler()
    print("Testing DateTimeHandlerTool...")
    
    local tool = Tool.create("datetime-handler")
    
    -- Test current time
    local result = tool:execute({
        parameters = {
            operation = "now",
            timezone = "UTC"
        }
    })
    
    assert(result.success == true, "Getting current time should succeed")
    assert(result.result.iso8601 ~= nil, "Should return ISO8601 format")
    
    -- Test parsing
    result = tool:execute({
        parameters = {
            operation = "parse",
            input = "2023-12-25T10:30:00Z"
        }
    })
    
    assert(result.success == true, "Parsing should succeed")
    assert(result.result.year == 2023, "Year mismatch")
    assert(result.result.month == 12, "Month mismatch")
    assert(result.result.day == 25, "Day mismatch")
    
    -- Test format conversion
    result = tool:execute({
        parameters = {
            operation = "format",
            input = "2023-12-25T10:30:00Z",
            format = "%Y-%m-%d %H:%M:%S"
        }
    })
    
    assert(result.success == true, "Formatting should succeed")
    assert(result.result.formatted == "2023-12-25 10:30:00", "Format mismatch")
    
    print("✓ DateTimeHandlerTool tests passed")
end

local function test_diff_calculator()
    print("Testing DiffCalculatorTool...")
    
    local tool = Tool.create("diff-calculator")
    
    -- Test text diff
    local result = tool:execute({
        parameters = {
            left = "Line 1\nLine 2\nLine 3",
            right = "Line 1\nLine 2 modified\nLine 3",
            format = "unified"
        }
    })
    
    assert(result.success == true, "Text diff should succeed")
    assert(result.result.has_differences == true, "Should detect differences")
    assert(result.result.summary.modified == 1, "Should have 1 modified line")
    
    -- Test JSON diff
    result = tool:execute({
        parameters = {
            left = '{"a": 1, "b": 2}',
            right = '{"a": 1, "b": 3, "c": 4}',
            format = "json"
        }
    })
    
    assert(result.success == true, "JSON diff should succeed")
    assert(result.result.has_differences == true, "Should detect JSON differences")
    
    print("✓ DiffCalculatorTool tests passed")
end

local function test_data_validation()
    print("Testing DataValidationTool...")
    
    local tool = Tool.create("data-validation")
    
    -- Test JSON schema validation
    local result = tool:execute({
        parameters = {
            data = {name = "John", age = 30},
            schema = {
                type = "object",
                properties = {
                    name = {type = "string"},
                    age = {type = "integer", minimum = 0}
                },
                required = {"name", "age"}
            }
        }
    })
    
    assert(result.success == true, "Validation should succeed")
    assert(result.result.valid == true, "Data should be valid")
    
    -- Test invalid data
    result = tool:execute({
        parameters = {
            data = {name = "John", age = -5},
            schema = {
                type = "object",
                properties = {
                    name = {type = "string"},
                    age = {type = "integer", minimum = 0}
                }
            }
        }
    })
    
    assert(result.success == true, "Validation execution should succeed")
    assert(result.result.valid == false, "Data should be invalid")
    assert(#result.result.errors > 0, "Should have validation errors")
    
    print("✓ DataValidationTool tests passed")
end

local function test_template_engine()
    print("Testing TemplateEngineTool...")
    
    local tool = Tool.create("template-engine")
    
    -- Test simple template
    local result = tool:execute({
        parameters = {
            template = "Hello, {{name}}!",
            data = {name = "World"},
            engine = "handlebars"
        }
    })
    
    assert(result.success == true, "Template rendering should succeed")
    assert(result.result.output == "Hello, World!", "Template output mismatch")
    
    -- Test template with iteration
    result = tool:execute({
        parameters = {
            template = "{{#each items}}{{this}}, {{/each}}",
            data = {items = {"apple", "banana", "orange"}},
            engine = "handlebars"
        }
    })
    
    assert(result.success == true, "Template with iteration should succeed")
    assert(string.find(result.result.output, "apple"), "Should contain apple")
    assert(string.find(result.result.output, "banana"), "Should contain banana")
    
    print("✓ TemplateEngineTool tests passed")
end

local function test_tool_chaining()
    print("Testing tool chaining...")
    
    -- Generate UUID, then hash it
    local uuid_tool = Tool.create("uuid-generator")
    local hash_tool = Tool.create("hash-calculator")
    
    local uuid_result = uuid_tool:execute({
        parameters = {version = "v4", count = 1}
    })
    assert(uuid_result.success == true, "UUID generation should succeed")
    
    local uuid = uuid_result.result.uuids[1]
    local hash_result = hash_tool:execute({
        parameters = {
            algorithm = "sha256",
            input = uuid
        }
    })
    assert(hash_result.success == true, "Hashing UUID should succeed")
    
    -- Encode the hash in base64
    local base64_tool = Tool.create("base64-encoder")
    local encode_result = base64_tool:execute({
        parameters = {
            operation = "encode",
            input = hash_result.result.hash
        }
    })
    assert(encode_result.success == true, "Base64 encoding should succeed")
    
    print("✓ Tool chaining tests passed")
end

local function test_error_propagation()
    print("Testing error propagation...")
    
    -- Test missing required parameters
    local tool = Tool.create("calculator")
    local ok, err = pcall(function()
        tool:execute({
            parameters = {
                operation = "evaluate"
                -- missing expression
            }
        })
    end)
    assert(not ok, "Should fail with missing parameter")
    assert(string.find(tostring(err), "expression") or string.find(tostring(err), "required"), 
           "Error should mention missing expression")
    
    -- Test invalid input
    local hash_tool = Tool.create("hash-calculator")
    ok, err = pcall(function()
        hash_tool:execute({
            parameters = {
                algorithm = "sha256",
                file_path = "/non/existent/file.txt"
            }
        })
    end)
    assert(not ok, "Should fail with non-existent file")
    
    print("✓ Error propagation tests passed")
end

-- Run all tests
local function run_all_tests()
    print("=== Running Lua Tool Integration Tests ===\n")
    
    test_hash_calculator()
    test_base64_encoder()
    test_uuid_generator()
    test_text_manipulator()
    test_calculator()
    test_datetime_handler()
    test_diff_calculator()
    test_data_validation()
    test_template_engine()
    test_tool_chaining()
    test_error_propagation()
    
    print("\n=== All tests passed! ===")
end

-- Execute tests
run_all_tests()