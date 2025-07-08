-- utility-tools-examples.lua
-- Focused examples for utility tools (refactored with llmspell-utils)

print("🔧 Utility Tools Examples")
print("=========================")

local Agent = require("llmspell.agent")
local agent = Agent.create("claude-3-sonnet-20240229")

print("\n1. UUID Generator Tool")
print("----------------------")

-- Generate different UUID versions
local uuid_v4 = agent:use_tool("uuid_generator", {
    operation = "generate",
    version = "v4",
    format = "hyphenated"
})
print("UUID v4:", uuid_v4)

local uuid_v5 = agent:use_tool("uuid_generator", {
    operation = "generate",
    version = "v5",
    namespace = "dns",
    name = "example.com"
})
print("UUID v5:", uuid_v5)

-- Generate component ID
local component_id = agent:use_tool("uuid_generator", {
    operation = "component_id",
    prefix = "tool",
    short = false
})
print("Component ID:", component_id)

-- Generate deterministic ID
local deterministic_id = agent:use_tool("uuid_generator", {
    operation = "deterministic",
    namespace = "agent",
    name = "my-agent"
})
print("Deterministic ID:", deterministic_id)

print("\n2. Base64 Encoder Tool")
print("----------------------")

-- Encode and decode text
local original = "Hello, LLMSpell! 🚀"
local encoded = agent:use_tool("base64_encoder", {
    operation = "encode",
    input = original
})
print("Original:", original)
print("Encoded:", encoded)

local decoded = agent:use_tool("base64_encoder", {
    operation = "decode",
    input = encoded
})
print("Decoded:", decoded)

-- URL-safe encoding
local url_safe = agent:use_tool("base64_encoder", {
    operation = "encode",
    input = "special/chars+in=url",
    url_safe = true
})
print("URL-safe encoded:", url_safe)

print("\n3. Hash Calculator Tool")
print("-----------------------")

-- Different hash algorithms
local text = "LLMSpell Phase 2 Security"

local md5_hash = agent:use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "md5",
    data = text
})
print("MD5:", md5_hash)

local sha256_hash = agent:use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "sha256",
    data = text
})
print("SHA256:", sha256_hash)

-- Verify hash
local verify_result = agent:use_tool("hash_calculator", {
    operation = "verify",
    algorithm = "sha256",
    data = text,
    expected_hash = sha256_hash
})
print("Hash verification:", verify_result)

print("\n4. Text Manipulator Tool")
print("------------------------")

local sample_text = "hello world from llmspell"

-- Case operations
local uppercase = agent:use_tool("text_manipulator", {
    operation = "uppercase",
    text = sample_text
})
print("Uppercase:", uppercase)

local lowercase = agent:use_tool("text_manipulator", {
    operation = "lowercase",
    text = "HELLO WORLD"
})
print("Lowercase:", lowercase)

-- Case conversion
local snake_case = agent:use_tool("text_manipulator", {
    operation = "snake_case",
    text = "HelloWorldFromLLMSpell"
})
print("Snake case:", snake_case)

local camel_case = agent:use_tool("text_manipulator", {
    operation = "camel_case",
    text = "hello_world_from_llmspell"
})
print("Camel case:", camel_case)

-- Text operations
local reversed = agent:use_tool("text_manipulator", {
    operation = "reverse",
    text = "LLMSpell"
})
print("Reversed:", reversed)

local replaced = agent:use_tool("text_manipulator", {
    operation = "replace",
    text = "Hello World",
    options = {
        from = "World",
        to = "LLMSpell"
    }
})
print("Replaced:", replaced)

-- Substring extraction
local substring = agent:use_tool("text_manipulator", {
    operation = "substring",
    text = "LLMSpell is awesome",
    options = {
        start = 0,
        ["end"] = 8
    }
})
print("Substring:", substring)

print("\n5. Calculator Tool")
print("------------------")

-- Basic arithmetic
local basic_calc = agent:use_tool("calculator", {
    operation = "evaluate",
    expression = "2 + 3 * 4 - 1"
})
print("2 + 3 * 4 - 1 =", basic_calc)

-- With variables
local var_calc = agent:use_tool("calculator", {
    operation = "evaluate",
    expression = "x^2 + y^2",
    variables = {
        x = 3,
        y = 4
    }
})
print("x^2 + y^2 (x=3, y=4) =", var_calc)

-- Validate expression
local validation = agent:use_tool("calculator", {
    operation = "validate",
    expression = "2 + 3 * (4 + 5)"
})
print("Expression validation:", validation)

-- Get available functions
local functions = agent:use_tool("calculator", {
    operation = "functions"
})
print("Available functions:", functions)

print("\n6. Date Time Handler Tool")
print("-------------------------")

-- Current date and time
local now = agent:use_tool("date_time_handler", {
    operation = "now"
})
print("Current time:", now)

-- Parse date
local parsed = agent:use_tool("date_time_handler", {
    operation = "parse",
    input = "2024-12-25T10:30:00Z"
})
print("Parsed date:", parsed)

-- Format date
local formatted = agent:use_tool("date_time_handler", {
    operation = "format",
    input = "2024-12-25T10:30:00Z",
    format = "%Y-%m-%d %H:%M:%S"
})
print("Formatted date:", formatted)

-- Date arithmetic
local future_date = agent:use_tool("date_time_handler", {
    operation = "add",
    input = "2024-01-01T00:00:00Z",
    amount = 30,
    unit = "days"
})
print("30 days from 2024-01-01:", future_date)

print("\n7. Diff Calculator Tool")
print("-----------------------")

-- Text diff
local old_text = "The quick brown fox\njumps over the lazy dog"
local new_text = "The quick brown fox\njumps over the lazy cat\nAnd runs away"

local unified_diff = agent:use_tool("diff_calculator", {
    old_text = old_text,
    new_text = new_text,
    format = "unified"
})
print("Unified diff:", unified_diff)

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

local json_diff = agent:use_tool("diff_calculator", {
    type = "json",
    old_json = old_json,
    new_json = new_json
})
print("JSON diff:", json_diff)

print("\n8. Data Validation Tool")
print("-----------------------")

-- Email validation
local email_validation = agent:use_tool("data_validation", {
    data = {
        email = "user@example.com",
        age = 25,
        name = "John Doe"
    },
    rules = {
        rules = {
            {field = "email", type = "email"},
            {field = "age", type = "number", min = 18, max = 100},
            {field = "name", type = "string", min_length = 2}
        }
    }
})
print("Email validation:", email_validation)

-- Complex validation
local complex_validation = agent:use_tool("data_validation", {
    data = {
        user = {
            profile = {
                username = "alice123",
                email = "alice@example.com"
            },
            settings = {
                notifications = true,
                theme = "dark"
            }
        }
    },
    rules = {
        rules = {
            {field = "user.profile.username", type = "string", min_length = 3},
            {field = "user.profile.email", type = "email"},
            {field = "user.settings.notifications", type = "boolean"}
        }
    }
})
print("Complex validation:", complex_validation)

print("\n9. Template Engine Tool")
print("-----------------------")

-- Handlebars template
local handlebars_result = agent:use_tool("template_engine", {
    template = "Hello, {{name}}! You have {{count}} new {{#if (gt count 1)}}messages{{else}}message{{/if}}.",
    context = {
        name = "Alice",
        count = 5
    },
    engine = "handlebars"
})
print("Handlebars result:", handlebars_result)

-- Tera template
local tera_result = agent:use_tool("template_engine", {
    template = "Welcome, {{ name | title }}! Your score is {{ score | round }}.",
    context = {
        name = "bob",
        score = 85.67
    },
    engine = "tera"
})
print("Tera result:", tera_result)

print("\n✅ All utility tools demonstrated successfully!")
print("These tools use shared utilities from llmspell-utils for consistency.")

return {
    tools_demonstrated = 9,
    categories = "utility",
    shared_utilities = true,
    status = "success"
}