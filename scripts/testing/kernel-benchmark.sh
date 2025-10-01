#!/usr/bin/env bash
# ABOUTME: Benchmark automation for llmspell performance testing
# ABOUTME: Supports running, comparing, and regression detection

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Benchmark configuration
KERNEL_FEATURES="lua"  # Required features for kernel benchmarks

usage() {
    cat <<EOF
${BOLD}Usage:${NC} $0 [OPTIONS] [BENCHMARK_NAME]

Run performance benchmarks for llmspell components

${BOLD}OPTIONS:${NC}
    -p, --package PACKAGE    Run benchmarks for specific package (default: all)
    -b, --baseline NAME      Save results as baseline NAME
    -c, --compare BASELINE   Compare current run against saved BASELINE
    -l, --list              List available benchmarks
    -h, --help              Show this help message
    -v, --verbose           Show detailed cargo output

${BOLD}EXAMPLES:${NC}
    $0                                    # Run all benchmarks
    $0 -p llmspell-kernel                # Run kernel benchmarks only
    $0 -b my-baseline                     # Run and save as baseline
    $0 -c my-baseline                     # Run and compare against baseline
    $0 -p llmspell-kernel kernel_performance  # Run specific benchmark

${BOLD}BASELINES:${NC}
    Saved baselines are stored in: target/criterion/
    To list saved baselines: ls -1 target/criterion/ | grep -v "^report$"

${BOLD}REPORTS:${NC}
    HTML reports: open target/criterion/report/index.html

EOF
}

log_info() {
    echo -e "${BLUE}==>${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*" >&2
}

list_benchmarks() {
    echo -e "${BOLD}Available benchmarks:${NC}"
    echo ""
    echo -e "${BOLD}llmspell-kernel:${NC}"
    echo "  ${GREEN}kernel_performance${NC} - Startup, messaging, tool invocation"
    echo "    Targets: <2s startup, <5ms message handling, <10ms tool invocation"
    echo "    Features: lua"
    echo ""
    echo -e "${BOLD}llmspell-tools:${NC}"
    echo "  ${GREEN}tool_initialization${NC} - Tool creation time (<10ms target)"
    echo "  ${GREEN}tool_operations${NC} - Tool execution performance"
    echo ""
    echo -e "${BOLD}llmspell-bridge:${NC}"
    echo "  ${GREEN}session_bench${NC} - Session management performance"
    echo "  ${GREEN}workflow_bridge_bench${NC} - Workflow execution"
    echo ""
    echo -e "${BOLD}llmspell-workflows:${NC}"
    echo "  ${GREEN}workflow_bench${NC} - Workflow performance"
    echo ""

    # List saved baselines if any exist
    if [[ -d "target/criterion" ]]; then
        local baselines=$(ls -1 target/criterion/ 2>/dev/null | grep -v "^report$" || true)
        if [[ -n "$baselines" ]]; then
            echo -e "${BOLD}Saved Baselines:${NC}"
            echo "$baselines" | sed 's/^/  /'
            echo ""
        fi
    fi
}

check_requirements() {
    log_info "Checking requirements..."

    # Check if we're in the project root
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "Not in project root. Expected to find Cargo.toml"
        exit 1
    fi

    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust toolchain."
        exit 1
    fi

    log_success "Requirements check passed"
}

run_benchmarks() {
    local package="${1:-}"
    local benchmark="${2:-}"
    local baseline="${3:-}"
    local compare="${4:-}"
    local verbose="${5:-false}"

    log_info "Starting benchmark run..."
    echo ""

    # Build cargo command
    local cargo_args="bench --workspace"

    if [[ -n "$package" ]]; then
        cargo_args="bench -p $package"
        log_info "Package: $package"
    else
        log_info "Package: all"
    fi

    if [[ -n "$benchmark" ]]; then
        cargo_args="$cargo_args --bench $benchmark"
        log_info "Benchmark: $benchmark"
    else
        log_info "Benchmark: all"
    fi

    # Add features for kernel benchmarks
    if [[ "$package" == "llmspell-kernel" ]] || [[ -z "$package" ]]; then
        cargo_args="$cargo_args --features $KERNEL_FEATURES"
        log_info "Features: $KERNEL_FEATURES"
    fi

    # Build criterion arguments
    local criterion_args=""
    if [[ -n "$baseline" ]]; then
        criterion_args="--save-baseline $baseline"
        log_info "Saving baseline: $baseline"
    elif [[ -n "$compare" ]]; then
        # Check if baseline exists
        if [[ ! -d "target/criterion/$compare" ]]; then
            log_error "Baseline '$compare' not found in target/criterion/"
            log_info "Available baselines:"
            ls -1 target/criterion/ 2>/dev/null | grep -v "^report$" || echo "  (none)"
            exit 1
        fi
        criterion_args="--baseline $compare"
        log_info "Comparing against: $compare"
    fi

    echo ""
    log_info "Running: cargo $cargo_args -- $criterion_args"
    echo ""

    # Change to project root
    cd "$PROJECT_ROOT"

    # Run benchmarks
    local start_time=$(date +%s)

    if [[ "$verbose" == "true" ]]; then
        cargo $cargo_args -- $criterion_args
    else
        # Suppress some verbose output but keep important info
        cargo $cargo_args -- $criterion_args 2>&1 | grep -E "(Benchmarking|time:|Found|change:|regressed|improved|Compiling|Finished)" || true
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    echo ""
    log_success "Benchmark run completed in ${duration}s"
    echo ""
}

show_summary() {
    local compare="${1:-}"

    echo -e "${BOLD}Summary:${NC}"
    echo ""

    if [[ -n "$compare" ]]; then
        log_info "Comparison results available"

        # Check for regression indicators in criterion output
        # This is a simplified check - full implementation would parse JSON
        log_warning "Regression detection: Check HTML reports for detailed analysis"
        echo "  Significant regression: >10% slower than baseline"
        echo "  Noise threshold: <5% is within measurement noise"
        echo ""
    fi

    log_info "View detailed HTML reports:"
    echo "  ${BOLD}open target/criterion/report/index.html${NC}"
    echo ""

    if [[ -d "target/criterion" ]]; then
        local report_size=$(du -sh target/criterion 2>/dev/null | cut -f1)
        log_info "Report directory size: $report_size"
    fi
}

check_regressions() {
    local compare="${1:-}"

    if [[ -z "$compare" ]]; then
        return 0
    fi

    log_info "Checking for regressions against baseline: $compare"

    # Future enhancement: Parse target/criterion/*/change/estimates.json
    # For Phase 10, provide guidance only
    echo ""
    log_info "Regression Detection:"
    echo "  • Check HTML reports for performance comparison"
    echo "  • Look for 'Performance has regressed' or 'Performance has improved' messages"
    echo "  • Review charts showing performance deltas"
    echo ""
    log_info "Acceptance Thresholds:"
    echo "  • ✓ Within 5%: No action needed (measurement noise)"
    echo "  • ⚠ 5-10%: Monitor, investigate if consistent across runs"
    echo "  • ✗ >10%: Likely regression, investigate immediately"
    echo ""
}

validate_baseline_name() {
    local name="$1"

    # Check for invalid characters
    if [[ ! "$name" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        log_error "Invalid baseline name: $name"
        log_info "Baseline names must contain only letters, numbers, hyphens, and underscores"
        exit 1
    fi
}

main() {
    local package=""
    local benchmark=""
    local baseline=""
    local compare=""
    local verbose="false"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--package)
                package="$2"
                shift 2
                ;;
            -b|--baseline)
                baseline="$2"
                validate_baseline_name "$baseline"
                shift 2
                ;;
            -c|--compare)
                compare="$2"
                validate_baseline_name "$compare"
                shift 2
                ;;
            -l|--list)
                list_benchmarks
                exit 0
                ;;
            -v|--verbose)
                verbose="true"
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                benchmark="$1"
                shift
                ;;
        esac
    done

    # Mutual exclusivity check
    if [[ -n "$baseline" && -n "$compare" ]]; then
        log_error "Cannot use --baseline and --compare together"
        exit 1
    fi

    # Run benchmarks
    check_requirements
    run_benchmarks "$package" "$benchmark" "$baseline" "$compare" "$verbose"
    check_regressions "$compare"
    show_summary "$compare"

    log_success "Done!"
}

main "$@"
