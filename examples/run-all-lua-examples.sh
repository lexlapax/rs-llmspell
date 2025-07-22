#!/bin/bash
# ABOUTME: Master script to run all Lua examples and generate comprehensive report
# ABOUTME: Orchestrates testing of tools, agents, workflows, and core functionality

# Set the llmspell command path
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
EXAMPLES_DIR="$PROJECT_ROOT/examples"

if [ -n "$LLMSPELL_CMD" ]; then
    # Use the provided command
    echo "Using provided LLMSPELL_CMD: $LLMSPELL_CMD"
elif [ -x "$PROJECT_ROOT/target/debug/llmspell" ]; then
    LLMSPELL_CMD="$PROJECT_ROOT/target/debug/llmspell"
    echo "Using llmspell binary: $LLMSPELL_CMD"
elif [ -x "$PROJECT_ROOT/target/release/llmspell" ]; then
    LLMSPELL_CMD="$PROJECT_ROOT/target/release/llmspell"
    echo "Using llmspell binary: $LLMSPELL_CMD"
elif command -v cargo &> /dev/null; then
    LLMSPELL_CMD="cargo run --bin llmspell --"
    echo "Using cargo run as llmspell command"
else
    echo "Error: llmspell binary not found and cargo not available"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🚀 LLMSpell Complete Lua Examples Test Suite"
echo "==========================================="
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Check if we're in the right directory
if [[ $(basename "$PWD") == "examples" ]]; then
    examples_dir="."
else
    examples_dir="examples"
fi

# Initialize overall counters
total_passed=0
total_failed=0
total_skipped=0
overall_start=$(date +%s)

# Function to run a test suite
run_suite() {
    local suite_name=$1
    local script_path=$2
    local emoji=$3
    
    echo ""
    echo -e "${BLUE}${emoji} Running ${suite_name}...${NC}"
    echo "----------------------------------------"
    
    if [ -f "$script_path" ]; then
        # Make script executable
        chmod +x "$script_path"
        
        # Run the script and capture output
        suite_output=$("$script_path" 2>&1)
        suite_exit=$?
        
        # Extract stats from output (looking for summary lines)
        passed=$(echo "$suite_output" | grep -oE "Passed: [0-9]+" | grep -oE "[0-9]+$" | tail -1)
        failed=$(echo "$suite_output" | grep -oE "Failed: [0-9]+" | grep -oE "[0-9]+$" | tail -1)
        skipped=$(echo "$suite_output" | grep -oE "Skipped: [0-9]+" | grep -oE "[0-9]+$" | tail -1)
        
        # Default to 0 if not found
        passed=${passed:-0}
        failed=${failed:-0}
        skipped=${skipped:-0}
        
        # Update totals
        total_passed=$((total_passed + passed))
        total_failed=$((total_failed + failed))
        total_skipped=$((total_skipped + skipped))
        
        # Show summary
        if [ $suite_exit -eq 0 ] && [ $failed -eq 0 ]; then
            echo -e "${GREEN}✅ ${suite_name} completed successfully${NC}"
        else
            echo -e "${RED}❌ ${suite_name} had failures${NC}"
        fi
        echo "   Passed: $passed, Failed: $failed, Skipped: $skipped"
        
        # Option to show detailed output
        if [ "$VERBOSE" = "1" ]; then
            echo ""
            echo "Detailed output:"
            echo "$suite_output"
        fi
    else
        echo -e "${YELLOW}⚠️  ${suite_name} test script not found at: $script_path${NC}"
    fi
}

# Function to run individual Lua files
run_lua_files() {
    local dir=$1
    local pattern=$2
    local name=$3
    
    echo ""
    echo -e "${BLUE}🔧 Running ${name}...${NC}"
    echo "----------------------------------------"
    
    local files=($(ls $dir/$pattern 2>/dev/null | sort))
    local passed=0
    local failed=0
    
    for file in "${files[@]}"; do
        basename_file=$(basename "$file")
        echo -n "Testing $basename_file... "
        
        # Run with timeout
        timeout 30 $LLMSPELL_CMD run "$file" > /tmp/lua_test_output.log 2>&1
        exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}✅${NC}"
            ((passed++))
        else
            echo -e "${RED}❌ (exit code: $exit_code)${NC}"
            ((failed++))
            if [ "$VERBOSE" = "1" ]; then
                echo "Error output:"
                cat /tmp/lua_test_output.log | head -20
            fi
        fi
    done
    
    total_passed=$((total_passed + passed))
    total_failed=$((total_failed + failed))
    
    echo "   Passed: $passed, Failed: $failed"
    rm -f /tmp/lua_test_output.log
}

# Check for verbose flag
if [ "$1" = "-v" ] || [ "$1" = "--verbose" ]; then
    VERBOSE=1
    echo "Verbose mode enabled"
fi

# API Key check
echo -e "${YELLOW}⚠️  API Key Status:${NC}"
if [ -n "$OPENAI_API_KEY" ]; then
    echo -e "   ${GREEN}✓${NC} OPENAI_API_KEY is set"
else
    echo -e "   ${RED}✗${NC} OPENAI_API_KEY is not set"
fi
if [ -n "$ANTHROPIC_API_KEY" ]; then
    echo -e "   ${GREEN}✓${NC} ANTHROPIC_API_KEY is set"
else
    echo -e "   ${RED}✗${NC} ANTHROPIC_API_KEY is not set"
fi

# Run test suites in order
echo ""
echo "============================================"
echo "Starting Test Execution"
echo "============================================"

# 1. Core functionality tests
run_lua_files "$examples_dir/lua" "test-*.lua" "Core Functionality Tests"

# 2. Tool examples (no API needed)
run_suite "Tool Examples" "$examples_dir/run-all-tools-examples.sh" "🔨"

# 3. Workflow examples (some may need API)
run_suite "Workflow Examples" "$examples_dir/run-workflow-examples.sh" "🔄"

# 4. Agent examples (need API)
if [ -n "$OPENAI_API_KEY" ] || [ -n "$ANTHROPIC_API_KEY" ]; then
    run_suite "Agent Examples" "$examples_dir/run-agent-examples.sh" "🤖"
else
    echo ""
    echo -e "${YELLOW}⏭️  Skipping Agent Examples (no API keys)${NC}"
    echo "----------------------------------------"
fi

# Calculate final stats
overall_end=$(date +%s)
overall_duration=$((overall_end - overall_start))
grand_total=$((total_passed + total_failed + total_skipped))

# Print final summary
echo ""
echo "============================================"
echo -e "${BLUE}📊 Complete Test Summary${NC}"
echo "============================================"
echo "Total examples tested: $grand_total"
echo -e "✅ Passed: ${GREEN}$total_passed${NC}"
echo -e "❌ Failed: ${RED}$total_failed${NC}"
echo -e "⏭️  Skipped: ${YELLOW}$total_skipped${NC}"
echo "⏱️  Total time: ${overall_duration} seconds"

if [ $grand_total -gt 0 ]; then
    if command -v bc >/dev/null 2>&1; then
        success_rate=$(echo "scale=1; $total_passed * 100 / $grand_total" | bc)
    else
        success_rate=$(awk "BEGIN {printf \"%.1f\", $total_passed * 100 / $grand_total}")
    fi
    echo "📈 Overall success rate: ${success_rate}%"
fi

echo ""
if [ $total_failed -eq 0 ]; then
    echo -e "${GREEN}✨ All tests passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed. Review the output above.${NC}"
    echo ""
    echo "Tips for debugging:"
    echo "  - Run with -v or --verbose for detailed output"
    echo "  - Check individual test scripts for specific failures"
    echo "  - Ensure API keys are set for agent tests"
    echo "  - Review example files for any recent changes"
    exit 1
fi