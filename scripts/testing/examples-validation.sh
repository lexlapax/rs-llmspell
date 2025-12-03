#!/bin/bash
# examples-validation.sh - Validate all examples work with specified profiles
# Usage: ./scripts/testing/examples-validation.sh [category]
# Categories: getting-started, features, cookbook, applications, all

set -euo pipefail

# Configuration
EXAMPLES_DIR="examples/script-users"
TIMEOUT_SECONDS=30
FAILED=0
SKIPPED=0
PASSED=0

# Determine llmspell binary path (prefer built binary over cargo run for speed)
if [[ -x "./target/debug/llmspell" ]]; then
    LLMSPELL_BIN="./target/debug/llmspell"
elif [[ -x "./target/release/llmspell" ]]; then
    LLMSPELL_BIN="./target/release/llmspell"
else
    LLMSPELL_BIN="cargo run --quiet --bin llmspell --"
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to extract profile from example header
get_profile() {
    local file="$1"
    # Lua files use -- for comments, not #
    grep "^-- Profile:" "$file" | head -1 | awk '{print $3}' || echo "minimal"
}

# Function to check if example requires API key
requires_api_key() {
    local file="$1"
    # Check for API KEY mentions in comments (API keys are usually in Prerequisites section)
    # Use grep with -- to prevent -- in pattern from being interpreted as option
    grep -q -- "API_KEY" "$file" 2>/dev/null
}

# Function to validate single example
validate_example() {
    local example="$1"
    local profile=$(get_profile "$example")
    local basename=$(basename "$example")

    echo -n "Testing: $basename with profile '$profile' ... "

    # Skip if requires API key and not available
    if requires_api_key "$example"; then
        if [[ -z "${OPENAI_API_KEY:-}" ]] && [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
            echo -e "${YELLOW}SKIPPED${NC} (API key required)"
            ((SKIPPED++)) || true
            return 0
        fi
    fi

    # Run example with timeout
    if timeout ${TIMEOUT_SECONDS}s $LLMSPELL_BIN -p "$profile" run "$example" &>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++)) || true
    else
        echo -e "${RED}FAILED${NC}"
        ((FAILED++)) || true
        # Log failure details
        echo "  Profile: $profile"
        echo "  File: $example"
        echo "  Run command to debug:"
        echo "    $LLMSPELL_BIN -p $profile run $example"
    fi
}

# Main validation logic
main() {
    local category="${1:-all}"

    echo "========================================="
    echo "  LLMSpell Examples Validation"
    echo "  Category: $category"
    echo "  Timeout: ${TIMEOUT_SECONDS}s per example"
    echo "========================================="
    echo ""

    case "$category" in
        getting-started)
            echo "Validating getting-started examples (REQUIRED: 100% pass rate)..."
            for example in "$EXAMPLES_DIR"/getting-started/*.lua; do
                [[ -f "$example" ]] || continue
                validate_example "$example"
            done
            ;;

        features)
            echo "Validating features examples..."
            for example in "$EXAMPLES_DIR"/features/*.lua; do
                [[ -f "$example" ]] || continue
                validate_example "$example"
            done
            ;;

        cookbook)
            echo "Validating cookbook examples (TARGET: 90%+ pass rate)..."
            for example in "$EXAMPLES_DIR"/cookbook/*.lua; do
                [[ -f "$example" ]] || continue
                validate_example "$example"
            done
            ;;

        applications)
            echo "Validating applications (may require API keys)..."
            for app_dir in "$EXAMPLES_DIR"/applications/*/; do
                main_lua="${app_dir}main.lua"
                [[ -f "$main_lua" ]] || continue
                validate_example "$main_lua"
            done
            ;;

        all)
            echo "Validating ALL examples..."
            main getting-started
            echo ""
            main features
            echo ""
            main cookbook
            echo ""
            main applications
            ;;

        *)
            echo "Error: Unknown category '$category'"
            echo "Usage: $0 [getting-started|features|cookbook|applications|all]"
            exit 1
            ;;
    esac

    echo ""
    echo "========================================="
    echo "  Results"
    echo "========================================="
    echo -e "${GREEN}Passed:${NC}  $PASSED"
    echo -e "${YELLOW}Skipped:${NC} $SKIPPED (API keys not available)"
    echo -e "${RED}Failed:${NC}  $FAILED"
    echo ""

    if [[ "$FAILED" -gt 0 ]]; then
        echo -e "${RED}VALIDATION FAILED${NC}"
        exit 1
    else
        echo -e "${GREEN}VALIDATION PASSED${NC}"
        exit 0
    fi
}

# Run main with all arguments
main "$@"
