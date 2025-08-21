-- Simple Document Generator: Tests state persistence and file generation
-- This example creates a simple document using just 3 agents

-- JSON is available as a global in llmspell
local json = JSON

-- Configuration
local OUTPUT_DIR = arg and arg[1] or "/tmp/doc-gen-test"
local TOPIC = arg and arg[2] or "Building a REST API with Rust"

print("📝 Simple Document Generator")
print("============================")
print("Topic: " .. TOPIC)
print("Output: " .. OUTPUT_DIR)
print("")

-- Check if State is available
if not State then
    error("State global not available. Please enable state persistence in config.")
end

-- Create agents for document generation
print("🤖 Creating Document Generation Agents...")

-- 1. Analyzer Agent - Analyzes the topic
local analyzer = Agent.builder()
    :name("analyzer")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :system_prompt([[You are a technical analyst. 
    Analyze the given topic and provide:
    - Key concepts (3-5 items)
    - Main challenges (2-3 items)  
    - Best practices (3-5 items)
    Output as JSON with keys: concepts, challenges, practices]])
    :build()
print("  ✓ Analyzer Agent")

-- 2. Writer Agent - Writes the content
local writer = Agent.builder()
    :name("writer")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :system_prompt([[You are a technical writer.
    Based on the analysis provided, write a comprehensive README.md document.
    Include sections for Overview, Key Concepts, Best Practices, and Challenges.
    Make it clear and informative with markdown formatting.]])
    :build()
print("  ✓ Writer Agent")

-- 3. Example Agent - Generates code examples
local example_gen = Agent.builder()
    :name("example_gen")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :system_prompt([[You are a code expert.
    Generate 2-3 practical code examples for the topic.
    Each example should have a title, description, and working code.
    Output as JSON with array of examples, each having: title, description, code]])
    :build()
print("  ✓ Example Generator Agent")

-- Create a sequential workflow
print("\n🔄 Creating Sequential Workflow...")

-- Use sequential with table configuration
local workflow = Workflow.sequential({
    name = "doc_workflow",
    steps = {
        {
            name = "analyze",
            type = "agent",
            agent = "analyzer",
            input = TOPIC
        },
        {
            name = "write",
            type = "agent",
            agent = "writer", 
            input = TOPIC .. " Analysis: {{analyze.output}}"
        },
        {
            name = "examples",
            type = "agent",
            agent = "example_gen",
            input = TOPIC
        }
    }
})

print("  ✓ Workflow created with 3 steps")

-- Execute the workflow
print("\n▶️  Executing Document Generation Workflow...")
print("  (This will make real LLM calls and may take 30-60 seconds)")

local start_time = os.time()
local result = workflow:execute({ text = TOPIC })
local end_time = os.time()

print("\n✅ Workflow completed in " .. (end_time - start_time) .. " seconds")

-- Debug: Print what we got back from the workflow
print("\nDEBUG: Result type: " .. type(result))
if result then
    print("DEBUG: Result fields:")
    for k, v in pairs(result) do
        print("  " .. tostring(k) .. " = " .. tostring(v))
    end
end

-- For now, we'll iterate through all state keys to find our outputs
-- This is a workaround until we can properly extract the execution_id
local workflow_id = "unknown"
print("Workflow ID: " .. workflow_id)

-- Collect outputs from State
print("\n📦 Collecting Outputs from State...")

-- Function to find agent output by scanning state keys
local function find_agent_output(agent_name)
    -- Try to list all keys (if State supports it)
    -- For now, we'll try to find outputs by pattern matching
    -- This is a workaround since we don't have the execution_id
    
    -- First, let's try to get all state keys if possible
    print("  Searching for " .. agent_name .. " output...")
    
    -- Since we don't have the execution_id, let's try a different approach
    -- We'll use the result from the workflow execution which should contain the outputs
    if result and result.text then
        print("  ✓ Found output in result for " .. agent_name)
        return result.text
    end
    
    print("  ⚠️  No output found for " .. agent_name)
    return nil
end

-- Since we can't easily get the execution_id, use the workflow result directly
local outputs = {}
if result and result.text then
    -- The final output contains all the generated content
    outputs.analysis = nil  -- We'll need to parse this from the result
    outputs.content = result.text  -- The main content is in the result
    outputs.examples = nil  -- We'll need to parse this from the result
    
    print("  ✓ Retrieved workflow output (" .. string.len(result.text) .. " bytes)")
else
    print("  ⚠️  No workflow output available")
end

-- Generate files
print("\n📁 Generating Document Files...")

-- Create output directory
print("Creating output directory: " .. OUTPUT_DIR)
local dir_result = Tool.invoke("file-manager", {
    operation = "create_dir",
    path = OUTPUT_DIR,
    recursive = true
})

if not dir_result or not dir_result.success then
    print("⚠️  Warning: Could not create output directory")
    print("Error: " .. (dir_result and dir_result.error or "unknown"))
else
    print("  ✓ Output directory created")
    
    -- Write README if we have content
    if outputs.content then
        local readme_path = OUTPUT_DIR .. "/README.md"
        print("  Writing: " .. readme_path)
        
        local write_result = Tool.invoke("file-manager", {
            operation = "write",
            path = readme_path,
            content = outputs.content
        })
        
        if write_result and write_result.success then
            print("  ✓ README.md written")
        else
            print("  ⚠️  Failed to write README.md")
        end
    end
    
    -- Write examples if we have them
    if outputs.examples then
        local examples_path = OUTPUT_DIR .. "/examples.json"
        print("  Writing: " .. examples_path)
        
        local write_result = Tool.invoke("file-manager", {
            operation = "write",
            path = examples_path,
            content = outputs.examples
        })
        
        if write_result and write_result.success then
            print("  ✓ examples.json written")
        else
            print("  ⚠️  Failed to write examples.json")
        end
    end
    
    -- Write analysis if we have it
    if outputs.analysis then
        local analysis_path = OUTPUT_DIR .. "/analysis.json"
        print("  Writing: " .. analysis_path)
        
        local write_result = Tool.invoke("file-manager", {
            operation = "write",
            path = analysis_path,
            content = outputs.analysis
        })
        
        if write_result and write_result.success then
            print("  ✓ analysis.json written")
        else
            print("  ⚠️  Failed to write analysis.json")
        end
    end
    
    -- List generated files
    print("\n📄 Checking Generated Files:")
    local list_result = Tool.invoke("file-manager", {
        operation = "list",
        path = OUTPUT_DIR
    })
    
    if list_result and list_result.success and list_result.result and list_result.result.entries then
        for _, entry in ipairs(list_result.result.entries) do
            print("  - " .. entry.name .. " (" .. (entry.size or 0) .. " bytes)")
        end
    else
        print("  Could not list directory contents")
    end
end

print("\n🎉 Simple Document Generator Complete!")