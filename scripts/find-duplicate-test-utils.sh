#!/bin/bash
# Script to find duplicate test utilities across the codebase

echo "=== Finding duplicate test helper functions across llmspell crates ==="
echo ""

# Find all create_test_* functions
echo "1. create_test_* functions by crate:"
echo "-----------------------------------"
for crate in llmspell-*; do
    if [ -d "$crate" ] && [ "$crate" != "llmspell-testing" ]; then
        count=$(find "$crate" -name "*.rs" -type f | xargs grep -h "fn create_test_" 2>/dev/null | wc -l)
        if [ $count -gt 0 ]; then
            echo "$crate: $count functions"
            find "$crate" -name "*.rs" -type f | xargs grep -n "fn create_test_" 2>/dev/null | head -5 | sed 's/^/  /'
        fi
    fi
done

echo ""
echo "2. Common test helper patterns:"
echo "-------------------------------"
# Look for common patterns
patterns=(
    "create_test_context"
    "create_test_manager"
    "create_test_agent"
    "create_test_tool"
    "create_test_workflow"
    "create_test_state"
    "create_test_event"
    "create_test_hook"
)

for pattern in "${patterns[@]}"; do
    count=$(find . -name "*.rs" -path "*/llmspell-*" ! -path "*/llmspell-testing/*" -type f | xargs grep -h "fn $pattern" 2>/dev/null | wc -l)
    if [ $count -gt 1 ]; then
        echo "$pattern: found in $count locations"
    fi
done

echo ""
echo "3. Crates using llmspell-testing:"
echo "---------------------------------"
grep -r "llmspell-testing" */Cargo.toml 2>/dev/null | grep -E "dev-dependencies|dependencies" | cut -d: -f1 | sort -u | xargs dirname | sort -u

echo ""
echo "4. Crates NOT using llmspell-testing (excluding foundational):"
echo "--------------------------------------------------------------"
for crate in llmspell-*; do
    if [ -d "$crate" ] && [ "$crate" != "llmspell-testing" ]; then
        # Skip foundational crates
        if [[ ! "$crate" =~ ^llmspell-(core|utils|storage|security|config|state-traits|cli)$ ]]; then
            if ! grep -q "llmspell-testing" "$crate/Cargo.toml" 2>/dev/null; then
                echo "$crate"
            fi
        fi
    fi
done

echo ""
echo "5. Summary:"
echo "-----------"
total_helpers=$(find . -name "*.rs" -path "*/llmspell-*" ! -path "*/llmspell-testing/*" -type f | xargs grep -h "fn create_test_" 2>/dev/null | wc -l)
echo "Total test helper functions outside llmspell-testing: $total_helpers"