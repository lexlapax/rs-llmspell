-- Profile: research (recommended)
-- Run with: llmspell -p research run main.lua
-- Full stack with trace logging

-- ============================================================
-- LLMSPELL APPLICATION SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: applications
-- Application ID: 10 - Research Chat v1.0.0
-- Complexity: ADVANCED
-- Expected Output: Research findings + interactive chat session with shared memory
-- Version: 1.0.0
-- Tags: research, chat, composition, phase-13, workflow-template-delegation

-- HOW TO RUN:
-- 1. Basic usage:
--    llmspell app run research-chat --topic "Rust async programming"
--
-- 2. With custom parameters:
--    llmspell app run research-chat --topic "Rust ownership" --max_sources 15 --question "Explain borrowing"
--
-- 3. Debug mode:
--    llmspell app run research-chat --topic "Tokio runtime" --debug
--
-- ABOUTME: Research-Chat - "I need to research a topic and ask questions about it"
-- ABOUTME: Demonstrates workflow-template delegation with session-based memory sharing

-- Get command-line arguments (provided by llmspell app runtime)
local args = args or {}

-- Generate unique session ID for memory sharing across template steps
local session_id = "research-chat-" .. os.date("%Y%m%d-%H%M%S")

print("=== Research-Chat v1.0 (Phase 13 Composition Demo) ===")
print("Workflow-Template Delegation with Shared Memory\n")
print("Session ID: " .. session_id)
print("Topic: " .. (args.topic or "Rust async programming"))
print("")

-- ============================================================
-- Create Sequential Workflow with Template Steps
-- ============================================================

print("1. Creating research workflow with template delegation...")

-- Create sequential workflow using builder pattern
local workflow = Workflow.builder()
    :name("research-chat")
    :description("AI research with conversational follow-up")
    :sequential()

-- Add template steps using builder pattern
workflow = workflow
    :add_template_step("research", "research-assistant", {
        topic = args.topic or "Rust async programming",
        max_sources = tonumber(args.max_sources) or 10,
        session_id = session_id,              -- Memory anchor
        memory_enabled = true,                -- Enable RAG storage
        depth = "comprehensive",              -- Research depth
    })
    :add_template_step("chat", "interactive-chat", {
        system_prompt = "You are an expert assistant. Reference the research findings from the previous step to answer questions accurately and comprehensively.",
        message = args.question or "Summarize the key findings from the research",
        session_id = session_id,              -- Same session = shared memory
        memory_enabled = true,                -- Enable RAG retrieval
        max_turns = tonumber(args.max_turns) or 1,
        temperature = 0.7,
    })
    :build()

print("  ✅ Created workflow with 2 template steps (research + chat)")
print("")

-- ============================================================
-- Execute Workflow
-- ============================================================

print("2. Executing workflow...")
print("  → Research phase: Gathering information on '" .. (args.topic or "Rust async programming") .. "'")
print("  → Chat phase: Answering question with research context")
print("")

-- Execute the workflow (synchronous execution)
local result = workflow:execute()

-- ============================================================
-- Display Results
-- ============================================================

print("")
print("3. Results:")
print("=============================================================")

if result and result.success then
    print("  ✅ Workflow Status: SUCCESS")
    print("")
    print("  📊 Execution Summary:")
    print("    • Steps Completed: 2 (research + chat)")
    print("    • Session ID: " .. session_id)
    print("    • Memory Sharing: ACTIVE")
    print("    • Templates Used: research-assistant, interactive-chat")
    print("")

    -- Display research step output
    if result.outputs and result.outputs.research then
        print("  📚 Research Phase Output:")
        local research_output = result.outputs.research
        if type(research_output) == "string" then
            -- Truncate long outputs for display
            local display_output = research_output
            if #research_output > 500 then
                display_output = research_output:sub(1, 500) .. "\n    ... (truncated, see full output in memory)"
            end
            print("    " .. display_output:gsub("\n", "\n    "))
        elseif type(research_output) == "table" and research_output.content then
            local content = research_output.content
            if #content > 500 then
                content = content:sub(1, 500) .. "\n    ... (truncated)"
            end
            print("    " .. content:gsub("\n", "\n    "))
        end
        print("")
    end

    -- Display chat step output
    if result.outputs and result.outputs.chat then
        print("  💬 Chat Phase Output:")
        local chat_output = result.outputs.chat
        if type(chat_output) == "string" then
            print("    " .. chat_output:gsub("\n", "\n    "))
        elseif type(chat_output) == "table" and chat_output.content then
            print("    " .. chat_output.content:gsub("\n", "\n    "))
        end
        print("")
    end

    print("  🔄 To Continue This Conversation:")
    print("    llmspell template exec interactive-chat \\")
    print("      --param session_id=" .. session_id .. " \\")
    print("      --param message=\"Your next question here\"")
    print("")
    print("  🧠 Memory Status:")
    print("    • Research findings stored in session: " .. session_id)
    print("    • Chat can access research context via session_id")
    print("    • Run additional chat commands with same session_id to continue")
    print("")
else
    print("  ❌ Workflow Status: FAILED")
    if result and result.error then
        print("  Error: " .. tostring(result.error))
    end
    print("")
end

print("=============================================================")
print("")

-- ============================================================
-- Architecture Demonstration
-- ============================================================

print("📘 What This Demonstrates (Phase 13 Completion):")
print("")
print("  1. Workflow-Template Delegation:")
print("     • Workflows can execute templates as steps via StepType::Template")
print("     • TemplateBridge flows: TemplateBridge → WorkflowBridge → Workflows → StepExecutionContext")
print("")
print("  2. Session-Based Memory Sharing:")
print("     • Research template stores findings in RAG (session_id anchor)")
print("     • Chat template retrieves research context (same session_id)")
print("     • Memory persists across template executions")
print("")
print("  3. Composition Pattern (Option E):")
print("     • Templates = reusable AI behaviors")
print("     • Workflows = orchestration of templates")
print("     • Result = flexible, composable AI applications")
print("")

print("✅ Research-Chat v1.0 Complete!")
print("   Phase 13 (Adaptive Memory System) - Workflow-Template Integration Validated")
print("")
