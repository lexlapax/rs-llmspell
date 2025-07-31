#!/usr/bin/env llmspell

-- knowledge_base.lua - Building a knowledge base with artifacts
-- This example shows how to create a searchable knowledge base using sessions and artifacts

print("=== Knowledge Base Example ===\n")

-- Create a knowledge base session
local kb_session = Session.create({
    name = "Company Knowledge Base",
    description = "Centralized knowledge base for company documentation",
    tags = {"knowledge-base", "documentation", "searchable"},
    metadata = {
        version = "1.0",
        department = "Engineering",
        access_level = "internal"
    }
})

print("Created knowledge base session: " .. kb_session)
Session.setCurrent(kb_session)

-- Helper function to add document to knowledge base
local function addDocument(title, content, category, tags)
    local doc_id = Artifact.store(
        kb_session,
        "user_input",
        title .. ".md",
        content,
        {
            title = title,
            category = category,
            tags = tags,
            indexed = true,
            last_updated = os.time(),
            word_count = #content:gsub("%S+", ""),
            searchable = true
        }
    )
    return doc_id
end

-- Helper function to add FAQ entry
local function addFAQ(question, answer, category)
    local faq_content = string.format("## Q: %s\n\n**A:** %s", question, answer)
    return addDocument(
        "FAQ: " .. question:sub(1, 50),
        faq_content,
        "faq",
        {category, "faq", "q&a"}
    )
end

-- 1. Add technical documentation
print("1. Adding technical documentation:")

local api_docs = addDocument(
    "API Reference Guide",
    [[# API Reference Guide

## Authentication
All API requests require authentication using API keys or OAuth tokens.

### API Key Authentication
Include your API key in the `X-API-Key` header:
```
X-API-Key: your-api-key-here
```

### OAuth 2.0
For OAuth authentication, obtain an access token through the standard OAuth flow.

## Endpoints

### GET /api/v1/users
Retrieve user information.

**Parameters:**
- `limit` (optional): Number of results to return
- `offset` (optional): Pagination offset

**Response:**
```json
{
  "users": [...],
  "total": 100,
  "limit": 20,
  "offset": 0
}
```

### POST /api/v1/sessions
Create a new session.

**Request Body:**
```json
{
  "name": "Session Name",
  "metadata": {}
}
```

## Rate Limiting
API requests are limited to 1000 requests per hour per API key.
]],
    "technical",
    {"api", "reference", "authentication", "endpoints"}
)

print("  Added: API Reference Guide")

local arch_docs = addDocument(
    "System Architecture Overview",
    [[# System Architecture Overview

## Core Components

### 1. Session Manager
The SessionManager is the central component that orchestrates all session operations.
It manages session lifecycle, persistence, and state management.

### 2. Artifact Storage
Content-addressed storage system using BLAKE3 hashing for deduplication.
Automatic compression for artifacts larger than 10KB.

### 3. Event System
Pub/sub event system for component communication and workflow coordination.

### 4. Hook System
Extensible hook system for lifecycle events and custom processing.

## Data Flow
1. User requests → API Gateway
2. API Gateway → Session Manager
3. Session Manager → Storage Backend
4. Storage Backend → Persistence Layer

## Scalability Considerations
- Horizontal scaling through session sharding
- Read replicas for high-traffic scenarios
- Caching layer for frequently accessed artifacts
]],
    "technical",
    {"architecture", "system-design", "components", "scalability"}
)

print("  Added: System Architecture Overview")

-- 2. Add process documentation
print("\n2. Adding process documentation:")

local onboarding = addDocument(
    "Employee Onboarding Guide",
    [[# Employee Onboarding Guide

## Week 1: Orientation
- [ ] Complete HR paperwork
- [ ] Set up workstation
- [ ] Obtain access badges
- [ ] Meet team members
- [ ] Review company policies

## Week 2: Training
- [ ] Complete security training
- [ ] Attend product overview sessions
- [ ] Set up development environment
- [ ] Review coding standards

## Week 3-4: Ramp Up
- [ ] Work on starter project
- [ ] Pair programming sessions
- [ ] Code review participation
- [ ] Documentation review

## Resources
- Internal Wiki: https://wiki.company.com
- Training Portal: https://training.company.com
- IT Support: helpdesk@company.com
]],
    "process",
    {"onboarding", "hr", "training", "new-employee"}
)

print("  Added: Employee Onboarding Guide")

-- 3. Add FAQ entries
print("\n3. Adding FAQ entries:")

addFAQ(
    "How do I reset my password?",
    "You can reset your password by clicking 'Forgot Password' on the login page, or contact IT support at helpdesk@company.com.",
    "account"
)

addFAQ(
    "What are the working hours?",
    "Core hours are 10 AM - 4 PM in your local timezone. Outside of core hours, you have flexibility to manage your schedule.",
    "hr"
)

addFAQ(
    "How do I request time off?",
    "Submit time off requests through the HR portal at least 2 weeks in advance. Your manager will receive a notification for approval.",
    "hr"
)

print("  Added: 3 FAQ entries")

-- 4. Add troubleshooting guides
print("\n4. Adding troubleshooting guides:")

local troubleshoot = addDocument(
    "Common Issues and Solutions",
    [[# Troubleshooting Guide

## Build Failures

### Issue: "Module not found" error
**Solution:**
1. Run `npm install` or `cargo build` to install dependencies
2. Check that all required environment variables are set
3. Verify you're on the correct branch

### Issue: Test failures in CI
**Solution:**
1. Ensure tests pass locally first
2. Check for timing-dependent tests
3. Verify test data is properly initialized

## Runtime Errors

### Issue: "Connection refused" error
**Solution:**
1. Check if required services are running
2. Verify network connectivity
3. Check firewall settings

### Issue: "Out of memory" error
**Solution:**
1. Increase heap size in configuration
2. Check for memory leaks
3. Review recent code changes for inefficiencies
]],
    "troubleshooting",
    {"debug", "errors", "solutions", "technical"}
)

print("  Added: Troubleshooting Guide")

-- 5. Create a searchable index
print("\n5. Building searchable index:")

-- Simple search function
local function searchKnowledgeBase(query)
    local results = {}
    local query_lower = query:lower()
    
    -- Get all artifacts
    local all_docs = Artifact.list(kb_session)
    
    for _, doc in ipairs(all_docs) do
        local score = 0
        
        -- Search in title
        if doc.metadata.title and doc.metadata.title:lower():find(query_lower) then
            score = score + 10
        end
        
        -- Search in tags
        if doc.metadata.tags then
            for _, tag in ipairs(doc.metadata.tags) do
                if tag:lower():find(query_lower) then
                    score = score + 5
                end
            end
        end
        
        -- Search in category
        if doc.metadata.category and doc.metadata.category:lower():find(query_lower) then
            score = score + 3
        end
        
        -- If we need to search content, retrieve it
        if score > 0 then
            local full_doc = Artifact.get(kb_session, doc.id)
            if full_doc and full_doc.content:lower():find(query_lower) then
                score = score + 1
            end
            
            table.insert(results, {
                doc = doc,
                score = score
            })
        end
    end
    
    -- Sort by score
    table.sort(results, function(a, b) return a.score > b.score end)
    
    return results
end

-- Test search functionality
local search_queries = {"api", "onboarding", "password", "error"}

for _, query in ipairs(search_queries) do
    local results = searchKnowledgeBase(query)
    print(string.format("  Search '%s': found %d results", query, #results))
    if #results > 0 then
        print(string.format("    Top result: %s (score: %d)", 
            results[1].doc.metadata.title or results[1].doc.name, 
            results[1].score))
    end
end

-- 6. Generate knowledge base statistics
print("\n6. Knowledge base statistics:")

local stats = {
    total_documents = 0,
    by_category = {},
    by_type = {},
    total_size = 0,
    tags = {}
}

local all_artifacts = Artifact.list(kb_session)

for _, artifact in ipairs(all_artifacts) do
    stats.total_documents = stats.total_documents + 1
    stats.total_size = stats.total_size + artifact.size
    
    -- Count by category
    local category = artifact.metadata.category or "uncategorized"
    stats.by_category[category] = (stats.by_category[category] or 0) + 1
    
    -- Count by type
    stats.by_type[artifact.artifact_type] = (stats.by_type[artifact.artifact_type] or 0) + 1
    
    -- Collect tags
    if artifact.metadata.tags then
        for _, tag in ipairs(artifact.metadata.tags) do
            stats.tags[tag] = (stats.tags[tag] or 0) + 1
        end
    end
end

print("  Total documents: " .. stats.total_documents)
print("  Total size: " .. string.format("%.2f KB", stats.total_size / 1024))
print("\n  By category:")
for cat, count in pairs(stats.by_category) do
    print("    " .. cat .. ": " .. count)
end
print("\n  Popular tags:")
local tag_list = {}
for tag, count in pairs(stats.tags) do
    table.insert(tag_list, {tag = tag, count = count})
end
table.sort(tag_list, function(a, b) return a.count > b.count end)
for i = 1, math.min(5, #tag_list) do
    print("    " .. tag_list[i].tag .. ": " .. tag_list[i].count)
end

-- 7. Export knowledge base summary
print("\n7. Exporting knowledge base summary:")

local summary = {
    session_id = kb_session,
    created_at = os.time(),
    statistics = stats,
    categories = {},
    recent_updates = {}
}

-- Group documents by category
for _, artifact in ipairs(all_artifacts) do
    local category = artifact.metadata.category or "uncategorized"
    if not summary.categories[category] then
        summary.categories[category] = {}
    end
    table.insert(summary.categories[category], {
        id = artifact.id,
        name = artifact.name,
        title = artifact.metadata.title,
        tags = artifact.metadata.tags,
        updated = artifact.metadata.last_updated
    })
end

-- Save summary as artifact
local summary_json = Tool.execute("json-processor", {
    operation = "stringify",
    input = summary,
    pretty = true
})

Artifact.store(
    kb_session,
    "system_generated",
    "kb_summary.json",
    summary_json.result,
    {
        description = "Knowledge base summary and index",
        auto_generated = true,
        timestamp = os.time()
    }
)

print("  Exported knowledge base summary")

-- Complete the session
Session.save(kb_session)
print("\n✓ Knowledge base example completed!")
print("  Session " .. kb_session .. " saved with " .. stats.total_documents .. " documents")

-- Summary
print("\n=== Summary ===")
print("This example demonstrated building a knowledge base with:")
print("  • Structured document storage with rich metadata")
print("  • Multiple content categories (technical, process, FAQ)")
print("  • Simple search functionality")
print("  • Tag-based organization")
print("  • Statistical analysis")
print("  • Export capabilities")
print("\nExtend this pattern to build:")
print("  • Documentation portals")
print("  • Help centers")
print("  • Internal wikis")
print("  • Searchable archives")