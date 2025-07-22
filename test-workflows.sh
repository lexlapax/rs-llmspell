#!/bin/bash
# Simple workflow test script

echo "Testing Workflow Examples"
echo "========================"

# Test each working workflow
workflows=(
    "workflow-basics-sequential.lua"
    "workflow-basics-conditional.lua" 
    "workflow-basics-parallel.lua"
    "workflow-sequential.lua"
    "workflow-conditional.lua"
    "workflow-parallel.lua"
)

passed=0
failed=0

for workflow in "${workflows[@]}"; do
    echo -e "\n[$workflow]"
    if cargo run --bin llmspell -- run "examples/lua/workflows/$workflow" > /dev/null 2>&1; then
        echo "✅ PASSED"
        ((passed++))
    else
        echo "❌ FAILED"
        ((failed++))
    fi
done

echo -e "\nSummary:"
echo "========"
echo "✅ Passed: $passed"
echo "❌ Failed: $failed"
echo "📊 Success rate: $(( passed * 100 / (passed + failed) ))%"