-- Application: WebApp Creator v2.1 (Focused Test Version)
-- Purpose: Generate web applications using 5 key AI agents for faster testing
-- Architecture: State-based workflow with proper output collection
-- Prerequisites: OPENAI_API_KEY and ANTHROPIC_API_KEY environment variables
-- Expected Output: Complete web application with essential components
-- Version: 2.1.0 (Focused version for testing infrastructure)

print("=== WebApp Creator v2.1 - Focused Test Version ===\n")

-- ============================================================
-- Configuration and Setup
-- ============================================================

local json = JSON  -- Global JSON provided by llmspell

-- Load user input
local input_file = ARGS and ARGS.input or "user-input-ecommerce.lua"

-- If the input file doesn't start with /, try to find it relative to this script
if not input_file:match("^/") then
    -- Try to find the script directory
    local script_dir = "examples/script-users/applications/webapp-creator"
    local possible_paths = {
        input_file,  -- Try current directory first
        script_dir .. "/" .. input_file,  -- Try script directory
        "./" .. input_file,  -- Try explicit current directory
    }
    
    for _, path in ipairs(possible_paths) do
        local file = io.open(path, "r")
        if file then
            file:close()
            input_file = path
            break
        end
    end
end

print("Loading requirements from: " .. input_file)
local user_input = dofile(input_file)

if not user_input or not user_input.requirements then
    error("Failed to load user input from " .. input_file)
end

-- Project configuration
local project_name = user_input.project.name or "webapp_project"
local safe_project_name = project_name:lower():gsub("%s+", "-"):gsub("[^%w%-_]", "")
-- Use absolute path that matches config's allowed_paths
local base_output_dir = ARGS and (ARGS.output or ARGS["output-dir"]) or "/tmp"
-- Convert relative path to absolute if needed
if base_output_dir:sub(1, 1) ~= "/" then
    base_output_dir = "/Users/spuri/projects/lexlapax/rs-llmspell/" .. base_output_dir
end
local project_dir = base_output_dir .. "/" .. safe_project_name

print("📋 Project: " .. project_name)
print("📁 Output: " .. project_dir)
print()

-- ============================================================
-- Helper Functions
-- ============================================================

-- Collect workflow outputs from state
function collect_workflow_outputs(workflow_id, step_names, agent_id_map)
    local outputs = {}
    
    if Debug then
        Debug.info("Collecting outputs for workflow: " .. tostring(workflow_id), "webapp.state")
    end
    
    for _, step_name in ipairs(step_names) do
        -- Use the actual agent ID with timestamp if available
        local actual_agent_id = agent_id_map and agent_id_map[step_name] or step_name
        local key = string.format("workflow:%s:agent:%s:output", workflow_id, actual_agent_id)
        local output = State.get(key)
        
        if Debug then
            if output then
                Debug.debug("Retrieved " .. step_name .. " output from key: " .. key, "webapp.state")
            else
                Debug.warn("No output for " .. step_name .. " at key: " .. key, "webapp.state")
            end
        end
        
        outputs[step_name] = output or ""
    end
    
    return outputs
end

-- File generation helper
function generate_file(path, content)
    Tool.invoke("file_operations", {
        operation = "write",
        path = path,
        input = type(content) == "table" and JSON.stringify(content) or content
    })
    print("  ✅ Generated: " .. path)
end

-- ============================================================
-- Agent Creation (5 Key Agents for Testing)
-- ============================================================

print("🤖 Creating 5 Key Agents for Testing...\n")

local agents = {}
local timestamp = os.time()

-- 1. Requirements Analyst - Analyze and structure requirements
agents.requirements_analyst = Agent.builder()
    :name("requirements_analyst_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a requirements analyst. Extract and structure software requirements from user input.
    Output a JSON object with: functional_requirements, non_functional_requirements, constraints, and priorities.]])
    :build()
print("  1. Requirements Analyst: " .. (agents.requirements_analyst and "✓" or "✗"))

-- 2. System Architect - Create high-level architecture
agents.system_architect = Agent.builder()
    :name("system_architect_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a system architect. Create high-level system architecture.
    Output a JSON object with: components (array), interactions (array), deployment_diagram.]])
    :build()
print("  2. System Architect: " .. (agents.system_architect and "✓" or "✗"))

-- 3. Frontend Developer - Generate frontend code
agents.frontend_developer = Agent.builder()
    :name("frontend_developer_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.4)
    :system_prompt([[You are a frontend developer. Generate React component code.
    Output complete React components with TypeScript, including App.tsx and all child components.]])
    :build()
print("  3. Frontend Developer: " .. (agents.frontend_developer and "✓" or "✗"))

-- 4. Backend Developer - Generate backend code
agents.backend_developer = Agent.builder()
    :name("backend_developer_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a backend developer. Generate backend server code.
    Output complete Node.js/Express server code with all routes, middleware, and database connections.]])
    :build()
print("  4. Backend Developer: " .. (agents.backend_developer and "✓" or "✗"))

-- 5. Documentation Writer - Generate project documentation
agents.documentation_writer = Agent.builder()
    :name("documentation_writer_" .. timestamp)
    :type("llm")
    :model("gpt-3.5-turbo")
    :temperature(0.4)
    :system_prompt([[You are a documentation writer. Generate comprehensive README and docs.
    Output markdown documentation including setup, usage, API reference, and contributing guidelines.]])
    :build()
print("  5. Documentation Writer: " .. (agents.documentation_writer and "✓" or "✗"))

-- ============================================================
-- Workflow Execution with State-Based Collection
-- ============================================================

print("\n📊 Starting Workflow Execution...\n")

-- Create workflow for sequential agent execution
local webapp_workflow = Workflow.builder()
    :name("webapp_creator_focused_workflow")
    :description("Generate web application with 5 key agents")
    :sequential()

-- Add each agent as a workflow step in logical order
local agent_order = {
    "requirements_analyst", "system_architect", "frontend_developer", "backend_developer", "documentation_writer"
}

local agent_names = {}
local agent_ids = {}  -- Store actual agent IDs with timestamps
for _, name in ipairs(agent_order) do
    local agent = agents[name]
    agent_names[#agent_names + 1] = name
    
    -- The agent was created with name "name_timestamp", use that
    local agent_id = name .. "_" .. timestamp
    agent_ids[name] = agent_id  -- Map base name to actual ID
    
    -- Add step - use simple text input instead of complex JSON
    local step_input = user_input.requirements  -- Simple text input
    
    -- Debug: Check all values before add_step
    print(string.format("  Adding step: name='%s', agent='%s', input=%s chars", 
        tostring(name), tostring(agent_id), string.len(step_input)))
    
    webapp_workflow:add_step({
        name = name,
        type = "agent",  -- Required by Lua workflow parser
        agent = agent_id,  -- Use the agent's registered name
        input = step_input  -- Pass simple text input to each agent
    })
end

-- Build and execute workflow
webapp_workflow = webapp_workflow:build()
print("\nExecuting workflow with " .. #agent_names .. " agents...")

-- Workflow expects input with a "text" field for the sequential workflow
local workflow_input = {
    text = user_input.requirements,  -- Pass the requirements as the main text
    context = user_input  -- Pass the full user_input as context
}
local result = webapp_workflow:execute(workflow_input)  -- Pass formatted input

-- ============================================================
-- State-Based Output Collection
-- ============================================================

print("\n📦 Collecting Outputs from State...\n")

-- Debug: show what's in the result
if Debug then
    Debug.debug("Workflow result type: " .. type(result), "webapp.workflow")
    if result then
        for k, v in pairs(result) do
            Debug.debug("Result field: " .. k .. " = " .. tostring(v), "webapp.workflow")
        end
    end
end

if result then
    -- Extract workflow ID from metadata
    local workflow_id = nil
    
    -- Check if metadata exists and has the execution_id in extra
    if result.metadata and type(result.metadata) == "table" then
        if result.metadata.extra and type(result.metadata.extra) == "table" then
            workflow_id = result.metadata.extra.execution_id or result.metadata.extra.workflow_id
            
            -- Also check if there are agent_outputs already collected
            if result.metadata.extra.agent_outputs then
                print("Agent outputs already collected in metadata")
                -- Use the pre-collected outputs if available
            end
        end
    end
    
    -- Fallback to other possible locations
    if not workflow_id then
        workflow_id = result.workflow_id or result.execution_id or result.id
    end
    
    if workflow_id then
        print("Workflow ID: " .. workflow_id)
        
        -- Collect all outputs (or use pre-collected ones from metadata)
        local outputs = nil
        if result.metadata and result.metadata.extra and result.metadata.extra.agent_outputs then
            -- Use pre-collected outputs from metadata
            outputs = result.metadata.extra.agent_outputs
            print("Using pre-collected agent outputs from workflow metadata")
        else
            -- Fallback to manual collection from state
            outputs = collect_workflow_outputs(workflow_id, agent_names, agent_ids)
        end
        
        -- ============================================================
        -- File Generation Pipeline
        -- ============================================================
        
        print("\n📁 Generating Project Files...\n")
        
        -- Create project directory (recursive to create parent dirs)
        Tool.invoke("file_operations", {
            operation = "mkdir",
            path = project_dir,
            recursive = true
        })
        
        -- Generate core files from agent outputs
        generate_file(project_dir .. "/requirements.json", outputs.requirements_analyst)
        generate_file(project_dir .. "/architecture.json", outputs.system_architect)
        
        -- Frontend Code
        Tool.invoke("file_operations", {
            operation = "mkdir",
            path = project_dir .. "/frontend"
        })
        generate_file(project_dir .. "/frontend/src/App.tsx", outputs.frontend_developer)
        generate_file(project_dir .. "/frontend/package.json", {
            name = safe_project_name .. "-frontend",
            version = "1.0.0",
            dependencies = {
                react = "^18.2.0",
                typescript = "^5.0.0"
            }
        })
        
        -- Backend Code
        Tool.invoke("file_operations", {
            operation = "mkdir",
            path = project_dir .. "/backend"
        })
        generate_file(project_dir .. "/backend/src/server.js", outputs.backend_developer)
        generate_file(project_dir .. "/backend/package.json", {
            name = safe_project_name .. "-backend",
            version = "1.0.0",
            dependencies = {
                express = "^4.18.0",
                postgresql = "^14.0.0"
            }
        })
        
        -- Documentation
        generate_file(project_dir .. "/README.md", outputs.documentation_writer)
        
        print("\n✅ WebApp Generation Complete!")
        print("📁 Project generated at: " .. project_dir)
        print("\n📊 Files Generated:")
        
        -- Count and show files generated
        local files_generated = {
            project_dir .. "/requirements.json",
            project_dir .. "/architecture.json", 
            project_dir .. "/frontend/src/App.tsx",
            project_dir .. "/frontend/package.json",
            project_dir .. "/backend/src/server.js",
            project_dir .. "/backend/package.json",
            project_dir .. "/README.md"
        }
        
        for i, file in ipairs(files_generated) do
            print("  " .. i .. ". " .. file)
        end
        
    else
        print("❌ No workflow_id found in result")
    end
else
    print("❌ Workflow execution failed")
    if result and result.error then
        print("Error: " .. tostring(result.error))
    end
end

print("\n=== WebApp Creator v2.1 Focused Test Complete ===")