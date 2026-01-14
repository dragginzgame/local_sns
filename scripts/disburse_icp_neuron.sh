#!/bin/bash
# Script to disburse an ICP neuron to a receiver principal
#
# Usage:
#   bash scripts/disburse_icp_neuron.sh [principal] [neuron_id|receiver_principal] [receiver_principal] [amount_e8s]
#
# Arguments (all optional - interactive prompts if not provided):
#   principal          - Optional: Principal of the owner of the neuron
#                       If not provided, shows participant selection menu
#   neuron_id         - Optional: Neuron ID (number)
#                       If not provided, shows neuron selection menu
#   receiver_principal - Optional: Principal to receive the disbursed tokens
#                       If not provided, prompts interactively
#   amount_e8s        - Optional: Amount to disburse in e8s (if not provided, full disbursement)
#
# Interactive flow:
#   1. Select principal (if not provided)
#   2. Select neuron (if not provided)
#   3. Enter receiver principal (if not provided)
#   4. Enter amount (optional, if not provided full disbursement)
#
# Example:
#   bash scripts/disburse_icp_neuron.sh
#   bash scripts/disburse_icp_neuron.sh 2laou-ygqmf-... receiver-principal-...
#   bash scripts/disburse_icp_neuron.sh 2laou-ygqmf-... 12345 receiver-principal-... 100000000

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

print_header "Disburse ICP Neuron"

# Build command arguments - pass through whatever was provided
CMD_ARGS=("disburse-icp-neuron")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"
