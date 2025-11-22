-- Recommended profile: rag-dev
-- Run with: llmspell -p rag-dev run rag-multi-tenant.lua
-- RAG features with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: RAG-01 - Multi-Tenant RAG System v0.8.0
-- Complexity Level: PRODUCTION
-- Real-World Use Case: Enterprise SaaS with isolated customer knowledge bases
-- Pattern Category: RAG & Vector Storage
--
-- Purpose: Production-ready multi-tenant RAG pattern demonstrating tenant isolation,
--          separate collections, quota management, and cross-tenant operations.
--          Essential for SaaS applications where each customer needs their own
--          isolated knowledge base with security guarantees.
-- Architecture: Tenant-scoped vector collections with namespace isolation
-- Crates Showcased: llmspell-rag, llmspell-tenancy, llmspell-security, llmspell-storage
-- Key Features:
--   • Tenant-specific collections with complete isolation
--   • Per-tenant quota management and usage tracking
--   • Cross-tenant search with permissions (admin only)
--   • Tenant migration and backup strategies
--   • Security policies for data isolation
--   • Performance optimization per tenant tier
--   • Audit logging for compliance
--
-- Prerequisites:
--   • LLMSpell installed and built with Phase 8 features
--   • Multi-tenant RAG configuration (see configs/rag-multi-tenant.toml)
--   • Embedding provider configured
--   • Sufficient memory for multiple vector indices
--
-- HOW TO RUN:
-- ./target/debug/llmspell -c examples/script-users/configs/rag-multi-tenant.toml \
--   run examples/script-users/cookbook/rag-multi-tenant.lua
--
-- EXPECTED OUTPUT:
-- 3 tenants created with isolated RAG collections
-- Documents ingested per tenant with no cross-contamination
-- Tenant-specific searches returning only own data
-- Admin cross-tenant search demonstration
-- Usage metrics and quota tracking
--
-- Time to Complete: <20 seconds
-- Production Notes: Use persistent vector storage, implement regular backups,
--                   monitor per-tenant resource usage, enforce quotas strictly,
--                   audit all cross-tenant operations, use encryption at rest.
-- ============================================================

print("=== Multi-Tenant RAG System ===")
print("Pattern RAG-01: PRODUCTION - Enterprise tenant isolation\n")

-- ============================================================
-- Pattern 1: Tenant Management System
-- ============================================================

print("1. Setting Up Tenant Management")
print("-" .. string.rep("-", 40))

-- Tenant registry and management
local TenantManager = {}
TenantManager.__index = TenantManager

function TenantManager:new()
    return setmetatable({
        tenants = {},
        current_tenant = nil,
        admin_mode = false,
        usage_tracking = {},
        quotas = {
            starter = {
                max_documents = 100,
                max_searches_per_day = 1000,
                max_storage_mb = 50,
                vector_dimensions = 384
            },
            professional = {
                max_documents = 1000,
                max_searches_per_day = 10000,
                max_storage_mb = 500,
                vector_dimensions = 768
            },
            enterprise = {
                max_documents = 100000,
                max_searches_per_day = 100000,
                max_storage_mb = 10000,
                vector_dimensions = 1536
            }
        }
    }, self)
end

function TenantManager:create_tenant(id, name, tier)
    tier = tier or "starter"
    
    local tenant = {
        id = id,
        name = name,
        tier = tier,
        collection = "tenant_" .. id,
        created_at = os.time(),
        quotas = self.quotas[tier],
        usage = {
            documents = 0,
            searches_today = 0,
            storage_mb = 0,
            last_reset = os.date("%Y-%m-%d")
        },
        metadata = {}
    }
    
    self.tenants[id] = tenant
    self.usage_tracking[id] = {
        operations = {},
        alerts = {}
    }
    
    print(string.format("   Created tenant: %s (%s tier)", name, tier))
    return tenant
end

function TenantManager:set_current_tenant(tenant_id)
    if not self.tenants[tenant_id] then
        return false, "Tenant not found"
    end
    
    self.current_tenant = tenant_id
    
    -- Configure RAG for this tenant's collection
    RAG.configure({
        collection = self.tenants[tenant_id].collection,
        dimensions = self.tenants[tenant_id].quotas.vector_dimensions
    })
    
    print(string.format("   Switched to tenant: %s", self.tenants[tenant_id].name))
    return true
end

function TenantManager:check_quota(tenant_id, operation, amount)
    local tenant = self.tenants[tenant_id]
    if not tenant then
        return false, "Tenant not found"
    end
    
    -- Reset daily counters if needed
    local today = os.date("%Y-%m-%d")
    if tenant.usage.last_reset ~= today then
        tenant.usage.searches_today = 0
        tenant.usage.last_reset = today
    end
    
    -- Check specific quota
    if operation == "ingest" then
        if tenant.usage.documents + amount > tenant.quotas.max_documents then
            return false, "Document quota exceeded"
        end
    elseif operation == "search" then
        if tenant.usage.searches_today + amount > tenant.quotas.max_searches_per_day then
            return false, "Daily search quota exceeded"
        end
    elseif operation == "storage" then
        if tenant.usage.storage_mb + amount > tenant.quotas.max_storage_mb then
            return false, "Storage quota exceeded"
        end
    end
    
    return true
end

function TenantManager:update_usage(tenant_id, operation, amount)
    local tenant = self.tenants[tenant_id]
    if not tenant then
        return
    end
    
    if operation == "ingest" then
        tenant.usage.documents = tenant.usage.documents + amount
    elseif operation == "search" then
        tenant.usage.searches_today = tenant.usage.searches_today + amount
    elseif operation == "storage" then
        tenant.usage.storage_mb = tenant.usage.storage_mb + amount
    end
    
    -- Track operation
    table.insert(self.usage_tracking[tenant_id].operations, {
        operation = operation,
        amount = amount,
        timestamp = os.time()
    })
    
    -- Check for alerts
    local usage_percent = 0
    if operation == "ingest" then
        usage_percent = (tenant.usage.documents / tenant.quotas.max_documents) * 100
    elseif operation == "search" then
        usage_percent = (tenant.usage.searches_today / tenant.quotas.max_searches_per_day) * 100
    end
    
    if usage_percent > 90 then
        table.insert(self.usage_tracking[tenant_id].alerts, {
            type = "quota_warning",
            operation = operation,
            usage_percent = usage_percent,
            timestamp = os.time()
        })
        print(string.format("   ⚠️ Tenant %s: %s usage at %.1f%%", 
            tenant.name, operation, usage_percent))
    end
end

-- Initialize tenant manager
local tenant_manager = TenantManager:new()

-- Create sample tenants
print("   Creating sample tenants:")
tenant_manager:create_tenant("acme_corp", "Acme Corporation", "enterprise")
tenant_manager:create_tenant("startup_inc", "Startup Inc", "starter")
tenant_manager:create_tenant("tech_co", "Tech Company", "professional")

print()

-- ============================================================
-- Pattern 2: Tenant-Isolated Document Ingestion
-- ============================================================

print("2. Tenant-Isolated Document Ingestion")
print("-" .. string.rep("-", 40))

function ingest_for_tenant(tenant_manager, tenant_id, documents)
    -- Switch to tenant context
    local success, err = tenant_manager:set_current_tenant(tenant_id)
    if not success then
        print("   ✗ " .. err)
        return 0
    end
    
    local tenant = tenant_manager.tenants[tenant_id]
    local ingested = 0
    
    print(string.format("\n   Ingesting for %s:", tenant.name))
    
    for i, doc in ipairs(documents) do
        -- Check quota before ingesting
        local can_ingest, quota_err = tenant_manager:check_quota(tenant_id, "ingest", 1)
        if not can_ingest then
            print(string.format("   ✗ Document %d: %s", i, quota_err))
            break
        end
        
        -- Add tenant metadata to document
        doc.metadata = doc.metadata or {}
        doc.metadata.tenant_id = tenant_id
        doc.metadata.tenant_name = tenant.name
        doc.metadata.ingested_at = os.time()
        
        -- Ingest document into tenant-specific scope
        local result = RAG.ingest(doc, {
            scope = "tenant",
            scope_id = tenant_id
        })
        
        if result and result.success then
            ingested = ingested + 1
            tenant_manager:update_usage(tenant_id, "ingest", 1)
            
            -- Estimate storage (simplified)
            local storage_mb = string.len(doc.content) / (1024 * 1024)
            tenant_manager:update_usage(tenant_id, "storage", storage_mb)
            
            print(string.format("   ✓ Document %d: %s", 
                i, doc.metadata.title or "Untitled"))
        else
            print(string.format("   ✗ Document %d failed: %s", 
                i, result and result.error or "Unknown"))
        end
    end
    
    print(string.format("   Total ingested: %d/%d", ingested, #documents))
    return ingested
end

-- Tenant-specific documents
local acme_documents = {
    {
        content = "Acme Corporation Internal Policy: All employees must complete security training quarterly. Data classification levels include Public, Internal, Confidential, and Restricted. Restricted data requires encryption at rest and in transit.",
        metadata = {title = "Security Policy", category = "policy", classification = "internal"}
    },
    {
        content = "Acme Product Roadmap 2024: Q1 - Launch new API gateway. Q2 - Implement multi-region support. Q3 - Release mobile SDK. Q4 - Enterprise SSO integration.",
        metadata = {title = "Product Roadmap", category = "planning", classification = "confidential"}
    },
    {
        content = "Acme Customer Success Guide: Key metrics include NPS score, churn rate, and expansion revenue. Monthly business reviews should focus on value realization and growth opportunities.",
        metadata = {title = "Customer Success", category = "operations", classification = "internal"}
    }
}

local startup_documents = {
    {
        content = "Startup Inc MVP Features: User authentication, basic CRUD operations, payment integration with Stripe, and email notifications. Focus on core functionality over advanced features.",
        metadata = {title = "MVP Specification", category = "development"}
    },
    {
        content = "Startup Inc Go-to-Market Strategy: Target early adopters through Product Hunt launch, content marketing on dev.to, and partnership with accelerators. Initial pricing at $29/month.",
        metadata = {title = "GTM Strategy", category = "marketing"}
    }
}

local tech_documents = {
    {
        content = "Tech Company Architecture: Microservices deployed on Kubernetes, PostgreSQL for transactional data, Redis for caching, and Kafka for event streaming. All services must implement circuit breakers.",
        metadata = {title = "System Architecture", category = "technical"}
    },
    {
        content = "Tech Company SLA: 99.95% uptime guarantee, <100ms API response time for 95th percentile, 24/7 support for enterprise customers, and 4-hour response time for critical issues.",
        metadata = {title = "Service Level Agreement", category = "legal"}
    },
    {
        content = "Tech Company Data Pipeline: Ingestion via Kinesis, processing with Apache Spark, storage in S3 and Redshift, visualization through Tableau. Daily ETL jobs run at 2 AM UTC.",
        metadata = {title = "Data Infrastructure", category = "data"}
    }
}

-- Ingest documents for each tenant
ingest_for_tenant(tenant_manager, "acme_corp", acme_documents)
ingest_for_tenant(tenant_manager, "startup_inc", startup_documents)
ingest_for_tenant(tenant_manager, "tech_co", tech_documents)

print()

-- ============================================================
-- Pattern 3: Tenant-Isolated Search
-- ============================================================

print("3. Tenant-Isolated Search")
print("-" .. string.rep("-", 40))

function search_for_tenant(tenant_manager, tenant_id, query)
    -- Switch to tenant context
    tenant_manager:set_current_tenant(tenant_id)
    
    local tenant = tenant_manager.tenants[tenant_id]
    
    -- Check search quota
    local can_search, quota_err = tenant_manager:check_quota(tenant_id, "search", 1)
    if not can_search then
        print(string.format("   ✗ %s: %s", tenant.name, quota_err))
        return nil
    end
    
    print(string.format("\n   Searching for %s: '%s'", tenant.name, query))
    
    -- Perform search in tenant scope
    local result = RAG.search(query, {
        limit = 3,
        scope = "tenant",
        scope_id = tenant_id
    })
    
    tenant_manager:update_usage(tenant_id, "search", 1)
    
    if result and result.success and result.results then
        for i, doc in ipairs(result.results) do
            print(string.format("   %d. [%.3f] %s", 
                i, 
                doc.score or 0,
                doc.metadata and doc.metadata.title or "Untitled"
            ))
            
            -- Verify tenant isolation
            if doc.metadata and doc.metadata.tenant_id then
                if doc.metadata.tenant_id ~= tenant_id then
                    print("   ⚠️ SECURITY VIOLATION: Cross-tenant data leak detected!")
                end
            end
        end
        return result.results
    else
        print("   No results found")
        return nil
    end
end

-- Test tenant isolation with searches
print("\nTesting tenant isolation:")

-- Each tenant searches for "security" - should only see their own data
search_for_tenant(tenant_manager, "acme_corp", "security")
search_for_tenant(tenant_manager, "startup_inc", "security")
search_for_tenant(tenant_manager, "tech_co", "security")

-- Acme searches for "MVP" - should find nothing (Startup's data)
print("\n   Isolation test - Acme searching for 'MVP':")
search_for_tenant(tenant_manager, "acme_corp", "MVP")

print()

-- ============================================================
-- Pattern 4: Cross-Tenant Admin Operations
-- ============================================================

print("4. Cross-Tenant Admin Operations")
print("-" .. string.rep("-", 40))

function admin_cross_tenant_search(tenant_manager, query)
    print(string.format("\n   Admin global search: '%s'", query))
    
    if not tenant_manager.admin_mode then
        print("   ✗ Admin mode required for cross-tenant search")
        return
    end
    
    local all_results = {}
    
    -- Search across all tenants
    for tenant_id, tenant in pairs(tenant_manager.tenants) do
        tenant_manager:set_current_tenant(tenant_id)
        
        local result = RAG.search(query, {
            limit = 2,
            scope = "tenant",
            scope_id = tenant_id
        })
        
        if result and result.success and result.results then
            for _, doc in ipairs(result.results) do
                -- Add tenant info to results
                doc.search_tenant = tenant.name
                table.insert(all_results, doc)
            end
        end
    end
    
    -- Sort by score
    table.sort(all_results, function(a, b)
        return (a.score or 0) > (b.score or 0)
    end)
    
    print(string.format("   Found %d results across all tenants:", #all_results))
    for i, doc in ipairs(all_results) do
        if i > 5 then break end  -- Limit display
        print(string.format("   %d. [%.3f] %s (Tenant: %s)", 
            i,
            doc.score or 0,
            doc.metadata and doc.metadata.title or "Untitled",
            doc.search_tenant
        ))
    end
    
    -- Audit log
    print("\n   📝 Audit: Admin cross-tenant search performed")
    print(string.format("      Query: '%s'", query))
    print(string.format("      Timestamp: %s", os.date()))
    print(string.format("      Results: %d documents", #all_results))
    
    return all_results
end

-- Enable admin mode (would require authentication in production)
print("\n   Enabling admin mode...")
tenant_manager.admin_mode = true
print("   ✓ Admin mode enabled")

-- Admin searches across all tenants
admin_cross_tenant_search(tenant_manager, "architecture")
admin_cross_tenant_search(tenant_manager, "customer")

print()

-- ============================================================
-- Pattern 5: Tenant Migration
-- ============================================================

print("5. Tenant Migration and Backup")
print("-" .. string.rep("-", 40))

function migrate_tenant_data(tenant_manager, source_id, target_id)
    print(string.format("\n   Migrating from %s to %s", source_id, target_id))
    
    local source = tenant_manager.tenants[source_id]
    local target = tenant_manager.tenants[target_id]
    
    if not source or not target then
        print("   ✗ Invalid tenant IDs")
        return false
    end
    
    -- Simulate migration steps
    local steps = {
        {name = "Validate permissions", success = true},
        {name = "Create backup", success = true},
        {name = "Copy vector indices", success = true},
        {name = "Update metadata", success = true},
        {name = "Verify data integrity", success = true},
        {name = "Switch collections", success = true}
    }
    
    for _, step in ipairs(steps) do
        if step.success then
            print(string.format("   ✓ %s", step.name))
        else
            print(string.format("   ✗ %s - Migration failed", step.name))
            return false
        end
    end
    
    print("   ✅ Migration completed successfully")
    return true
end

function backup_tenant_data(tenant_manager, tenant_id)
    local tenant = tenant_manager.tenants[tenant_id]
    if not tenant then
        return nil
    end
    
    print(string.format("\n   Creating backup for %s", tenant.name))
    
    -- Get tenant statistics for this tenant
    local stats = RAG.get_stats("tenant", tenant_id)
    
    local backup = {
        tenant_id = tenant_id,
        tenant_name = tenant.name,
        timestamp = os.time(),
        collection = tenant.collection,
        usage = tenant.usage,
        checksum = string.format("%x", math.random(0, 0xFFFFFFFF))  -- Simulated
    }
    
    print(string.format("   ✓ Backup created: %s", backup.checksum))
    print(string.format("     Documents: %d", tenant.usage.documents))
    print(string.format("     Storage: %.2f MB", tenant.usage.storage_mb))
    
    return backup
end

-- Demonstrate migration and backup
backup_tenant_data(tenant_manager, "acme_corp")
-- migrate_tenant_data(tenant_manager, "startup_inc", "tech_co")  -- Commented to preserve data

print()

-- ============================================================
-- Pattern 6: Usage Analytics
-- ============================================================

print("6. Tenant Usage Analytics")
print("-" .. string.rep("-", 40))

function generate_usage_report(tenant_manager)
    print("\n   📊 Usage Report for All Tenants:")
    print("   " .. string.rep("-", 60))
    
    for tenant_id, tenant in pairs(tenant_manager.tenants) do
        print(string.format("\n   %s (%s tier):", tenant.name, tenant.tier))
        
        -- Document usage
        local doc_percent = (tenant.usage.documents / tenant.quotas.max_documents) * 100
        print(string.format("     Documents: %d/%d (%.1f%%)",
            tenant.usage.documents,
            tenant.quotas.max_documents,
            doc_percent
        ))
        
        -- Search usage
        local search_percent = (tenant.usage.searches_today / tenant.quotas.max_searches_per_day) * 100
        print(string.format("     Searches today: %d/%d (%.1f%%)",
            tenant.usage.searches_today,
            tenant.quotas.max_searches_per_day,
            search_percent
        ))
        
        -- Storage usage
        local storage_percent = (tenant.usage.storage_mb / tenant.quotas.max_storage_mb) * 100
        print(string.format("     Storage: %.2f/%.0f MB (%.1f%%)",
            tenant.usage.storage_mb,
            tenant.quotas.max_storage_mb,
            storage_percent
        ))
        
        -- Alerts
        local tracking = tenant_manager.usage_tracking[tenant_id]
        if tracking and #tracking.alerts > 0 then
            print("     ⚠️ Alerts:")
            for _, alert in ipairs(tracking.alerts) do
                print(string.format("        - %s: %.1f%% usage",
                    alert.operation,
                    alert.usage_percent
                ))
            end
        end
        
        -- Recommendations
        if doc_percent > 80 or search_percent > 80 or storage_percent > 80 then
            print("     💡 Recommendation: Consider upgrading to higher tier")
        end
    end
    
    print("\n   " .. string.rep("-", 60))
end

generate_usage_report(tenant_manager)

print()

-- ============================================================
-- Pattern 7: Cleanup Scope Data
-- ============================================================

print("7. Tenant Data Cleanup")
print("-" .. string.rep("-", 40))

function cleanup_tenant_data(tenant_id)
    print(string.format("\n   Cleaning up data for tenant: %s", tenant_id))
    
    -- Use RAG cleanup for tenant scope
    local result = RAG.cleanup_scope({
        scope = "tenant_" .. tenant_id,
        confirm = true  -- Safety check
    })
    
    if result and result.success then
        print("   ✓ Tenant data cleaned up successfully")
        if result.stats then
            print(string.format("     Documents removed: %d", result.stats.documents_removed or 0))
            print(string.format("     Storage freed: %.2f MB", result.stats.storage_freed_mb or 0))
        end
    else
        print("   ✗ Cleanup failed: " .. (result and result.error or "Unknown"))
    end
    
    return result
end

-- Example: Clean up a tenant (commented to preserve demo data)
-- cleanup_tenant_data("startup_inc")

print()
print("🎯 Key Takeaways:")
print("   • Complete tenant isolation with separate collections")
print("   • Per-tenant quota management and enforcement")
print("   • Usage tracking with alerts and recommendations")
print("   • Admin cross-tenant search with audit logging")
print("   • Migration and backup capabilities")
print("   • Cleanup operations for GDPR compliance")
print()
print("🔐 Security Best Practices:")
print("   • Always verify tenant context before operations")
print("   • Audit all cross-tenant operations")
print("   • Implement rate limiting per tenant")
print("   • Use encryption at rest for sensitive data")
print("   • Regular backups with integrity checks")
print("   • Monitor for quota violations and anomalies")

-- Return success with statistics
return {
    success = true,
    tenants_created = 3,
    total_documents = tenant_manager.tenants["acme_corp"].usage.documents +
                      tenant_manager.tenants["startup_inc"].usage.documents +
                      tenant_manager.tenants["tech_co"].usage.documents
}