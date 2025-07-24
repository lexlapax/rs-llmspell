-- ABOUTME: Simple single agent test to debug parameter passing
-- ABOUTME: Minimal example to test agent invocation with correct parameters

print("=== Simple Agent Test ===")

-- Create a research agent (like in coordinator example)
local agent = Agent.create({
    name = "research_agent",
    model = "openai/gpt-4o-mini",
    system_prompt = [[
You are a research specialist. You:
1. Gather relevant information on topics
2. Verify facts and sources
3. Summarize findings concisely
4. Identify knowledge gaps
5. Suggest areas for deeper investigation
]],
    temperature = 0.4
})

print("Agent created successfully")

-- Test 1: Simple prompt
print("\nTest 1: Simple prompt")
local result1 = agent:invoke({text = "What are the benefits of smart home technology?"})
if result1 and result1.text then
    print("Response 1:", result1.text)
else
    print("No response or error:", tostring(result1))
end

-- Test 2: Complex prompt with data (like coordinator example)
print("\nTest 2: Complex prompt with structured data")
local market_data = {
    product = "Smart Home Hub",
    current_market_size = "$2.5B",
    growth_rate = "15% annually",
    competitors = {
        "TechCorp (35%)",
        "HomeSmart (25%)",
        "ConnectAll (20%)",
        "Others (20%)"
    }
}

-- Convert market data to string
local market_info = string.format([[
Product: %s
Market Size: %s
Growth Rate: %s
Competitors: %s
]], market_data.product, market_data.current_market_size, market_data.growth_rate, 
table.concat(market_data.competitors, ", "))

local complex_prompt = string.format([[
Research and analyze this smart home market data:

%s

Provide insights on:
1. Market opportunities
2. Competitive landscape
3. Growth potential
4. Key challenges
]], market_info)

local result2 = agent:invoke({text = complex_prompt})
if result2 and result2.text then
    print("Response 2:", result2.text)
else
    print("No response or error:", tostring(result2))
end

print("\n=== Test Complete ===")