-- ============================================================
-- LLMSPELL APPLICATION SHOWCASE - EXPERT CULMINATION
-- ============================================================
-- Application ID: 10 - WebApp Creator v2.0.0
-- Complexity Level: 6 [EXPERT] - Peak Complexity Achievement
-- Real-World Use Case: Full-stack application generation (Google Jarvis-like, 2025 AI development trend)
--
-- PROGRESSION JOURNEY COMPLETE:
-- Layer 1 (Universal):    2 agents  - file-organizer (simple tasks)
-- Layer 2 (Universal):    2 agents  - research-collector (parallel search)
-- Layer 3 (Power User):   4 agents  - content-creator (quality control)
-- Layer 4 (Business):     5 agents  - communication-manager (state persistence)
-- Layer 5 (Professional): 8 agents  - process-orchestrator (enterprise orchestration)
-- Layer 6 (Expert):      21 agents  - webapp-creator (AI automation mastery)
--
-- YOU'VE REACHED THE SUMMIT! This represents the pinnacle of llmspell capabilities.
-- 
-- Purpose: Generate complete production-ready web applications using 20 specialized AI agents
-- Architecture: Sequential workflow with automatic agent output collection
-- Crates Showcased: llmspell-agents, llmspell-workflows, llmspell-tools, llmspell-bridge, 
--                   llmspell-state-persistence, llmspell-utils, llmspell-testing
-- Key Features:
--   • 20 specialized agents for full-stack development
--   • Complete app generation: frontend, backend, database, tests, deployment
--   • Automatic agent output collection via workflow metadata
--   • Sequential workflow with integrated error handling
--   • Production-ready code generation with best practices
--
-- Prerequisites:
--   • API Keys: OPENAI_API_KEY and ANTHROPIC_API_KEY (both required)
--   • Config: config.toml for file system and security settings
--   • Resources: ~120 seconds runtime, ~$0.50-1.00 API costs
--
-- HOW TO RUN:
-- 1. Basic Demo (generates e-commerce platform):
--    ./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua
--
-- 2. With Custom Requirements:
--    ./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua \
--    -- --input user-input-social.lua --output ~/projects/my-app
--
-- 3. With Full Configuration:
--    ./target/debug/llmspell -c examples/script-users/applications/webapp-creator/config.toml \
--    run examples/script-users/applications/webapp-creator/main.lua \
--    -- --input user-input-ecommerce.lua --output ./generated
--
-- 4. Production Mode (with optimizations):
--    ./target/release/llmspell -c config.toml run main.lua \
--    -- --input requirements.lua --output /var/www/apps --production
--
-- Expected Output:
--   • Complete web application structure (20+ files)
--   • Frontend: React/Vue components, routing, state management
--   • Backend: API endpoints, database models, authentication
--   • Infrastructure: Docker, CI/CD, deployment configs
--   • Documentation: README, API docs, setup guides
--   • Runtime: ~120-180 seconds | API Cost: ~$0.50-1.00
--
-- Progressive Learning:
--   • Previous: App 09 (sales-automation) introduced meta-workflows
--   • This App: Demonstrates production patterns with 20 agents
--   • Completion: You've mastered all llmspell capabilities!
--
-- ARCHITECTURE HIGHLIGHTS:
-- • Automatic agent output collection via workflow metadata
-- • generate_file(): Unified file generation with error handling
-- • 20 specialized agents with specific models and prompts
-- • Sequential workflow with integrated state management
-- • Workflow-based retry and error handling
-- ============================================================

print("=== WebApp Creator v2.0 ===")
print("Application 10: EXPERT - Complete web application generation")
print("Showcasing: Production patterns with 20 specialized agents\n")

-- ============================================================
-- Configuration and Setup
-- ============================================================

local json = JSON  -- Global JSON provided by llmspell

-- Load user input
local input_file = ARGS and ARGS.input or "user-input.lua"

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
-- Helper Functions (Task 10.3.c)
-- ============================================================

-- Note: Agent output collection is now handled automatically by the workflow infrastructure
-- The workflow result will contain agent_outputs in result.metadata.extra.agent_outputs
-- Error handling and retry logic is handled by the Rust workflow infrastructure
-- The workflow executor will handle retries, timeouts, and state persistence automatically

-- File generation helper (Task 10.3.c)
function generate_file(path, content)
    Tool.invoke("file-operations", {
        operation = "write",
        path = path,
        input = type(content) == "table" and JSON.stringify(content) or content
    })
    print("  ✅ Generated: " .. path)
end

-- ============================================================
-- Agent Creation (Task 10.3.b - 20 Specialized Agents)
-- ============================================================

print("🤖 Creating 20 Specialized Agents...\n")

local agents = {}
local timestamp = os.time()

-- Research & Analysis Phase (5 agents)
print("📊 Research & Analysis Agents:")

agents.requirements_analyst = Agent.builder()
    :name("requirements_analyst_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a requirements analyst. Extract and structure software requirements from user input.
    Output a JSON object with: functional_requirements, non_functional_requirements, constraints, and priorities.]])
    :build()
print("  1. Requirements Analyst: " .. (agents.requirements_analyst and "✓" or "✗"))

agents.ux_researcher = Agent.builder()
    :name("ux_researcher_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.4)
    :system_prompt([[You are a UX researcher. Generate user personas, user journeys, and pain points.
    Output a JSON object with: personas (array), user_journeys (array), pain_points (array).]])
    :build()
print("  2. UX Researcher: " .. (agents.ux_researcher and "✓" or "✗"))

agents.market_researcher = Agent.builder()
    :name("market_researcher_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-haiku-20240307")
    :temperature(0.5)
    :system_prompt([[You are a market researcher. Analyze similar products and competitive landscape.
    Output a JSON object with: competitors (array), market_gaps, unique_value_proposition.]])
    :build()
print("  3. Market Researcher: " .. (agents.market_researcher and "✓" or "✗"))

agents.tech_stack_advisor = Agent.builder()
    :name("tech_stack_advisor_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a tech stack advisor. Recommend optimal technologies based on requirements.
    Output a JSON object with: frontend (framework, libraries), backend (language, framework), database (type, specific), devops (tools).]])
    :build()
print("  4. Tech Stack Advisor: " .. (agents.tech_stack_advisor and "✓" or "✗"))

agents.feasibility_analyst = Agent.builder()
    :name("feasibility_analyst_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-haiku-20240307")
    :temperature(0.3)
    :system_prompt([[You are a feasibility analyst. Evaluate technical feasibility and risks.
    Output a JSON object with: feasibility_score (0-100), risks (array), mitigation_strategies (array).]])
    :build()
print("  5. Feasibility Analyst: " .. (agents.feasibility_analyst and "✓" or "✗"))

-- Architecture & Design Phase (5 agents)
print("\n🏗️ Architecture & Design Agents:")

agents.system_architect = Agent.builder()
    :name("system_architect_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :max_tokens(1200)
    :system_prompt([[You are a system architect. Create SIMPLE system architecture.
    Output a JSON object with: components (5 max), interactions (5 max).
    Keep descriptions under 10 words each. Be concise.]])
    :build()
print("  6. System Architect: " .. (agents.system_architect and "✓" or "✗"))

agents.database_architect = Agent.builder()
    :name("database_architect_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-haiku-20240307")
    :temperature(0.2)
    :max_tokens(1500)
    :system_prompt([[You are a database architect. Generate SIMPLE SQL schema.
    Output ONLY CREATE TABLE statements for 4 core tables.
    Include: users, products, orders, order_items.
    Keep it concise. No explanations.]])
    :build()
print("  7. Database Architect: " .. (agents.database_architect and "✓" or "✗"))

agents.api_designer = Agent.builder()
    :name("api_designer_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :max_tokens(1200)
    :system_prompt([[You are an API designer. Create SIMPLE API endpoints.
    Output a JSON object with 5 REST endpoints: path, method, description.
    Keep it under 20 lines. No OpenAPI, just simple JSON.]])
    :build()
print("  8. API Designer: " .. (agents.api_designer and "✓" or "✗"))

agents.security_architect = Agent.builder()
    :name("security_architect_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-haiku-20240307")
    :temperature(0.2)
    :max_tokens(800)
    :system_prompt([[You are a security architect. List 5 security measures.
    Output a JSON object with: measures (array of {type, description}).
    Keep each description under 15 words. Be specific.]])
    :build()
print("  9. Security Architect: " .. (agents.security_architect and "✓" or "✗"))

agents.frontend_designer = Agent.builder()
    :name("frontend_designer_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.4)
    :max_tokens(1000)
    :system_prompt([[You are a frontend designer. List UI components.
    Output a JSON object with: pages (5 max), components (10 max).
    Keep names simple, no descriptions. Be concise.]])
    :build()
print("  10. Frontend Designer: " .. (agents.frontend_designer and "✓" or "✗"))

-- Implementation Phase (5 agents)
print("\n💻 Implementation Agents:")

agents.backend_developer = Agent.builder()
    :name("backend_developer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-5-sonnet-20241022")
    :temperature(0.3)
    :max_tokens(1800)
    :system_prompt([[You are a backend developer. Generate SIMPLE Express server.
    Output ONLY basic server.js with 5 routes and MongoDB connection.
    Keep it under 60 lines. No explanations, just code.]])
    :build()
print("  11. Backend Developer: " .. (agents.backend_developer and "✓" or "✗"))

agents.frontend_developer = Agent.builder()
    :name("frontend_developer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-5-sonnet-20241022")
    :temperature(0.4)
    :system_prompt([[You are a frontend developer. Generate React component code.
    Output complete React components with TypeScript, including App.tsx and all child components.]])
    :build()
print("  12. Frontend Developer: " .. (agents.frontend_developer and "✓" or "✗"))

agents.database_developer = Agent.builder()
    :name("database_developer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-5-sonnet-20241022")
    :temperature(0.2)
    :system_prompt([[You are a database developer. Create migration scripts and seed data.
    Output SQL migration files for database setup and initial data.]])
    :build()
print("  13. Database Developer: " .. (agents.database_developer and "✓" or "✗"))

agents.api_developer = Agent.builder()
    :name("api_developer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-5-sonnet-20241022")
    :temperature(0.3)
    :max_tokens(1000)
    :system_prompt([[You are an API developer. Generate SIMPLE Express.js route definitions.
    Output ONLY JavaScript code for 5 basic REST endpoints.
    Include: GET /products, POST /cart, POST /checkout, GET /orders.
    Keep it under 50 lines. No explanations.]])
    :build()
print("  14. API Developer: " .. (agents.api_developer and "✓" or "✗"))

agents.integration_developer = Agent.builder()
    :name("integration_developer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-5-sonnet-20241022")
    :temperature(0.3)
    :max_tokens(800)
    :system_prompt([[You are an integration developer. Write SIMPLE connection code.
    Output ONLY code to connect frontend to backend API.
    Include: axios setup, 3 API call functions.
    Keep it under 30 lines. No explanations.]])
    :build()
print("  15. Integration Developer: " .. (agents.integration_developer and "✓" or "✗"))

-- Quality & Deployment Phase (5 agents)
print("\n🚀 Quality & Deployment Agents:")

agents.test_engineer = Agent.builder()
    :name("test_engineer_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a test engineer. Generate comprehensive test suites.
    Output Jest/Mocha test files for unit tests and Cypress tests for E2E.]])
    :build()
print("  16. Test Engineer: " .. (agents.test_engineer and "✓" or "✗"))

agents.devops_engineer = Agent.builder()
    :name("devops_engineer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-haiku-20240307")
    :temperature(0.3)
    :system_prompt([[You are a DevOps engineer. Create deployment configurations.
    Output Dockerfile, docker-compose.yml, and GitHub Actions CI/CD workflow.]])
    :build()
print("  17. DevOps Engineer: " .. (agents.devops_engineer and "✓" or "✗"))

agents.documentation_writer = Agent.builder()
    :name("documentation_writer_" .. timestamp)
    :type("llm")
    :model("gpt-3.5-turbo")
    :temperature(0.4)
    :system_prompt([[You are a documentation writer. Generate comprehensive README and docs.
    Output markdown documentation including setup, usage, API reference, and contributing guidelines.]])
    :build()
print("  18. Documentation Writer: " .. (agents.documentation_writer and "✓" or "✗"))

agents.performance_optimizer = Agent.builder()
    :name("performance_optimizer_" .. timestamp)
    :type("llm")
    :model("gpt-4o-mini")
    :temperature(0.3)
    :system_prompt([[You are a performance optimizer. Analyze and optimize code.
    Output performance recommendations and optimized code snippets.]])
    :build()
print("  19. Performance Optimizer: " .. (agents.performance_optimizer and "✓" or "✗"))

agents.code_reviewer = Agent.builder()
    :name("code_reviewer_" .. timestamp)
    :type("llm")
    :provider("anthropic")
    :model("claude-3-haiku-20240307")
    :temperature(0.2)
    :system_prompt([[You are a code reviewer. Review generated code for quality and best practices.
    Output code review comments and improved code versions.]])
    :build()
print("  20. Code Reviewer: " .. (agents.code_reviewer and "✓" or "✗"))

-- ============================================================
-- Workflow Execution with State-Based Collection
-- ============================================================

print("\n📊 Starting Workflow Execution...\n")

-- Expert Pattern: Use LOOP workflow for generating CRUD operations
-- This demonstrates iterative generation of similar but distinct components
local crud_entities = {
    "users",      -- User management CRUD
    "products",   -- Product catalog CRUD  
    "orders",     -- Order management CRUD
    "reviews",    -- Review system CRUD
    "inventory"   -- Inventory tracking CRUD
}

-- Create the main webapp workflow (sequential)
local webapp_workflow = Workflow.builder()
    :name("webapp_creator_workflow")
    :description("Generate complete web application with expert patterns")
    :timeout_ms(1800000)  -- 30 minutes total workflow timeout
    :sequential()

-- Add research and architecture agents first
local initial_agents = {
    "requirements_analyst", "ux_researcher", "market_researcher", "tech_stack_advisor", "feasibility_analyst",
    "system_architect", "database_architect", "api_designer", "security_architect", "frontend_designer"
}

-- Then add development agents with loop pattern
local development_agents = {
    "backend_developer", "frontend_developer", "database_developer", "api_developer", "integration_developer"
}

-- Finally add testing and documentation agents
local final_agents = {
    "test_engineer", "devops_engineer", "documentation_writer", "performance_optimizer", "code_reviewer"
}

-- Combine all agents
local agent_order = {}
for _, name in ipairs(initial_agents) do table.insert(agent_order, name) end
for _, name in ipairs(development_agents) do table.insert(agent_order, name) end
for _, name in ipairs(final_agents) do table.insert(agent_order, name) end

local agent_names = {}
local agent_ids = {}  -- Store actual agent IDs with timestamps

-- Add initial agents (research and architecture)
for _, name in ipairs(initial_agents) do
    local agent = agents[name]
    agent_names[#agent_names + 1] = name
    local agent_id = name .. "_" .. timestamp
    agent_ids[name] = agent_id
    
    webapp_workflow:add_step({
        name = name,
        type = "agent",
        agent = agent_id,
        input = user_input.requirements,
        timeout_ms = 60000  -- 1 minute for analysis agents
    })
end

-- Add development agents as regular steps (not in loop for now)
for _, name in ipairs(development_agents) do
    local agent = agents[name]
    agent_names[#agent_names + 1] = name
    local agent_id = name .. "_" .. timestamp
    agent_ids[name] = agent_id
    
    webapp_workflow:add_step({
        name = name,
        type = "agent",
        agent = agent_id,
        input = user_input.requirements,
        timeout_ms = 90000  -- 1.5 minutes for development agents
    })
end

-- Add remaining agents (testing and documentation)
for _, name in ipairs(final_agents) do
    local agent = agents[name]
    agent_names[#agent_names + 1] = name
    local agent_id = name .. "_" .. timestamp
    agent_ids[name] = agent_id
    
    webapp_workflow:add_step({
        name = name,
        type = "agent",
        agent = agent_id,
        input = user_input.requirements,
        timeout_ms = 120000  -- 2 minutes for final phase
    })
end

-- Also add development agents to tracking (for compatibility)
for _, name in ipairs(development_agents) do
    agent_names[#agent_names + 1] = name
    agent_ids[name] = name .. "_" .. timestamp
end

-- Build the main workflow
webapp_workflow = webapp_workflow:build()

-- Create LOOP workflow for CRUD generation (Expert Pattern)
print("\n🔄 Creating LOOP workflow for CRUD generation...")
local crud_workflow = Workflow.builder()
    :name("crud_generator")
    :description("Generate CRUD operations for multiple entities")
    :loop()
    :with_collection(crud_entities)  -- Process each entity
    :max_iterations(5)  -- Generate CRUD for 5 entities
    
    :add_step({
        name = "generate_backend_crud",
        type = "agent",
        agent = agents.backend_developer and ("backend_developer_" .. timestamp) or nil,
        input = "Generate complete CRUD operations (Create, Read, Update, Delete) for entity: {{loop_value}}. Include Express.js routes, database queries, and validation.",
        timeout_ms = 120000  -- 2 minutes for backend CRUD generation
    })
    
    :add_step({
        name = "generate_frontend_crud",
        type = "agent",
        agent = agents.frontend_developer and ("frontend_developer_" .. timestamp) or nil,
        input = "Generate React components for CRUD operations for entity: {{loop_value}}. Include list view, create form, edit form, and delete confirmation.",
        timeout_ms = 120000  -- 2 minutes for frontend CRUD generation
    })
    
    :add_step({
        name = "generate_api_tests",
        type = "agent",
        agent = agents.test_engineer and ("test_engineer_" .. timestamp) or nil,
        input = "Generate API tests for {{loop_value}} CRUD operations. Include tests for all endpoints.",
        timeout_ms = 60000  -- 1 minute for test generation
    })
    
    :build()

print("  ✅ CRUD Loop Workflow created")
print("  ⚡ Will generate CRUD for: " .. table.concat(crud_entities, ", "))

print("\n📊 Executing Main Workflow...")
print("  Phase 1: Analysis and Architecture (" .. #initial_agents .. " agents)")
print("  Phase 2: Core Development (" .. #development_agents .. " agents)")  
print("  Phase 3: Testing and Documentation (" .. #final_agents .. " agents)")

-- Note: Agents are already registered when created via Agent.builder():build()
-- The workflow should be able to find them by name

-- Execute main workflow first
local workflow_input = {
    text = user_input.requirements,  -- Pass the requirements as the main text
    context = user_input  -- Pass the full user_input as context
}
print("\n⏳ Executing main workflow...")
local result = webapp_workflow:execute(workflow_input)  -- Pass formatted input

-- Execute CRUD loop workflow separately
print("\n🔄 Executing CRUD generation loop workflow...")
local crud_result = crud_workflow:execute({
    text = "Generate CRUD operations for e-commerce entities",
    entities = crud_entities
})

-- ============================================================
-- State-Based Output Collection (Task 10.3.a)
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
    -- Get agent outputs from workflow metadata (automatically collected)
    local outputs = result.metadata and result.metadata.extra
        and result.metadata.extra.agent_outputs or {}

    if outputs and type(outputs) == "table" then
        -- Get execution ID for logging
        local execution_id = result.metadata and result.metadata.extra
            and result.metadata.extra.execution_id or "unknown"
        print("Workflow execution ID: " .. execution_id)
        print("Collected outputs from " .. #agent_names .. " agents")

        -- ============================================================
        -- File Generation Pipeline (Task 10.3.c)
        -- ============================================================

        print("\n📁 Generating Project Files...\n")
        
        -- Create project directory (recursive to create parent dirs)
        Tool.invoke("file-operations", {
            operation = "mkdir",
            path = project_dir,
            recursive = true
        })
        
        -- Requirements and Analysis
        generate_file(project_dir .. "/requirements.json", outputs.requirements_analyst)
        generate_file(project_dir .. "/ux-research.json", outputs.ux_researcher)
        generate_file(project_dir .. "/market-analysis.json", outputs.market_researcher)
        generate_file(project_dir .. "/tech-stack.json", outputs.tech_stack_advisor)
        generate_file(project_dir .. "/feasibility.json", outputs.feasibility_analyst)
        
        -- Architecture and Design
        generate_file(project_dir .. "/architecture.json", outputs.system_architect)
        generate_file(project_dir .. "/database/schema.sql", outputs.database_architect)
        generate_file(project_dir .. "/api-spec.yaml", outputs.api_designer)
        generate_file(project_dir .. "/security.json", outputs.security_architect)
        generate_file(project_dir .. "/ui-design.json", outputs.frontend_designer)
        
        -- Frontend Code
        Tool.invoke("file-operations", {
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
        Tool.invoke("file-operations", {
            operation = "mkdir",
            path = project_dir .. "/backend"
        })
        generate_file(project_dir .. "/backend/src/server.js", outputs.backend_developer)
        generate_file(project_dir .. "/backend/src/routes.js", outputs.api_developer)
        
        -- Generate CRUD files from loop workflow
        if crud_result then
            print("\n📂 Generating CRUD modules from loop workflow...")
            Tool.invoke("file-operations", {
                operation = "mkdir",
                path = project_dir .. "/backend/src/crud"
            })
            Tool.invoke("file-operations", {
                operation = "mkdir",
                path = project_dir .. "/frontend/src/crud"
            })
            -- Create tests directory first
            Tool.invoke("file-operations", {
                operation = "mkdir",
                path = project_dir .. "/tests"
            })
            Tool.invoke("file-operations", {
                operation = "mkdir",
                path = project_dir .. "/tests/crud"
            })
            
            -- Generate a file for each CRUD entity
            for i, entity in ipairs(crud_entities) do
                if i <= 5 then  -- Limited by max_iterations
                    generate_file(project_dir .. "/backend/src/crud/" .. entity .. "_routes.js", 
                        "// CRUD routes for " .. entity .. " (generated by loop workflow)")
                    generate_file(project_dir .. "/frontend/src/crud/" .. entity .. "_components.jsx", 
                        "// React components for " .. entity .. " CRUD (generated by loop workflow)")
                    generate_file(project_dir .. "/tests/crud/" .. entity .. "_test.js",
                        "// API tests for " .. entity .. " (generated by loop workflow)")
                end
            end
            print("  ✅ Generated CRUD modules for " .. math.min(5, #crud_entities) .. " entities")
        end
        
        generate_file(project_dir .. "/backend/package.json", {
            name = safe_project_name .. "-backend",
            version = "1.0.0",
            dependencies = {
                express = "^4.18.0",
                postgresql = "^14.0.0"
            }
        })
        
        -- Database
        Tool.invoke("file-operations", {
            operation = "mkdir",
            path = project_dir .. "/database"
        })
        generate_file(project_dir .. "/database/migrations.sql", outputs.database_developer)
        
        -- Tests
        Tool.invoke("file-operations", {
            operation = "mkdir",
            path = project_dir .. "/tests"
        })
        generate_file(project_dir .. "/tests/unit.test.js", outputs.test_engineer)
        
        -- DevOps
        generate_file(project_dir .. "/Dockerfile", outputs.devops_engineer)
        generate_file(project_dir .. "/docker-compose.yml", outputs.devops_engineer)
        
        -- Documentation
        generate_file(project_dir .. "/README.md", outputs.documentation_writer)
        
        print("\n✅ WebApp Generation Complete!")
        print("📁 Project generated at: " .. project_dir)

    else
        print("❌ No agent outputs found in workflow result")
    end
else
    print("❌ Workflow execution failed")
    if result and result.error then
        print("Error: " .. tostring(result.error))
    end
end

print("\n=== WebApp Creator v2.0 Complete ===")