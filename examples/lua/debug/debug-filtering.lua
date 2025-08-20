-- Module Filtering Example
-- Demonstrates advanced filtering capabilities for targeted debugging

print("🎯 LLMSpell Debug - Module Filtering")
print("=" .. string.rep("=", 45))

-- Helper function to demonstrate logging from different modules
local function log_from_modules()
    Debug.info("Core system initialized", "system.core")
    Debug.info("Database connection established", "database.connection")
    Debug.warn("High memory usage detected", "system.memory")
    Debug.info("User authentication successful", "auth.login")
    Debug.error("Failed to process payment", "payment.processor")
    Debug.debug("Cache hit for user data", "cache.user")
    Debug.info("Workflow step 1 completed", "workflow.step1")
    Debug.info("Workflow step 2 started", "workflow.step2")
    Debug.info("Agent task assigned", "agent.coordinator")
    Debug.debug("Internal agent state updated", "agent.internal.state")
    Debug.info("Tool execution finished", "tool.calculator")
    Debug.warn("Test assertion failed", "unit.test")
    Debug.info("Integration test passed", "integration.test")
end

-- 1. Default Behavior (All Modules)
print("\n📋 1. Default Behavior - All Modules Logged")
print("-" .. string.rep("-", 50))

-- Clear any existing filters
Debug.clearModuleFilters()

print("Current filter state:")
local summary = Debug.getFilterSummary()
print("  Default enabled: " .. tostring(summary.default_enabled))
print("  Total rules: " .. summary.total_rules)

print("\nLogging from various modules:")
log_from_modules()

-- 2. Simple Pattern Filtering
print("\n🔍 2. Simple Pattern Filtering")
print("-" .. string.rep("-", 35))

-- Enable only workflow modules
Debug.clearModuleFilters()
Debug.addModuleFilter("workflow.*", true)

print("Filter: Only 'workflow.*' modules enabled")
print("Expected: Only workflow.step1 and workflow.step2 should appear")
log_from_modules()

-- 3. Wildcard Patterns
print("\n🌟 3. Wildcard Patterns")
print("-" .. string.rep("-", 25))

Debug.clearModuleFilters()
Debug.addModuleFilter("*.test", false)  -- Disable all test modules
Debug.addModuleFilter("agent.*", true)  -- Enable all agent modules

print("Filters:")
print("  *.test = false (disable all test modules)")
print("  agent.* = true (enable all agent modules)")
print("Expected: No test modules, but agent modules should appear")
log_from_modules()

-- 4. Hierarchical Patterns
print("\n🏗️  4. Hierarchical Patterns")
print("-" .. string.rep("-", 30))

Debug.clearModuleFilters()
Debug.addModuleFilter("system.*", true)     -- Enable all system modules
Debug.addModuleFilter("agent.*", true)      -- Enable all agent modules
Debug.addModuleFilter("agent.internal.*", false)  -- But disable internal agent modules

print("Filters:")
print("  system.* = true (enable all system modules)")
print("  agent.* = true (enable all agent modules)")
print("  agent.internal.* = false (disable internal agent modules)")
print("Expected: system.* and agent.* except agent.internal.*")
log_from_modules()

-- 5. Advanced Filter Rules
print("\n🚀 5. Advanced Filter Rules")
print("-" .. string.rep("-", 30))

Debug.clearModuleFilters()

-- Use advanced filter with regex
Debug.addAdvancedFilter("^(database|auth)\\..*", "regex", true)  -- Enable database and auth
Debug.addAdvancedFilter(".*\\.test$", "regex", false)           -- Disable anything ending in .test
Debug.addAdvancedFilter("payment", "exact", true)               -- Enable exact match for payment

print("Advanced filters:")
print("  Regex: '^(database|auth)\\..*' = true")
print("  Regex: '.*\\.test$' = false")
print("  Exact: 'payment' = true")
print("Expected: database.*, auth.*, no *.test, but 'payment' modules")
log_from_modules()

-- 6. Filter Priority and Specificity
print("\n⚡ 6. Filter Priority and Specificity")
print("-" .. string.rep("-", 40))

Debug.clearModuleFilters()

-- Demonstrate that more specific rules take precedence
Debug.addModuleFilter("workflow.*", false)      -- Disable all workflow
Debug.addModuleFilter("workflow.step1", true)   -- But enable step1 specifically

print("Filters (testing priority):")
print("  workflow.* = false (disable all workflow)")
print("  workflow.step1 = true (enable step1 specifically)")
print("Expected: Only workflow.step1 should appear (specific overrides general)")
log_from_modules()

-- 7. Filter Summary and Management
print("\n📊 7. Filter Summary and Management")
print("-" .. string.rep("-", 40))

-- Add multiple filters to demonstrate management
Debug.clearModuleFilters()
Debug.addModuleFilter("system.*", true)
Debug.addModuleFilter("database.*", true)
Debug.addModuleFilter("*.test", false)
Debug.addAdvancedFilter("auth\\..*", "regex", true)

print("Current filter configuration:")
local current_summary = Debug.getFilterSummary()
print("  Default enabled: " .. tostring(current_summary.default_enabled))
print("  Total rules: " .. current_summary.total_rules)

print("\nDetailed filter rules:")
for i = 1, #current_summary.rules do
    local rule = current_summary.rules[i]
    print("  " .. rule.pattern_type .. ": '" .. rule.pattern .. "' = " .. tostring(rule.enabled))
end

-- 8. Dynamic Filter Adjustment
print("\n🔄 8. Dynamic Filter Adjustment")
print("-" .. string.rep("-", 35))

print("Testing before filter change:")
log_from_modules()

-- Remove a specific filter
print("\nRemoving 'system.*' filter...")
local removed = Debug.removeModuleFilter("system")
print("Filter removed: " .. tostring(removed))

print("Testing after removing system.* filter:")
log_from_modules()

-- 9. Performance Impact Demo
print("\n⚡ 9. Performance Impact Demonstration")
print("-" .. string.rep("-", 45))

-- Measure performance with and without filtering
local function performance_test(description, setup_func)
    setup_func()
    
    local timer = Debug.timer("filter_performance_" .. description)
    
    -- Run many log operations
    for i = 1, 1000 do
        Debug.info("Test message " .. i, "performance.test.module" .. (i % 10))
    end
    
    local duration = timer:stop()
    print(description .. ": " .. string.format("%.2f", duration) .. "ms for 1000 log calls")
    
    return duration
end

-- Test with all modules enabled
local all_enabled_time = performance_test("All modules enabled", function()
    Debug.clearModuleFilters()
    Debug.setDefaultFilterEnabled(true)
end)

-- Test with restrictive filtering
local filtered_time = performance_test("Restrictive filtering", function()
    Debug.clearModuleFilters()
    Debug.setDefaultFilterEnabled(false)
    Debug.addModuleFilter("performance.test.module1", true)  -- Only allow one module
end)

print("\nPerformance impact:")
print("  Filtering overhead: " .. string.format("%.2f", filtered_time - all_enabled_time) .. "ms")
print("  Overhead per call: " .. string.format("%.4f", (filtered_time - all_enabled_time) / 1000) .. "ms")

-- 10. Preset Filter Configurations
print("\n🎨 10. Preset Filter Configurations")
print("-" .. string.rep("-", 40))

-- Note: These would be implemented in Rust, showing what they would do
print("Available preset configurations:")
print("  errors_only: Only show error and warning messages")
print("  development: Focus on workflow, agent, and tool modules")
print("  production: Focus on security, performance, error, and audit")
print("  component('agent'): Focus on specific component debugging")

print("\nExample of what each preset would filter:")

-- Simulate errors_only preset
print("\nSimulating 'errors_only' preset:")
Debug.clearModuleFilters()
Debug.setDefaultFilterEnabled(false)
Debug.addModuleFilter("error", true)
Debug.addModuleFilter("warn", true)
-- In real implementation, this would be based on log level, not module name

-- 11. Best Practices Demo
print("\n💡 11. Best Practices")
print("-" .. string.rep("-", 25))

print("Best practices for module filtering:")
print("1. Use hierarchical naming: 'component.subcomponent.function'")
print("2. Start broad, then narrow down: 'workflow.*' -> 'workflow.step1'")
print("3. Use exact matches for critical debugging")
print("4. Combine with log levels for fine control")
print("5. Document your filter strategies")

-- Final cleanup
Debug.clearModuleFilters()
print("\n✅ Module filtering example complete!")
print("🔧 Tip: Combine module filtering with log levels for powerful debugging control")