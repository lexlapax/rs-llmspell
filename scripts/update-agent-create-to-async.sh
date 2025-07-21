#!/bin/bash
# Script to update Agent.create to Agent.createAsync in all Lua examples
# Date: 2025-07-21
# Part of Task 3.3.24 - Fix async/coroutine errors

echo "=== Updating Agent.create to Agent.createAsync ==="
echo "This script will update all Lua examples to use the async-safe version"
echo

# Function to update a file
update_file() {
    local file="$1"
    if [ -f "$file" ]; then
        # Check if file contains Agent.create
        if grep -q "Agent\.create(" "$file"; then
            echo "Updating: $file"
            # Create backup
            cp "$file" "${file}.bak"
            
            # Replace Agent.create with Agent.createAsync
            # Careful to not replace Agent.createAsync, Agent.createFrom, etc.
            sed -i '' 's/Agent\.create(/Agent.createAsync(/g' "$file"
            
            # Verify the change
            if grep -q "Agent\.createAsync(" "$file"; then
                echo "  ✓ Updated successfully"
                # Remove backup if successful
                rm "${file}.bak"
            else
                echo "  ✗ Update failed, restoring backup"
                mv "${file}.bak" "$file"
            fi
        fi
    fi
}

# Update test files in root
echo "Updating test files in root directory..."
update_file "test_agent.lua"
update_file "test-llm-agent.lua"

# Update examples directory
echo -e "\nUpdating examples directory..."
for file in examples/*.lua; do
    update_file "$file"
done

# Update examples/lua/agents directory
echo -e "\nUpdating examples/lua/agents directory..."
for file in examples/lua/agents/*.lua; do
    update_file "$file"
done

# Update examples/lua/workflows directory
echo -e "\nUpdating examples/lua/workflows directory..."
for file in examples/lua/workflows/*.lua; do
    update_file "$file"
done

# Update examples/lua directory
echo -e "\nUpdating examples/lua directory..."
for file in examples/lua/*.lua; do
    update_file "$file"
done

# Update llmspell-bridge/examples directory
echo -e "\nUpdating llmspell-bridge/examples directory..."
for file in llmspell-bridge/examples/*.lua; do
    update_file "$file"
done

# Update llmspell-workflows/examples directories
echo -e "\nUpdating llmspell-workflows/examples directories..."
for dir in llmspell-workflows/examples/*/; do
    if [ -d "$dir" ]; then
        for file in "$dir"*.lua; do
            update_file "$file"
        done
    fi
done

echo -e "\n=== Update Complete ==="
echo "All Agent.create calls have been updated to Agent.createAsync"
echo "This prevents 'attempt to yield from outside a coroutine' errors"