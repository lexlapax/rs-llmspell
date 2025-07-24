#!/bin/bash

# Script to analyze and suggest tags for integration tests
# This script analyzes test files and suggests appropriate ignore tags based on their characteristics

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Function to print info
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to analyze a test file and suggest tags
analyze_test_file() {
    local file=$1
    local suggested_tags=""
    local reasons=""
    
    # Check if it's already ignored
    if grep -q "#\[ignore" "$file" 2>/dev/null; then
        existing_ignore=$(grep -o '#\[ignore[^]]*\]' "$file" | head -1)
        echo -e "${YELLOW}Already has: $existing_ignore${NC}"
        return
    fi
    
    # Check for external network dependencies
    if grep -qE "(httpbin\.org|example\.com|https?://|reqwest::|Client::new)" "$file" 2>/dev/null; then
        suggested_tags="external"
        reasons="$reasons - Uses external HTTP endpoints\n"
    fi
    
    # Check if it's in a tool-specific test directory
    if [[ "$file" =~ llmspell-tools/tests/ ]]; then
        if [[ -z "$suggested_tags" ]]; then
            suggested_tags="tool,integration"
        else
            suggested_tags="$suggested_tags,tool,integration"
        fi
        reasons="$reasons - Tool integration test\n"
    fi
    
    # Check if it's a bridge/runtime test
    if [[ "$file" =~ llmspell-bridge/tests/ ]]; then
        if [[ -z "$suggested_tags" ]]; then
            suggested_tags="bridge,integration"
        else
            suggested_tags="$suggested_tags,bridge,integration"
        fi
        reasons="$reasons - Bridge/runtime integration test\n"
    fi
    
    # Check for LLM provider tests
    if grep -qE "(OpenAI|Anthropic|Gemini|LLMProvider|mock_provider)" "$file" 2>/dev/null; then
        if [[ -z "$suggested_tags" ]]; then
            suggested_tags="llm,integration"
        else
            suggested_tags="$suggested_tags,llm"
        fi
        reasons="$reasons - Tests LLM provider functionality\n"
    fi
    
    # Check for slow operations
    if grep -qE "(tokio::time::sleep|Duration::from_secs\([5-9]|[0-9]{2,}\)|large_file|benchmark|stress_test)" "$file" 2>/dev/null; then
        if [[ -z "$suggested_tags" ]]; then
            suggested_tags="slow"
        else
            suggested_tags="$suggested_tags,slow"
        fi
        reasons="$reasons - Contains slow operations (sleep/large data)\n"
    fi
    
    # Check for database operations
    if grep -qE "(database|postgres|mysql|sqlite|rocksdb|sled)" "$file" 2>/dev/null; then
        if [[ -z "$suggested_tags" ]]; then
            suggested_tags="database,integration"
        else
            suggested_tags="$suggested_tags,database"
        fi
        reasons="$reasons - Tests database functionality\n"
    fi
    
    # Check for file system operations beyond basic read/write
    if grep -qE "(TempDir|tempfile|std::fs::|tokio::fs::)" "$file" 2>/dev/null; then
        if ! grep -q "integration" <<< "$suggested_tags" 2>/dev/null; then
            if [[ -z "$suggested_tags" ]]; then
                suggested_tags="integration"
            else
                suggested_tags="$suggested_tags,integration"
            fi
            reasons="$reasons - Uses file system operations\n"
        fi
    fi
    
    # If no specific tags, check if it's in tests/ directory (likely integration)
    if [[ -z "$suggested_tags" ]] && [[ "$file" =~ /tests/ ]] && [[ ! "$file" =~ /src/ ]]; then
        suggested_tags="integration"
        reasons="$reasons - Located in tests/ directory\n"
    fi
    
    # Print suggestions
    if [[ -n "$suggested_tags" ]]; then
        echo -e "${GREEN}Suggested tags: #[ignore = \"$suggested_tags\"]${NC}"
        echo -e "${MAGENTA}Reasons:${NC}"
        echo -e "$reasons"
    else
        echo -e "${BLUE}No special tags needed (standard integration test)${NC}"
    fi
}

# Main script
echo "🏷️  Analyzing Integration Tests for Tag Suggestions"
echo "=================================================="
echo ""

# Check for dry-run mode
DRY_RUN=true
if [[ "$1" == "--apply" ]]; then
    DRY_RUN=false
    print_warning "Running in APPLY mode - will modify files!"
    echo ""
fi

# Find all test files
print_info "Finding all test files..."
TEST_FILES=$(find . -name "*.rs" -path "*/tests/*" -type f | grep -v "/target/" | sort)
TEST_COUNT=$(echo "$TEST_FILES" | wc -l)

echo "Found $TEST_COUNT test files"
echo ""

# Analyze each file
for file in $TEST_FILES; do
    echo "📄 ${file#./}"
    analyze_test_file "$file"
    echo ""
done

# Summary
echo "=================================================="
echo "Summary:"
echo ""
echo "Tag meanings:"
echo "  external    - Requires network access to external services"
echo "  tool        - Tests in llmspell-tools package"
echo "  bridge      - Tests in llmspell-bridge package"
echo "  llm         - Tests LLM provider functionality"
echo "  slow        - Takes significant time to run"
echo "  database    - Requires database access"
echo "  integration - General integration test"
echo ""

if $DRY_RUN; then
    print_info "This was a dry run. To apply suggested tags, run:"
    echo "  $0 --apply"
    echo ""
    print_warning "Note: Automatic application is not yet implemented."
    print_warning "Please manually add the suggested #[ignore] tags to appropriate tests."
else
    print_error "Automatic tag application not yet implemented."
    print_info "Please manually add the suggested tags to your test files."
fi