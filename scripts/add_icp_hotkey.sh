#!/bin/bash
# Script to add a hotkey to an ICP neuron
# This is a wrapper around the Rust binary's add-hotkey command for ICP neurons
#
# Usage:
#   bash scripts/add_icp_hotkey.sh [hotkey_principal]
#
# Arguments (all optional - interactive prompts if not provided):
#   hotkey_principal - Optional: Principal to add as a hotkey to the ICP neuron
#                     If not provided, prompts interactively
#
# Interactive flow:
#   1. Enter hotkey principal (if not provided)
#
# Note: Uses the ICP neuron from the SNS deployment data (the neuron that created the SNS proposal)
#
# Example:
#   bash scripts/add_icp_hotkey.sh
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

# All arguments are optional - Rust code handles interactive flow

# Get script directory (should be in local_sns/scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_SNS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to local_sns root directory
cd "$LOCAL_SNS_ROOT"

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    print_error "dfx is not running. Start it with: dfx start --clean --system-canisters"
    exit 1
fi

# Check if deployment data exists
DEPLOYMENT_DATA="$LOCAL_SNS_ROOT/generated/sns_deployment_data.json"
if [ ! -f "$DEPLOYMENT_DATA" ]; then
    print_error "Deployment data not found at: $DEPLOYMENT_DATA"
    print_info "Please deploy an SNS first (option 9 in menu, or run deploy_local_sns.sh)"
    exit 1
fi

print_header "Add Hotkey to ICP Neuron"

# Build command arguments - pass through whatever was provided
CMD_ARGS=("add-hotkey" "icp")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"
