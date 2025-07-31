-- ABOUTME: Example demonstrating basic session management operations
-- ABOUTME: Shows session creation, lifecycle management, persistence, and querying

-- CONFIG: Requires runtime integration (see README.md for current status)
-- WHY: Sessions provide conversation context, artifact storage, and execution tracking
-- STATUS: Session/Artifact globals implemented but not yet integrated into CLI runtime
-- TODO: Runtime needs to initialize SessionManager - see llmspell-bridge/src/runtime.rs

print("📝 Basic Session Operations Example")
print("===================================")

-- This example demonstrates:
-- 1. Creating sessions with metadata
-- 2. Managing session lifecycle (suspend, resume, complete)
-- 3. Session persistence (save/load)
-- 4. Thread-local session context
-- 5. Querying and filtering sessions
-- 6. Error handling

-- Helper function to print session info
local function print_session(session)
    print(string.format("  📋 ID: %s", session.id))
    print(string.format("     Name: %s", session.name))
    print(string.format("     Status: %s", session.status))
    print(string.format("     Created: %s", session.created_at))
    if session.description then
        print(string.format("     Description: %s", session.description))
    end
    if session.tags and #session.tags > 0 then
        print(string.format("     Tags: %s", table.concat(session.tags, ", ")))
    end
end

-- Step 1: Create a new session with metadata
print("\n1. Creating Sessions with Metadata")
print(string.rep("-", 40))
local session_id = Session.create({
    name = "Data Analysis Session",
    description = "Analyzing Q4 2024 sales data",
    tags = {"analysis", "sales", "q4-2024"},
    metadata = {
        department = "sales",
        analyst = "john.doe",
        priority = "high"
    }
})
print("✅ Created session:", session_id)

-- Step 2: Get session metadata
print("\n2. Retrieving Session Metadata")
print(string.rep("-", 40))
local session = Session.get(session_id)
print_session(session)

-- Step 3: Create a child session
print("\n3. Parent-Child Session Relationships")
print(string.rep("-", 40))
local child_id = Session.create({
    name = "Regional Analysis",
    description = "Deep dive into EMEA region",
    parent_session_id = session_id,
    tags = {"analysis", "emea"}
})
print("✅ Created child session:", child_id)

-- Step 4: List all active sessions
print("\n4. Listing and Querying Sessions")
print(string.rep("-", 40))
local sessions = Session.list()
print(string.format("Found %d active sessions:", #sessions))
for i, s in ipairs(sessions) do
    print(string.format("  %d. %s (%s)", i, s.name, s.id))
end

-- Step 5: Suspend and resume a session
print("\n5. Session Lifecycle Management")
print(string.rep("-", 40))
print("📦 Suspending session...")
Session.suspend(session_id)

local suspended = Session.get(session_id)
print("  Status after suspend:", suspended.status)

print("▶️  Resuming session...")
Session.resume(session_id)

local resumed = Session.get(session_id)
print("  Status after resume:", resumed.status)

-- Step 6: Save session to persistent storage
print("\n6. Session Persistence")
print(string.rep("-", 40))
print("💾 Saving session to storage...")
Session.save(session_id)
print("✅ Session saved successfully")

-- Step 7: Thread-local session context
print("\n7. Thread-Local Session Context")
print(string.rep("-", 40))
print("Current session (before):", Session.getCurrent() or "none")

Session.setCurrent(session_id)
print("Current session (after set):", Session.getCurrent())

-- Operations can now use the current session implicitly
-- (demonstrated in artifacts.lua)

Session.setCurrent(nil)
print("Current session (after clear):", Session.getCurrent() or "none")

-- Step 8: Query sessions with filters
print("\n8. Advanced Session Queries")
print(string.rep("-", 40))
local filtered = Session.list({
    tags = {"analysis"},
    status = "active",
    limit = 5
})
print(string.format("Found %d sessions matching filters:", #filtered))
for _, s in ipairs(filtered) do
    print(string.format("  - %s", s.name))
end

-- Step 9: Error handling
print("\n9. Error Handling")
print(string.rep("-", 40))
local success, err = pcall(Session.get, "invalid-session-id")
if not success then
    print("✓ Expected error caught:", tostring(err):match("([^:]+)$"))
end

-- Step 10: Complete sessions
print("\n10. Session Cleanup")
print(string.rep("-", 40))
Session.complete(child_id)
print("✅ Child session completed")

Session.complete(session_id)
print("✅ Parent session completed")

-- Try to list again - completed sessions are removed from active
local remaining = Session.list()
print(string.format("\nRemaining active sessions: %d", #remaining))

-- Summary
print("\n\n🎉 Session Operations Completed!")
print("================================")
print("\nDemonstrated capabilities:")
print("  ✓ Session creation with rich metadata")
print("  ✓ Parent-child session relationships")
print("  ✓ Lifecycle management (suspend/resume/complete)")
print("  ✓ Persistence with save/load")
print("  ✓ Thread-local context management")
print("  ✓ Query filtering and listing")
print("  ✓ Proper error handling")
print("\nKey takeaways:")
print("  • Sessions track conversation context and artifacts")
print("  • Completed sessions are removed from active storage")
print("  • Thread-local context enables implicit session use")
print("  • Persistence allows recovery across restarts")