-- ABOUTME: Conditional workflow example demonstrating branch-based execution
-- ABOUTME: Shows how to use Workflow.conditional() for decision-based processing

-- Conditional Workflow Example
-- Demonstrates branching logic and condition-based execution

-- Note: All workflow methods are now synchronous - no helpers needed

print("=== Conditional Workflow Example ===\n")

-- Example 1: Simple Conditional Workflow
print("Example 1: Simple Conditional Workflow")
print("-" .. string.rep("-", 38))

-- Set up test data (State will be available in Phase 5)
local test_data = {
    user_type = "premium",
    account_balance = 1500
}

local simple_conditional = Workflow.conditional({
    name = "user_type_handler",
    description = "Different processing based on user type",
    
    branches = {
        -- Premium user branch
        {
            name = "premium_branch",
            condition = {
                type = "tool",
                tool = "json_processor",
                input = {
                    input = test_data,
                    operation = "query",
                    query = '.user_type == "premium"'
                }
            },
            steps = {
                {
                    name = "premium_welcome",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Welcome Premium User! Your balance: ${{balance}}",
                        variables = {
                            balance = test_data.account_balance
                        }
                    }
                },
                {
                    name = "apply_discount",
                    type = "tool",
                    tool = "calculator",
                    input = { input = tostring(test_data.account_balance) .. " * 0.20" }  -- 20% discount
                }
            }
        },
        -- Regular user branch
        {
            name = "regular_branch",
            condition = {
                type = "tool",
                tool = "json_processor",
                input = {
                    input = test_data,
                    operation = "query",
                    query = '.user_type == "regular"'
                }
            },
            steps = {
                {
                    name = "regular_welcome",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Welcome! Consider upgrading to premium.",
                        variables = {}
                    }
                },
                {
                    name = "standard_processing",
                    type = "tool",
                    tool = "calculator",
                    input = { input = tostring(test_data.account_balance) .. " * 0.05" }  -- 5% discount
                }
            }
        },
        -- Default branch (always executes if no other conditions match)
        {
            name = "default_branch",
            condition = { type = "always" },
            steps = {
                {
                    name = "default_message",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Unknown user type - applying standard processing",
                        operation = "uppercase"
                    }
                }
            }
        }
    },
    
    execute_default_on_no_match = true
})

print("Executing simple conditional workflow...")
local simple_result = simple_conditional:execute()

if simple_result then
    print("Result:")
    print("- Executed branches: " .. (simple_result.data and simple_result.data.executed_branches or "N/A"))
    print("- Success: " .. tostring(simple_result.success))
else
    print("Execution error: Unknown error")
end

-- Example 2: Multi-Condition Workflow
print("\n\nExample 2: Multi-Condition Workflow")
print("-" .. string.rep("-", 35))

-- Set up complex conditions
local environment_data = {
    temperature = 75,
    humidity = 65,
    time_of_day = "afternoon"
}

local multi_condition = Workflow.conditional({
    name = "environment_controller",
    description = "Complex environmental control logic",
    
    branches = {
        -- Hot and humid conditions
        {
            name = "cooling_mode",
            condition = {
                type = "and",
                conditions = {
                    {
                        type = "tool",
                        tool = "json_processor",
                        input = {
                            input = environment_data,
                            operation = "query",
                            query = ".temperature > 70"
                        }
                    },
                    {
                        type = "tool",
                        tool = "json_processor",
                        input = {
                            input = environment_data,
                            operation = "query",
                            query = ".humidity > 60"
                        }
                    }
                }
            },
            steps = {
                {
                    name = "activate_cooling",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = [[
Climate Control: COOLING MODE
Temperature: {{temp}}°F
Humidity: {{humidity}}%
Action: AC on, Dehumidifier on
]],
                        variables = {
                            temp = environment_data.temperature,
                            humidity = environment_data.humidity
                        }
                    }
                }
            }
        },
        -- Cold conditions
        {
            name = "heating_mode",
            condition = {
                type = "tool",
                tool = "json_processor",
                input = {
                    input = environment_data,
                    operation = "query",
                    query = ".temperature < 60"
                }
            },
            steps = {
                {
                    name = "activate_heating",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Climate Control: HEATING MODE - Temp: {{temp}}°F",
                        variables = { temp = environment_data.temperature }
                    }
                }
            }
        },
        -- Night mode
        {
            name = "night_mode",
            condition = {
                type = "or",
                conditions = {
                    {
                        type = "tool",
                        tool = "json_processor",
                        input = {
                            input = environment_data,
                            operation = "query",
                            query = '.time_of_day == "night"'
                        }
                    },
                    {
                        type = "tool",
                        tool = "json_processor",
                        input = {
                            input = environment_data,
                            operation = "query",
                            query = '.time_of_day == "late_evening"'
                        }
                    }
                }
            },
            steps = {
                {
                    name = "night_settings",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Activating night mode: Lower fan speed, dim lights",
                        operation = "lowercase"
                    }
                }
            }
        },
        -- Comfortable conditions (default)
        {
            name = "comfort_mode",
            condition = { type = "always" },
            steps = {
                {
                    name = "maintain_comfort",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Climate Control: COMFORT MODE - Maintaining current settings",
                        variables = {}
                    }
                }
            }
        }
    },
    
    -- Execute all matching branches
    execute_all_matching = true
})

print("Executing multi-condition workflow...")
local multi_result = multi_condition:execute()

if multi_result then
    print("Results:")
    print("- Matched branches: " .. (multi_result.data and multi_result.data.matched_branches or "N/A"))
    print("- Total branches evaluated: " .. (multi_result.data and multi_result.data.total_branches or "N/A"))
else
    print("Execution error: Unknown error")
end

-- Example 3: Dynamic Condition Workflow
print("\n\nExample 3: Dynamic Condition Workflow")
print("-" .. string.rep("-", 37))

-- Function to create dynamic conditions
local function create_price_workflow(threshold, order_total)
    return Workflow.conditional({
        name = "dynamic_pricing",
        description = "Dynamic pricing based on threshold",
        
        branches = {
            -- High value order
            {
                name = "high_value",
                condition = {
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = { order_total = order_total, threshold = threshold },
                        operation = "query",
                        query = ".order_total >= .threshold"
                    }
                },
                steps = {
                    {
                        name = "apply_vip_discount",
                        type = "tool",
                        tool = "calculator",
                        input = { input = string.format("%d * 0.15", order_total) }
                    },
                    {
                        name = "add_gift",
                        type = "tool",
                        tool = "template_engine",
                        input = {
                            template = "VIP Order: ${{total}} - Free gift included!",
                            variables = { total = order_total }
                        }
                    }
                }
            },
            -- Standard order
            {
                name = "standard_value",
                condition = { type = "always" },
                steps = {
                    {
                        name = "standard_discount",
                        type = "tool",
                        tool = "calculator",
                        input = { input = string.format("%d * 0.05", order_total) }
                    }
                }
            }
        }
    })
end

-- Test with different order values
local test_orders = {500, 1500, 2500}
local current_order_total = 0

for _, amount in ipairs(test_orders) do
    current_order_total = amount
    
    -- Create workflow with dynamic threshold
    local threshold = 1000
    local pricing_workflow = create_price_workflow(threshold, amount)
    
    print(string.format("\nProcessing order of $%d (threshold: $%d)", amount, threshold))
    local result = pricing_workflow:execute()
    
    if result and result.success then
        print("- Branch executed: " .. (result.data and result.data.executed_branches > 0 and "found" or "none"))
    elseif result then
        print("- Workflow failed")
    else
        print("- Execution error: " .. tostring(err))
    end
end

-- Example 4: Simple Condition Types
print("\n\nExample 4: Simple Condition Types")
print("-" .. string.rep("-", 33))

-- Demonstrate different condition types
local condition_types = Workflow.conditional({
    name = "condition_demo",
    description = "Demonstrate different condition types",
    
    branches = {
        -- String condition
        {
            name = "string_condition",
            condition = {
                type = "tool",
                tool = "text_manipulator",
                input = {
                    input = "test",
                    operation = "uppercase"
                }
            },
            steps = {
                {
                    name = "string_result",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "String condition matched",
                        variables = {}
                    }
                }
            }
        },
        -- Always condition (fallback)
        {
            name = "fallback",
            condition = { type = "always" },
            steps = {
                {
                    name = "fallback_result",
                    type = "tool", 
                    tool = "text_manipulator",
                    input = {
                        input = "Fallback condition executed",
                        operation = "lowercase"
                    }
                }
            }
        }
    }
})

print("Executing condition types demonstration...")
local types_result = condition_types:execute()
if types_result then
    print("Condition types demo completed: " .. (types_result.success and "Success" or "Failed"))
else
    print("Execution error: Unknown error")
end

-- Example 5: Condition with Step Output
print("\n\nExample 5: Condition Based on Step Output")
print("-" .. string.rep("-", 41))

local step_output_workflow = Workflow.conditional({
    name = "validation_workflow",
    description = "Branch based on validation results",
    
    -- Pre-steps that run before condition evaluation
    pre_steps = {
        {
            name = "validate_input",
            type = "tool",
            tool = "data_validation",
            input = {
                input = { username = "john_doe", email = "john@example.com" },
                schema = {
                    type = "object",
                    required = {"username", "email"},
                    properties = {
                        username = { type = "string", minLength = 3 },
                        email = { type = "string", format = "email" }
                    }
                }
            }
        }
    },
    
    branches = {
        -- Valid data branch
        {
            name = "valid_data",
            condition = {
                type = "step_output_contains",
                step_name = "validate_input",
                substring = "valid"
            },
            steps = {
                {
                    name = "process_valid",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "✓ Validation passed - Processing user registration",
                        variables = {}
                    }
                },
                {
                    name = "create_account",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                }
            }
        },
        -- Invalid data branch
        {
            name = "invalid_data",
            condition = {
                type = "not",
                condition = {
                    type = "step_output_contains",
                    step_name = "validate_input",
                    substring = "valid"
                }
            },
            steps = {
                {
                    name = "handle_invalid",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "✗ Validation failed - Please check your input",
                        operation = "uppercase"
                    }
                }
            }
        }
    }
})

print("Executing step output conditional workflow...")
local step_output_result = step_output_workflow:execute()
if step_output_result then
    print("Validation workflow completed")
else
    print("Execution error: Unknown error")
end

-- Performance test
print("\n\n=== Conditional Workflow Performance ===")

local perf_workflow = Workflow.conditional({
    name = "performance_test",
    branches = {
        {
            name = "fast_branch",
            condition = { type = "always" },
            steps = {
                {
                    name = "quick_op",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "42 * 2" }
                }
            }
        }
    }
})

-- Benchmark
local iterations = 20
local total_time = 0

for i = 1, iterations do
    local start = os.clock()
    perf_workflow:execute()
    total_time = total_time + (os.clock() - start) * 1000
end

print(string.format("Average execution time: %.2f ms", total_time / iterations))

-- Summary
print("\n=== Conditional Workflow Summary ===")
print("Examples demonstrated:")
print("1. Simple user type branching")
print("2. Multi-condition logic (AND/OR)")
print("3. Dynamic condition creation")
print("4. Nested conditional workflows")
print("5. Conditions based on step output")
print("\nKey features:")
print("- Multiple condition types")
print("- Execute all matching branches option")
print("- Default branch support")
print("- Custom condition evaluation")
print("- Pre-step execution")

print("\n=== Conditional Workflow Example Complete ===")