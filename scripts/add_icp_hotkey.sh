#!/bin/bash
# Script to add a hotkey to an ICP neuron
# This is a wrapper around the Rust binary's add-hotkey command for ICP neurons
#
# Usage:
#   bash scripts/add_icp_hotkey.sh <hotkey_principal>
#
# Arguments:
#   hotkey_principal - Principal to add as a hotkey to the ICP neuron
#
# Note: Uses the ICP neuron from the SNS deployment data (the neuron that created the SNS proposal)
#
# Example:
#   bash scripts/add_icp_hotkey.sh your-hotkey-principal

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

# Check arguments
if [ $# -lt 1 ]; then
    echo ""
    echo "Usage: $0 <hotkey_principal>"
    echo ""
    echo "Arguments:"
    echo "  hotkey_principal - Principal to add as a hotkey to the ICP neuron"
    echo ""
    echo "Note: Uses the ICP neuron from the SNS deployment data"
    echo "      (the neuron that was used to create the SNS proposal)"
    echo ""
    echo "Example:"
    echo "  $0 your-hotkey-principal"
    exit 1
fi

HOTKEY_PRINCIPAL="$1"

# Get script directory (should be in local_sns/scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_SNS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to local_sns root directory
cd "$LOCAL_SNS_ROOT"

print_header "Add Hotkey to ICP Neuron"

print_info "Hotkey: $HOTKEY_PRINCIPAL"

DEPLOYMENT_DATA="generated/sns_deployment_data.json"

# Check if deployment data exists
if [ ! -f "$DEPLOYMENT_DATA" ]; then
    print_error "Deployment data not found at: $DEPLOYMENT_DATA"
    print_info "Please run deploy_local_sns.sh first to create an SNS"
    exit 1
fi

# Extract ICP neuron ID from deployment data (optional, just for info)
if command -v jq &> /dev/null; then
    ICP_NEURON_ID=$(jq -r '.icp_neuron_id // empty' "$DEPLOYMENT_DATA" 2>/dev/null)
    if [ -n "$ICP_NEURON_ID" ] && [ "$ICP_NEURON_ID" != "null" ]; then
        print_info "ICP Neuron ID (from deployment data): $ICP_NEURON_ID"
    fi
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

# Use the Rust binary's add-hotkey command for ICP
print_header "Adding Hotkey via Rust Binary"

print_info "Note: ICP neurons don't use permission types - hotkeys have full control like the owner"
print_info "Note: The owner_principal argument is ignored for ICP neurons (uses default dfx identity)"

cargo run --bin local_sns -- add-hotkey icp "$HOTKEY_PRINCIPAL"

if [ $? -eq 0 ]; then
    print_success "Hotkey added successfully!"
else
    print_error "Failed to add hotkey"
    exit 1
fi
