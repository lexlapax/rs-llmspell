-- Application: Production Data Pipeline v2.0
-- Purpose: Production ETL with LLM-powered quality analysis and anomaly detection
-- Prerequisites: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variables
-- Expected Output: Multi-phase data pipeline with parallel processing and LLM analysis
-- Version: 0.7.0
-- Tags: application, data-pipeline, production, workflow, agents, parallel
--
-- HOW TO RUN:
-- 1. Basic (no API keys): ./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
-- 2. With config: LLMSPELL_CONFIG=examples/script-users/applications/data-pipeline/config.toml ./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
-- 3. Full features: export OPENAI_API_KEY="sk-..." && export ANTHROPIC_API_KEY="sk-ant-..." && ./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
--
-- ABOUTME: Production ETL pipeline demonstrating nested workflows and LLM analysis
-- ABOUTME: Shows Sequential + Parallel workflow composition with 5 specialized agents

print("=== Production Data Pipeline v2.0 ===")
print("Blueprint-compliant architecture demonstration\n")

-- ============================================================
-- Configuration
-- ============================================================

local config = {
    pipeline_name = "production_data_pipeline_v2",
    models = {
        enricher = "openai/gpt-3.5-turbo",
        quality = "openai/gpt-4o-mini",
        anomaly = "openai/gpt-4o-mini", 
        patterns = "anthropic/claude-3-haiku-20240307",
        report = "anthropic/claude-3-5-sonnet-20241022"
    },
    files = {
        input = "/tmp/pipeline_input.txt",
        output = "/tmp/pipeline_output.txt",
        report = "/tmp/pipeline_report.txt"
    }
}

-- ============================================================
-- Step 1: Create LLM Agents (5 per blueprint)
-- ============================================================

print("1. Creating 5 LLM Agents per blueprint...")

-- Store agent names for workflow steps
local agent_names = {}
local timestamp = os.time()

-- Data Enricher Agent
agent_names.enricher = "data_enricher_" .. timestamp
local data_enricher = Agent.builder()
    :name(agent_names.enricher)
    :description("Adds contextual information to data")
    :type("llm")
    :model(config.models.enricher)
    :temperature(0.3)
    :max_tokens(300)
    :custom_config({
        system_prompt = "You are a data enrichment specialist. Add context and metadata to data records."
    })
    :build()

print(data_enricher and "  ✅ Data Enricher Agent created" or "  ⚠️ Data Enricher needs API key")

-- Quality Analyzer Agent
agent_names.quality = "quality_analyzer_" .. timestamp
local quality_analyzer = Agent.builder()
    :name(agent_names.quality)
    :description("Identifies data quality issues")
    :type("llm")
    :model(config.models.quality)
    :temperature(0.2)
    :max_tokens(500)
    :custom_config({
        system_prompt = "You are a data quality expert. Find missing values, inconsistencies, and quality issues."
    })
    :build()

print(quality_analyzer and "  ✅ Quality Analyzer Agent created" or "  ⚠️ Quality Analyzer needs API key")

-- Anomaly Detector Agent
agent_names.anomaly = "anomaly_detector_" .. timestamp
local anomaly_detector = Agent.builder()
    :name(agent_names.anomaly)
    :description("Finds outliers and anomalies")
    :type("llm")
    :model(config.models.anomaly)
    :temperature(0.3)
    :max_tokens(400)
    :custom_config({
        system_prompt = "You are an anomaly detection specialist. Identify outliers and unusual patterns."
    })
    :build()

print(anomaly_detector and "  ✅ Anomaly Detector Agent created" or "  ⚠️ Anomaly Detector needs API key")

-- Pattern Finder Agent
agent_names.patterns = "pattern_finder_" .. timestamp
local pattern_finder = Agent.builder()
    :name(agent_names.patterns)
    :description("Discovers patterns and trends")
    :type("llm")
    :model(config.models.patterns)
    :temperature(0.4)
    :max_tokens(500)
    :custom_config({
        system_prompt = "You are a pattern recognition expert. Find recurring patterns and correlations."
    })
    :build()

print(pattern_finder and "  ✅ Pattern Finder Agent created" or "  ⚠️ Pattern Finder needs API key")

-- Report Generator Agent
agent_names.report = "report_generator_" .. timestamp
local report_generator = Agent.builder()
    :name(agent_names.report)
    :description("Creates comprehensive insights report")
    :type("llm")
    :model(config.models.report)
    :temperature(0.6)
    :max_tokens(800)
    :custom_config({
        system_prompt = "You are a report writer. Create executive summaries with key findings and recommendations."
    })
    :build()

print(report_generator and "  ✅ Report Generator Agent created" or "  ⚠️ Report Generator needs API key")

-- ============================================================
-- Step 2: Prepare Sample Data
-- ============================================================

print("\n2. Preparing sample data files...")

-- Create sample input data
local sample_data = [[
Customer Records:
- ID: 1001, Name: John Doe, Region: NA, Revenue: $45000, Status: Active
- ID: 1002, Name: Jane Smith, Region: EU, Revenue: $32000, Status: Active
- ID: 1003, Name: Bob Johnson, Region: APAC, Revenue: $999999, Status: Inactive (ANOMALY: Unusual revenue)
- ID: 1004, Name: Alice Brown, Region: NA, Revenue: $28000, Status: Active
- ID: 1005, Name: Charlie Wilson, Region: EU, Revenue: -$500, Status: Error (ANOMALY: Negative revenue)

Transaction Records:
- TXN001: Amount: $1500, Status: Completed, Date: 2024-11-15
- TXN002: Amount: $2200, Status: Pending, Date: 2024-11-16
- TXN003: Amount: $50000, Status: Failed, Date: 2024-11-16 (ANOMALY: Large failed transaction)
- TXN004: Amount: $800, Status: Completed, Date: 2024-11-16
- TXN005: Amount: $1200, Status: Completed, Date: 2024-11-16

Product Inventory:
- PROD001: Electronics, Stock: 150, Price: $299
- PROD002: Clothing, Stock: 0, Price: $49 (ISSUE: Out of stock)
- PROD003: Food, Stock: 500, Price: $15
- PROD004: Books, Stock: 75, Price: $25
- PROD005: Electronics, Stock: -10, Price: $199 (ERROR: Negative stock)
]]

-- Save sample data to input file
Tool.invoke("file_operations", {
    operation = "write",
    path = config.files.input,
    input = sample_data
})
print("  ✅ Created input data file: " .. config.files.input)

-- ============================================================
-- Step 3: Create Nested Workflows
-- ============================================================

print("\n3. Creating nested workflows...")

-- 3.1: Extract Phase - Read input data
local extract_workflow = Workflow.builder()
    :name("extract_phase")
    :description("Extract data from input file")
    :sequential()
    
    :add_step({
        name = "read_input",
        type = "tool",
        tool = "file_operations",
        input = {
            operation = "read",
            path = config.files.input
        }
    })
    
    :build()

print("  ✅ Extract workflow created")

-- 3.2: Analysis Phase - Parallel analysis with different agents
local analysis_workflow = Workflow.builder()
    :name("analysis_phase")
    :description("Parallel analysis by multiple agents")
    :parallel()

-- Add quality analysis if agent exists
if quality_analyzer then
    analysis_workflow:add_step({
        name = "quality_analysis",
        type = "agent",
        agent = agent_names.quality,  -- Use stored agent name
        input = "Analyze this data for quality issues:\n" .. sample_data
    })
end

-- Add anomaly detection if agent exists
if anomaly_detector then
    analysis_workflow:add_step({
        name = "anomaly_detection",
        type = "agent",
        agent = agent_names.anomaly,  -- Use stored agent name
        input = "Detect anomalies in this data:\n" .. sample_data
    })
end

-- Add pattern finding if agent exists
if pattern_finder then
    analysis_workflow:add_step({
        name = "pattern_discovery",
        type = "agent",
        agent = agent_names.patterns,  -- Use stored agent name
        input = "Find patterns in this data:\n" .. sample_data
    })
end

-- If no agents available, add a simple tool step
if not quality_analyzer and not anomaly_detector and not pattern_finder then
    analysis_workflow:add_step({
        name = "basic_analysis",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "word_count",
            input = sample_data
        }
    })
end

analysis_workflow = analysis_workflow:build()
print("  ✅ Analysis workflow created")

-- 3.3: Report Phase - Generate final report
local report_workflow = Workflow.builder()
    :name("report_phase")
    :description("Generate final report")
    :sequential()

if report_generator then
    report_workflow:add_step({
        name = "generate_report",
        type = "agent",
        agent = agent_names.report,  -- Use stored agent name
        input = "Generate an executive report for this data pipeline analysis:\n" .. sample_data
    })
else
    -- Fallback to simple file write
    report_workflow:add_step({
        name = "write_report",
        type = "tool",
        tool = "file_operations",
        input = {
            operation = "write",
            path = config.files.report,
            input = "Data Pipeline Report\n" .. 
                   "====================\n" ..
                   "Pipeline executed successfully.\n" ..
                   "Input data processed.\n" ..
                   "Analysis completed.\n"
        }
    })
end

report_workflow = report_workflow:build()
print("  ✅ Report workflow created")

-- 3.4: Main Pipeline - Sequential orchestration of all phases
local main_pipeline = Workflow.builder()
    :name("data_pipeline_main")
    :description("Main data pipeline orchestrator")
    :sequential()
    
    -- Phase 1: Extract
    :add_step({
        name = "extraction",
        type = "workflow",
        workflow = extract_workflow
    })
    
    -- Phase 2: Analysis
    :add_step({
        name = "analysis",
        type = "workflow",
        workflow = analysis_workflow
    })
    
    -- Phase 3: Report
    :add_step({
        name = "reporting",
        type = "workflow",
        workflow = report_workflow
    })
    
    :build()

print("  ✅ Main pipeline workflow created (3 nested phases)")

-- ============================================================
-- Step 4: Execute Pipeline
-- ============================================================

print("\n4. Executing data pipeline...")
print("=" .. string.rep("=", 50))

local start_time = os.time()
local result = main_pipeline:execute({
    initial_context = {
        pipeline_name = config.pipeline_name,
        start_time = start_time
    }
})
local end_time = os.time()

-- ============================================================
-- Step 5: Display Results
-- ============================================================

print("\n5. Pipeline Results:")
print("=" .. string.rep("=", 50))

if result then
    print("  ✅ Pipeline Status: COMPLETED")
    print("  ⏱️  Execution Time: " .. (end_time - start_time) .. " seconds")
    
    -- Display phase results
    if result.extraction then
        print("\n  📥 Extraction Phase: ✅ Completed")
    end
    
    if result.analysis then
        print("  🔍 Analysis Phase: ✅ Completed")
        if quality_analyzer or anomaly_detector or pattern_finder then
            print("    • LLM analysis performed")
        else
            print("    • Basic analysis performed (no API keys)")
        end
    end
    
    if result.reporting then
        print("  📊 Reporting Phase: ✅ Completed")
        if report_generator then
            print("    • Executive report generated")
        else
            print("    • Basic report created")
        end
    end
    
    -- Save final output
    Tool.invoke("file_operations", {
        operation = "write",
        path = config.files.output,
        input = "Pipeline Execution Summary\n" ..
                "===========================\n" ..
                "Status: SUCCESS\n" ..
                "Duration: " .. (end_time - start_time) .. " seconds\n" ..
                "Phases Completed: 3/3\n" ..
                "Timestamp: " .. os.date("%Y-%m-%d %H:%M:%S") .. "\n"
    })
    
    print("\n  💾 Output saved to: " .. config.files.output)
    if report_generator or not quality_analyzer then
        print("  📄 Report saved to: " .. config.files.report)
    end
else
    print("  ❌ Pipeline Status: FAILED")
    print("  ⚠️ Check logs for details")
end

print("\n" .. "=" .. string.rep("=", 50))
print("Pipeline execution complete!")
print("\nThis example demonstrates:")
print("  • Nested workflow composition (Sequential + Parallel)")
print("  • 5 specialized LLM agents (when API keys available)")
print("  • Tool integration for file operations")
print("  • Graceful degradation without API keys")
print("  • Blueprint v2.0 architecture (simplified for llmspell)")