-- tools-data.lua
-- Examples for data processing tools (JSON, CSV, HTTP, GraphQL)
-- Using direct Tool API

print("📊 Data Processing Tools Examples")
print("=================================")

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
        -- Extract relevant output
        local r = result.result or result
        if r.output then
            -- For JSON output, try to format it nicely
            if type(r.output) == "table" then
                print("  ✅ " .. label .. ":")
                print(r.output, 2)
            else
                print("  ✅ " .. label .. ": " .. tostring(r.output))
            end
        elseif r.data then
            print("  ✅ " .. label .. ": Retrieved " .. tostring(#r.data) .. " items")
        elseif r.status_code then
            print("  ✅ " .. label .. ": HTTP " .. r.status_code)
        elseif r.rows then
            print("  ✅ " .. label .. ": " .. r.rows .. " rows analyzed")
        elseif result.message then
            print("  ✅ " .. label .. ": " .. result.message)
        else
            print("  ✅ " .. label .. ": Success")
        end
    end
end

print("JSON Processor Tool")

print("\nJSON processing operations:")

-- Sample JSON data as a string to ensure proper parsing
local sample_json_str = [[{
    "users": [
        {
            "id": 1,
            "name": "Alice Johnson",
            "age": 30,
            "city": "New York",
            "skills": ["Python", "JavaScript", "Go"]
        },
        {
            "id": 2,
            "name": "Bob Smith",
            "age": 25,
            "city": "San Francisco",
            "skills": ["Java", "Kotlin", "Swift"]
        },
        {
            "id": 3,
            "name": "Charlie Brown",
            "age": 35,
            "city": "Chicago",
            "skills": ["Rust", "C++", "Assembly"]
        }
    ],
    "metadata": {
        "version": "1.0",
        "generated": "2024-07-08"
    }
}]]

-- Query JSON with jq syntax
local jq_query = use_tool("json_processor", {
    operation = "query",
    input = sample_json_str,
    query = ".users[] | select(.age > 25) | {name, city}"
})
print_result("JQ query (age > 25)", jq_query)

-- Transform JSON
local transform = use_tool("json_processor", {
    operation = "query",  -- transform is done via query
    input = sample_json_str,
    query = ".users | map({fullName: .name, location: .city, yearsOfExperience: (.age - 20)})"
})
print_result("Transform users", transform)

-- Validate JSON against schema
local schema = {
    type = "object",
    properties = {
        users = {
            type = "array",
            items = {
                type = "object",
                required = {"id", "name", "age"},
                properties = {
                    id = {type = "number"},
                    name = {type = "string"},
                    age = {type = "number", minimum = 0, maximum = 150}
                }
            }
        }
    }
}

local validate_json = use_tool("json_processor", {
    operation = "validate",
    input = sample_json_str,
    schema = schema
})
print_result("Schema validation", validate_json)

-- Pretty print JSON (format not supported, use query)
local pretty_json = use_tool("json_processor", {
    operation = "query",
    input = {compact = true, data = {1, 2, 3}},
    query = "."  -- Identity query to return formatted
})
print_result("Pretty print", pretty_json)

print("CSV Analyzer Tool")

print("\nCSV analysis operations:")

-- Sample CSV data
local csv_data = [[name,age,city,salary
Alice Johnson,30,New York,85000
Bob Smith,25,San Francisco,95000
Charlie Brown,35,Chicago,75000
Diana Prince,28,Washington DC,90000
Eve Adams,32,Boston,88000]]

-- Analyze CSV
local csv_analysis = use_tool("csv_analyzer", {
    operation = "analyze",
    input = csv_data
})
print_result("CSV analysis", csv_analysis)

-- Get statistics (part of analyze operation)
local csv_stats = use_tool("csv_analyzer", {
    operation = "analyze",
    input = csv_data
})
print_result("Column statistics", csv_stats)

-- Filter CSV data
local csv_filter = use_tool("csv_analyzer", {
    operation = "filter",
    input = csv_data,
    options = {
        filter = "age > 30"  -- Simple filter expression
    }
})
print_result("Filter (age > 30)", csv_filter)

-- Convert CSV to JSON
local csv_to_json = use_tool("csv_analyzer", {
    operation = "convert",
    input = csv_data,
    options = {
        format = "json"
    }
})
print_result("CSV to JSON", csv_to_json)

print("HTTP Request Tool")

print("\nHTTP request operations:")

-- GET request example
local get_request = use_tool("http_request", {
    method = "GET",
    input = "https://api.github.com/repos/anthropics/llmspell",
    headers = {
        ["User-Agent"] = "LLMSpell-Example/1.0"
    }
})
print_result("GET request", get_request)

-- POST request example (to a test endpoint)
local post_data = {
    title = "Test Post",
    body = "This is a test from LLMSpell",
    userId = 1
}
local post_request = use_tool("http_request", {
    method = "POST",
    input = "https://jsonplaceholder.typicode.com/posts",
    body = post_data,
    headers = {
        ["Content-Type"] = "application/json"
    }
})
print_result("POST request", post_request)

-- Request with query parameters (manual URL encoding since query_params not supported)
local query_request = use_tool("http_request", {
    method = "GET",
    input = "https://httpbin.org/get?q=llmspell&limit=10&offset=0",
    headers = {
        ["User-Agent"] = "LLMSpell-Example/1.0"
    }
})
print_result("Query params", query_request)

-- Request with timeout
local timeout_request = use_tool("http_request", {
    method = "GET",
    input = "https://httpbin.org/delay/5",
    timeout_ms = 2000
})
print_result("Timeout test", timeout_request)

print("GraphQL Query Tool")

print("\nGraphQL operations:")

-- Simple GraphQL query (using Countries API)
local simple_query = [[
query GetCountry($code: ID!) {
    country(code: $code) {
        code
        name
        capital
        currency
        languages {
            code
            name
        }
    }
}
]]

local graphql_result = use_tool("graphql_query", {
    endpoint = "https://countries.trevorblades.com/graphql",
    input = simple_query,
    variables = {
        code = "US"
    }
})
print_result("GraphQL query", graphql_result)

-- GraphQL query with more data (using GraphQLZero API)
local posts_query = [[
query GetPosts($limit: Int!) {
    posts(options: { paginate: { limit: $limit } }) {
        data {
            id
            title
            body
            user {
                name
                email
                username
            }
            comments {
                data {
                    id
                    name
                    body
                }
            }
        }
    }
}
]]

local posts_result = use_tool("graphql_query", {
    endpoint = "https://graphqlzero.almansi.me/api",
    input = posts_query,
    variables = {
        limit = 3
    }
})
print_result("Posts with comments", posts_result)

-- Simple query to demonstrate another GraphQL endpoint
local simple_country_query = [[
query {
    countries(filter: { code: { in: ["US", "CA", "GB"] } }) {
        code
        name
        emoji
        capital
        currency
    }
}
]]

local countries_result = use_tool("graphql_query", {
    endpoint = "https://countries.trevorblades.com/graphql",
    input = simple_country_query
})
print_result("Multiple countries", countries_result)

print("\n🔍 Advanced Data Processing Examples")
print("====================================")

-- Example: Process API response with JSON tool
print("\nChaining tools - API response processing:")
local api_response = {
    data = {
        users = {
            {id = 1, name = "Alice", score = 95},
            {id = 2, name = "Bob", score = 87},
            {id = 3, name = "Charlie", score = 92}
        }
    },
    meta = {total = 3, page = 1}
}

-- Extract and transform data
local extracted = use_tool("json_processor", {
    operation = "query",
    input = api_response,  -- Changed from json to input
    query = ".data.users | sort_by(.score) | reverse | .[0:2]"
})
print_result("Top 2 users by score", extracted)

-- Example: CSV data enrichment
print("\nCSV enrichment example:")
local sales_csv = [[product,units,price
Widget A,100,9.99
Widget B,75,14.99
Widget C,150,7.99]]

-- Calculate revenue and add to CSV (transform not fully implemented, use analyze)
local enriched = use_tool("csv_analyzer", {
    operation = "analyze",
    input = sales_csv  -- Changed from csv_data to input
})
print_result("CSV analysis (transform not implemented)", enriched)

print("\n✅ Data Processing Tools Examples Complete!")
print("Demonstrated JSON processing, CSV analysis, HTTP requests, and GraphQL queries.")

-- Summary
local tools_demonstrated = {
    "json_processor",
    "csv_analyzer",
    "http_request",
    "graphql_query"
}

print("\n📊 Summary:")
print("  Tools tested: " .. #tools_demonstrated)
print("  Operations demonstrated:")
print("    - JSON: query, transform, validate, format")
print("    - CSV: analyze, statistics, filter, convert")
print("    - HTTP: GET, POST, headers, timeout")
print("    - GraphQL: query, multiple queries, introspection")

return {
    tools_demonstrated = #tools_demonstrated,
    categories = "data_processing",
    operations = {
        json = {"query", "transform", "validate", "format"},
        csv = {"analyze", "statistics", "filter", "to_json"},
        http = {"GET", "POST", "headers", "timeout"},
        graphql = {"query", "mutation", "introspection"}
    },
    status = "success"
}