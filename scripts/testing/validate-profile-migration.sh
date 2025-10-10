#!/bin/bash
# Validate all lua files work with updated builtin profiles
# Part of Phase 11b.4 - Configuration Consolidation

set -e

echo "=== Validating Lua File Profile Migration ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED=()
PASSED=()
SKIPPED=()
TOTAL=0

# Find all lua files in examples
while IFS= read -r lua_file; do
    TOTAL=$((TOTAL + 1))

    # Extract first -p profile from HOW TO RUN section
    # Look for patterns like: "llmspell -p profile-name run"
    # Check first 60 lines of file (covers full header section including long application headers)
    profile=$(head -60 "$lua_file" | grep -o -- '-p [a-z-]*' | head -1 | awk '{print $2}')

    if [ -z "$profile" ]; then
        echo -e "${YELLOW}⚠${NC} Skipped: $lua_file (no profile found)"
        SKIPPED+=("$lua_file")
        continue
    fi

    echo -n "Testing: $lua_file with profile '$profile'... "

    # Try to run with timeout (1 second is enough to validate profile loads)
    # We don't need the script to complete, just verify profile loads without error
    if timeout 2 ./target/debug/llmspell -p "$profile" run "$lua_file" 2>&1 | grep -q "Profile.*not found\|Unknown profile\|Invalid profile"; then
        echo -e "${RED}✗ FAIL${NC}"
        FAILED+=("$lua_file (profile: $profile)")
    else
        echo -e "${GREEN}✓ PASS${NC}"
        PASSED+=("$lua_file")
    fi
done < <(find examples/script-users -name "*.lua" -type f | sort)

# Also check top-level examples/*.lua
while IFS= read -r lua_file; do
    TOTAL=$((TOTAL + 1))

    # Extract first -p profile from HOW TO RUN section (first 60 lines)
    profile=$(head -60 "$lua_file" | grep -o -- '-p [a-z-]*' | head -1 | awk '{print $2}')

    if [ -z "$profile" ]; then
        echo -e "${YELLOW}⚠${NC} Skipped: $lua_file (no profile found)"
        SKIPPED+=("$lua_file")
        continue
    fi

    echo -n "Testing: $lua_file with profile '$profile'... "

    if timeout 2 ./target/debug/llmspell -p "$profile" run "$lua_file" 2>&1 | grep -q "Profile.*not found\|Unknown profile\|Invalid profile"; then
        echo -e "${RED}✗ FAIL${NC}"
        FAILED+=("$lua_file (profile: $profile)")
    else
        echo -e "${GREEN}✓ PASS${NC}"
        PASSED+=("$lua_file")
    fi
done < <(find examples -maxdepth 1 -name "*.lua" -type f | sort)

# Summary
echo ""
echo "==================================="
echo "VALIDATION SUMMARY"
echo "==================================="
echo "Total files: $TOTAL"
echo -e "${GREEN}Passed: ${#PASSED[@]}${NC}"
echo -e "${YELLOW}Skipped: ${#SKIPPED[@]}${NC}"
echo -e "${RED}Failed: ${#FAILED[@]}${NC}"
echo ""

if [ ${#FAILED[@]} -gt 0 ]; then
    echo -e "${RED}FAILED FILES:${NC}"
    printf '%s\n' "${FAILED[@]}"
    echo ""
    exit 1
else
    echo -e "${GREEN}✅ All lua files validated successfully!${NC}"
    echo ""

    if [ ${#SKIPPED[@]} -gt 0 ]; then
        echo -e "${YELLOW}Skipped files (no profile specified):${NC}"
        printf '%s\n' "${SKIPPED[@]}"
        echo ""
    fi

    exit 0
fi
