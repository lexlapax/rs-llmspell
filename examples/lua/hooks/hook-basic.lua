-- ABOUTME: Basic hook registration and unregistration example
-- ABOUTME: Demonstrates fundamental hook system usage with simple registration patterns

print("=== Basic Hook Registration Example ===")
print("Demonstrates: Hook.register(), handle:unregister(), Hook.unregister()")
print()

-- Example 1: Basic hook registration with default priority
print("1. Basic hook registration:")
local handle1 = Hook.register("BeforeAgentInit", function(context)
    print("   🪝 Hook fired: Agent initializing -", context.component_id.name)
    return "continue"
end)

print("   ✅ Hook registered with ID:", handle1:id())
print("   📍 Hook point:", handle1:hook_point())
print()

-- Example 2: Hook with explicit normal priority
print("2. Hook with explicit priority:")
local handle2 = Hook.register("AfterAgentInit", function(context)
    print("   🪝 Hook fired: Agent initialized -", context.component_id.name)
    return "continue"
end, "normal")

print("   ✅ Hook registered with ID:", handle2:id())
print()

-- Example 3: Hook with different hook point
print("3. Tool execution hook:")
local handle3 = Hook.register("BeforeToolExecution", function(context)
    print("   🪝 Hook fired: About to execute tool -", context.component_id.name)
    return "continue"
end, "normal")

print("   ✅ Hook registered with ID:", handle3:id())
print()

-- List all registered hooks
print("4. Listing all registered hooks:")
local all_hooks = Hook.list()
print("   📋 Total hooks registered:", #all_hooks)
for i, hook in ipairs(all_hooks) do
    print(string.format("   %d. %s (%s priority, %s)", 
          i, hook.name, hook.priority, hook.language))
end
print()

-- Example 4: Unregistration using handle method
print("5. Unregistering hooks:")
print("   Unregistering handle1 using handle:unregister()...")
local unregistered1 = handle1:unregister()
print("   ✅ Unregistered successfully:", unregistered1)

-- Example 5: Unregistration using standalone function
print("   Unregistering handle2 using Hook.unregister()...")
local unregistered2 = Hook.unregister(handle2)
print("   ✅ Unregistered successfully:", unregistered2)

-- Try to unregister the same hook again (should return false)
print("   Trying to unregister handle1 again...")
local unregistered_again = handle1:unregister()
print("   ❌ Already unregistered:", not unregistered_again)
print()

-- Check remaining hooks
print("6. Remaining hooks after cleanup:")
local remaining_hooks = Hook.list()
print("   📋 Remaining hooks:", #remaining_hooks)
for i, hook in ipairs(remaining_hooks) do
    print(string.format("   %d. %s", i, hook.name))
end
print()

-- Clean up the last hook
print("7. Final cleanup:")
Hook.unregister(handle3)
local final_hooks = Hook.list()
print("   🧹 Final hook count:", #final_hooks)

print()
print("✨ Basic hook example complete!")
print("   Key concepts demonstrated:")
print("   • Hook registration with Hook.register()")
print("   • Handle methods: id(), hook_point(), unregister()")
print("   • Standalone Hook.unregister() function")
print("   • Hook listing with Hook.list()")
print("   • Proper cleanup patterns")