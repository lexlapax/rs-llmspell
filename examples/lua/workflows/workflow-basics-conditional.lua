-- ABOUTME: Basic conditional workflow example using only tool steps
-- ABOUTME: Demonstrates branching logic without custom condition functions

-- Load workflow helpers for async execution
local helpers = dofile("examples/lua/workflows/workflow-helpers.lua")
-- Load tool helpers for async tool invocation
local tool_helpers = dofile("examples/lua/tools/tool-helpers.lua")

print("=== Basic Conditional Workflow Example ===\n")

-- Example 1: Simple Value-Based Branching
print("Example 1: Simple Value-Based Branching")
print("-" .. string.rep("-", 39))

-- Create test data
local test_value = 75
tool_helpers.invokeTool("file_operations", {
    operation = "write",
    path = "/tmp/test_value.txt",
    content = tostring(test_value)
})

local simple_conditional = Workflow.conditional({
    name = "value_branching",
    description = "Branch based on numeric value",
    
    branches = {
        -- High value branch
        {
            name = "high_value_branch",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = '{"value": ' .. test_value .. ', "threshold": 50}',
                    query = '.value > .threshold'
                }
            },
            steps = {
                {
                    name = "high_message",
                    tool = "template_engine",
                    input = {
                        template = "High value detected: {{value}} (above threshold)",
                        variables = { value = test_value }
                    }
                }
            }
        },
        -- Low value branch
        {
            name = "low_value_branch",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = '{"value": ' .. test_value .. ', "threshold": 50}',
                    query = '.value <= .threshold'
                }
            },
            steps = {
                {
                    name = "low_message",
                    tool = "template_engine",
                    input = {
                        template = "Low value detected: {{value}} (at or below threshold)",
                        variables = { value = test_value }
                    }
                }
            }
        }
    }
})

print("Executing value-based branching...")
local result, err = helpers.executeWorkflow(simple_conditional)

if result and result.success then
    print("✓ Conditional workflow completed!")
    print("Branches executed: " .. (result.data and result.data.executed_branches or "N/A"))
else
    print("✗ Workflow failed: " .. tostring(err or (result and result.error)))
end

-- Example 2: String Pattern Matching
print("\n\nExample 2: String Pattern Matching")
print("-" .. string.rep("-", 34))

-- Test different email formats
local test_email = "user@example.com"

local pattern_conditional = Workflow.conditional({
    name = "email_validation",
    description = "Validate and process email based on domain",
    
    branches = {
        -- Corporate email branch
        {
            name = "corporate_email",
            condition = {
                tool = "data_validation",
                input = {
                    input = test_email,
                    schema = {
                        type = "string",
                        pattern = ".*@(company|corp|business)\\.com$"
                    }
                }
            },
            steps = {
                {
                    name = "corporate_process",
                    tool = "template_engine",
                    input = {
                        template = "Corporate email detected: {{email}}\nRoute to business team.",
                        variables = { email = test_email }
                    }
                }
            }
        },
        -- Personal email branch
        {
            name = "personal_email",
            condition = {
                tool = "data_validation",
                input = {
                    input = test_email,
                    schema = {
                        type = "string",
                        pattern = ".*@(gmail|yahoo|hotmail|outlook)\\.com$"
                    }
                }
            },
            steps = {
                {
                    name = "personal_process",
                    tool = "template_engine",
                    input = {
                        template = "Personal email detected: {{email}}\nRoute to consumer team.",
                        variables = { email = test_email }
                    }
                }
            }
        },
        -- Default branch (always true)
        {
            name = "default_branch",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = '{"always": true}',
                    query = '.always'
                }
            },
            steps = {
                {
                    name = "default_process",
                    tool = "template_engine",
                    input = {
                        template = "Standard email: {{email}}\nRoute to general queue.",
                        variables = { email = test_email }
                    }
                }
            }
        }
    }
})

print("Executing pattern matching workflow...")
local pattern_result, err = helpers.executeWorkflow(pattern_conditional)

if pattern_result and pattern_result.success then
    print("✓ Pattern matching completed!")
    print("Result: " .. (pattern_result.data and pattern_result.data.final_output or "N/A"))
else
    print("✗ Pattern matching failed: " .. tostring(err))
end

-- Example 3: Multi-Criteria Decision
print("\n\nExample 3: Multi-Criteria Decision")
print("-" .. string.rep("-", 34))

-- Product data for decision making
local product = {
    name = "Premium Widget",
    price = 150,
    stock = 5,
    category = "electronics"
}

-- Save product data
local product_json = tool_helpers.invokeTool("json_processor", {
    operation = "stringify",
    input = product
})
if product_json then
    tool_helpers.invokeTool("file_operations", {
        operation = "write",
        path = "/tmp/product.json",
        content = product_json.output
    })
end

local decision_workflow = Workflow.conditional({
    name = "product_decision",
    description = "Make decisions based on product attributes",
    
    branches = {
        -- Premium product branch
        {
            name = "premium_product",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = product_json and product_json.output or "{}",
                    query = '.price > 100 and .category == "electronics"'
                }
            },
            steps = {
                {
                    name = "premium_pricing",
                    tool = "calculator",
                    input = { input = tostring(product.price) .. " * 1.2" }  -- 20% markup
                },
                {
                    name = "premium_message",
                    tool = "template_engine",
                    input = {
                        template = "Premium product: {{name}}\nAdjusted price: ${{price}}",
                        variables = {
                            name = product.name,
                            price = "{{step:premium_pricing:output}}"
                        }
                    }
                }
            }
        },
        -- Low stock branch
        {
            name = "low_stock",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = product_json and product_json.output or "{}",
                    query = '.stock < 10'
                }
            },
            steps = {
                {
                    name = "reorder_quantity",
                    tool = "calculator",
                    input = { input = "50 - " .. tostring(product.stock) }
                },
                {
                    name = "stock_alert",
                    tool = "template_engine",
                    input = {
                        template = "Low stock alert: {{name}}\nCurrent: {{current}}\nReorder: {{reorder}} units",
                        variables = {
                            name = product.name,
                            current = product.stock,
                            reorder = "{{step:reorder_quantity:output}}"
                        }
                    }
                }
            }
        },
        -- Standard processing (default)
        {
            name = "standard",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = '{"value": true}',
                    query = '.value'
                }
            },
            steps = {
                {
                    name = "standard_process",
                    tool = "template_engine",
                    input = {
                        template = "Standard processing: {{name}} (${{price}})",
                        variables = {
                            name = product.name,
                            price = product.price
                        }
                    }
                }
            }
        }
    },
    
    -- Execute all matching branches
    execute_default_on_no_match = false
})

print("Executing multi-criteria decision...")
local decision_result, err = helpers.executeWorkflow(decision_workflow)

if decision_result and decision_result.success then
    print("✓ Decision workflow completed!")
    print("Matched branches: " .. (decision_result.data and decision_result.data.matched_branches or "0"))
else
    print("✗ Decision workflow failed: " .. tostring(err))
end

-- Example 4: File Existence Branching
print("\n\nExample 4: File Existence Branching")
print("-" .. string.rep("-", 35))

local file_conditional = Workflow.conditional({
    name = "file_processing",
    description = "Process based on file existence",
    
    branches = {
        -- File exists branch
        {
            name = "file_exists",
            condition = {
                tool = "file_operations",
                input = {
                    operation = "exists",
                    path = "/tmp/inventory.json"
                }
            },
            steps = {
                {
                    name = "read_file",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/inventory.json"
                    }
                },
                {
                    name = "process_data",
                    tool = "json_processor",
                    input = {
                        operation = "query",
                        input = "{{step:read_file:output}}",
                        query = '.items | length'
                    }
                },
                {
                    name = "report",
                    tool = "template_engine",
                    input = {
                        template = "File found! Contains {{count}} items.",
                        variables = {
                            count = "{{step:process_data:output}}"
                        }
                    }
                }
            }
        },
        -- File missing branch
        {
            name = "file_missing",
            condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = '{"missing": true}',
                    query = '.missing'
                }
            },
            steps = {
                {
                    name = "create_default",
                    tool = "template_engine",
                    input = {
                        template = '{"items": [], "created": "{{timestamp}}"}',
                        variables = {
                            timestamp = os.date("%Y-%m-%dT%H:%M:%S")
                        }
                    }
                },
                {
                    name = "save_default",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/default_inventory.json",
                        content = "{{step:create_default:output}}"
                    }
                }
            }
        }
    }
})

print("Executing file existence branching...")
local file_result, err = helpers.executeWorkflow(file_conditional)

if file_result and file_result.success then
    print("✓ File processing completed!")
else
    print("✗ File processing failed: " .. tostring(err))
end

-- Summary
print("\n\n=== Basic Conditional Workflow Summary ===")
print("Key concepts demonstrated:")
print("1. Conditions use tool outputs (json_processor, data_validation)")
print("2. Multiple branches can execute (not just first match)")
print("3. Default branches using always-true conditions")
print("4. Complex conditions via jq queries in json_processor")
print("\nNo custom condition functions needed!")

print("\n=== Example Complete ===")