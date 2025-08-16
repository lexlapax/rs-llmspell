-- ABOUTME: Example of conditional workflow for content routing based on classification
-- ABOUTME: Demonstrates agent classification conditions and multi-path routing

print("=== Conditional Content Routing Example ===\n")

-- Create mock classifier agent (in production, use real LLM agent)
local classifier_agent = Agent and Agent.builder and Agent.builder()
    :name("content_classifier")
    :description("Classifies content type")
    :type("llm")
    :model("openai/gpt-3.5-turbo")
    :temperature(0.2)
    :build() or nil

-- Store agent name for workflow reference
local agent_names = {}
if classifier_agent then
    agent_names.classifier = "content_classifier"
end

-- Create specialized workflows for each content type
local blog_workflow = Workflow.builder()
    :name("blog_creation")
    :description("Process blog content")
    :sequential()
    :add_step({
        name = "format_blog",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Blog: ",
            suffix = " [FORMATTED FOR BLOG]"
        }
    })
    :build()

local social_workflow = Workflow.builder()
    :name("social_creation")
    :description("Process social media content")
    :sequential()
    :add_step({
        name = "format_social",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Social: ",
            suffix = " [FORMATTED FOR SOCIAL MEDIA]"
        }
    })
    :build()

local email_workflow = Workflow.builder()
    :name("email_creation")
    :description("Process email content")
    :sequential()
    :add_step({
        name = "format_email",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Email: ",
            suffix = " [FORMATTED FOR EMAIL]"
        }
    })
    :build()

-- Create main conditional routing workflow
local content_router = Workflow.builder()
    :name("content_routing_system")
    :description("Routes content to appropriate workflow based on classification")
    :conditional()
    
    -- Initial classification step
    :add_step({
        name = "classify",
        type = classifier_agent and "agent" or "tool",
        agent = agent_names.classifier,
        tool = not classifier_agent and "text_manipulator" or nil,
        input = classifier_agent and "Classify this content as 'blog', 'social', or 'email': {{content}}" or {
            operation = "analyze",
            input = "{{content}}"
        }
    })
    
    -- Condition function to check classification result
    :condition(function(ctx)
        -- In production, this would check actual agent output
        -- For demo, we check if content contains "blog"
        local content = ctx.input or ""
        return string.match(content:lower(), "blog") ~= nil
    end)
    
    -- Route to blog workflow if condition is true
    :add_then_step({
        name = "route_to_blog",
        type = "workflow",
        workflow = blog_workflow
    })
    
    -- Route to social workflow as first else option
    :add_else_step({
        name = "route_to_social",
        type = "workflow",
        workflow = social_workflow
    })
    
    -- Route to email workflow as second else option
    :add_else_step({
        name = "route_to_email",
        type = "workflow",
        workflow = email_workflow
    })
    
    :build()

print("✅ Created conditional content routing workflow")
print("  • Classification step: " .. (classifier_agent and "LLM agent" or "Text analyzer"))
print("  • Blog route: Sequential workflow for blog formatting")
print("  • Social route: Sequential workflow for social media")
print("  • Email route: Sequential workflow for email formatting")

-- Test the routing with different content types
local test_contents = {
    "Write a blog post about AI technology",
    "Create social media announcement for product launch",
    "Draft email newsletter for subscribers"
}

print("\n📝 Testing content routing:")
for i, content in ipairs(test_contents) do
    print("\n  Test " .. i .. ": " .. content)
    
    -- Execute the routing workflow
    local result = content_router:execute({
        input = content,
        content = content
    })
    
    if result then
        print("    → Routing completed successfully")
    else
        print("    → Routing simulation (no execution in demo)")
    end
end

print("\n=== Conditional Routing Example Complete ===")
print("This example demonstrates:")
print("  1. Agent-based content classification")
print("  2. Conditional routing based on classification")
print("  3. Multiple else branches for different content types")
print("  4. Nested workflow execution in each branch")
print("\nTo run with real LLM agents, set OPENAI_API_KEY or ANTHROPIC_API_KEY")