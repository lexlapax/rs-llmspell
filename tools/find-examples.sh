#!/bin/bash
# LLMSpell Example Finder Tool
# Helps users discover relevant examples based on search criteria

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default to examples directory
EXAMPLES_DIR="${1:-examples/script-users}"

print_usage() {
    echo "Usage: $0 [search_term] [options]"
    echo ""
    echo "Search for LLMSpell examples by keyword, tag, or pattern."
    echo ""
    echo "Options:"
    echo "  --tag TAG        Search by tag (e.g., --tag beginner)"
    echo "  --feature FEAT   Search by feature (agent, tool, workflow, state)"
    echo "  --complexity LVL Search by complexity (beginner, intermediate, advanced)"
    echo "  --list-tags      List all available tags"
    echo "  --list-features  List all features with examples"
    echo "  --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 agent                    # Find all agent examples"
    echo "  $0 --tag beginner           # Find beginner examples"
    echo "  $0 --feature workflow       # Find workflow examples"
    echo "  $0 'state.*persistence'     # Regex search for state persistence"
}

list_tags() {
    echo -e "${BLUE}Available Tags:${NC}"
    echo "  beginner     - Getting started examples"
    echo "  intermediate - Core feature demonstrations"
    echo "  advanced     - Complex patterns and integrations"
    echo "  production   - Production-ready applications"
    echo "  tools        - Tool usage examples"
    echo "  agents       - Agent orchestration examples"
    echo "  workflows    - Workflow patterns"
    echo "  state        - State management examples"
}

list_features() {
    echo -e "${BLUE}Features with Examples:${NC}"
    echo ""
    echo -e "${GREEN}Agents:${NC}"
    find "$EXAMPLES_DIR" -name "*agent*.lua" -type f | head -5 | xargs -I {} basename {}
    echo ""
    echo -e "${GREEN}Tools:${NC}"
    find "$EXAMPLES_DIR" -name "*tool*.lua" -type f | head -5 | xargs -I {} basename {}
    echo ""
    echo -e "${GREEN}Workflows:${NC}"
    find "$EXAMPLES_DIR" -name "*workflow*.lua" -type f | head -5 | xargs -I {} basename {}
    echo ""
    echo -e "${GREEN}State:${NC}"
    find "$EXAMPLES_DIR" -name "*state*.lua" -type f | head -5 | xargs -I {} basename {}
}

search_by_tag() {
    local tag=$1
    echo -e "${BLUE}Examples tagged with '${tag}':${NC}"
    
    case "$tag" in
        beginner)
            find "$EXAMPLES_DIR/getting-started" -name "*.lua" -type f
            ;;
        intermediate)
            find "$EXAMPLES_DIR/features" -name "*.lua" -type f | head -10
            ;;
        advanced)
            find "$EXAMPLES_DIR/advanced" -name "*.lua" -type f
            find "$EXAMPLES_DIR/workflows" -name "*complex*.lua" -type f
            ;;
        production)
            find "$EXAMPLES_DIR/applications" -type d -maxdepth 1 -mindepth 1
            ;;
        tools)
            find "$EXAMPLES_DIR" -name "*tool*.lua" -type f
            ;;
        agents)
            find "$EXAMPLES_DIR" -name "*agent*.lua" -type f
            ;;
        workflows)
            find "$EXAMPLES_DIR" -name "*workflow*.lua" -type f
            ;;
        state)
            find "$EXAMPLES_DIR" -name "*state*.lua" -type f
            ;;
        *)
            echo "Unknown tag: $tag"
            list_tags
            ;;
    esac
}

search_by_feature() {
    local feature=$1
    echo -e "${BLUE}Examples featuring '${feature}':${NC}"
    
    case "$feature" in
        agent|agents)
            find "$EXAMPLES_DIR" -name "*agent*.lua" -type f -exec echo "  {}" \;
            ;;
        tool|tools)
            find "$EXAMPLES_DIR" -name "*tool*.lua" -type f -exec echo "  {}" \;
            ;;
        workflow|workflows)
            find "$EXAMPLES_DIR" -name "*workflow*.lua" -type f -exec echo "  {}" \;
            ;;
        state)
            find "$EXAMPLES_DIR" -name "*state*.lua" -type f -exec echo "  {}" \;
            ;;
        hook|hooks)
            find "$EXAMPLES_DIR" -name "*hook*.lua" -type f -exec echo "  {}" \;
            ;;
        *)
            echo "Searching for '$feature' in file contents..."
            grep -r -l "$feature" "$EXAMPLES_DIR" --include="*.lua" | head -20
            ;;
    esac
}

search_by_complexity() {
    local level=$1
    echo -e "${BLUE}Examples at '${level}' complexity:${NC}"
    
    case "$level" in
        beginner)
            echo -e "${GREEN}Getting Started:${NC}"
            ls -1 "$EXAMPLES_DIR/getting-started/"*.lua 2>/dev/null || true
            ;;
        intermediate)
            echo -e "${GREEN}Features:${NC}"
            ls -1 "$EXAMPLES_DIR/features/"*.lua 2>/dev/null | head -10 || true
            echo -e "${GREEN}Basic Workflows:${NC}"
            ls -1 "$EXAMPLES_DIR/workflows/"*basics*.lua 2>/dev/null || true
            ;;
        advanced)
            echo -e "${GREEN}Advanced:${NC}"
            ls -1 "$EXAMPLES_DIR/advanced/"*.lua 2>/dev/null || true
            echo -e "${GREEN}Complex Workflows:${NC}"
            ls -1 "$EXAMPLES_DIR/workflows/"*complex*.lua 2>/dev/null || true
            ls -1 "$EXAMPLES_DIR/workflows/"*nested*.lua 2>/dev/null || true
            ;;
        expert)
            echo -e "${GREEN}Production Applications:${NC}"
            ls -d "$EXAMPLES_DIR/applications/"*/ 2>/dev/null || true
            ;;
        *)
            echo "Unknown complexity level: $level"
            echo "Valid levels: beginner, intermediate, advanced, expert"
            ;;
    esac
}

general_search() {
    local term=$1
    echo -e "${BLUE}Searching for '${term}':${NC}"
    echo ""
    echo -e "${GREEN}In filenames:${NC}"
    find "$EXAMPLES_DIR" -name "*${term}*.lua" -type f | head -10
    
    echo ""
    echo -e "${GREEN}In file contents:${NC}"
    grep -r "$term" "$EXAMPLES_DIR" --include="*.lua" -l | head -10
}

# Main script logic
if [ $# -eq 0 ] || [ "$1" == "--help" ]; then
    print_usage
    exit 0
fi

case "$1" in
    --list-tags)
        list_tags
        ;;
    --list-features)
        list_features
        ;;
    --tag)
        search_by_tag "$2"
        ;;
    --feature)
        search_by_feature "$2"
        ;;
    --complexity)
        search_by_complexity "$2"
        ;;
    *)
        general_search "$1"
        ;;
esac

echo ""
echo -e "${YELLOW}Tip: Use 'llmspell run <example-path>' to run any example${NC}"