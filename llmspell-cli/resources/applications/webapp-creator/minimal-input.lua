-- WebApp Creator Minimal Input Example
-- This demonstrates the minimum required configuration for WebApp Creator
-- Use this as a starting template for simple applications

return {
    -- Project Basic Information (REQUIRED)
    project = {
        name = "SimpleApp",
        description = "A basic todo list web application",
        version = "1.0.0"
    },
    
    -- Main Requirements (REQUIRED)
    requirements = [[
Create a simple todo list application where users can:
- Add new tasks
- Mark tasks as complete
- Delete tasks
- View all tasks in a clean interface
    ]],
    
    -- Technical Preferences (OPTIONAL - defaults will be used if not specified)
    technical = {
        -- Minimal technical specifications
        frontend = {
            framework = "React",  -- or "Vue", "Angular", "vanilla"
            styling = "CSS"       -- or "Tailwind", "Bootstrap"
        },
        backend = {
            runtime = "Node.js",  -- or "Python", "Go"
            database = "SQLite"   -- or "PostgreSQL", "MongoDB"
        }
    },
    
    -- UX Requirements (OPTIONAL - but recommended)
    ux = {
        -- Simple UX guidance
        target_users = "General users who need task management",
        design_style = "Clean and minimalist",
        
        -- Core features only
        must_have_features = {
            "Task creation and editing",
            "Task completion tracking",
            "Simple, intuitive interface"
        }
    },
    
    -- Output Preferences (OPTIONAL - defaults to generating everything)
    output = {
        -- Control what gets generated
        generate = {
            frontend_code = true,
            backend_code = true,
            database_schema = true,
            documentation = true
        },
        
        -- Simplified options
        include_comments = true,  -- Add explanatory comments in code
        include_tests = false     -- Skip test generation for simplicity
    },
    
    -- Advanced Options (OPTIONAL - using defaults for simplicity)
    advanced = {
        -- Keep costs low with fewer iterations
        max_iterations = 2,      -- Reduced from 5
        max_cost = 5.00,        -- Lower budget
        
        -- Simplified model selection
        preferred_models = {
            -- Use GPT-3.5 for cost efficiency
            default = "openai/gpt-3.5-turbo"
        }
    }
}

--[[
USAGE:
------
Run with this minimal configuration:

./target/release/llmspell run \
  examples/script-users/applications/webapp-creator/main.lua \
  -- --input minimal-input.lua --output ./simple-app

This will generate a basic todo application with:
- Frontend: React with simple CSS
- Backend: Node.js REST API
- Database: SQLite for easy setup
- Documentation: Basic README

Expected generation time: ~60-90 seconds (using GPT-3.5-turbo)
Expected cost: < $1.00

CUSTOMIZATION TIPS:
------------------
1. Change the framework:
   - Set technical.frontend.framework to "Vue" or "Angular"
   - Set technical.backend.runtime to "Python" for FastAPI

2. Add more features:
   - Expand the requirements text with additional functionality
   - Add items to ux.must_have_features

3. Increase quality:
   - Set advanced.preferred_models.default to "openai/gpt-4"
   - Increase advanced.max_iterations for more refinement

4. Add authentication:
   - Include "User authentication and accounts" in requirements
   - The system will automatically add login/signup functionality
--]]