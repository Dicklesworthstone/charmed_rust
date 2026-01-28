#!/usr/bin/env bash
# Run demo_showcase E2E test suite
#
# Usage:
#   ./scripts/demo_showcase_e2e.sh              # Run all E2E tests
#   ./scripts/demo_showcase_e2e.sh smoke        # Run smoke tour tests only
#   ./scripts/demo_showcase_e2e.sh navigation   # Run navigation tests only
#   ./scripts/demo_showcase_e2e.sh settings     # Run settings tests only
#   ./scripts/demo_showcase_e2e.sh --no-color   # Run without ANSI colors
#   ./scripts/demo_showcase_e2e.sh --clean      # Clean artifacts before running

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARTIFACTS_DIR="$PROJECT_ROOT/target/demo_showcase_e2e"

# Colors for output (can be disabled with --no-color)
USE_COLOR=true
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log_info() {
    if $USE_COLOR; then
        echo -e "${BLUE}[INFO]${NC} $1"
    else
        echo "[INFO] $1"
    fi
}

log_success() {
    if $USE_COLOR; then
        echo -e "${GREEN}[SUCCESS]${NC} $1"
    else
        echo "[SUCCESS] $1"
    fi
}

log_warn() {
    if $USE_COLOR; then
        echo -e "${YELLOW}[WARN]${NC} $1"
    else
        echo "[WARN] $1"
    fi
}

log_error() {
    if $USE_COLOR; then
        echo -e "${RED}[ERROR]${NC} $1"
    else
        echo "[ERROR] $1"
    fi
}

log_header() {
    if $USE_COLOR; then
        echo -e "\n${BOLD}${CYAN}$1${NC}\n"
    else
        echo ""
        echo "$1"
        echo ""
    fi
}

# Show usage help
usage() {
    cat << EOF
demo_showcase E2E Test Runner

Usage:
  $0 [OPTIONS] [FILTER]

Options:
  --no-color    Disable colored output
  --clean       Remove artifacts before running
  --verbose     Show all test output (not just failures)
  --help        Show this help message

Filters:
  smoke         Run smoke tour tests (e2e_smoke_tour)
  navigation    Run navigation tests (e2e_navigation_tests)
  settings      Run settings tests (e2e_settings)
  files         Run files page tests (e2e_files)
  wizard        Run wizard tests (e2e_wizard)
  <pattern>     Run tests matching custom pattern

Environment Variables:
  DEMO_SEED     Deterministic seed (default: 42)
  NO_COLOR      Set to disable ANSI colors in test output

Examples:
  $0                      # Run all E2E tests
  $0 smoke                # Run smoke tour only
  $0 navigation --clean   # Clean and run navigation tests
  $0 e2e_nav_sidebar      # Run specific test pattern
EOF
}

# Clean artifacts directory
clean_artifacts() {
    if [ -d "$ARTIFACTS_DIR" ]; then
        log_info "Cleaning artifacts directory: $ARTIFACTS_DIR"
        rm -rf "$ARTIFACTS_DIR"
    fi
}

# Print artifact locations on failure
show_artifacts() {
    if [ -d "$ARTIFACTS_DIR" ]; then
        echo ""
        log_header "Test Artifacts"
        log_info "Artifacts saved to: $ARTIFACTS_DIR"
        echo ""

        # Find and list recent failed scenarios
        local failed_dirs=$(find "$ARTIFACTS_DIR" -name "summary.txt" -mmin -5 2>/dev/null)
        if [ -n "$failed_dirs" ]; then
            log_warn "Recent failure summaries:"
            for summary in $failed_dirs; do
                local scenario_dir=$(dirname "$summary")
                local scenario_name=$(basename "$(dirname "$scenario_dir")")
                echo "  $scenario_name: $scenario_dir"
            done
            echo ""
            log_info "View a failure with: cat <scenario_dir>/summary.txt"
        fi
    fi
}

# Run the tests
run_tests() {
    local filter="${1:-}"
    local cargo_args="-p demo_showcase --lib"
    local test_filter=""

    # Map friendly names to test patterns
    case "$filter" in
        smoke)
            test_filter="e2e_smoke_tour"
            ;;
        navigation)
            test_filter="e2e_navigation_tests"
            ;;
        settings)
            test_filter="e2e_settings"
            ;;
        files)
            test_filter="e2e_files"
            ;;
        wizard)
            test_filter="e2e_wizard"
            ;;
        "")
            test_filter="e2e_"  # All E2E tests
            ;;
        *)
            test_filter="$filter"  # Custom pattern
            ;;
    esac

    log_header "Running demo_showcase E2E Tests"

    if [ -n "$test_filter" ]; then
        log_info "Filter: $test_filter"
    fi
    log_info "Seed: ${DEMO_SEED:-42}"
    log_info "Artifacts: $ARTIFACTS_DIR"
    echo ""

    # Set environment for deterministic tests
    export DEMO_SEED="${DEMO_SEED:-42}"

    # Optionally disable colors in test output
    if ! $USE_COLOR; then
        export NO_COLOR=1
    fi

    # Build test arguments
    local test_args=""
    if [ -n "$test_filter" ]; then
        test_args="$test_filter"
    fi

    # Run tests
    local start_time=$(date +%s)

    if $VERBOSE; then
        cargo test $cargo_args $test_args -- --nocapture
    else
        cargo test $cargo_args $test_args
    fi
    local exit_code=$?

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    echo ""
    if [ $exit_code -eq 0 ]; then
        log_success "All tests passed! (${duration}s)"
    else
        log_error "Some tests failed! (${duration}s)"
        show_artifacts
        exit $exit_code
    fi
}

# Main
main() {
    local filter=""
    local clean=false
    VERBOSE=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --no-color)
                USE_COLOR=false
                RED="" GREEN="" YELLOW="" BLUE="" CYAN="" BOLD="" NC=""
                shift
                ;;
            --clean)
                clean=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                filter="$1"
                shift
                ;;
        esac
    done

    # Change to project root
    cd "$PROJECT_ROOT"

    # Clean if requested
    if $clean; then
        clean_artifacts
    fi

    # Run tests
    run_tests "$filter"
}

main "$@"
