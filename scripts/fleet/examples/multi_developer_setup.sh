#!/bin/bash
# Multi-Developer Fleet Setup Example
# Shows how multiple developers can work with isolated kernel environments

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo "=== Multi-Developer Fleet Setup Example ==="
echo ""

# Ensure we're in fleet directory
cd "$(dirname "$0")/.."

# Clean start
echo "Cleaning up any existing kernels..."
./llmspell-fleet stop-all 2>/dev/null || true
./llmspell-fleet cleanup

echo ""
echo -e "${GREEN}Scenario 1: Three developers with different LLM providers${NC}"
echo "==========================================================="

# Developer A: Using OpenAI
echo ""
echo -e "${BLUE}Developer A: OpenAI Configuration${NC}"
echo "Creating OpenAI config..."
cat > configs/openai.toml << 'EOF'
[kernel]
language = "lua"
debug = false

[llm]
provider = "openai"
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"

[logging]
level = "info"
EOF

# Note: In production, you'd use: ./llmspell-fleet spawn openai.toml lua
# For demo, we'll use default config
./llmspell-fleet spawn
KERNEL_A=$(./llmspell-fleet list | grep kernel- | head -1 | awk '{print $1}')
PORT_A=$(./llmspell-fleet list | grep "$KERNEL_A" | awk '{print $5}')
echo -e "${GREEN}✓ Developer A kernel: $KERNEL_A on port $PORT_A${NC}"

# Developer B: Using Anthropic
echo ""
echo -e "${BLUE}Developer B: Anthropic Configuration${NC}"
echo "Creating Anthropic config..."
cat > configs/anthropic.toml << 'EOF'
[kernel]
language = "lua"
debug = false

[llm]
provider = "anthropic"
model = "claude-3-opus"
api_key_env = "ANTHROPIC_API_KEY"

[logging]
level = "info"
EOF

# Note: In production, you'd use: ./llmspell-fleet spawn anthropic.toml lua
# For demo, we'll spawn another with default config
./llmspell-fleet spawn
KERNEL_B=$(./llmspell-fleet list | grep kernel- | tail -2 | head -1 | awk '{print $1}')
PORT_B=$(./llmspell-fleet list | grep "$KERNEL_B" | awk '{print $5}')
echo -e "${GREEN}✓ Developer B kernel: $KERNEL_B on port $PORT_B${NC}"

# Developer C: Using Local Model
echo ""
echo -e "${BLUE}Developer C: Local Model Configuration${NC}"
echo "Creating local model config..."
cat > configs/local.toml << 'EOF'
[kernel]
language = "lua"
debug = true

[llm]
provider = "local"
model = "llama2"
endpoint = "http://localhost:11434"

[logging]
level = "debug"
EOF

# Note: In production, you'd use: ./llmspell-fleet spawn local.toml lua
# For demo, we'll spawn another with default config
./llmspell-fleet spawn
KERNEL_C=$(./llmspell-fleet list | grep kernel- | tail -1 | awk '{print $1}')
PORT_C=$(./llmspell-fleet list | grep "$KERNEL_C" | awk '{print $5}')
echo -e "${GREEN}✓ Developer C kernel: $KERNEL_C on port $PORT_C${NC}"

echo ""
echo -e "${YELLOW}Current Fleet Status:${NC}"
./llmspell-fleet list

echo ""
echo -e "${GREEN}Scenario 2: Collaborative Session Sharing${NC}"
echo "=========================================="
echo ""
echo "Multiple developers can connect to the same kernel for pair programming:"
echo ""
echo "Developer A and B collaborating on the same kernel:"
echo "  Developer A: jupyter console --existing ~/.llmspell/fleet/$KERNEL_A.json"
echo "  Developer B: jupyter console --existing ~/.llmspell/fleet/$KERNEL_A.json"
echo ""
echo "Both developers will share:"
echo "  - Same script runtime state"
echo "  - Same variables and functions"
echo "  - Same debug breakpoints"
echo "  - Real-time collaboration"

echo ""
echo -e "${GREEN}Scenario 3: Resource-Limited Development${NC}"
echo "========================================="
echo ""
echo "Creating resource-constrained kernel for testing..."

# Create a test script to demonstrate resource usage
cat > test_memory.lua << 'EOF'
-- Test memory usage
local data = {}
for i = 1, 1000000 do
    data[i] = string.rep("x", 100)
end
print("Allocated large data structure")
EOF

echo "Resource limits can be applied at OS level:"
echo "  Memory limit: ulimit -m 524288 && ./llmspell-fleet spawn"
echo "  CPU priority: nice -n 10 ./llmspell-fleet spawn"
echo "  Docker limits: See docker-compose.yml with mem_limit and cpus"

echo ""
echo -e "${GREEN}Scenario 4: Team Development Workflow${NC}"
echo "======================================"
echo ""
echo "1. Frontend team uses JavaScript kernel:"
python3 fleet_manager.py spawn --language javascript 2>/dev/null || echo "  (JavaScript support in future release)"

echo ""
echo "2. Backend team uses Python kernel:"
python3 fleet_manager.py spawn --language python 2>/dev/null || echo "  (Python support in future release)"

echo ""
echo "3. Data team uses Lua kernel with data libraries:"
echo "  Already running: $KERNEL_C with debug mode for data exploration"

echo ""
echo -e "${YELLOW}Fleet Metrics:${NC}"
python3 fleet_manager.py metrics | jq -r '
    "Total Kernels: \(.total_kernels)",
    "Total Memory: \([.kernels[].memory_mb] | add | round)MB",
    "Average CPU: \([.kernels[].cpu_percent] | add / length | round)%"
'

echo ""
echo -e "${GREEN}Scenario 5: Development to Production Pipeline${NC}"
echo "=============================================="
echo ""
echo "Development kernels (interactive):"
echo "  - Debug enabled"
echo "  - Verbose logging"
echo "  - Small resource limits"
echo ""
echo "Production kernels (background):"
echo "  - Debug disabled"
echo "  - Error logging only"
echo "  - Larger resource limits"
echo "  - Health monitoring"

echo ""
echo -e "${YELLOW}Connection Instructions:${NC}"
echo "=============================="
echo ""
echo "Each developer connects to their kernel using:"
echo "1. Jupyter Console:"
for kernel in $KERNEL_A $KERNEL_B $KERNEL_C; do
    echo "   jupyter console --existing ~/.llmspell/fleet/$kernel.json"
done

echo ""
echo "2. Direct TCP connection:"
echo "   Port $PORT_A: Developer A (OpenAI)"
echo "   Port $PORT_B: Developer B (Anthropic)"
echo "   Port $PORT_C: Developer C (Local)"

echo ""
echo "3. Via HTTP API:"
echo "   curl http://localhost:9550/kernels  # If HTTP service is running"

echo ""
echo -e "${GREEN}✓ Multi-developer setup complete!${NC}"
echo ""
echo "To clean up this example:"
echo "  ./llmspell-fleet stop-all"
echo ""