-- ABOUTME: Example of multi-branch conditional workflow with complex routing logic
-- ABOUTME: Demonstrates N-branch routing beyond simple then/else patterns

print("=== Multi-Branch Conditional Workflow Example ===\n")

-- Create priority classification workflow
local priority_router = Workflow.builder()
    :name("priority_routing_system")
    :description("Routes tasks based on priority level with multiple branches")
    :conditional()
    
    -- Initial priority assessment step
    :add_step({
        name = "assess_priority",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "analyze",
            input = "{{task_description}}",
            analysis_type = "priority_detection"
        }
    })
    
    -- First condition: Check for CRITICAL priority
    :condition(function(ctx)
        local task = ctx.task_description or ""
        return string.match(task:lower(), "critical") or 
               string.match(task:lower(), "urgent") or
               string.match(task:lower(), "emergency")
    end)
    
    -- CRITICAL path: Immediate escalation
    :add_then_step({
        name = "critical_escalation",
        type = "workflow",
        workflow = Workflow.builder()
            :name("critical_handler")
            :parallel()  -- Multiple actions in parallel
            :add_step({
                name = "notify_manager",
                type = "tool",
                tool = "webhook_caller",
                input = {
                    url = "https://example.com/critical",
                    method = "POST",
                    data = { priority = "CRITICAL", task = "{{task_description}}" }
                }
            })
            :add_step({
                name = "create_incident",
                type = "tool",
                tool = "file_operations",
                input = {
                    operation = "write",
                    path = "/tmp/incident.log",
                    input = "CRITICAL: {{task_description}}"
                }
            })
            :build()
    })
    
    -- HIGH priority path (first else)
    :add_else_step({
        name = "high_priority_path",
        type = "workflow",
        workflow = Workflow.builder()
            :name("high_handler")
            :sequential()
            :add_step({
                name = "assign_senior",
                type = "tool",
                tool = "text_manipulator",
                input = {
                    operation = "append",
                    input = "HIGH PRIORITY: ",
                    suffix = " - Assigned to senior team"
                }
            })
            :build()
    })
    
    -- MEDIUM priority path (second else)
    :add_else_step({
        name = "medium_priority_path",
        type = "workflow",
        workflow = Workflow.builder()
            :name("medium_handler")
            :sequential()
            :add_step({
                name = "queue_task",
                type = "tool",
                tool = "text_manipulator",
                input = {
                    operation = "append",
                    input = "MEDIUM PRIORITY: ",
                    suffix = " - Added to standard queue"
                }
            })
            :build()
    })
    
    -- LOW priority path (third else - default)
    :add_else_step({
        name = "low_priority_path",
        type = "workflow",
        workflow = Workflow.builder()
            :name("low_handler")
            :sequential()
            :add_step({
                name = "backlog_task",
                type = "tool",
                tool = "text_manipulator",
                input = {
                    operation = "append",
                    input = "LOW PRIORITY: ",
                    suffix = " - Added to backlog"
                }
            })
            :build()
    })
    
    :build()

print("✅ Created multi-branch priority routing workflow")
print("  • Branches: CRITICAL → HIGH → MEDIUM → LOW")
print("  • Each branch has specialized handling workflow")
print("  • Demonstrates N-way routing pattern")

-- Create department routing workflow
local department_router = Workflow.builder()
    :name("department_routing_system")
    :description("Routes requests to appropriate department")
    :conditional()
    
    :add_step({
        name = "analyze_request",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "analyze",
            input = "{{request}}"
        }
    })
    
    -- Engineering department condition
    :condition(function(ctx)
        local req = ctx.request or ""
        return string.match(req:lower(), "bug") or 
               string.match(req:lower(), "feature") or
               string.match(req:lower(), "code")
    end)
    
    :add_then_step({
        name = "route_to_engineering",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Request: ",
            suffix = " [ROUTED TO: Engineering]"
        }
    })
    
    -- Sales department (first else)
    :add_else_step({
        name = "route_to_sales",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Request: ",
            suffix = " [ROUTED TO: Sales]"
        }
    })
    
    -- Support department (second else)
    :add_else_step({
        name = "route_to_support",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Request: ",
            suffix = " [ROUTED TO: Support]"
        }
    })
    
    -- Marketing department (third else)
    :add_else_step({
        name = "route_to_marketing",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Request: ",
            suffix = " [ROUTED TO: Marketing]"
        }
    })
    
    -- General inquiry (fourth else - default)
    :add_else_step({
        name = "route_to_general",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "append",
            input = "Request: ",
            suffix = " [ROUTED TO: General Inquiry]"
        }
    })
    
    :build()

print("\n✅ Created department routing workflow")
print("  • Branches: Engineering → Sales → Support → Marketing → General")
print("  • Demonstrates 5-way routing pattern")

-- Test scenarios
local test_scenarios = {
    { 
        workflow = priority_router,
        name = "Priority Router",
        tests = {
            "CRITICAL: System is down!",
            "Please handle this task when possible",
            "Regular maintenance needed"
        }
    },
    {
        workflow = department_router,
        name = "Department Router",
        tests = {
            "Found a bug in the login system",
            "Need pricing information",
            "How do I reset my password?"
        }
    }
}

print("\n📝 Testing multi-branch routing:")
for _, scenario in ipairs(test_scenarios) do
    print("\n  " .. scenario.name .. ":")
    for i, test in ipairs(scenario.tests) do
        print("    Test " .. i .. ": " .. test)
        -- In production, execute: scenario.workflow:execute({ input = test })
    end
end

print("\n=== Multi-Branch Example Complete ===")
print("This example demonstrates:")
print("  1. N-way conditional routing (more than then/else)")
print("  2. Priority-based task routing")
print("  3. Department-based request routing")
print("  4. Default fallback branches")
print("  5. Nested workflows in each branch")
print("\nFuture Enhancement: add_branch() API for dynamic branch addition")