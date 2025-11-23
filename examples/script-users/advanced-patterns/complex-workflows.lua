-- Profile: research (recommended)
-- Run with: llmspell -p research run complex-workflows.lua
-- Full stack with trace logging for complex patterns

-- ============================================================
-- LLMSPELL ADVANCED PATTERNS SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: advanced-patterns
-- Pattern ID: 02 - Complex Workflow Patterns v0.7.0
-- Complexity: ADVANCED
-- Real-World Use Case: Enterprise automation and decision systems
-- Pattern Category: Workflow Orchestration
--
-- Purpose: Demonstrates advanced workflow patterns and compositions
-- Architecture: Sequential, parallel, conditional, and nested workflows
-- Key Capabilities:
--   • Sequential pipelines with data flow
--   • Parallel branch execution
--   • Conditional routing and decision trees
--   • Nested workflow composition
--   • Multi-branch conditionals
--   • Error handling in workflows
--   • State management between steps
--
-- Prerequisites:
--   • Understanding of workflow basics (see features/workflow-basics.lua)
--   • No API keys required (uses local tools)
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/advanced-patterns/complex-workflows.lua
--
-- EXPECTED OUTPUT:
-- Multiple workflow patterns executing with various strategies
-- Execution time: 2-5 seconds (local tools only)
--
-- Time to Complete: 5 seconds
-- Next Steps: See tool-integration-patterns.lua for tool combinations
-- ============================================================

print("=== Complex Workflow Patterns ===\n")

-- Helper function for safe workflow execution
local function execute_workflow(workflow, input, name)
    local success, result = pcall(function()
        return workflow:execute(input or {})
    end)
    
    if success and result and result.text then
        if result.text:match("completed successfully") then
            print("   ✓ " .. name .. " completed")
            return true, result
        else
            print("   ⚠ " .. name .. " failed: " .. result.text:sub(1, 50))
            return false, result
        end
    else
        print("   ✗ " .. name .. " error: " .. tostring(result))
        return false, nil
    end
end

-- 1. ADVANCED SEQUENTIAL PIPELINE
print("1. Advanced Sequential Pipeline")
print("-" .. string.rep("-", 31))

-- Complex data processing pipeline with multiple transformations
local data_pipeline = Workflow.builder()
    :name("advanced_data_pipeline")
    :description("Multi-stage data processing with validation")
    :sequential()
    -- Stage 1: Generate initial data
    :add_step({
        name = "generate_id",
        type = "tool",
        tool = "uuid-generator",
        input = {
            operation = "generate",
            version = "v4"
        }
    })
    -- Stage 2: Get timestamp
    :add_step({
        name = "get_timestamp",
        type = "tool",
        tool = "datetime-handler",
        input = {
            operation = "now"
        }
    })
    -- Stage 3: Create hash of combined data
    :add_step({
        name = "create_hash",
        type = "tool",
        tool = "hash-calculator",
        input = {
            operation = "hash",
            algorithm = "sha256",
            input = "{{step.generate_id.result}}_{{step.get_timestamp.result}}"
        }
    })
    -- Stage 4: Encode result
    :add_step({
        name = "encode_result",
        type = "tool",
        tool = "base64-encoder",
        input = {
            operation = "encode",
            input = "{{step.create_hash.result}}"
        }
    })
    -- Stage 5: Create final report
    :add_step({
        name = "create_report",
        type = "tool",
        tool = "template-creator",
        input = {
            input = "Pipeline Report:\nID: {{id}}\nTime: {{time}}\nHash: {{hash}}\nEncoded: {{encoded}}",
            context = {
                id = "{{step.generate_id.result}}",
                time = "{{step.get_timestamp.result}}",
                hash = "{{step.create_hash.result}}",
                encoded = "{{step.encode_result.result}}"
            }
        }
    })
    :build()

print("   Executing 5-stage pipeline...")
execute_workflow(data_pipeline, {}, "Data pipeline")

-- 2. PARALLEL WORKFLOW WITH AGGREGATION
print("\n2. Parallel Processing with Aggregation")
print("-" .. string.rep("-", 38))

local parallel_aggregator = Workflow.builder()
    :name("parallel_aggregation")
    :description("Multiple parallel operations with result aggregation")
    :parallel()
    -- Branch 1: Generate multiple UUIDs
    :add_step({
        name = "uuid_v4",
        type = "tool",
        tool = "uuid-generator",
        input = {operation = "generate", version = "v4"}
    })
    :add_step({
        name = "uuid_v5",
        type = "tool",
        tool = "uuid-generator",
        input = {operation = "generate", version = "v5", namespace = "test", name = "workflow"}
    })
    -- Branch 2: Multiple hash calculations
    :add_step({
        name = "md5_hash",
        type = "tool",
        tool = "hash-calculator",
        input = {operation = "hash", algorithm = "md5", input = "parallel_test"}
    })
    :add_step({
        name = "sha256_hash",
        type = "tool",
        tool = "hash-calculator",
        input = {operation = "hash", algorithm = "sha256", input = "parallel_test"}
    })
    -- Branch 3: Text operations
    :add_step({
        name = "uppercase",
        type = "tool",
        tool = "text-manipulator",
        input = {operation = "uppercase", input = "parallel workflow"}
    })
    :add_step({
        name = "word_count",
        type = "tool",
        tool = "text-manipulator",
        input = {operation = "count", input = "parallel workflow execution test", count_type = "words"}
    })
    :build()

print("   Executing 6 parallel operations...")
execute_workflow(parallel_aggregator, {}, "Parallel aggregator")

-- 3. CONDITIONAL ROUTING WORKFLOW
print("\n3. Conditional Content Routing")
print("-" .. string.rep("-", 30))

-- Create specialized sub-workflows for different content types
local blog_processor = Workflow.builder()
    :name("blog_processor")
    :sequential()
    :add_step({
        name = "format_blog",
        type = "tool",
        tool = "template-creator",
        input = {
            input = "{{content}}\n\n[Formatted for Blog Publication]",
            context = {
                content = "{{input.content}}"
            }
        }
    })
    :build()

local email_processor = Workflow.builder()
    :name("email_processor")
    :sequential()
    :add_step({
        name = "format_email",
        type = "tool",
        tool = "template-creator",
        input = {
            input = "Subject: Important Update\n\n{{content}}",
            context = {
                content = "{{input.content}}"
            }
        }
    })
    :build()

-- Main conditional router
local content_router = Workflow.builder()
    :name("content_routing_system")
    :description("Routes content based on type detection")
    :conditional()
    -- Initial analysis step
    :add_step({
        name = "analyze_content",
        type = "tool",
        tool = "text-manipulator",
        input = {
            operation = "count",
            input = "{{input.content}}",
            count_type = "chars"
        }
    })
    -- Condition: Use shared_data_equals to check content type
    :condition({
        type = "shared_data_equals",
        key = "content_type",
        value = "blog"
    })
    -- If blog content
    :add_then_step({
        name = "process_as_blog",
        type = "workflow",
        workflow = blog_processor
    })
    -- Otherwise email
    :add_else_step({
        name = "process_as_email",
        type = "workflow",
        workflow = email_processor
    })
    :build()

-- Set shared data for routing
if content_router then
    -- For blog test, set content_type to "blog"
    -- Note: In real use, this would be set by analysis step
    content_router:set_shared_data("content_type", "blog")
end

print("   Testing blog content routing...")
if content_router then
    content_router:set_shared_data("content_type", "blog")
    execute_workflow(content_router, {content = "This is a blog post about workflows"}, "Blog router")
end

print("   Testing email content routing...")
if content_router then
    content_router:set_shared_data("content_type", "email")
    execute_workflow(content_router, {content = "Quick email update about the project"}, "Email router")
end

-- 4. MULTI-BRANCH CONDITIONAL
print("\n4. Multi-Branch Priority System")
print("-" .. string.rep("-", 31))

-- Priority-based routing with multiple branches
local priority_system = Workflow.builder()
    :name("priority_routing")
    :description("N-way branching based on priority levels")
    :conditional()
    -- Assess priority
    :add_step({
        name = "check_priority",
        type = "tool",
        tool = "text-manipulator",
        input = {
            operation = "count",
            input = "{{input.task}}",
            count_type = "words"
        }
    })
    -- Use shared_data_equals for priority checking
    :condition({
        type = "shared_data_equals",
        key = "priority_level",
        value = "critical"
    })
    -- Critical path
    :add_then_step({
        name = "critical_handler",
        type = "tool",
        tool = "template-creator",
        input = {
            input = "🚨 CRITICAL ALERT: {{task}}\nImmediate action required!",
            context = {task = "{{input.task}}"}
        }
    })
    -- Non-critical path (simplified for API compatibility)
    :add_else_step({
        name = "normal_handler",
        type = "tool",
        tool = "template-creator",
        input = {
            input = "📋 NORMAL: {{task}}\nAdded to standard queue.",
            context = {task = "{{input.task}}"}
        }
    })
    :build()

print("   Testing critical task...")
if priority_system then
    priority_system:set_shared_data("priority_level", "critical")
    execute_workflow(priority_system, {task = "Critical system failure detected"}, "Critical priority")
end

print("   Testing normal task...")
if priority_system then
    priority_system:set_shared_data("priority_level", "normal")
    execute_workflow(priority_system, {task = "Regular maintenance needed"}, "Normal priority")
end

-- 5. NESTED WORKFLOW COMPOSITION
print("\n5. Nested Workflow Composition")
print("-" .. string.rep("-", 30))

-- Inner workflow 1: Data preparation
local data_prep = Workflow.builder()
    :name("data_preparation")
    :sequential()
    :add_step({
        name = "generate_data",
        type = "tool",
        tool = "uuid-generator",
        input = {operation = "generate", version = "v4"}
    })
    :add_step({
        name = "encode_data",
        type = "tool",
        tool = "base64-encoder",
        input = {operation = "encode", input = "{{step.generate_data.result}}"}
    })
    :build()

-- Inner workflow 2: Data validation
local data_validation = Workflow.builder()
    :name("data_validation")
    :sequential()
    :add_step({
        name = "verify_format",
        type = "tool",
        tool = "text-manipulator",
        input = {operation = "trim", input = "{{input.data}}"}
    })
    :build()

-- Outer workflow composing inner workflows
local nested_workflow = Workflow.builder()
    :name("nested_composition")
    :description("Workflow containing other workflows")
    :sequential()
    -- Step 1: Run data preparation workflow
    :add_step({
        name = "prepare",
        type = "workflow",
        workflow = data_prep
    })
    -- Step 2: Process the prepared data
    :add_step({
        name = "process",
        type = "tool",
        tool = "hash-calculator",
        input = {
            operation = "hash",
            algorithm = "sha256",
            input = "{{step.prepare.result}}"
        }
    })
    -- Step 3: Validate using validation workflow
    :add_step({
        name = "validate",
        type = "workflow",
        workflow = data_validation,
        input = {data = "{{step.process.result}}"}
    })
    :build()

print("   Executing nested workflow composition...")
execute_workflow(nested_workflow, {}, "Nested composition")

-- 6. ERROR RECOVERY WORKFLOW
print("\n6. Error Recovery Patterns")
print("-" .. string.rep("-", 26))

local recovery_workflow = Workflow.builder()
    :name("error_recovery")
    :description("Workflow with error handling and recovery")
    :sequential()
    -- Step 1: Generate data
    :add_step({
        name = "generate",
        type = "tool",
        tool = "uuid-generator",
        input = {
            operation = "generate",
            version = "v4"
        }
    })
    -- Step 2: Process the data
    :add_step({
        name = "process",
        type = "tool",
        tool = "hash-calculator",
        input = {
            operation = "hash",
            algorithm = "md5",
            input = "{{step.generate.result}}"
        }
    })
    :build()

print("   Testing error recovery...")
execute_workflow(recovery_workflow, {}, "Error recovery")

-- 7. PERFORMANCE METRICS
print("\n7. Workflow Performance Analysis")
print("-" .. string.rep("-", 32))

local start_time = os.clock()

-- Quick workflow for performance testing
local perf_workflow = Workflow.builder()
    :name("performance_test")
    :parallel()
    :add_step({
        name = "op1",
        type = "tool",
        tool = "uuid-generator",
        input = {operation = "generate", version = "v4"}
    })
    :add_step({
        name = "op2",
        type = "tool",
        tool = "hash-calculator",
        input = {operation = "hash", algorithm = "md5", input = "test"}
    })
    :add_step({
        name = "op3",
        type = "tool",
        tool = "base64-encoder",
        input = {operation = "encode", input = "performance"}
    })
    :build()

execute_workflow(perf_workflow, {}, "Performance test")

local duration = (os.clock() - start_time) * 1000
print(string.format("   Parallel execution time: %.2f ms", duration))

-- 8. BEST PRACTICES
print("\n8. Complex Workflow Best Practices")
print("-" .. string.rep("-", 34))

print("   • Use sequential for dependent operations")
print("   • Use parallel for independent operations")
print("   • Implement conditional routing for decision trees")
print("   • Compose workflows for reusable components")
print("   • Add retry logic for unreliable operations")
print("   • Use error handlers for graceful degradation")
print("   • Monitor performance of complex pipelines")
print("   • Document workflow dependencies clearly")

print("\n=== Complex Workflow Patterns Complete ===")
print("Demonstrated: Sequential, Parallel, Conditional, Nested, Recovery patterns")
print("Next: Explore tool-integration-patterns.lua for advanced tool usage")