#!/bin/bash
# llmspell-easy - Simplified launcher for LLMSpell applications
# Zero configuration wrapper that handles everything for non-technical users

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_error() {
    echo -e "${RED}❌ Error: $1${NC}" >&2
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Auto-detect llmspell binary location
find_llmspell() {
    # Check common locations in order of preference
    local locations=(
        "./target/debug/llmspell"
        "./target/release/llmspell"
        "/usr/local/bin/llmspell"
        "/usr/bin/llmspell"
        "$HOME/.cargo/bin/llmspell"
        "$(which llmspell 2>/dev/null || true)"
    )
    
    for loc in "${locations[@]}"; do
        if [ -x "$loc" ]; then
            echo "$loc"
            return 0
        fi
    done
    
    return 1
}

# Check if API keys are set
check_api_keys() {
    local has_openai=false
    local has_anthropic=false
    
    if [ -n "$OPENAI_API_KEY" ]; then
        has_openai=true
    fi
    
    if [ -n "$ANTHROPIC_API_KEY" ]; then
        has_anthropic=true
    fi
    
    if ! $has_openai && ! $has_anthropic; then
        return 1
    fi
    
    return 0
}

# Main script
main() {
    echo -e "${BLUE}🚀 LLMSpell Easy Launcher${NC}"
    echo ""
    
    # Find llmspell binary
    print_info "Looking for llmspell..."
    if ! LLMSPELL=$(find_llmspell); then
        print_error "Cannot find llmspell binary!"
        echo ""
        echo "Please install llmspell first:"
        echo "  1. Build from source: cargo build --release"
        echo "  2. Or install: cargo install llmspell-cli"
        echo ""
        echo "Then run this script again."
        exit 1
    fi
    print_success "Found llmspell at: $LLMSPELL"
    
    # Check API keys
    print_info "Checking API keys..."
    if ! check_api_keys; then
        print_warning "No API keys found!"
        echo ""
        echo "You need at least one API key to use LLMSpell."
        echo ""
        echo "Would you like to set up API keys now? (y/n)"
        read -r response
        
        if [[ "$response" == "y" || "$response" == "Y" ]]; then
            echo ""
            echo "Running interactive setup..."
            "$LLMSPELL" setup
            
            # Reload environment if config was created
            if [ -f "$HOME/.llmspell/config.toml" ]; then
                print_success "Setup complete! Please run this script again."
                exit 0
            fi
        else
            echo ""
            echo "To set up API keys manually:"
            echo "  export OPENAI_API_KEY='your-key-here'"
            echo "  export ANTHROPIC_API_KEY='your-key-here'"
            echo ""
            echo "Get your keys from:"
            echo "  OpenAI: https://platform.openai.com/api-keys"
            echo "  Anthropic: https://console.anthropic.com/settings/keys"
            exit 1
        fi
    fi
    print_success "API keys configured"
    
    # Parse command
    if [ $# -eq 0 ]; then
        # No arguments - show available apps
        echo ""
        "$LLMSPELL" apps list
        echo ""
        echo "Usage: $0 <app-name> [arguments]"
        echo ""
        echo "Examples:"
        echo "  $0 file-organizer"
        echo "  $0 research-collector"
        echo "  $0 content-creator"
        exit 0
    fi
    
    # Get the app name
    APP_NAME="$1"
    shift
    
    # Validate app name
    case "$APP_NAME" in
        file-organizer|research-collector|content-creator|communication-manager|process-orchestrator|code-review-assistant|webapp-creator)
            print_info "Starting $APP_NAME..."
            ;;
        list)
            "$LLMSPELL" apps list
            exit 0
            ;;
        help|--help|-h)
            echo "Usage: $0 <app-name> [arguments]"
            echo ""
            echo "Available applications:"
            echo "  file-organizer         - Organize messy files"
            echo "  research-collector     - Research any topic"
            echo "  content-creator        - Create content efficiently"
            echo "  communication-manager  - Manage business communications"
            echo "  process-orchestrator   - Orchestrate complex processes"
            echo "  code-review-assistant  - Review code quality"
            echo "  webapp-creator         - Create web applications"
            echo ""
            echo "Other commands:"
            echo "  list                   - List all applications"
            echo "  help                   - Show this help"
            exit 0
            ;;
        *)
            print_error "Unknown application: $APP_NAME"
            echo ""
            echo "Available applications:"
            "$LLMSPELL" apps list 2>/dev/null | grep -E "^[a-z-]+" | awk '{print "  " $1}'
            echo ""
            echo "Run '$0 help' for more information"
            exit 1
            ;;
    esac
    
    # Run the application
    echo ""
    exec "$LLMSPELL" apps "$APP_NAME" "$@"
}

# Run main function
main "$@"