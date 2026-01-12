#!/bin/bash
# Script to fetch ICP neuron information
# This queries the ICP Governance canister for neuron details
#
# Usage:
#   bash scripts/get_icp_neuron.sh [neuron_id]
#
# Arguments:
#   neuron_id - Optional: Specific neuron ID to query. If not provided, uses neuron from deployment data
#
# Example:
#   bash scripts/get_icp_neuron.sh
#   bash scripts/get_icp_neuron.sh 1281960829742175837

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

# Standard NNS canister IDs for local development
GOVERNANCE_CANISTER="rrkah-fqaaa-aaaaa-aaaaq-cai"

# Get script directory (should be in local_sns/scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_SNS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to local_sns root directory
cd "$LOCAL_SNS_ROOT"

print_header "Fetching ICP Neuron Information"

# Determine neuron ID
if [ $# -ge 1 ]; then
    NEURON_ID="$1"
    print_info "Using provided neuron ID: $NEURON_ID"
else
    # Try to get from deployment data
    DEPLOYMENT_DATA="generated/sns_deployment_data.json"
    if [ -f "$DEPLOYMENT_DATA" ]; then
        if command -v jq &> /dev/null; then
            NEURON_ID=$(jq -r '.icp_neuron_id // empty' "$DEPLOYMENT_DATA" 2>/dev/null)
        else
            # Fallback: try to extract with grep/sed
            NEURON_ID=$(cat "$DEPLOYMENT_DATA" | grep -o '"icp_neuron_id"[[:space:]]*:[[:space:]]*[0-9]*' | sed 's/.*"icp_neuron_id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/' | head -1)
        fi

        if [ -z "$NEURON_ID" ] || [ "$NEURON_ID" = "null" ]; then
            print_error "No neuron ID provided and deployment data not found or missing icp_neuron_id"
            print_info "Usage: $0 [neuron_id]"
            exit 1
        fi
        print_info "Using neuron ID from deployment data: $NEURON_ID"
    else
        print_error "No neuron ID provided and deployment data not found"
        print_info "Usage: $0 [neuron_id]"
        exit 1
    fi
fi

print_info "ICP Governance canister: $GOVERNANCE_CANISTER"

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

print_header "Querying ICP Neuron"

# Use the Rust binary to get neuron information
if [ -n "$NEURON_ID" ]; then
    cargo run --bin local_sns -- get-icp-neuron "$NEURON_ID"
else
    cargo run --bin local_sns -- get-icp-neuron
fi

if [ $? -eq 0 ]; then
    print_success "Query complete"
else
    print_error "Failed to query neuron"
    exit 1
fi
