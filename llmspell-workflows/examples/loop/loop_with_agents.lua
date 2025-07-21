-- ABOUTME: Loop workflows with agent integration for intelligent iteration
-- ABOUTME: Demonstrates AI-powered loops for content processing and decision making

-- Agent-based content processing loop
-- Process multiple documents with AI analysis

-- Create content analysis agent
local analyzer = Agent.createAsync({
    name = "content_analyzer",
    model = "gpt-4",
    system_prompt = "You are a content analyst. Analyze text and provide insights on quality, sentiment, and key topics.",
    temperature = 0.5
})

-- Create summary agent
local summarizer = Agent.createAsync({
    name = "summarizer",
    model = "gpt-3.5-turbo",
    system_prompt = "Create concise summaries of content, highlighting key points.",
    temperature = 0.3
})

-- Documents to process
local documents = {
    {
        id = "doc1",
        title = "Introduction to Workflow Automation",
        content = "Workflow automation helps streamline business processes by reducing manual tasks..."
    },
    {
        id = "doc2", 
        title = "Best Practices for API Design",
        content = "When designing APIs, it's important to follow RESTful principles and maintain consistency..."
    },
    {
        id = "doc3",
        title = "Machine Learning in Production",
        content = "Deploying ML models requires careful consideration of scalability and monitoring..."
    }
}

State.set("documents", documents)

-- Document processing loop with AI
local doc_processor = Workflow.loop({
    name = "ai_document_processor",
    description = "Process documents with AI analysis and summarization",
    
    iterator = {
        collection = State.get("documents")
    },
    
    body = {
        -- Analyze document with AI
        {
            name = "analyze_content",
            type = "agent",
            agent = analyzer,
            input = {
                prompt = [[
Analyze this document:
Title: {{loop:current_item.title}}
Content: {{loop:current_item.content}}

Provide:
1. Quality score (1-10)
2. Main topics (comma-separated)
3. Sentiment (positive/neutral/negative)
4. Recommendations
]]
            }
        },
        
        -- Generate summary
        {
            name = "summarize_content",
            type = "agent",
            agent = summarizer,
            input = {
                prompt = "Create a 2-sentence summary of: {{loop:current_item.content}}"
            }
        },
        
        -- Extract quality score using text manipulation
        {
            name = "extract_score",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:analyze_content:output}}",
                operation = "extract",
                pattern = "Quality score: (\\d+)"
            }
        },
        
        -- Store results
        {
            name = "store_analysis",
            type = "custom",
            execute = function(context)
                local analyses = State.get("document_analyses") or {}
                
                analyses[context.current_item.id] = {
                    title = context.current_item.title,
                    analysis = context.steps.analyze_content.output,
                    summary = context.steps.summarize_content.output,
                    quality_score = tonumber(context.steps.extract_score.output) or 0,
                    processed_at = os.date("%Y-%m-%d %H:%M:%S")
                }
                
                State.set("document_analyses", analyses)
                
                return {
                    success = true,
                    output = "Analysis stored for: " .. context.current_item.id
                }
            end
        }
    },
    
    -- Continue if quality score is above threshold
    continue_condition = {
        type = "custom",
        evaluate = function(context)
            local score = tonumber(context.steps.extract_score.output) or 0
            return score >= 5  -- Only continue if quality is acceptable
        end
    },
    
    error_strategy = "continue"
})

print("Starting AI document processing...")
local doc_result = doc_processor:execute()
print("Documents processed: " .. doc_result.data.completed_iterations)

-- Display results
local analyses = State.get("document_analyses") or {}
for id, analysis in pairs(analyses) do
    print("\n" .. analysis.title .. " - Score: " .. analysis.quality_score)
end

-- Iterative improvement loop with AI feedback
local improver = Agent.createAsync({
    name = "content_improver",
    model = "gpt-4",
    system_prompt = "You improve text based on feedback. Make specific enhancements.",
    temperature = 0.7
})

local improvement_workflow = Workflow.loop({
    name = "iterative_improver",
    description = "Iteratively improve content based on AI feedback",
    
    -- Continue while quality score < 8
    iterator = {
        while_condition = {
            type = "custom",
            evaluate = function()
                local score = State.get("current_quality_score") or 0
                return score < 8
            end
        }
    },
    
    -- Maximum 5 improvement iterations
    max_iterations = 5,
    
    body = {
        -- Get current content
        {
            name = "get_content",
            type = "custom",
            execute = function()
                return {
                    success = true,
                    output = State.get("content_to_improve") or "Initial draft content here."
                }
            end
        },
        
        -- Analyze current quality
        {
            name = "analyze_quality",
            type = "agent",
            agent = analyzer,
            input = {
                prompt = [[
Rate this content quality (1-10) and explain why:
{{step:get_content:output}}

Format: "Score: X\nReason: ..."
]]
            }
        },
        
        -- Extract and store score
        {
            name = "extract_score",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:analyze_quality:output}}",
                operation = "extract",
                pattern = "Score: (\\d+)"
            },
            on_complete = function(result)
                State.set("current_quality_score", tonumber(result.output) or 0)
            end
        },
        
        -- Improve content if needed
        {
            name = "improve_content",
            type = "agent",
            agent = improver,
            input = {
                prompt = [[
Improve this content based on the feedback:

Content: {{step:get_content:output}}
Feedback: {{step:analyze_quality:output}}

Provide only the improved content.
]]
            },
            on_complete = function(result)
                State.set("content_to_improve", result.output)
                State.set("improvement_iteration", 
                    (State.get("improvement_iteration") or 0) + 1)
            end
        }
    },
    
    -- Track improvements
    on_iteration_complete = function(index, results)
        local history = State.get("improvement_history") or {}
        table.insert(history, {
            iteration = index,
            score = State.get("current_quality_score"),
            content_preview = string.sub(State.get("content_to_improve"), 1, 100) .. "..."
        })
        State.set("improvement_history", history)
    end
})

-- Initialize content for improvement
State.set("content_to_improve", "This is basic content that needs improvement for clarity.")
State.set("current_quality_score", 0)
State.set("improvement_iteration", 0)

print("\n\nStarting iterative improvement workflow...")
local improve_result = improvement_workflow:execute()

print("Improvement iterations: " .. improve_result.data.completed_iterations)
print("Final quality score: " .. State.get("current_quality_score"))

-- Customer feedback processing loop
local feedback_processor = Workflow.loop({
    name = "feedback_analyzer",
    description = "Process customer feedback with AI categorization",
    
    iterator = {
        collection = {
            "The product works great but the UI could be more intuitive",
            "Excellent customer service! Very happy with my purchase",
            "Shipping was delayed and packaging was damaged",
            "Feature request: Would love to see dark mode added",
            "Bug report: Login fails when using special characters"
        }
    },
    
    body = {
        -- Categorize feedback
        {
            name = "categorize",
            type = "agent",
            agent = Agent.createAsync({
                name = "categorizer",
                model = "gpt-3.5-turbo",
                system_prompt = "Categorize feedback into: praise, complaint, feature_request, or bug_report"
            }),
            input = {
                prompt = "Categorize this feedback (respond with only the category): {{loop:current_item}}"
            }
        },
        
        -- Analyze sentiment
        {
            name = "analyze_sentiment",
            type = "agent",
            agent = Agent.createAsync({
                name = "sentiment_analyzer",
                model = "gpt-3.5-turbo",
                system_prompt = "Analyze sentiment as positive, negative, or neutral"
            }),
            input = {
                prompt = "What is the sentiment of: {{loop:current_item}}"
            }
        },
        
        -- Generate response based on category
        {
            name = "generate_response",
            type = "agent",
            agent = Agent.createAsync({
                name = "responder",
                model = "gpt-4",
                system_prompt = "Generate appropriate customer responses"
            }),
            input = {
                prompt = [[
Generate a response for this {{step:categorize:output}} feedback:
"{{loop:current_item}}"

Be professional and helpful.
]]
            }
        },
        
        -- Store categorized feedback
        {
            name = "store_feedback",
            type = "custom",
            execute = function(context)
                local feedback_data = State.get("categorized_feedback") or {
                    praise = {},
                    complaint = {},
                    feature_request = {},
                    bug_report = {}
                }
                
                local category = context.steps.categorize.output
                local clean_category = category:gsub("%s+", "_"):lower()
                
                if feedback_data[clean_category] then
                    table.insert(feedback_data[clean_category], {
                        feedback = context.current_item,
                        sentiment = context.steps.analyze_sentiment.output,
                        response = context.steps.generate_response.output,
                        index = context.current_index
                    })
                end
                
                State.set("categorized_feedback", feedback_data)
                
                return {
                    success = true,
                    output = "Stored as: " .. clean_category
                }
            end
        }
    }
})

print("\n\nProcessing customer feedback...")
local feedback_result = feedback_processor:execute()

-- Display categorized results
local categorized = State.get("categorized_feedback") or {}
print("\nFeedback Summary:")
for category, items in pairs(categorized) do
    print("- " .. category .. ": " .. #items .. " items")
end