-- ABOUTME: Example of creating and executing a conditional workflow in Lua
-- ABOUTME: Demonstrates branching logic based on conditions

-- Helper function to check a condition
function check_threshold(value, threshold)
    return value > threshold
end

-- Create a conditional workflow with dynamic branching
local threshold = 50
local input_value = 75

local workflow = Workflow.conditional({
    name = "threshold_checker",
    description = "Workflow that branches based on a threshold check",
    
    -- Condition can be a string expression or actual boolean
    condition = tostring(check_threshold(input_value, threshold)),
    
    -- Then branch: executed when condition is true
    then_branch = {
        name = "above_threshold_processing",
        tool = "calculator",
        parameters = {
            operation = "multiply",
            a = input_value,
            b = 1.5  -- Apply 50% bonus
        }
    },
    
    -- Else branch: executed when condition is false
    else_branch = {
        name = "below_threshold_processing",
        tool = "calculator",
        parameters = {
            operation = "multiply",
            a = input_value,
            b = 0.8  -- Apply 20% penalty
        }
    }
})

-- Execute the workflow
print(string.format("Checking value %d against threshold %d...", input_value, threshold))
local result = Workflow.execute(workflow)

if result.success then
    print("Conditional workflow completed!")
    local branch_taken = result.branch_taken or "unknown"
    print(string.format("Branch taken: %s", branch_taken))
    print(string.format("Result: %s", tostring(result.output)))
else
    print("Workflow failed:", result.error)
end

-- Example: Dynamic condition based on previous workflow output
print("\n--- Dynamic Condition Example ---")

-- First workflow: calculate a value
local calc_workflow = Workflow.sequential({
    name = "calculate_score",
    steps = {
        {
            name = "base_score",
            tool = "calculator",
            parameters = { operation = "add", a = 30, b = 40 }
        }
    }
})

local calc_result = Workflow.execute(calc_workflow)
local score = calc_result.outputs and calc_result.outputs.base_score or 0

-- Second workflow: conditional based on first workflow's output
local decision_workflow = Workflow.conditional({
    name = "score_decision",
    condition = tostring(score >= 60),  -- Pass/fail threshold
    
    then_branch = {
        name = "pass_action",
        tool = "text_manipulator",
        parameters = {
            operation = "format",
            template = "Congratulations! You passed with score: {score}",
            values = { score = score }
        }
    },
    
    else_branch = {
        name = "fail_action",
        tool = "text_manipulator",
        parameters = {
            operation = "format",
            template = "Sorry, you failed. Your score: {score}. Required: 60",
            values = { score = score }
        }
    }
})

local decision_result = Workflow.execute(decision_workflow)
if decision_result.success then
    print("Decision result:", decision_result.output)
end

-- Example: Nested conditionals
print("\n--- Nested Conditional Example ---")

local grade_workflow = Workflow.conditional({
    name = "grade_calculator",
    condition = tostring(score >= 90),
    
    then_branch = {
        name = "grade_a",
        tool = "text_manipulator",
        parameters = {
            operation = "constant",
            value = "Grade: A - Excellent!"
        }
    },
    
    else_branch = {
        -- This could be another conditional workflow for more complex branching
        name = "grade_other",
        tool = "text_manipulator",
        parameters = {
            operation = "constant",
            value = score >= 80 and "Grade: B - Good!" or 
                    score >= 70 and "Grade: C - Average" or
                    score >= 60 and "Grade: D - Below Average" or
                    "Grade: F - Fail"
        }
    }
})

local grade_result = Workflow.execute(grade_workflow)
if grade_result.success then
    print("Grade result:", grade_result.output)
end