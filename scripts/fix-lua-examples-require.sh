#!/bin/bash
# ABOUTME: Script to remove 'require("llmspell")' from Lua examples
# ABOUTME: Updates examples to use global APIs directly

echo "🔧 Fixing Lua Examples - Removing require statements"
echo "=================================================="

# Find all Lua files in examples directory
lua_files=$(find examples/lua -name "*.lua" -type f)

count=0
for file in $lua_files; do
    # Check if file contains the require statement
    if grep -q 'require("llmspell")' "$file"; then
        echo "Fixing: $file"
        
        # Remove the line with require("llmspell")
        sed -i.bak '/local llmspell = require("llmspell")/d' "$file"
        
        # Also remove any standalone require("llmspell") lines
        sed -i.bak '/require("llmspell")/d' "$file"
        
        # Clean up backup files
        rm -f "${file}.bak"
        
        ((count++))
    fi
done

echo ""
echo "✅ Fixed $count files"
echo ""

# Show which files still might reference llmspell variable
echo "Checking for remaining 'llmspell' variable usage..."
remaining=$(grep -l "llmspell\." examples/lua/**/*.lua 2>/dev/null || true)

if [ -n "$remaining" ]; then
    echo "⚠️  These files still reference 'llmspell' variable:"
    echo "$remaining"
else
    echo "✅ No remaining llmspell variable references found"
fi

echo ""
echo "✨ Lua examples updated to use global APIs!"