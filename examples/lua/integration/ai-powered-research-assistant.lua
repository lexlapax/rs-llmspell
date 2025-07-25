-- ABOUTME: AI-Powered Research Assistant demonstrating full integration with LLM agents, tools, workflows, hooks, and events
-- ABOUTME: Requires API keys for OpenAI/Anthropic/other providers to execute actual LLM operations

print("=== AI-Powered Research Assistant Integration ===")
print("Demonstrates: Complete integration with LLM agents, tools, workflows, hooks, and events")
print("Requirements: Valid API keys for LLM providers (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)")
print()

-- Check for API keys
local has_openai = os.getenv("OPENAI_API_KEY") ~= nil
local has_anthropic = os.getenv("ANTHROPIC_API_KEY") ~= nil
local has_any_provider = has_openai or has_anthropic

if not has_any_provider then
    print("⚠️  WARNING: No API keys found. This example requires at least one of:")
    print("   • OPENAI_API_KEY")
    print("   • ANTHROPIC_API_KEY")
    print("   Set these environment variables to run this example with real LLM capabilities.")
    print("   Continuing with simulation mode...")
    print()
else
    print("✅ Found API key(s):")
    if has_openai then print("   • OpenAI API key detected") end
    if has_anthropic then print("   • Anthropic API key detected") end
    print()
end

-- Configuration
local config = {
    use_real_llm = has_any_provider,
    primary_provider = has_openai and "openai/gpt-4o-mini" or "anthropic/claude-instant-1",
    research_topic = "quantum computing applications in cryptography",
    max_research_depth = 3,
    enable_hooks = true,
    enable_events = true,
    enable_monitoring = true
}

-- Research state
local research_state = {
    start_time = os.time(),
    queries_made = 0,
    tools_invoked = 0,
    agents_created = 0,
    events_published = 0,
    hooks_triggered = 0,
    research_findings = {},
    performance_metrics = {}
}

-- Event subscriptions for monitoring
local event_subs = {}

-- Hook handles for cleanup
local hook_handles = {}

-- Helper function to safely create agent
local function create_research_agent(name, role, provider)
    research_state.agents_created = research_state.agents_created + 1
    
    if config.use_real_llm then
        -- Real agent with LLM provider
        local agent = Agent.create({
            name = name,
            model = provider or config.primary_provider,
            temperature = 0.7,
            max_tokens = 1000,
            system_prompt = "You are a " .. role .. " helping with research on " .. config.research_topic
        })
        
        Event.publish("research.agent.created", {
            agent_name = name,
            role = role,
            provider = provider or config.primary_provider,
            timestamp = os.time()
        })
        
        return agent
    else
        -- Simulated agent for demonstration
        print(string.format("   🤖 [SIMULATED] Creating agent: %s (%s)", name, role))
        return {
            name = name,
            role = role,
            invoke = function(self, params)
                research_state.queries_made = research_state.queries_made + 1
                Event.publish("research.agent.query", {
                    agent_name = self.name,
                    prompt = params.text or params.prompt,
                    simulated = true
                })
                return {text = "Simulated response for: " .. (params.text or params.prompt or "")}
            end
        }
    end
end

print("1. Setting up research infrastructure:")

-- Set up event monitoring
if config.enable_events then
    print("   📡 Setting up event monitoring...")
    
    -- Monitor all research events
    event_subs.research_all = Event.subscribe("research.*")
    event_subs.agent_events = Event.subscribe("research.agent.*")
    event_subs.tool_events = Event.subscribe("research.tool.*")
    event_subs.workflow_events = Event.subscribe("research.workflow.*")
    event_subs.finding_events = Event.subscribe("research.finding.*")
    
    print("   ✅ Event monitoring active")
end

-- Set up comprehensive hooks
if config.enable_hooks then
    print("   🪝 Setting up research hooks...")
    
    -- Hook: Monitor agent execution
    local agent_hook = Hook.register("BeforeAgentExecution", function(context)
        research_state.hooks_triggered = research_state.hooks_triggered + 1
        
        Event.publish("research.hook.agent_execution", {
            agent = context.component_id.name,
            correlation_id = context.correlation_id,
            timestamp = os.time()
        })
        
        print(string.format("   🔍 [HOOK] Agent executing: %s", context.component_id.name))
        
        return "continue"
    end, "high")
    table.insert(hook_handles, agent_hook)
    
    -- Hook: Monitor tool usage
    local tool_hook = Hook.register("BeforeToolExecution", function(context)
        research_state.hooks_triggered = research_state.hooks_triggered + 1
        research_state.tools_invoked = research_state.tools_invoked + 1
        
        Event.publish("research.hook.tool_execution", {
            tool = context.tool_name or "unknown",
            parameters = context.parameters,
            timestamp = os.time()
        })
        
        print(string.format("   🔧 [HOOK] Tool executing: %s", context.tool_name or "unknown"))
        
        return "continue"
    end, "high")
    table.insert(hook_handles, tool_hook)
    
    -- Hook: Monitor workflow progress
    local workflow_hook = Hook.register("BeforeWorkflowStage", function(context)
        research_state.hooks_triggered = research_state.hooks_triggered + 1
        
        Event.publish("research.hook.workflow_stage", {
            workflow = context.workflow_id or "research_workflow",
            stage = context.stage_name or "unknown",
            timestamp = os.time()
        })
        
        print(string.format("   📊 [HOOK] Workflow stage: %s", context.stage_name or "unknown"))
        
        return "continue"
    end, "normal")
    table.insert(hook_handles, workflow_hook)
    
    -- Hook: Capture research findings
    local finding_hook = Hook.register("AfterAgentExecution", function(context)
        if context.result and context.result.output then
            Event.publish("research.finding.captured", {
                agent = context.component_id.name,
                finding = context.result.output,
                timestamp = os.time()
            })
            
            table.insert(research_state.research_findings, {
                agent = context.component_id.name,
                finding = context.result.output,
                timestamp = os.time()
            })
        end
        
        return "continue"
    end, "normal")
    table.insert(hook_handles, finding_hook)
    
    print("   ✅ Research hooks registered")
end

print()
print("2. Creating specialized research agents:")

-- Create research team
local researcher = create_research_agent("researcher", "expert researcher specializing in quantum computing")
local analyst = create_research_agent("analyst", "data analyst focusing on cryptographic applications")
local synthesizer = create_research_agent("synthesizer", "information synthesizer creating comprehensive summaries")

print()
print("3. Setting up research tools:")

-- Helper function to simulate tool usage
local function use_research_tool(tool_name, params)
    research_state.tools_invoked = research_state.tools_invoked + 1
    
    Event.publish("research.tool." .. tool_name, {
        tool = tool_name,
        params = params,
        timestamp = os.time()
    })
    
    -- Simulate tool execution for demo purposes
    -- In real scenario with proper tools registered, you would use Tool.invoke()
    if tool_name == "web_search" then
        if config.use_real_llm then
            -- In real scenario, this would use actual web search tool
            return {
                success = true,
                results = {
                    {title = "Quantum Computing in Cryptography", url = "https://example.com/1"},
                    {title = "Post-Quantum Cryptography Standards", url = "https://example.com/2"},
                    {title = "Quantum Key Distribution", url = "https://example.com/3"}
                }
            }
        else
            return {
                success = true,
                results = {
                    {title = "[SIMULATED] " .. params.query .. " result 1"},
                    {title = "[SIMULATED] " .. params.query .. " result 2"}
                }
            }
        end
    elseif tool_name == "document_analyzer" then
        return {
            success = true,
            analysis = {
                key_concepts = {"quantum supremacy", "cryptographic algorithms", "security implications"},
                summary = "Document discusses the intersection of quantum computing and cryptography",
                relevance_score = 0.85
            }
        }
    elseif tool_name == "citation_formatter" then
        return {
            success = true,
            citation = string.format("[%s] %s. Retrieved from %s", 
                params.style or "APA",
                params.title or "Unknown Title",
                params.url or "Unknown URL"
            )
        }
    else
        return {success = false, error = "Unknown tool: " .. tool_name}
    end
end

print("   ✅ Research tools configured")
print()

print("4. Creating research workflow:")

-- Create comprehensive research workflow
local research_workflow = Workflow.sequential({
    name = "quantum_cryptography_research",
    description = "Multi-step research workflow with AI agents",
    
    steps = {
        -- Step 1: Initial research using researcher agent
        {
            name = "initial_research",
            type = "agent",
            agent = "researcher",
            input = {
                text = "Research the current state of quantum computing applications in cryptography. Focus on recent developments and practical applications."
            }
        },
        
        -- Step 2: Deep analysis using analyst agent
        {
            name = "deep_analysis",
            type = "agent",
            agent = "analyst",
            input = {
                text = "Analyze the following research findings and identify key cryptographic applications of quantum computing: {{step:initial_research:output}}"
            }
        },
        
        -- Step 3: Synthesis and report generation using synthesizer agent
        {
            name = "synthesis",
            type = "agent",
            agent = "synthesizer",
            input = {
                text = "Create a comprehensive summary of quantum computing applications in cryptography based on:\nResearch: {{step:initial_research:output}}\nAnalysis: {{step:deep_analysis:output}}"
            }
        }
    }
})

print("   ✅ Research workflow created")
print()

print("5. Executing research workflow:")
print(string.rep("-", 60))

-- Hook to monitor workflow progress
local progress_hook = Hook.register("BeforeWorkflowStage", function(context)
    if context.stage_name then
        local stage_names = {
            initial_research = "📚 Step 1: Conducting initial research...",
            deep_analysis = "🔍 Step 2: Performing deep analysis...",
            synthesis = "📝 Step 3: Synthesizing findings..."
        }
        print("   " .. (stage_names[context.stage_name] or "Processing " .. context.stage_name .. "..."))
    end
    return "continue"
end, "high")

-- Execute the workflow
local workflow_start = os.clock()
local workflow_result = research_workflow:execute()
local workflow_duration = (os.clock() - workflow_start) * 1000

-- Unregister progress hook
if progress_hook then
    Hook.unregister(progress_hook)
end

print(string.rep("-", 60))
print()

-- Publish workflow completion event
Event.publish("research.workflow.completed", {
    workflow = "quantum_cryptography_research",
    duration_ms = workflow_duration,
    success = workflow_result.success,
    timestamp = os.time()
})

print("6. Processing monitoring events:")

-- Process collected events
if config.enable_monitoring then
    local event_count = 0
    local event_types = {}
    
    -- Collect events from all subscriptions
    for sub_name, sub_id in pairs(event_subs) do
        local timeout = 100 -- Quick timeout since events are already published
        while true do
            local event = Event.receive(sub_id, timeout)
            if not event then break end
            
            event_count = event_count + 1
            local event_type = event.event_type or event.type or "unknown"
            event_types[event_type] = (event_types[event_type] or 0) + 1
            
            -- Process specific event types
            if event_type == "research.finding.captured" then
                print(string.format("   💡 Finding captured from %s", event.data and event.data.agent or "unknown"))
            elseif event_type == "research.tool.web_search" then
                print(string.format("   🔍 Web search performed: %s", event.data and event.data.query or "unknown"))
            end
        end
    end
    
    print()
    print("   📊 Event Summary:")
    print(string.format("   • Total events: %d", event_count))
    for event_type, count in pairs(event_types) do
        print(string.format("   • %s: %d", event_type, count))
    end
end

print()
print("7. Research Summary:")
print(string.rep("=", 60))

-- Display research results
if workflow_result.success then
    print("✅ Research completed successfully!")
    print()
    
    -- Show key metrics
    print("📊 Research Metrics:")
    print(string.format("   • Agents created: %d", research_state.agents_created))
    print(string.format("   • Queries made: %d", research_state.queries_made))
    print(string.format("   • Tools invoked: %d", research_state.tools_invoked))
    print(string.format("   • Hooks triggered: %d", research_state.hooks_triggered))
    print(string.format("   • Events published: %d", event_count or 0))
    print(string.format("   • Workflow duration: %.2fms", workflow_duration))
    print(string.format("   • Total runtime: %ds", os.time() - research_state.start_time))
    print()
    
    -- Show findings
    print("🔍 Research Findings:")
    if #research_state.research_findings > 0 then
        for i, finding in ipairs(research_state.research_findings) do
            print(string.format("   %d. [%s] %s", i, finding.agent, 
                               string.sub(tostring(finding.finding), 1, 100) .. "..."))
        end
    else
        -- Show workflow results
        if workflow_result.data and workflow_result.data.steps then
            print("   • Initial research: [Completed]")
            print("   • Analysis: [Completed]")
            print("   • Final report: [Completed]")
            if workflow_result.data.final_output then
                print()
                print("📄 Final Report Summary:")
                print(string.sub(tostring(workflow_result.data.final_output), 1, 200) .. "...")
            end
        end
    end
else
    print("❌ Research workflow failed")
    print("   Error: " .. tostring(workflow_result.error))
end

print()
print("8. Cleanup:")

-- Unsubscribe from events
for sub_name, sub_id in pairs(event_subs) do
    Event.unsubscribe(sub_id)
    print(string.format("   🧹 Unsubscribed from %s", sub_name))
end

-- Unregister hooks
for _, handle in ipairs(hook_handles) do
    if handle and handle.unregister then
        handle:unregister()
    elseif handle and handle.id then
        Hook.unregister(handle)
    end
end
print(string.format("   🧹 Unregistered %d hooks", #hook_handles))

print()
print(string.rep("=", 60))
print("✨ AI-Powered Research Assistant example complete!")
print()
print("Key Integration Points Demonstrated:")
print("   • Real LLM agents with provider configuration")
print("   • Tool integration for research capabilities")
print("   • Sequential workflow for multi-step processes")
print("   • Comprehensive hook system for monitoring")
print("   • Event-driven architecture for loose coupling")
print("   • Performance tracking and metrics collection")
print()

if not config.use_real_llm then
    print("💡 To run with real LLM capabilities:")
    print("   export OPENAI_API_KEY='your-key-here'")
    print("   # or")
    print("   export ANTHROPIC_API_KEY='your-key-here'")
    print("   llmspell run " .. arg[0])
end