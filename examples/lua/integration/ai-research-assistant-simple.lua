-- ABOUTME: Simplified AI-Powered Research Assistant demonstrating LLM agents with hooks and events
-- ABOUTME: Shows practical integration of agents, hooks, and events for real-world use cases

print("=== AI-Powered Research Assistant (Simplified) ===")
print("Demonstrates: LLM agents with hooks and events integration")
print("Requirements: OPENAI_API_KEY or ANTHROPIC_API_KEY for real agent execution")
print()
print("Note: This example demonstrates hook and event integration patterns.")
print("      Real LLM agent execution may require additional setup.")
print("      Simulation mode (no API keys) demonstrates the full workflow.")
print()

-- Check for API keys
local has_openai = os.getenv("OPENAI_API_KEY") ~= nil
local has_anthropic = os.getenv("ANTHROPIC_API_KEY") ~= nil
local has_any_provider = has_openai or has_anthropic

if not has_any_provider then
    print("⚠️  WARNING: No API keys found. Running in simulation mode.")
    print("   To use real LLM agents, set OPENAI_API_KEY or ANTHROPIC_API_KEY")
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
    provider = has_openai and "gpt-4o-mini" or has_anthropic and "claude-instant-1" or "mock",
    research_topic = "quantum computing applications in cryptography"
}

-- Research state tracking
local research_state = {
    hooks_triggered = 0,
    events_published = 0,
    agents_executed = 0,
    findings = {}
}

print("1. Setting up event monitoring:")

-- Create event subscriptions
local event_subs = {
    all_research = Event.subscribe("research.*"),
    agent_events = Event.subscribe("research.agent.*"),
    findings = Event.subscribe("research.finding.*")
}

print("   ✅ Event subscriptions created")

print()
print("2. Setting up monitoring hooks:")

-- Hook: Monitor agent execution
local agent_hook = Hook.register("BeforeAgentExecution", function(context)
    research_state.hooks_triggered = research_state.hooks_triggered + 1
    
    print(string.format("   🔍 [HOOK] Agent '%s' starting execution", 
          context.component_id and context.component_id.name or "unknown"))
    
    Event.publish("research.agent.executing", {
        agent = context.component_id and context.component_id.name or "unknown",
        timestamp = os.time()
    })
    
    return "continue"
end, "high")

-- Hook: Capture agent results
local result_hook = Hook.register("AfterAgentExecution", function(context)
    research_state.hooks_triggered = research_state.hooks_triggered + 1
    
    if context.result and context.result.output then
        print(string.format("   💡 [HOOK] Agent '%s' completed with result", 
              context.component_id and context.component_id.name or "unknown"))
        
        Event.publish("research.finding.captured", {
            agent = context.component_id and context.component_id.name or "unknown",
            finding = context.result.output,
            timestamp = os.time()
        })
        
        table.insert(research_state.findings, {
            agent = context.component_id and context.component_id.name or "unknown",
            content = context.result.output
        })
    end
    
    return "continue"
end, "normal")

print("   ✅ Monitoring hooks registered")

print()
print("3. Creating research agents:")

-- Helper to create agent (real or simulated)
local function create_agent(name, role, system_prompt)
    if config.use_real_llm then
        -- Create real agent
        local agent = Agent.create({
            name = name,
            model = config.provider,
            temperature = 0.7,
            system_prompt = system_prompt
        })
        
        Event.publish("research.agent.created", {
            name = name,
            role = role,
            provider = config.provider,
            timestamp = os.time()
        })
        
        print(string.format("   ✅ Created real agent: %s (%s)", name, config.provider))
        
        -- Initialize the agent
        local init_success, init_err = pcall(function()
            if agent.initialize then
                agent:initialize()
            end
        end)
        
        if not init_success then
            print(string.format("   ⚠️  Warning: Could not initialize agent %s: %s", name, tostring(init_err)))
        end
        
        return agent
    else
        -- Create simulated agent
        print(string.format("   🤖 Created simulated agent: %s", name))
        
        Event.publish("research.agent.created", {
            name = name,
            role = role,
            simulated = true,
            timestamp = os.time()
        })
        
        return {
            name = name,
            invoke = function(self, params)
                research_state.agents_executed = research_state.agents_executed + 1
                
                -- Simulate response based on role
                local responses = {
                    researcher = "Based on my research, quantum computing poses both opportunities and threats to cryptography. Key findings include: 1) Quantum computers can break RSA and ECC encryption, 2) Post-quantum cryptography algorithms are being developed, 3) Quantum key distribution offers unbreakable communication.",
                    analyst = "Analysis reveals three critical areas: 1) Timeline - practical quantum computers capable of breaking encryption are 10-20 years away, 2) Impact - current public key infrastructure needs complete overhaul, 3) Solutions - NIST has standardized several post-quantum algorithms.",
                    synthesizer = "Summary: Quantum computing represents a paradigm shift for cryptography. Organizations should: 1) Begin transitioning to post-quantum algorithms, 2) Implement crypto-agility in systems, 3) Monitor quantum computing developments closely. The transition period is critical for maintaining security."
                }
                
                local response = responses[self.name] or "Simulated response for: " .. (params.text or "unknown query")
                
                Event.publish("research.agent.response", {
                    agent = self.name,
                    response_length = #response,
                    simulated = true,
                    timestamp = os.time()
                })
                
                return {text = response}
            end
        }
    end
end

-- Create research team
local researcher = create_agent(
    "researcher",
    "Research Specialist",
    "You are an expert researcher specializing in quantum computing and cryptography. Provide detailed, accurate information about current developments and implications."
)

local analyst = create_agent(
    "analyst", 
    "Security Analyst",
    "You are a security analyst focused on cryptographic systems. Analyze the impact of quantum computing on current and future security infrastructure."
)

local synthesizer = create_agent(
    "synthesizer",
    "Technical Writer", 
    "You are a technical writer who creates clear, actionable summaries. Synthesize complex technical information into practical recommendations."
)

print()
print("4. Conducting research with event-driven coordination:")
print(string.rep("-", 60))

-- Small delay to ensure agents are ready
os.execute("sleep 0.5")

-- Phase 1: Initial Research
print("📚 Phase 1: Initial Research")
Event.publish("research.phase.started", {phase = "initial_research", timestamp = os.time()})

local success, research_result = pcall(function()
    return researcher:invoke({
        text = "What are the current and emerging applications of quantum computing in cryptography? Include both threats and opportunities."
    })
end)

if not success then
    print("   ⚠️  Agent invocation failed: " .. tostring(research_result))
    research_result = {text = "Error invoking agent: " .. tostring(research_result)}
end

print("   ✅ Research completed")
Event.publish("research.phase.completed", {
    phase = "initial_research", 
    result_size = research_result and research_result.text and #research_result.text or 0,
    timestamp = os.time()
})

-- Brief pause for event processing
os.execute("sleep 1")

-- Phase 2: Deep Analysis
print()
print("🔍 Phase 2: Deep Analysis")
Event.publish("research.phase.started", {phase = "analysis", timestamp = os.time()})

local analysis_prompt = "Analyze the following research on quantum computing in cryptography and identify critical security implications: " .. 
                       (research_result and research_result.text or "No research data available")

local analysis_success, analysis_result = pcall(function()
    return analyst:invoke({
        text = analysis_prompt
    })
end)

if not analysis_success then
    print("   ⚠️  Analysis failed: " .. tostring(analysis_result))
    analysis_result = {text = "Error during analysis: " .. tostring(analysis_result)}
end

print("   ✅ Analysis completed")
Event.publish("research.phase.completed", {
    phase = "analysis",
    timestamp = os.time()
})

-- Phase 3: Synthesis and Recommendations
print()
print("📝 Phase 3: Synthesis and Recommendations")
Event.publish("research.phase.started", {phase = "synthesis", timestamp = os.time()})

local synthesis_prompt = string.format(
    "Create an executive summary with actionable recommendations based on:\n\nResearch: %s\n\nAnalysis: %s",
    research_result and research_result.text or "No research data",
    analysis_result and analysis_result.text or "No analysis data"
)

local synthesis_success, synthesis_result = pcall(function()
    return synthesizer:invoke({
        text = synthesis_prompt
    })
end)

if not synthesis_success then
    print("   ⚠️  Synthesis failed: " .. tostring(synthesis_result))
    synthesis_result = {text = "Error during synthesis: " .. tostring(synthesis_result)}
end

print("   ✅ Synthesis completed")
Event.publish("research.phase.completed", {
    phase = "synthesis",
    timestamp = os.time()
})

print(string.rep("-", 60))

-- Process monitoring events
print()
print("5. Processing monitoring events:")

local processed_events = 0
local event_summary = {}

-- Collect events from all subscriptions
for sub_name, sub_id in pairs(event_subs) do
    while true do
        local event = Event.receive(sub_id, 100) -- 100ms timeout
        if not event then break end
        
        processed_events = processed_events + 1
        if event.type then
            event_summary[event.type] = (event_summary[event.type] or 0) + 1
        end
        
        -- Log interesting events
        if event.type == "research.finding.captured" then
            print(string.format("   💡 Finding captured from %s", event.data.agent))
        elseif event.type == "research.agent.executing" then
            print(string.format("   🤖 Agent %s executed", event.data.agent))
        end
    end
end

print(string.format("   📊 Processed %d events", processed_events))

-- Show event type summary
if next(event_summary) then
    print("   Event types:")
    for event_type, count in pairs(event_summary) do
        print(string.format("     • %s: %d", event_type, count))
    end
end

print()
print("6. Research Summary:")
print(string.rep("=", 60))

-- Display results
if config.use_real_llm then
    print("🔬 RESEARCH FINDINGS:")
    print(research_result and research_result.text or "No research data")
    print()
    
    print("📊 ANALYSIS:")
    print(analysis_result and analysis_result.text or "No analysis data")
    print()
    
    print("📋 EXECUTIVE SUMMARY:")
    print(synthesis_result and synthesis_result.text or "No synthesis data")
else
    print("🔬 RESEARCH FINDINGS (Simulated):")
    print("   " .. (research_result and research_result.text or "No data"))
    print()
    
    print("📊 ANALYSIS (Simulated):")
    print("   " .. (analysis_result and analysis_result.text or "No data"))
    print()
    
    print("📋 RECOMMENDATIONS (Simulated):")
    print("   " .. (synthesis_result and synthesis_result.text or "No data"))
end

print()
print("📈 Execution Metrics:")
print(string.format("   • Agents created: 3"))
print(string.format("   • Agents executed: %d", config.use_real_llm and 3 or research_state.agents_executed))
print(string.format("   • Hooks triggered: %d", research_state.hooks_triggered))
print(string.format("   • Events published: %d", processed_events))
print(string.format("   • Findings captured: %d", #research_state.findings))

-- Cleanup
print()
print("7. Cleanup:")

-- Unsubscribe from events
for sub_name, sub_id in pairs(event_subs) do
    Event.unsubscribe(sub_id)
end
print("   ✅ Unsubscribed from events")

-- Unregister hooks
Hook.unregister(agent_hook)
Hook.unregister(result_hook)
print("   ✅ Unregistered hooks")

print()
print(string.rep("=", 60))
print("✨ AI-Powered Research Assistant example complete!")
print()
print("Key Integration Points Demonstrated:")
print("   • Real LLM agents (when API keys available)")
print("   • Event-driven coordination between agents")
print("   • Hook-based monitoring and data capture")
print("   • Multi-phase research workflow")
print("   • Comprehensive metrics and monitoring")

if not config.use_real_llm then
    print()
    print("💡 To run with real LLM agents:")
    print("   export OPENAI_API_KEY='your-key-here'")
    print("   llmspell run examples/lua/integration/ai-research-assistant-simple.lua")
end