#!/bin/bash
# ABOUTME: Master script to run all Lua examples in the new organized structure
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
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🚀 LLMSpell Master Examples Test Suite${NC}"
echo -e "${PURPLE}======================================${NC}"
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Project Root: $PROJECT_ROOT"
echo "Examples Directory: $EXAMPLES_DIR"
echo ""

# Export the command for sub-scripts
export LLMSPELL_CMD

# Check if we have API keys
api_keys_available=false
if [ -n "$OPENAI_API_KEY" ] || [ -n "$ANTHROPIC_API_KEY" ]; then
    api_keys_available=true
    echo -e "${GREEN}✅ API keys found - will run full test suite${NC}"
else
    echo -e "${YELLOW}⚠️  No API keys found - some examples may be skipped${NC}"
fi
echo ""

# Initialize master counters
master_passed=0
master_failed=0
master_skipped=0
master_start_time=$(date +%s)

# Array to track suite results
declare -a suite_results

# Function to run a test suite
run_suite() {
    local suite_name="$1"
    local script_path="$2"
    local emoji="$3"
    
    echo -e "${CYAN}${emoji} Running ${suite_name} Test Suite${NC}"
    echo "============================================="
    
    suite_start=$(date +%s)
    
    if [ -f "$script_path" ]; then
        # Run the script and capture exit code
        bash "$script_path"
        suite_exit_code=$?
        
        suite_end=$(date +%s)
        suite_duration=$((suite_end - suite_start))
        
        if [ $suite_exit_code -eq 0 ]; then
            echo -e "${GREEN}✅ ${suite_name} suite completed successfully in ${suite_duration}s${NC}"
            suite_results+=("✅ $suite_name: SUCCESS (${suite_duration}s)")
        else
            echo -e "${RED}❌ ${suite_name} suite failed after ${suite_duration}s${NC}"
            suite_results+=("❌ $suite_name: FAILED (${suite_duration}s)")
        fi
    else
        echo -e "${RED}❌ Script not found: $script_path${NC}"
        suite_results+=("❌ $suite_name: SCRIPT NOT FOUND")
    fi
    
    echo ""
    return $suite_exit_code
}

# Run individual test suites
echo -e "${BLUE}📋 Running Test Suites in Order${NC}"
echo "================================"
echo ""

# 1. Tools Examples
run_suite "Tool Examples" "$EXAMPLES_DIR/run-all-tools-examples.sh" "🔧"
tools_result=$?

# 2. Agent Examples (may require API keys)
if [ "$api_keys_available" = true ]; then
    run_suite "Agent Examples" "$EXAMPLES_DIR/run-all-agent-examples.sh" "🤖"
    agents_result=$?
else
    echo -e "${YELLOW}⏭️  Skipping Agent Examples - No API keys available${NC}"
    suite_results+=("⏭️  Agent Examples: SKIPPED (No API keys)")
    agents_result=0
    echo ""
fi

# 3. Workflow Examples
run_suite "Workflow Examples" "$EXAMPLES_DIR/run-workflow-examples.sh" "🔄"
workflows_result=$?

# Calculate master totals
master_end_time=$(date +%s)
master_total_duration=$((master_end_time - master_start_time))

# Count results
total_suites=${#suite_results[@]}
passed_suites=$(echo "${suite_results[@]}" | grep -o "SUCCESS" | wc -l)
failed_suites=$(echo "${suite_results[@]}" | grep -o "FAILED" | wc -l)
skipped_suites=$(echo "${suite_results[@]}" | grep -o "SKIPPED" | wc -l)

# Print master summary
echo -e "${PURPLE}============================================================${NC}"
echo -e "${PURPLE}📊 Master Test Suite Summary Report${NC}"
echo -e "${PURPLE}============================================================${NC}"
echo "Total suites: $total_suites"
echo -e "${GREEN}✅ Passed: $passed_suites${NC}"
echo -e "${RED}❌ Failed: $failed_suites${NC}"
echo -e "${YELLOW}⏭️  Skipped: $skipped_suites${NC}"
echo -e "${BLUE}⏱️  Total time: ${master_total_duration} seconds${NC}"

if [ $total_suites -gt 0 ]; then
    if command -v bc >/dev/null 2>&1; then
        success_rate=$(echo "scale=1; $passed_suites * 100 / $total_suites" | bc)
    else
        success_rate=$(awk "BEGIN {printf \"%.1f\", $passed_suites * 100 / $total_suites}")
    fi
    echo -e "${PURPLE}📈 Success rate: ${success_rate}%${NC}"
fi

echo ""
echo -e "${BLUE}📋 Suite Results:${NC}"
for result in "${suite_results[@]}"; do
    echo "   $result"
done

echo ""
echo -e "${BLUE}📁 Examples tested from new organized structure:${NC}"
echo "   • script-users/getting-started/ (learning path)"
echo "   • script-users/features/ (feature demonstrations)"
echo "   • script-users/advanced/ (complex patterns)"
echo "   • script-users/workflows/ (workflow patterns)"
echo "   • tests-as-examples/ (test runners and benchmarks)"

echo ""
if [ $failed_suites -eq 0 ]; then
    echo -e "${GREEN}✨ All available test suites completed successfully!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some test suites failed. Check individual reports above.${NC}"
    exit 1
fi