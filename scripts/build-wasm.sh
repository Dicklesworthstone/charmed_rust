#!/usr/bin/env bash
# Build WASM packages for charmed-wasm
#
# Usage:
#   ./scripts/build-wasm.sh          # Build all targets
#   ./scripts/build-wasm.sh bundler  # Build only bundler target
#   ./scripts/build-wasm.sh web      # Build only web target
#   ./scripts/build-wasm.sh nodejs   # Build only nodejs target

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRATE_DIR="$PROJECT_ROOT/crates/charmed-wasm"
PKG_DIR="$PROJECT_ROOT/pkg"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if wasm-pack is installed
check_wasm_pack() {
    if ! command -v wasm-pack &> /dev/null; then
        log_error "wasm-pack is not installed"
        log_info "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
        exit 1
    fi
    log_info "Using wasm-pack $(wasm-pack --version)"
}

# Check if wasm32 target is installed
check_wasm_target() {
    if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
        log_warn "wasm32-unknown-unknown target not installed, installing..."
        rustup target add wasm32-unknown-unknown
    fi
}

# Build for bundlers (webpack, vite, parcel, etc.)
build_bundler() {
    log_info "Building for bundlers (webpack, vite, parcel)..."
    wasm-pack build "$CRATE_DIR" \
        --target bundler \
        --out-dir "$PKG_DIR/bundler" \
        --release \
        -- --features console_error_panic_hook
    log_success "Bundler build complete: $PKG_DIR/bundler"
}

# Build for direct web usage (<script type="module">)
build_web() {
    log_info "Building for web (ESM modules)..."
    wasm-pack build "$CRATE_DIR" \
        --target web \
        --out-dir "$PKG_DIR/web" \
        --release \
        -- --features console_error_panic_hook
    log_success "Web build complete: $PKG_DIR/web"
}

# Build for Node.js
build_nodejs() {
    log_info "Building for Node.js..."
    wasm-pack build "$CRATE_DIR" \
        --target nodejs \
        --out-dir "$PKG_DIR/nodejs" \
        --release \
        -- --features console_error_panic_hook
    log_success "Node.js build complete: $PKG_DIR/nodejs"
}

# Build all targets
build_all() {
    build_bundler
    build_web
    build_nodejs
}

# Show package sizes
show_sizes() {
    log_info "Package sizes:"
    echo ""
    for target in bundler web nodejs; do
        if [ -d "$PKG_DIR/$target" ]; then
            wasm_file=$(find "$PKG_DIR/$target" -name "*.wasm" -type f 2>/dev/null | head -1)
            if [ -n "$wasm_file" ]; then
                size=$(du -h "$wasm_file" | cut -f1)
                gzip_size=$(gzip -c "$wasm_file" | wc -c | awk '{printf "%.1fK", $1/1024}')
                echo "  $target: $size (gzipped: $gzip_size)"
            fi
        fi
    done
    echo ""
}

# Main
main() {
    log_info "Building charmed-wasm WASM packages"
    echo ""

    check_wasm_pack
    check_wasm_target

    # Create pkg directory
    mkdir -p "$PKG_DIR"

    # Build based on argument
    case "${1:-all}" in
        bundler)
            build_bundler
            ;;
        web)
            build_web
            ;;
        nodejs)
            build_nodejs
            ;;
        all)
            build_all
            ;;
        *)
            log_error "Unknown target: $1"
            echo "Usage: $0 [bundler|web|nodejs|all]"
            exit 1
            ;;
    esac

    show_sizes
    log_success "Build complete! Packages in $PKG_DIR/"
}

main "$@"
