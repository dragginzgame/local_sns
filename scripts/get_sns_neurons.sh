#!/bin/bash
# Script to fetch SNS neurons for a given principal
# This is a wrapper around the Rust binary's list-sns-neurons command
#
# Usage:
#   bash scripts/get_sns_neurons.sh [principal]
#
# Arguments:
#   principal - Optional: Principal to query neurons for
#              If not provided, shows participant selection menu
#
# Example:
#   bash scripts/get_sns_neurons.sh
#   bash scripts/get_sns_neurons.sh qc2qr-5u5mz-3ny2c-rzvkj-3z2lh-4uawd-5ggw7-pfwno-ghsmf-gqfau-oqe

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Check arguments - principal is now optional
if [ $# -ge 1 ]; then
    PRINCIPAL="$1"
else
    PRINCIPAL=""
fi

# Get script directory (should be in local_sns/scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_SNS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to local_sns root directory
cd "$LOCAL_SNS_ROOT"

print_header "Fetching SNS Neurons"

if [ -n "$PRINCIPAL" ]; then
    print_info "Principal: $PRINCIPAL"
else
    print_info "No principal specified - will show participant selection"
fi

DEPLOYMENT_DATA="generated/sns_deployment_data.json"

# Check if deployment data exists
if [ ! -f "$DEPLOYMENT_DATA" ]; then
    print_error "Deployment data not found at: $DEPLOYMENT_DATA"
    print_info "Please run deploy_local_sns.sh first to create an SNS"
    exit 1
fi

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    print_error "dfx is not running. Start it with: dfx start --clean --system-canisters"
    exit 1
fi

# Check if Rust toolchain is available
if ! command -v cargo &> /dev/null; then
    print_error "cargo is not installed. Please install Rust toolchain."
    exit 1
fi

# Build the binary if needed
print_info "Building local_sns binary..."
if cargo build --bin local_sns --release 2>/dev/null; then
    print_success "Binary built successfully"
else
    print_warning "Release build failed, trying dev build..."
    cargo build --bin local_sns
fi

# Use the Rust binary's list-sns-neurons command
print_header "Querying SNS Governance via Rust Binary"

if [ -n "$PRINCIPAL" ]; then
    cargo run --bin local_sns -- list-sns-neurons "$PRINCIPAL"
else
    cargo run --bin local_sns -- list-sns-neurons
fi
