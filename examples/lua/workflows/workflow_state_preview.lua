-- workflow_state_preview.lua: Preview of Phase 3.3.19 workflow state management
-- This example shows how state will work with workflows (full implementation pending)

print("=== Workflow State Management Preview ===\n")

-- The State global provides in-memory state management
print("1. Using the State global directly:")
State.set("counter", 0)
State.set("config", {
    retries = 3,
    timeout = 5000,
    debug = true
})

local counter = State.get("counter")
print("Counter value:", counter)

local config = State.get("config")
print("Config retries:", config.retries)

-- List all keys
local keys = State.list()
print("\nAll state keys:", table.concat(keys, ", "))

print("\n2. State in Sequential Workflows:")
-- When workflows gain state support, they'll use scoped state
local workflow = Workflow.sequential({
    name = "data_processing_pipeline",
    -- State will be scoped to this workflow
    state = {
        initial = {
            processed_count = 0,
            errors = {}
        }
    },
    steps = {
        {
            name = "load_data",
            type = "tool",
            tool = "file_reader",
            input = { path = "data.json" },
            -- Steps will be able to access workflow state
            on_complete = function(result, state)
                state.set("data", result.data)
                state.set("total_count", #result.data)
            end
        },
        {
            name = "process_items",
            type = "tool", 
            tool = "data_transformer",
            -- Input can reference state
            input = function(state)
                return {
                    items = state.get("data"),
                    config = state.get("config")
                }
            end,
            on_complete = function(result, state)
                local count = state.get("processed_count") or 0
                state.set("processed_count", count + result.processed)
            end
        },
        {
            name = "save_results",
            type = "tool",
            tool = "file_writer",
            input = function(state)
                return {
                    path = "output.json",
                    data = {
                        processed = state.get("processed_count"),
                        total = state.get("total_count"),
                        errors = state.get("errors")
                    }
                }
            end
        }
    }
})

print("\n3. State in Parallel Workflows:")
-- Parallel workflows will have thread-safe state access
local parallel_workflow = Workflow.parallel({
    name = "parallel_processor",
    state = {
        initial = {
            results = {},
            completed = 0
        }
    },
    branches = {
        {
            name = "branch_1",
            steps = {
                {
                    type = "tool",
                    tool = "web_scraper",
                    input = { url = "https://example.com/page1" },
                    on_complete = function(result, state)
                        -- Thread-safe state updates
                        local results = state.get("results") or {}
                        results.page1 = result.data
                        state.set("results", results)
                        
                        local completed = state.get("completed") or 0
                        state.set("completed", completed + 1)
                    end
                }
            }
        },
        {
            name = "branch_2", 
            steps = {
                {
                    type = "tool",
                    tool = "web_scraper",
                    input = { url = "https://example.com/page2" },
                    on_complete = function(result, state)
                        -- Thread-safe state updates
                        local results = state.get("results") or {}
                        results.page2 = result.data
                        state.set("results", results)
                        
                        local completed = state.get("completed") or 0
                        state.set("completed", completed + 1)
                    end
                }
            }
        }
    }
})

print("\n4. Cross-Workflow State Sharing:")
-- Workflows can access global state or other workflow states
State.set("shared_api_key", "sk-123456")

local workflow1 = Workflow.sequential({
    name = "data_fetcher",
    steps = {
        {
            type = "tool",
            tool = "api_client", 
            input = function(state)
                return {
                    api_key = State.get("shared_api_key"), -- Global state
                    endpoint = "/data"
                }
            end,
            on_complete = function(result, state)
                -- Save to global state for other workflows
                State.set("fetched_data", result.data)
            end
        }
    }
})

local workflow2 = Workflow.sequential({
    name = "data_processor",
    steps = {
        {
            type = "tool",
            tool = "data_analyzer",
            input = function(state)
                -- Read from global state set by workflow1
                return {
                    data = State.get("fetched_data")
                }
            end
        }
    }
})

print("\n5. State Persistence (Phase 5 Preview):")
print("In Phase 5, state will persist across sessions:")
print("- State.set() will write to disk (sled/rocksdb)")
print("- State.get() will read from persistent storage")
print("- State migrations for schema changes")
print("- Backup and restore capabilities")
print("- Distributed state synchronization")

print("\n6. State Scoping Patterns:")
-- Different scoping levels for isolation
print("- Global: State.get/set - shared across all workflows")
print("- Workflow: workflow.state.get/set - isolated to workflow")
print("- Step: step.state.get/set - isolated to step execution")
print("- Custom: State.namespace('cache').get/set - custom namespaces")

print("\n=== End of State Management Preview ===")

-- Note: This is a preview. Full workflow state integration
-- depends on Phase 5 persistent state implementation.
-- Currently, only the State global with in-memory storage is functional.