-- tools-utility.lua
-- Examples for utility tools (refactored with llmspell-utils)
-- Using direct Tool API

print("🔧 Utility Tools Examples")
print("=========================")

-- Load test helpers for better output
local TestHelpers = dofile("examples/test-helpers.lua")

-- Helper function to execute tool
local function use_tool(tool_name, params)
    return TestHelpers.execute_tool(tool_name, params)
end

-- Helper to print clean results
local function print_result(label, result)
    if result.error then
        print("  ❌ " .. label .. ": " .. result.error)
    elseif result.success == false then
        print("  ❌ " .. label .. ": " .. (result.message or "Failed"))
    else
        -- Extract the most relevant field from result
        local r = result.result or result
        local value = r.uuid or r.output or r.hash or r.result or r.datetime or 
                     r.formatted or r.value or r.valid or result.message
        print("  ✅ " .. label .. ": " .. tostring(value))
    end
end

TestHelpers.print_section("UUID Generator Tool")

-- Generate different UUID versions
print("\nGenerating UUIDs:")
local uuid_v4 = use_tool("uuid_generator", {
    operation = "generate",
    version = "v4",
    format = "hyphenated"
})
print_result("UUID v4", uuid_v4)

local uuid_v5 = use_tool("uuid_generator", {
    operation = "generate",
    version = "v5",
    namespace = "dns",
    name = "example.com"
})
print_result("UUID v5", uuid_v5)

-- Generate component ID
local component_id = use_tool("uuid_generator", {
    operation = "component_id",
    prefix = "tool",
    short = false
})
print_result("Component ID", component_id)

-- Generate deterministic ID
local deterministic_id = use_tool("uuid_generator", {
    operation = "deterministic",
    namespace = "agent",
    name = "my-agent"
})
print_result("Deterministic ID", deterministic_id)

TestHelpers.print_section("Base64 Encoder Tool")

-- Encode and decode text
local original = "Hello, LLMSpell! 🚀"
print("\nBase64 encoding/decoding:")
print("  Original: " .. original)

local encoded = use_tool("base64_encoder", {
    operation = "encode",
    input = original
})
print_result("Encoded", encoded)

if encoded.result and encoded.result.output then
    local decoded = use_tool("base64_encoder", {
        operation = "decode",
        input = encoded.result.output
    })
    print_result("Decoded", decoded)
end

-- URL-safe encoding
local url_safe = use_tool("base64_encoder", {
    operation = "encode",
    input = "special/chars+in=url",
    variant = "url_safe"
})
print_result("URL-safe", url_safe)

TestHelpers.print_section("Hash Calculator Tool")

-- Different hash algorithms
local text = "LLMSpell Phase 2 Security"
print("\nCalculating hashes for: " .. text)

local md5_hash = use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "md5",
    data = text
})
print_result("MD5", md5_hash)

local sha256_hash = use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "sha256",
    data = text
})
print_result("SHA256", sha256_hash)

-- Verify hash (if we got a valid hash)
if sha256_hash.result and sha256_hash.result.hash then
    local verify_result = use_tool("hash_calculator", {
        operation = "verify",
        algorithm = "sha256",
        data = text,
        expected_hash = sha256_hash.result.hash
    })
    print_result("Verification", verify_result)
end

TestHelpers.print_section("Text Manipulator Tool")

local sample_text = "hello world from llmspell"
print("\nText operations on: " .. sample_text)

-- Case operations
local uppercase = use_tool("text_manipulator", {
    operation = "uppercase",
    text = sample_text
})
print_result("Uppercase", uppercase)

local lowercase = use_tool("text_manipulator", {
    operation = "lowercase",
    text = "HELLO WORLD"
})
print_result("Lowercase", lowercase)

-- Case conversion
local snake_case = use_tool("text_manipulator", {
    operation = "snake_case",
    text = "HelloWorldFromLLMSpell"
})
print_result("Snake case", snake_case)

local camel_case = use_tool("text_manipulator", {
    operation = "camel_case",
    text = "hello_world_from_llmspell"
})
print_result("Camel case", camel_case)

-- Text operations
local reversed = use_tool("text_manipulator", {
    operation = "reverse",
    text = "LLMSpell"
})
print_result("Reversed", reversed)

local replaced = use_tool("text_manipulator", {
    operation = "replace",
    text = "Hello World",
    pattern = "World",
    replacement = "LLMSpell"
})
print_result("Replaced", replaced)

-- Substring extraction
local substring = use_tool("text_manipulator", {
    operation = "substring",
    text = "LLMSpell is awesome",
    start = 0,
    ["end"] = 8
})
print_result("Substring", substring)

TestHelpers.print_section("Calculator Tool")

-- Basic arithmetic
print("\nCalculations:")
local basic_calc = use_tool("calculator", {
    operation = "evaluate",
    expression = "2 + 3 * 4 - 1"
})
print_result("2 + 3 * 4 - 1", basic_calc)

-- With variables
local var_calc = use_tool("calculator", {
    operation = "evaluate",
    expression = "x^2 + y^2",
    variables = {
        x = 3,
        y = 4
    }
})
print_result("x^2 + y^2 (x=3, y=4)", var_calc)

-- Complex expression
local complex_calc = use_tool("calculator", {
    operation = "evaluate",
    expression = "(10 + 20) * 3 / 2"
})
print_result("(10 + 20) * 3 / 2", complex_calc)

TestHelpers.print_section("Date Time Handler Tool")

-- Current date and time
print("\nDate/Time operations:")
local now = use_tool("date_time_handler", {
    operation = "now"
})
print_result("Current time", now)

-- Parse date
local parsed = use_tool("date_time_handler", {
    operation = "parse",
    date_string = "2024-12-25T10:30:00Z"
})
print_result("Parsed date", parsed)

-- Date arithmetic
local future_date = use_tool("date_time_handler", {
    operation = "add",
    date_string = "2024-01-01T00:00:00Z",
    amount = 30,
    unit = "days"
})
print_result("30 days from 2024-01-01", future_date)

TestHelpers.print_section("Diff Calculator Tool")

-- Text diff
local old_text = "The quick brown fox\njumps over the lazy dog"
local new_text = "The quick brown fox\njumps over the lazy cat\nAnd runs away"

print("\nText diff:")
local unified_diff = use_tool("diff_calculator", {
    old_text = old_text,
    new_text = new_text,
    format = "unified"
})
if unified_diff.result and unified_diff.result.output then
    print(unified_diff.result.output)
else
    print_result("Diff", unified_diff)
end

-- JSON diff
local old_json = {
    name = "Alice",
    age = 25,
    city = "New York"
}
local new_json = {
    name = "Alice",
    age = 26,
    city = "San Francisco",
    country = "USA"
}

print("\nJSON diff:")
local json_diff = use_tool("diff_calculator", {
    content_type = "json",
    old_content = old_json,
    new_content = new_json
})
if json_diff.result and json_diff.result.output then
    print(json_diff.result.output)
else
    print_result("JSON diff", json_diff)
end

TestHelpers.print_section("Data Validation Tool")

-- Email validation
print("\nValidating data:")
local email_validation = use_tool("data_validation", {
    data = {
        email = "user@example.com",
        age = 25,
        name = "John Doe"
    },
    rules = {
        {field = "email", rule_type = "email"},
        {field = "age", rule_type = "number", min = 18, max = 100},
        {field = "name", rule_type = "string", min_length = 2}
    }
})
print_result("Email validation", email_validation)

-- Complex validation
local complex_data = {
    username = "alice123",
    email = "alice@example.com",
    notifications = true,
    theme = "dark"
}
local complex_validation = use_tool("data_validation", {
    data = complex_data,
    rules = {
        {field = "username", rule_type = "string", min_length = 3},
        {field = "email", rule_type = "email"},
        {field = "notifications", rule_type = "boolean"}
    }
})
print_result("Complex validation", complex_validation)

TestHelpers.print_section("Template Engine Tool")

-- Handlebars template
print("\nTemplate rendering:")
local handlebars_result = use_tool("template_engine", {
    template = "Hello, {{name}}! You have {{count}} new messages.",
    context = {
        name = "Alice",
        count = 5
    },
    engine = "handlebars"
})
print_result("Handlebars", handlebars_result)

-- Tera template  
local tera_result = use_tool("template_engine", {
    template = "Welcome, {{ name }}! Your score is {{ score }}.",
    context = {
        name = "bob",
        score = 85.67
    },
    engine = "tera"
})
print_result("Tera", tera_result)

print("\n✅ All utility tools demonstrated successfully!")
print("These tools use shared utilities from llmspell-utils for consistency.")

-- Summary
local tool_count = 9
local categories_tested = {
    "UUID Generation",
    "Base64 Encoding",
    "Hash Calculation",
    "Text Manipulation",
    "Mathematical Calculations",
    "Date/Time Handling",
    "Diff Calculation",
    "Data Validation",
    "Template Rendering"
}

print("\n📊 Summary:")
print("  Tools tested: " .. tool_count)
print("  Categories:")
for _, cat in ipairs(categories_tested) do
    print("    - " .. cat)
end

return {
    tools_demonstrated = tool_count,
    categories = "utility",
    shared_utilities = true,
    status = "success"
}