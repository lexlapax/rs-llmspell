-- tools-utility.lua
-- Examples for utility tools (refactored with llmspell-utils)
-- Using direct Tool API
-- NOTE: All utility tools have been migrated to use 'input' parameter in v0.3.0:
-- ✅ HashCalculatorTool: 'data' → 'input'
-- ✅ TextManipulatorTool: 'text' → 'input'
-- ✅ CalculatorTool: 'expression' → 'input'
-- ✅ TemplateEngineTool: 'template' → 'input'
-- ✅ DataValidationTool: 'data' → 'input'

print("🔧 Utility Tools Examples")
print("=========================")

-- Helper function to execute tool using synchronous API
local function use_tool(tool_name, params)
    local result = Tool.invoke(tool_name, params)
    
    -- Tool.invoke now returns structured results directly (no JSON parsing needed)
    if result then
        return result
    end
    
    -- Return error result if no result
    return {success = false, error = "Tool returned no result"}
end

-- Helper to print clean results
local function print_result(label, result)
    if result.error then
        print("  ❌ " .. label .. ": " .. result.error)
    elseif result.success == false then
        print("  ❌ " .. label .. ": " .. (result.message or "Failed"))
    else
        -- Result is already parsed by execute_tool, extract relevant field
        local value = nil
        if result.result then
            -- Extract the most relevant field from result
            value = result.result.uuid or 
                    result.result.output or 
                    result.result.hash or 
                    result.result.result or 
                    result.result.datetime or 
                    result.result.formatted or 
                    result.result.value or 
                    result.result.valid or
                    result.result.encoded or
                    result.result.decoded or
                    result.result.rendered
        elseif result.message then
            value = result.message
        elseif result.output then
            value = result.output
        end
        print("  ✅ " .. label .. ": " .. tostring(value))
    end
end

print("UUID Generator Tool")

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

print("Base64 Encoder Tool")

-- Encode and decode text
local original = "Hello, LLMSpell! 🚀"
print("\nBase64 encoding/decoding:")
print("  Original: " .. original)

local encoded = use_tool("base64_encoder", {
    operation = "encode",
    input = original
})
print_result("Encoded", encoded)

-- Parse the encoded result to get the output
local encoded_parsed, _ = encoded
if encoded_parsed and encoded_parsed.result and encoded_parsed.result.output then
    local decoded = use_tool("base64_encoder", {
        operation = "decode",
        input = encoded_parsed.result.output
    })
    print_result("Decoded", decoded)
end

-- URL-safe encoding
local url_safe = use_tool("base64_encoder", {
    operation = "encode",
    input = "special/chars+in=url",
    variant = "url-safe"
})
print_result("URL-safe", url_safe)

print("Hash Calculator Tool")

-- Different hash algorithms
local text = "LLMSpell Phase 2 Security"
print("\nCalculating hashes for: " .. text)

local md5_hash = use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "md5",
    input = text  -- Changed from 'data' to 'input'
})
print_result("MD5", md5_hash)

local sha256_hash = use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "sha256",
    input = text  -- Changed from 'data' to 'input'
})
print_result("SHA256", sha256_hash)

-- Verify hash (if we got a valid hash)
local sha256_parsed, _ = sha256_hash
if sha256_parsed and sha256_parsed.result and sha256_parsed.result.hash then
    local verify_result = use_tool("hash_calculator", {
        operation = "verify",
        algorithm = "sha256",
        input = text,  -- Changed from 'data' to 'input'
        expected_hash = sha256_parsed.result.hash
    })
    print_result("Verification", verify_result)
end

print("Text Manipulator Tool")

local sample_text = "hello world from llmspell"
print("\nText operations on: " .. sample_text)

-- Case operations
local uppercase = use_tool("text_manipulator", {
    operation = "uppercase",
    input = sample_text  -- Changed from 'text' to 'input'
})
print_result("Uppercase", uppercase)

local lowercase = use_tool("text_manipulator", {
    operation = "lowercase",
    input = "HELLO WORLD"  -- Changed from 'text' to 'input'
})
print_result("Lowercase", lowercase)

-- Case conversion
local snake_case = use_tool("text_manipulator", {
    operation = "snake_case",
    input = "HelloWorldFromLLMSpell"  -- Changed from 'text' to 'input'
})
print_result("Snake case", snake_case)

local camel_case = use_tool("text_manipulator", {
    operation = "camel_case",
    input = "hello_world_from_llmspell"  -- Changed from 'text' to 'input'
})
print_result("Camel case", camel_case)

-- Text operations
local reversed = use_tool("text_manipulator", {
    operation = "reverse",
    input = "LLMSpell"
})
print_result("Reversed", reversed)

local replaced = use_tool("text_manipulator", {
    operation = "replace",
    input = "Hello World",
    options = {
        from = "World",
        to = "LLMSpell"
    }
})
print_result("Replaced", replaced)

-- Substring extraction
local substring = use_tool("text_manipulator", {
    operation = "substring",
    input = "LLMSpell is awesome",
    options = {
        start = 0,
        ["end"] = 8  -- 'end' is a Lua keyword, so we use bracket notation
    }
})
print_result("Substring", substring)

print("Calculator Tool")

-- Basic arithmetic
print("\nCalculations:")
local basic_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "2 + 3 * 4 - 1"  -- Changed from 'expression' to 'input'
})
print_result("2 + 3 * 4 - 1", basic_calc)

-- With variables
local var_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "x^2 + y^2",  -- Changed from 'expression' to 'input'
    variables = {
        x = 3,
        y = 4
    }
})
print_result("x^2 + y^2 (x=3, y=4)", var_calc)

-- Complex expression
local complex_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "(10 + 20) * 3 / 2"  -- Changed from 'expression' to 'input'
})
print_result("(10 + 20) * 3 / 2", complex_calc)

-- Mathematical functions
print("\nMathematical functions:")

-- Trigonometry
local trig_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "sin(pi()/2) + cos(0) + tan(pi()/4)"  -- Changed from 'expression' to 'input'
})
print_result("sin(π/2) + cos(0) + tan(π/4)", trig_calc)

-- Square root and power
local sqrt_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "sqrt(25) + sqrt(16) + 2^3"  -- Changed from 'expression' to 'input'
})
print_result("sqrt(25) + sqrt(16) + 2^3", sqrt_calc)

-- Exponential and logarithm
local exp_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "exp(1) + ln(e()) + log(10, 100)"  -- Changed from 'expression' to 'input'
})
print_result("exp(1) + ln(e) + log₁₀(100)", exp_calc)

-- Combined with variables
local advanced_calc = use_tool("calculator", {
    operation = "evaluate",
    input = "sqrt(x^2 + y^2) * sin(theta)",  -- Changed from 'expression' to 'input'
    variables = {
        x = 3,
        y = 4,
        theta = 0.927  -- ~53.13 degrees in radians
    }
})
print_result("sqrt(x²+y²) * sin(θ) where x=3, y=4, θ=0.927", advanced_calc)

-- List available functions
local functions_list = use_tool("calculator", {
    operation = "functions"
})
local functions_parsed, _ = functions_list
if functions_parsed and functions_parsed.result then
    print("\nAvailable mathematical functions:")
    if functions_parsed.result.trigonometric then
        print("  Trigonometric: " .. table.concat(functions_parsed.result.trigonometric, ", "))
    end
    if functions_parsed.result.mathematical then
        print("  Mathematical: " .. table.concat(functions_parsed.result.mathematical, ", "))
    end
    if functions_parsed.result.constants then
        print("  Constants: " .. table.concat(functions_parsed.result.constants, ", "))
    end
end

print("Date Time Handler Tool")

-- Current date and time
print("\nDate/Time operations:")
local now = use_tool("date_time_handler", {
    operation = "now"
})
print_result("Current time", now)

-- Parse date
local parsed = use_tool("date_time_handler", {
    operation = "parse",
    input = "2024-12-25T10:30:00Z"
})
print_result("Parsed date", parsed)

-- Date arithmetic
local future_date = use_tool("date_time_handler", {
    operation = "add",
    input = "2024-01-01T00:00:00Z",
    amount = 30,
    unit = "days"
})
print_result("30 days from 2024-01-01", future_date)

print("Diff Calculator Tool")

-- Text diff
local old_text = "The quick brown fox\njumps over the lazy dog"
local new_text = "The quick brown fox\njumps over the lazy cat\nAnd runs away"

print("\nText diff:")
local unified_diff = use_tool("diff_calculator", {
    old_text = old_text,
    new_text = new_text,
    format = "unified"
})
local diff_parsed, _ = unified_diff
if diff_parsed and diff_parsed.result and diff_parsed.result.output then
    print(diff_parsed.result.output)
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
    type = "json",
    old_json = old_json,
    new_json = new_json
})
local json_diff_parsed, _ = json_diff
if json_diff_parsed and json_diff_parsed.result and json_diff_parsed.result.output then
    print(json_diff_parsed.result.output)
else
    print_result("JSON diff", json_diff)
end

print("Data Validation Tool")

-- Email validation
print("\nValidating data:")
local email_validation = use_tool("data_validation", {
    input = {  -- Changed from 'data' to 'input'
        email = "user@example.com",
        age = 25,
        name = "John Doe"
    },
    rules = {
        rules = {
            {type = "required"},
            {type = "type", expected = "object"}
        }
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
    input = complex_data,  -- Changed from 'data' to 'input'
    rules = {
        rules = {
            {type = "required"},
            {type = "type", expected = "object"}
        }
    }
})
print_result("Complex validation", complex_validation)

print("Template Engine Tool")

-- Handlebars template
print("\nTemplate rendering:")
local handlebars_result = use_tool("template_engine", {
    input = "Hello, {{name}}! You have {{count}} new messages.",  -- Changed from 'template' to 'input'
    context = {
        name = "Alice",
        count = 5
    },
    engine = "handlebars"
})
print_result("Handlebars", handlebars_result)

-- Tera template  
local tera_result = use_tool("template_engine", {
    input = "Welcome, {{ name }}! Your score is {{ score }}.",  -- Changed from 'template' to 'input'
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