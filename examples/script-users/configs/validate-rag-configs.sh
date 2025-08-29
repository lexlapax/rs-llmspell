#!/bin/bash
# RAG Configuration Validation Script
# Tests all provided RAG configurations for validity and functionality

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
LLMSPELL_BIN="${PROJECT_ROOT}/target/debug/llmspell"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build the project if needed
if [ ! -f "$LLMSPELL_BIN" ]; then
    echo -e "${YELLOW}Building llmspell...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --bin llmspell
fi

# Test configurations
CONFIGS=(
    "rag-basic.toml"
    "rag-development.toml"
    "rag-production.toml"
    "rag-performance.toml"
    "rag-multi-tenant.toml"
)

echo "=== RAG Configuration Validator ==="
echo ""

# Function to validate a config
validate_config() {
    local config=$1
    local config_path="$SCRIPT_DIR/$config"
    
    echo -n "Testing $config... "
    
    # Test 1: Config parsing
    if ! LLMSPELL_CONFIG="$config_path" timeout 5 "$LLMSPELL_BIN" exec 'print("Config loaded")' &>/dev/null; then
        echo -e "${RED}FAILED${NC} (config parsing error)"
        return 1
    fi
    
    # Test 2: RAG initialization
    local test_script=$(mktemp /tmp/test_rag_XXXXXX.lua)
    cat > "$test_script" << 'EOF'
-- Test RAG availability
if RAG then
    print("RAG API available: true")
    
    -- Test basic functionality
    local success, err = pcall(function()
        -- Check if we can access RAG methods
        if RAG.ingest and RAG.search then
            print("Core methods present: true")
        end
    end)
    
    if success then
        print("RAG initialization: SUCCESS")
    else
        print("RAG initialization: FAILED - " .. tostring(err))
        os.exit(1)
    end
else
    print("RAG API not available - check if enabled in config")
    os.exit(1)
end
EOF
    
    if LLMSPELL_CONFIG="$config_path" timeout 10 "$LLMSPELL_BIN" run "$test_script" &>/dev/null; then
        echo -e "${GREEN}OK${NC}"
        rm -f "$test_script"
        return 0
    else
        echo -e "${RED}FAILED${NC} (RAG initialization error)"
        rm -f "$test_script"
        return 1
    fi
}

# Run validation for each config
FAILED_CONFIGS=()
for config in "${CONFIGS[@]}"; do
    if ! validate_config "$config"; then
        FAILED_CONFIGS+=("$config")
    fi
done

echo ""
echo "=== Validation Summary ==="
if [ ${#FAILED_CONFIGS[@]} -eq 0 ]; then
    echo -e "${GREEN}All configurations passed validation!${NC}"
    exit 0
else
    echo -e "${RED}Failed configurations:${NC}"
    for config in "${FAILED_CONFIGS[@]}"; do
        echo "  - $config"
    done
    exit 1
fi