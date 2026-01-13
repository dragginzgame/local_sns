#!/bin/bash
# Script to create an ICP neuron
#
# Usage:
#   bash scripts/create_icp_neuron.sh [principal] [amount_e8s] [memo]
#
# Arguments (all optional - interactive prompts if not provided):
#   principal  - Optional: Principal to create the neuron for (defaults to dfx identity principal)
#               If not provided, uses default dfx identity
#   amount_e8s - Optional: Amount of ICP to stake in e8s
#               If not provided, prompts interactively
#   memo       - Optional: Memo to use for neuron creation (defaults to 1)
#               If not provided, prompts for memo or uses default 1
#
# Interactive flow:
#   1. Enter amount in e8s (if not provided)
#   2. Enter memo (if not provided, defaults to 1)
#
# Example:
#   bash scripts/create_icp_neuron.sh
#   bash scripts/create_icp_neuron.sh 100000000
#   bash scripts/create_icp_neuron.sh 2laou-ygqmf-... 100000000 1

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

print_header "Create ICP Neuron"

# Build command arguments - pass through whatever was provided
CMD_ARGS=("create-icp-neuron")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

