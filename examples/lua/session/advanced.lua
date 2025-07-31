-- ABOUTME: Example demonstrating advanced session patterns and best practices
-- ABOUTME: Shows session hierarchies, metadata management, and performance optimization

-- CONFIG: Requires runtime integration (see README.md for current status)
-- WHY: Advanced patterns enable complex workflows, better organization, and scalability
-- STATUS: Session/Artifact globals implemented but not yet integrated into CLI runtime
-- TODO: Runtime needs to initialize SessionManager - see llmspell-bridge/src/runtime.rs

print("🚀 Advanced Session Patterns Example")
print("====================================")

-- This example demonstrates:
-- 1. Complex session hierarchies
-- 2. Dynamic metadata management
-- 3. Session templates and cloning
-- 4. Bulk operations and performance
-- 5. Session metrics and analytics
-- 6. Advanced query patterns

-- Helper to create a session template
local function create_session_template(template_name, base_config)
    return {
        name = template_name .. " Session",
        tags = base_config.tags or {},
        metadata = base_config.metadata or {},
        description = base_config.description
    }
end

-- Helper to track session metrics
local function track_metrics(session_id, metric_type, value)
    -- In a real implementation, this would integrate with monitoring
    Artifact.store(
        session_id,
        "system_generated",
        "metrics_" .. os.time() .. ".json",
        JSON.stringify({
            metric = metric_type,
            value = value,
            timestamp = os.date("!%Y-%m-%dT%H:%M:%SZ")
        }),
        {
            mime_type = "application/json",
            metric_type = metric_type
        }
    )
end

-- Step 1: Complex session hierarchy
print("\n1. Building Session Hierarchy")
print(string.rep("-", 40))

-- Create root analysis session
local root_session = Session.create({
    name = "Q4 2024 Company Analysis",
    description = "Comprehensive quarterly analysis",
    tags = {"analysis", "q4-2024", "executive"},
    metadata = {
        quarter = "Q4",
        year = 2024,
        departments = {"sales", "marketing", "engineering", "support"},
        priority = "high"
    }
})
print("📊 Created root session:", root_session)

-- Create department sub-sessions
local dept_sessions = {}
local departments = {"sales", "marketing", "engineering", "support"}

for _, dept in ipairs(departments) do
    local dept_session = Session.create({
        name = dept:gsub("^%l", string.upper) .. " Analysis",
        description = "Q4 2024 " .. dept .. " department analysis",
        parent_session_id = root_session,
        tags = {"analysis", "q4-2024", dept},
        metadata = {
            department = dept,
            parent_type = "quarterly_analysis"
        }
    })
    dept_sessions[dept] = dept_session
    print("  📁 Created " .. dept .. " session:", dept_session:sub(1, 8) .. "...")
end

-- Create regional sub-sessions under sales
local regions = {"EMEA", "APAC", "AMERICAS"}
for _, region in ipairs(regions) do
    local region_session = Session.create({
        name = region .. " Sales Analysis",
        parent_session_id = dept_sessions.sales,
        tags = {"sales", region:lower(), "q4-2024"},
        metadata = {
            region = region,
            currency = region == "EMEA" and "EUR" or "USD"
        }
    })
    print("    📍 Created " .. region .. " session:", region_session:sub(1, 8) .. "...")
end

-- Step 2: Dynamic metadata management
print("\n2. Dynamic Metadata Management")
print(string.rep("-", 40))

-- Get sales session and update metadata dynamically
local sales_meta = Session.get(dept_sessions.sales)
print("📋 Current sales metadata:")
print("  Department:", sales_meta.custom_metadata.department)

-- Simulate adding metrics as analysis progresses
local updated_meta = sales_meta.custom_metadata
updated_meta.total_revenue = 5000000
updated_meta.growth_rate = 0.15
updated_meta.top_products = {"Product A", "Product B", "Product C"}
updated_meta.analysis_complete = false

-- In real implementation, we'd have Session.update_metadata
-- For now, demonstrate the pattern
print("📊 Updated with analysis metrics:")
print("  Revenue: $" .. updated_meta.total_revenue)
print("  Growth: " .. (updated_meta.growth_rate * 100) .. "%")

-- Step 3: Session templates
print("\n3. Session Templates and Cloning")
print(string.rep("-", 40))

-- Define templates for different session types
local templates = {
    agent_conversation = create_session_template("Agent Conversation", {
        tags = {"agent", "conversation"},
        metadata = {
            max_turns = 50,
            model = "gpt-4",
            temperature = 0.7
        },
        description = "Interactive agent conversation session"
    }),
    
    data_processing = create_session_template("Data Processing", {
        tags = {"data", "pipeline"},
        metadata = {
            pipeline_version = "2.0",
            checkpoints_enabled = true,
            batch_size = 1000
        },
        description = "Data processing pipeline session"
    }),
    
    research_task = create_session_template("Research Task", {
        tags = {"research", "analysis"},
        metadata = {
            sources = {},
            citations = {},
            max_depth = 3
        },
        description = "Research and information gathering session"
    })
}

-- Create sessions from templates
print("🎯 Creating sessions from templates:")
for template_name, template in pairs(templates) do
    local session = Session.create(template)
    print("  ✓ " .. template_name .. ":", session:sub(1, 8) .. "...")
end

-- Step 4: Bulk operations and performance
print("\n4. Bulk Operations and Performance")
print(string.rep("-", 40))

-- Simulate bulk session creation
local start_time = os.clock()
local bulk_sessions = {}

print("⚡ Creating 20 task sessions...")
for i = 1, 20 do
    local session = Session.create({
        name = "Task " .. i,
        tags = {"bulk", "task", "automated"},
        metadata = {
            task_id = i,
            batch = math.floor((i - 1) / 5) + 1
        }
    })
    bulk_sessions[i] = session
end

local create_time = os.clock() - start_time
print(string.format("  ✅ Created 20 sessions in %.3f seconds", create_time))
print(string.format("  ⚡ Average: %.3f ms per session", (create_time * 1000) / 20))

-- Bulk artifact storage
print("\n📦 Bulk artifact storage test...")
start_time = os.clock()

for i = 1, 10 do
    local session = bulk_sessions[i]
    for j = 1, 5 do
        Artifact.store(
            session,
            "tool_result",
            "result_" .. j .. ".txt",
            "Task " .. i .. " result " .. j,
            {tags = {"bulk-test"}}
        )
    end
end

local store_time = os.clock() - start_time
print(string.format("  ✅ Stored 50 artifacts in %.3f seconds", store_time))

-- Step 5: Session metrics and analytics
print("\n5. Session Analytics")
print(string.rep("-", 40))

-- Analyze session distribution
local all_sessions = Session.list()
local session_by_status = {}
local session_by_tag = {}

for _, session in ipairs(all_sessions) do
    -- Count by status
    session_by_status[session.status] = (session_by_status[session.status] or 0) + 1
    
    -- Count by tags
    if session.tags then
        for _, tag in ipairs(session.tags) do
            session_by_tag[tag] = (session_by_tag[tag] or 0) + 1
        end
    end
end

print("📊 Session distribution by status:")
for status, count in pairs(session_by_status) do
    print(string.format("  %s: %d sessions", status, count))
end

print("\n🏷️  Top tags:")
local tag_list = {}
for tag, count in pairs(session_by_tag) do
    table.insert(tag_list, {tag = tag, count = count})
end
table.sort(tag_list, function(a, b) return a.count > b.count end)

for i = 1, math.min(5, #tag_list) do
    print(string.format("  %s: %d sessions", tag_list[i].tag, tag_list[i].count))
end

-- Step 6: Advanced query patterns
print("\n6. Advanced Query Patterns")
print(string.rep("-", 40))

-- Query 1: Find all analysis sessions from Q4
print("🔍 Query 1: Q4 2024 analysis sessions")
local q4_sessions = Session.list({
    tags = {"q4-2024", "analysis"}
})
print("  Found:", #q4_sessions, "sessions")

-- Query 2: Find high-priority sessions
print("\n🔍 Query 2: High-priority active sessions")
local active_sessions = Session.list({status = "active"})
local high_priority = 0
for _, session in ipairs(active_sessions) do
    if session.custom_metadata and session.custom_metadata.priority == "high" then
        high_priority = high_priority + 1
    end
end
print("  Found:", high_priority, "high-priority sessions")

-- Query 3: Find sessions with specific metadata
print("\n🔍 Query 3: Sessions by department")
local dept_count = {}
for _, session in ipairs(all_sessions) do
    if session.custom_metadata and session.custom_metadata.department then
        local dept = session.custom_metadata.department
        dept_count[dept] = (dept_count[dept] or 0) + 1
    end
end
for dept, count in pairs(dept_count) do
    print("  " .. dept .. ":", count)
end

-- Cleanup bulk sessions
print("\n7. Cleanup")
print(string.rep("-", 40))
print("🧹 Cleaning up bulk sessions...")
for _, session in ipairs(bulk_sessions) do
    Session.complete(session)
end
print("✅ Bulk sessions completed")

-- Complete department sessions
for dept, session_id in pairs(dept_sessions) do
    Session.complete(session_id)
end
Session.complete(root_session)
print("✅ Hierarchy sessions completed")

-- Summary
print("\n\n🎉 Advanced Patterns Completed!")
print("===============================")
print("\nDemonstrated capabilities:")
print("  ✓ Complex session hierarchies (3 levels deep)")
print("  ✓ Dynamic metadata management")
print("  ✓ Session templates for consistency")
print("  ✓ Bulk operations with performance metrics")
print("  ✓ Session analytics and distribution")
print("  ✓ Advanced query patterns")
print("\nPerformance insights:")
print(string.format("  • Session creation: %.1f ms average", (create_time * 1000) / 20))
print(string.format("  • Bulk storage: %.1f artifacts/second", 50 / store_time))
print("\nBest practices:")
print("  • Use hierarchies for complex workflows")
print("  • Template common session types")
print("  • Track metrics for monitoring")
print("  • Use tags and metadata for queries")
print("  • Batch operations for performance")