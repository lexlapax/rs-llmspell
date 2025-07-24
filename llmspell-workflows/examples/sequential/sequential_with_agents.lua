-- ABOUTME: Sequential workflow demonstrating agent integration
-- ABOUTME: Shows how to use agents as workflow steps for complex decision making

-- Note: This example assumes Phase 3.3 agent infrastructure is implemented
-- It demonstrates the planned API for agent-workflow integration

-- Sequential workflow with agent steps
-- Demonstrates intelligent processing using LLM agents

-- Create an analysis agent
local analyzer = Agent.createAsync({
    name = "data_analyzer",
    model = "gpt-4",
    system_prompt = "You are a data analysis expert. Analyze the provided data and give insights.",
    temperature = 0.7
})

-- Create a decision agent
local decision_maker = Agent.createAsync({
    name = "decision_maker", 
    model = "gpt-4",
    system_prompt = "You are a decision-making assistant. Based on analysis, recommend next actions.",
    temperature = 0.5
})

-- Create sequential workflow with agent steps
local agent_workflow = Workflow.sequential({
    name = "intelligent_data_pipeline",
    description = "Process data with AI-powered analysis and decision making",
    
    steps = {
        -- Step 1: Read data file
        {
            name = "read_data",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/sample_data.csv"
            },
            -- Create sample data if file doesn't exist
            on_error = function(error)
                print("Creating sample data file...")
                Tools.get("file_operations"):execute({
                    operation = "write",
                    path = "/tmp/sample_data.csv",
                    content = [[
product,sales,revenue,region
Widget A,150,4500,North
Widget B,89,2670,South
Widget C,210,8400,East
Widget D,45,1350,West
Widget E,178,7120,North
]]
                })
                return { retry = true }
            end
        },
        
        -- Step 2: Analyze data with AI agent
        {
            name = "analyze_data",
            type = "agent",
            agent = analyzer,
            input = {
                prompt = "Analyze this CSV data and provide insights about sales patterns:\n\n{{step:read_data:output}}"
            }
        },
        
        -- Step 3: Parse CSV for calculations
        {
            name = "parse_csv",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = "{{step:read_data:output}}",
                operation = "analyze"
            }
        },
        
        -- Step 4: Make decisions based on analysis
        {
            name = "make_decisions",
            type = "agent",
            agent = decision_maker,
            input = {
                prompt = [[
Based on the following analysis, recommend actions:

AI Analysis:
{{step:analyze_data:output}}

Data Statistics:
{{step:parse_csv:output}}

Please provide:
1. Top performing products
2. Regions needing attention
3. Recommended actions
]]
            }
        },
        
        -- Step 5: Generate report
        {
            name = "generate_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
# Sales Analysis Report
Generated: {{timestamp}}

## AI Analysis
{{ai_analysis}}

## Recommendations
{{recommendations}}

## Data Summary
- Total Records: {{total_records}}
- Analysis Date: {{date}}
]],
                variables = {
                    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
                    ai_analysis = "{{step:analyze_data:output}}",
                    recommendations = "{{step:make_decisions:output}}",
                    total_records = "{{step:parse_csv:output.rows}}",
                    date = os.date("%Y-%m-%d")
                }
            }
        },
        
        -- Step 6: Save report
        {
            name = "save_report",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/sales_analysis_report.md",
                content = "{{step:generate_report:output}}"
            }
        }
    },
    
    error_strategy = "fail_fast",
    timeout_ms = 60000  -- 60 second timeout for AI operations
})

-- Execute the intelligent workflow
print("Starting intelligent data analysis workflow...")
local result = agent_workflow:execute()

if result.success then
    print("✓ Analysis completed successfully!")
    print("Report saved to: /tmp/sales_analysis_report.md")
else
    print("✗ Analysis failed: " .. (result.error and result.error.message or "Unknown error"))
end

-- Advanced Agent Workflow: Multi-stage content generation
local content_workflow = Workflow.sequential({
    name = "content_generation_pipeline",
    description = "Multi-stage content creation with different specialized agents",
    
    steps = {
        -- Research phase
        {
            name = "research",
            type = "agent",
            agent = Agent.createAsync({
                name = "researcher",
                model = "gpt-4",
                system_prompt = "You are a research assistant. Gather and summarize information on the given topic."
            }),
            input = {
                prompt = "Research the topic: Benefits of workflow automation in software development"
            }
        },
        
        -- Outline creation
        {
            name = "create_outline",
            type = "agent",
            agent = Agent.createAsync({
                name = "outliner",
                model = "gpt-3.5-turbo",
                system_prompt = "You are an expert at creating structured outlines for articles."
            }),
            input = {
                prompt = "Create a detailed outline for an article based on this research:\n\n{{step:research:output}}"
            }
        },
        
        -- Content writing
        {
            name = "write_content",
            type = "agent",
            agent = Agent.createAsync({
                name = "writer",
                model = "gpt-4",
                system_prompt = "You are a technical writer. Write clear, engaging content based on the provided outline.",
                temperature = 0.8
            }),
            input = {
                prompt = "Write a comprehensive article based on this outline:\n\n{{step:create_outline:output}}"
            }
        },
        
        -- Content review and editing
        {
            name = "edit_content",
            type = "agent",
            agent = Agent.createAsync({
                name = "editor",
                model = "gpt-4",
                system_prompt = "You are a professional editor. Review and improve the content for clarity and correctness.",
                temperature = 0.3
            }),
            input = {
                prompt = "Edit and improve this article:\n\n{{step:write_content:output}}"
            }
        },
        
        -- Format for publishing
        {
            name = "format_content",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:edit_content:output}}",
                operation = "format",
                options = {
                    format = "markdown",
                    add_toc = true
                }
            }
        }
    },
    
    -- Use state to track progress
    on_step_complete = function(step_name, result)
        State.set("last_completed_step", step_name)
        State.set(step_name .. "_word_count", #result.output:split(" "))
        print("✓ Completed: " .. step_name)
    end
})

print("\n\nStarting content generation pipeline...")
local content_result = content_workflow:execute()

if content_result.success then
    print("\n✓ Content generation completed!")
    print("Total execution time: " .. content_result.duration_ms .. "ms")
    
    -- Show word counts from state
    local steps = {"research", "create_outline", "write_content", "edit_content"}
    print("\nWord counts by step:")
    for _, step in ipairs(steps) do
        local count = State.get(step .. "_word_count") or 0
        print("- " .. step .. ": " .. count .. " words")
    end
end