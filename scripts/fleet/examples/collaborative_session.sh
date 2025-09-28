#!/bin/bash
# Collaborative Session Example
# Shows how multiple users can share the same kernel for pair programming

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== Collaborative Session Example ==="
echo ""

cd "$(dirname "$0")/.."

# Clean start
echo "Setting up collaborative kernel..."
./llmspell-fleet stop-all 2>/dev/null || true

# Create a shared kernel
echo -e "${BLUE}Creating shared development kernel...${NC}"
./llmspell-fleet spawn default.toml lua
SHARED_KERNEL=$(./llmspell-fleet list | grep kernel- | head -1 | awk '{print $1}')
SHARED_PORT=$(./llmspell-fleet list | grep "$SHARED_KERNEL" | awk '{print $5}')
CONNECTION_FILE="$HOME/.llmspell/fleet/$SHARED_KERNEL.json"

echo -e "${GREEN}✓ Shared kernel created: $SHARED_KERNEL on port $SHARED_PORT${NC}"
echo ""

# Create example Lua scripts for collaboration
echo -e "${YELLOW}Creating collaborative workspace...${NC}"
mkdir -p collaborative_workspace

# Script 1: Shared data structure
cat > collaborative_workspace/shared_data.lua << 'EOF'
-- Shared data structure for team collaboration
TeamData = {
    tasks = {},
    completed = 0,
    in_progress = 0,
    developers = {}
}

function TeamData:add_task(task, developer)
    table.insert(self.tasks, {
        description = task,
        assigned_to = developer,
        status = "pending",
        timestamp = os.time()
    })
    print("Task added: " .. task .. " (assigned to " .. developer .. ")")
end

function TeamData:complete_task(index)
    if self.tasks[index] then
        self.tasks[index].status = "completed"
        self.completed = self.completed + 1
        print("Task " .. index .. " completed!")
    end
end

function TeamData:list_tasks()
    print("\n=== Team Tasks ===")
    for i, task in ipairs(self.tasks) do
        print(i .. ". [" .. task.status .. "] " .. task.description ..
              " (" .. task.assigned_to .. ")")
    end
    print("==================\n")
end

-- Initialize some tasks
TeamData:add_task("Implement fleet manager", "Developer A")
TeamData:add_task("Write integration tests", "Developer B")
TeamData:add_task("Create documentation", "Developer C")

print("Shared TeamData structure initialized!")
print("All developers can now access and modify TeamData")
EOF

# Script 2: Collaborative debugging
cat > collaborative_workspace/debug_together.lua << 'EOF'
-- Collaborative debugging example
DebugSession = {
    breakpoints = {},
    watch_vars = {},
    logs = {}
}

function DebugSession:add_breakpoint(file, line, condition)
    table.insert(self.breakpoints, {
        file = file,
        line = line,
        condition = condition or "true",
        added_by = os.getenv("USER") or "unknown"
    })
    print("Breakpoint added at " .. file .. ":" .. line)
end

function DebugSession:watch(variable, value)
    self.watch_vars[variable] = value
    print("Watching variable: " .. variable .. " = " .. tostring(value))
end

function DebugSession:log(message)
    table.insert(self.logs, {
        timestamp = os.time(),
        user = os.getenv("USER") or "unknown",
        message = message
    })
    print("[LOG] " .. message)
end

function DebugSession:show_state()
    print("\n=== Debug Session State ===")
    print("Breakpoints: " .. #self.breakpoints)
    for _, bp in ipairs(self.breakpoints) do
        print("  " .. bp.file .. ":" .. bp.line .. " (by " .. bp.added_by .. ")")
    end
    print("\nWatched Variables:")
    for var, val in pairs(self.watch_vars) do
        print("  " .. var .. " = " .. tostring(val))
    end
    print("===========================\n")
end

-- Example usage
DebugSession:add_breakpoint("main.lua", 42, "x > 10")
DebugSession:watch("user_count", 0)
DebugSession:log("Debug session started")

print("Collaborative DebugSession initialized!")
EOF

# Script 3: Real-time code review
cat > collaborative_workspace/code_review.lua << 'EOF'
-- Real-time code review system
CodeReview = {
    files = {},
    comments = {},
    approvals = {}
}

function CodeReview:add_file(filename, content)
    self.files[filename] = {
        content = content,
        added_at = os.time(),
        reviewer_count = 0
    }
    print("File added for review: " .. filename)
end

function CodeReview:add_comment(filename, line, comment, reviewer)
    if not self.comments[filename] then
        self.comments[filename] = {}
    end
    table.insert(self.comments[filename], {
        line = line,
        comment = comment,
        reviewer = reviewer,
        timestamp = os.time()
    })
    print("Comment added to " .. filename .. ":" .. line .. " by " .. reviewer)
end

function CodeReview:approve(filename, reviewer)
    if not self.approvals[filename] then
        self.approvals[filename] = {}
    end
    self.approvals[filename][reviewer] = true
    print(reviewer .. " approved " .. filename)
end

function CodeReview:status()
    print("\n=== Code Review Status ===")
    for filename, _ in pairs(self.files) do
        local comment_count = self.comments[filename] and #self.comments[filename] or 0
        local approval_count = 0
        if self.approvals[filename] then
            for _, _ in pairs(self.approvals[filename]) do
                approval_count = approval_count + 1
            end
        end
        print(filename .. ": " .. comment_count .. " comments, " ..
              approval_count .. " approvals")
    end
    print("==========================\n")
end

-- Example usage
CodeReview:add_file("fleet_manager.py", "# Fleet manager implementation...")
CodeReview:add_comment("fleet_manager.py", 15, "Consider using async here", "Developer A")
CodeReview:approve("fleet_manager.py", "Developer B")

print("CodeReview system initialized for real-time collaboration!")
EOF

echo -e "${GREEN}✓ Collaborative workspace created${NC}"
echo ""

# Show how to connect
echo -e "${YELLOW}Connection Instructions for Team Members:${NC}"
echo "=========================================="
echo ""
echo "1. First developer starts the shared session:"
echo -e "   ${BLUE}jupyter console --existing $CONNECTION_FILE${NC}"
echo "   Then loads the collaborative scripts:"
echo "   In [1]: dofile('collaborative_workspace/shared_data.lua')"
echo "   In [2]: dofile('collaborative_workspace/debug_together.lua')"
echo "   In [3]: dofile('collaborative_workspace/code_review.lua')"
echo ""

echo "2. Other developers join the same session:"
echo -e "   ${BLUE}jupyter console --existing $CONNECTION_FILE${NC}"
echo "   They immediately have access to:"
echo "   - TeamData (shared task list)"
echo "   - DebugSession (collaborative debugging)"
echo "   - CodeReview (real-time code review)"
echo ""

echo "3. All developers can now collaborate:"
echo "   Developer A: TeamData:add_task('Fix bug #123', 'Developer A')"
echo "   Developer B: TeamData:complete_task(1)"
echo "   Developer C: DebugSession:add_breakpoint('main.lua', 100)"
echo "   Everyone: TeamData:list_tasks()"
echo ""

echo -e "${GREEN}Example: Simulating Collaborative Session${NC}"
echo "=========================================="

# Create a test script that simulates collaboration
cat > test_collaboration.lua << 'EOF'
-- Load collaborative modules
dofile('collaborative_workspace/shared_data.lua')
dofile('collaborative_workspace/debug_together.lua')
dofile('collaborative_workspace/code_review.lua')

print("\n=== Simulating Team Collaboration ===\n")

-- Developer A's actions
print("Developer A working...")
TeamData:add_task("Implement user authentication", "Developer A")
DebugSession:add_breakpoint("auth.lua", 25)
CodeReview:add_file("auth.lua", "function authenticate(user, pass)...")

-- Developer B's actions
print("\nDeveloper B working...")
TeamData:add_task("Write unit tests", "Developer B")
TeamData:complete_task(1)  -- Complete first task
DebugSession:watch("auth_token", "abc123")
CodeReview:add_comment("auth.lua", 10, "Add input validation", "Developer B")

-- Developer C's actions
print("\nDeveloper C working...")
TeamData:add_task("Update documentation", "Developer C")
DebugSession:log("Found issue in authentication flow")
CodeReview:approve("auth.lua", "Developer C")

-- Show final state
print("\n=== Final Collaborative State ===")
TeamData:list_tasks()
DebugSession:show_state()
CodeReview:status()

print("\nCollaboration simulation complete!")
print("In a real session, these actions would be performed by different developers")
print("connected to the same kernel simultaneously.")
EOF

echo ""
echo -e "${YELLOW}Testing collaborative features...${NC}"
echo "(This simulates what would happen with multiple developers)"
echo ""

# Note: We can't actually run Lua code here without the kernel, but we show what would happen
echo "Would execute in shared kernel:"
echo "  dofile('test_collaboration.lua')"
echo ""

echo -e "${GREEN}Benefits of Collaborative Sessions:${NC}"
echo "===================================="
echo "✓ Real-time state sharing"
echo "✓ Immediate visibility of changes"
echo "✓ No merge conflicts"
echo "✓ Pair programming support"
echo "✓ Shared debugging sessions"
echo "✓ Live code reviews"
echo "✓ Team learning and mentoring"
echo ""

echo -e "${RED}Important Considerations:${NC}"
echo "=========================="
echo "⚠ All users share the same runtime state"
echo "⚠ Changes affect everyone immediately"
echo "⚠ Need coordination for critical changes"
echo "⚠ Best for trusted team members"
echo "⚠ Consider using separate kernels for independent work"
echo ""

echo -e "${GREEN}Advanced Collaboration Patterns:${NC}"
echo "================================"
echo "1. Leader-Follower: One person drives, others observe and suggest"
echo "2. Round-Robin: Team members take turns making changes"
echo "3. Specialized Roles: Each person handles specific aspects"
echo "4. Review Mode: One kernel for development, another for review"
echo ""

echo -e "${YELLOW}Current kernel for collaboration:${NC}"
./llmspell-fleet list | grep "$SHARED_KERNEL"
echo ""

echo -e "${GREEN}✓ Collaborative session example complete!${NC}"
echo ""
echo "To test this yourself:"
echo "1. Open multiple terminals"
echo "2. Each terminal: jupyter console --existing $CONNECTION_FILE"
echo "3. Load the collaborative scripts and start working together!"
echo ""
echo "Cleanup: ./llmspell-fleet stop $SHARED_KERNEL"