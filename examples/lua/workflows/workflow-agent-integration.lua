-- ABOUTME: Workflow-Agent integration example demonstrating AI-powered automation
-- ABOUTME: Shows core patterns of combining agents and workflows for intelligent processing

-- Note: All workflow and agent methods are now synchronous - no helpers needed

print("=== Workflow-Agent Integration Example ===\n")

-- Core Concept 1: Agents as Intelligent Steps
-- Unlike deterministic tools, agents bring reasoning and context understanding
print("Concept 1: Agents as Intelligent Steps")
print("-" .. string.rep("-", 38))

-- Create a content analysis workflow where agents provide intelligence
local content_analysis = Workflow.sequential({
    name = "intelligent_content_analysis",
    description = "Analyze content using AI reasoning vs pure tools",
    
    steps = {
        -- Step 1: Create sample content
        {
            name = "create_sample",
            type = "tool", 
            tool = "template_engine",
            input = {
                template = [[Breaking: Local startup SecureVault raises $2M in Series A funding. 
The cybersecurity company, founded in 2022, plans to expand their team from 12 to 25 employees. 
CEO Sarah Chen says "This funding validates our approach to zero-trust architecture."
However, some industry experts question whether the market is oversaturated.]],
                variables = {}
            }
        },
        
        -- Step 2: Tool-based analysis (deterministic)
        {
            name = "tool_analysis",
            type = "tool",
            tool = "text_manipulator", 
            input = {
                input = "{{step:create_sample:output}}",
                operation = "word_count"
            }
        },
        
        -- Step 3: Agent-based analysis (intelligent)
        {
            name = "agent_analysis", 
            type = "agent",
            agent = "content_analyzer",
            input = {
                text = "{{step:create_sample:output}}",
                prompt = "Analyze this news content for: 1) Key business insights 2) Market sentiment 3) Strategic implications. Be concise."
            }
        },
        
        -- Step 4: Compare approaches
        {
            name = "comparison_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
=== Content Analysis Comparison ===
Tool Analysis (deterministic): {{tool_result}} words
Agent Analysis (intelligent): {{agent_result}}

Key Difference: Tools count, Agents understand meaning and context.
]],
                variables = {
                    tool_result = "{{step:tool_analysis:output}}",
                    agent_result = "{{step:agent_analysis:output}}"
                }
            }
        }
    }
})

-- Execute and show the difference
local analysis_result = content_analysis:execute()
if analysis_result and analysis_result.success then
    print("✓ Content analysis completed - shows agent intelligence vs tool determinism")
else
    print("✗ Analysis failed")
end

-- Core Concept 2: Agent-Driven Routing 
-- Agent decisions drive workflow branching logic
print("\n\nConcept 2: Agent-Driven Routing")
print("-" .. string.rep("-", 31))

-- Create sample customer emails to route
local customer_emails = {
    "Hi, I can't log into my account after password reset",
    "Your billing charged me twice this month, please refund", 
    "Love the new features! Any plans for mobile app?",
    "System is down, urgent - our production is affected"
}

-- Agent-driven email routing workflow
for i, email in ipairs(customer_emails) do
    print(string.format("\nRouting Email #%d: %s", i, email:sub(1, 40) .. "..."))
    
    local email_router = Workflow.conditional({
        name = "intelligent_email_router",
        description = "Route emails based on AI understanding",
        
        branches = {
            -- Technical issue branch
            {
                name = "technical_support",
                condition = {
                    type = "agent_decision",
                    agent = "email_classifier", 
                    input = {
                        email = email,
                        prompt = "Is this a technical support issue? Answer yes or no."
                    },
                    expected = "yes"
                },
                steps = {
                    {
                        name = "tech_response",
                        type = "tool",
                        tool = "template_engine", 
                        input = {
                            template = "→ Routed to TECHNICAL SUPPORT (Priority: High)",
                            variables = {}
                        }
                    }
                }
            },
            
            -- Billing issue branch  
            {
                name = "billing_support",
                condition = {
                    type = "agent_decision",
                    agent = "email_classifier",
                    input = {
                        email = email,
                        prompt = "Is this a billing or payment issue? Answer yes or no."
                    },
                    expected = "yes"
                },
                steps = {
                    {
                        name = "billing_response", 
                        type = "tool",
                        tool = "template_engine",
                        input = {
                            template = "→ Routed to BILLING DEPARTMENT",
                            variables = {}
                        }
                    }
                }
            },
            
            -- Default: General support
            {
                name = "general_support",
                condition = { type = "always" },
                steps = {
                    {
                        name = "general_response",
                        type = "tool", 
                        tool = "template_engine",
                        input = {
                            template = "→ Routed to GENERAL SUPPORT",
                            variables = {}
                        }
                    }
                }
            }
        }
    })
    
    local route_result = email_router:execute()
    if route_result and route_result.success then
        print("  ✓ Email routed successfully")
    end
end

-- Core Concept 3: Multi-Agent Collaboration
-- Specialized agents working together toward a common goal
print("\n\nConcept 3: Multi-Agent Collaboration") 
print("-" .. string.rep("-", 36))

-- Market research scenario with specialized agents
local market_research = Workflow.parallel({
    name = "collaborative_market_research",
    description = "Multiple specialized agents analyze different aspects",
    
    branches = {
        -- Competitive analysis agent
        {
            name = "competitive_analysis", 
            steps = {
                {
                    name = "analyze_competitors",
                    type = "agent",
                    agent = "competitive_analyst",
                    input = {
                        product = "AI-powered email assistant",
                        competitors = "Gmail Smart Compose, Outlook Editor, Grammarly",
                        prompt = "Analyze competitive landscape focusing on differentiation opportunities"
                    }
                }
            }
        },
        
        -- Market sizing agent
        {
            name = "market_sizing",
            steps = {
                {
                    name = "estimate_market",
                    type = "agent", 
                    agent = "market_analyst",
                    input = {
                        market = "Email productivity tools",
                        prompt = "Estimate total addressable market and growth trends"
                    }
                }
            }
        },
        
        -- Pricing strategy agent
        {
            name = "pricing_strategy",
            steps = {
                {
                    name = "recommend_pricing",
                    type = "agent",
                    agent = "pricing_strategist", 
                    input = {
                        product_type = "SaaS email assistant",
                        prompt = "Recommend pricing model and tiers for market entry"
                    }
                }
            }
        }
    },
    
    -- Synthesis agent combines all insights
    post_steps = {
        {
            name = "synthesize_research",
            type = "agent",
            agent = "research_synthesizer",
            input = {
                competitive_analysis = "{{branch:competitive_analysis:analyze_competitors:output}}",
                market_sizing = "{{branch:market_sizing:estimate_market:output}}", 
                pricing_strategy = "{{branch:pricing_strategy:recommend_pricing:output}}",
                prompt = "Synthesize all research into executive summary with key recommendations"
            }
        }
    }
})

print("Executing collaborative market research...")
local research_result = market_research:execute()
if research_result and research_result.success then
    print("✓ Multi-agent collaboration completed")
    print("  - Competitive analysis: Done")
    print("  - Market sizing: Done") 
    print("  - Pricing strategy: Done")
    print("  - Synthesis: Done")
else
    print("✗ Collaboration failed")
end

-- Core Concept 4: Stateful Agent Context
-- Agents maintaining context and learning across iterations
print("\n\nConcept 4: Stateful Agent Context")
print("-" .. string.rep("-", 33))

-- Document review process where agent builds understanding
local documents = {
    "Product Requirements Document v1.0",
    "Technical Architecture Specification", 
    "Security Audit Report",
    "User Experience Research Findings"
}

-- Initialize review context
Tool.invoke("file_operations", {
    operation = "write",
    path = "/tmp/review_context.json", 
    input = '{"reviewed_docs": [], "key_insights": [], "concerns": []}'
})

local document_reviewer = Workflow.loop({
    name = "stateful_document_review",
    description = "Agent builds context across document reviews",
    
    iterator = {
        collection = documents
    },
    
    body = {
        -- Read current context
        {
            name = "read_context",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/review_context.json"
            }
        },
        
        -- Agent review with accumulated context
        {
            name = "review_document",
            type = "agent", 
            agent = "document_reviewer",
            input = {
                document = "{{loop:current_item}}",
                previous_context = "{{step:read_context:output}}",
                prompt = "Review this document considering previous reviews. Build on earlier insights."
            }
        },
        
        -- Update context with new insights
        {
            name = "update_context",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:read_context:output}}",
                operation = "transform",
                query = ".reviewed_docs += [\"{{loop:current_item}}\"] | .key_insights += [\"New insight from {{loop:current_item}}\"]"
            }
        },
        
        -- Save updated context
        {
            name = "save_context", 
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/review_context.json",
                content = "{{step:update_context:output}}"
            }
        }
    }
})

print("Processing documents with stateful agent context...")
local review_result = document_reviewer:execute()
if review_result and review_result.success then
    print("✓ Stateful document review completed")
    print("  Agent built understanding across " .. #documents .. " documents")
else
    print("✗ Stateful review failed")
end

-- Core Concept 5: Hierarchical Agent Coordination  
-- Supervisor agent orchestrating worker agents
print("\n\nConcept 5: Hierarchical Agent Coordination")
print("-" .. string.rep("-", 42))

-- Project management scenario with supervisor and workers
local project_coordinator = Workflow.sequential({
    name = "hierarchical_project_management",
    description = "Supervisor agent coordinates worker agents",
    
    steps = {
        -- Supervisor creates project plan
        {
            name = "create_project_plan",
            type = "agent",
            agent = "project_supervisor", 
            input = {
                project = "Launch AI email assistant MVP",
                timeline = "8 weeks",
                prompt = "Create project plan with work breakdown structure"
            }
        },
        
        -- Supervisor delegates to specialist workers
        {
            name = "delegate_development",
            type = "parallel",
            workflow = Workflow.parallel({
                branches = {
                    -- Backend development worker
                    {
                        name = "backend_work",
                        steps = {
                            {
                                name = "backend_tasks",
                                type = "agent",
                                agent = "backend_developer",
                                input = {
                                    project_plan = "{{step:create_project_plan:output}}",
                                    prompt = "Extract backend development tasks and estimate effort"
                                }
                            }
                        }
                    },
                    
                    -- Frontend development worker  
                    {
                        name = "frontend_work",
                        steps = {
                            {
                                name = "frontend_tasks", 
                                type = "agent",
                                agent = "frontend_developer",
                                input = {
                                    project_plan = "{{step:create_project_plan:output}}",
                                    prompt = "Extract frontend development tasks and estimate effort"
                                }
                            }
                        }
                    },
                    
                    -- QA testing worker
                    {
                        name = "qa_work",
                        steps = {
                            {
                                name = "qa_tasks",
                                type = "agent", 
                                agent = "qa_engineer",
                                input = {
                                    project_plan = "{{step:create_project_plan:output}}",
                                    prompt = "Create QA strategy and testing timeline"
                                }
                            }
                        }
                    }
                }
            })
        },
        
        -- Supervisor consolidates worker outputs
        {
            name = "consolidate_plan",
            type = "agent",
            agent = "project_supervisor",
            input = {
                original_plan = "{{step:create_project_plan:output}}",
                backend_plan = "{{step:delegate_development:backend_work:backend_tasks:output}}",
                frontend_plan = "{{step:delegate_development:frontend_work:frontend_tasks:output}}", 
                qa_plan = "{{step:delegate_development:qa_work:qa_tasks:output}}",
                prompt = "Consolidate worker plans into final integrated project timeline"
            }
        }
    }
})

print("Executing hierarchical project coordination...")
local coord_result = project_coordinator:execute()
if coord_result and coord_result.success then
    print("✓ Hierarchical coordination completed")
    print("  - Supervisor created plan")
    print("  - Workers specialized on domains") 
    print("  - Integration achieved")
else
    print("✗ Coordination failed")
end

-- Summary of Integration Patterns
print("\n\n=== Integration Patterns Summary ===")
print("Core agent-workflow patterns demonstrated:")
print()
print("1. INTELLIGENT STEPS: Agents bring reasoning vs deterministic tools")
print("   - Tools: Count words, manipulate text") 
print("   - Agents: Understand meaning, provide insights")
print()
print("2. AGENT-DRIVEN ROUTING: AI decisions control workflow paths")
print("   - Agent analyzes content and determines routing")
print("   - More flexible than rule-based conditions")
print()
print("3. MULTI-AGENT COLLABORATION: Specialized agents work together")
print("   - Each agent focuses on domain expertise")
print("   - Parallel processing with synthesis")
print()
print("4. STATEFUL CONTEXT: Agents build understanding over time")
print("   - Context carries forward across iterations")
print("   - Agent learns and builds on previous insights")
print()
print("5. HIERARCHICAL COORDINATION: Supervisor orchestrates workers")
print("   - Clear delegation and consolidation patterns")
print("   - Scalable for complex multi-agent scenarios")

print("\n=== Key Benefits of Agent-Workflow Integration ===")
print("• Intelligence: Move beyond deterministic automation")
print("• Flexibility: Adapt to context and unexpected situations") 
print("• Scalability: Coordinate multiple specialized agents")
print("• Learning: Build context and improve over iterations")
print("• Orchestration: Manage complex multi-step processes")

print("\n=== Workflow-Agent Integration Example Complete ===")