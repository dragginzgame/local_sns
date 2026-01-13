#!/bin/bash
# Script to mint ICP tokens
#
# Usage:
#   bash scripts/mint_icp.sh [receiver_principal] [amount_e8s]
#
# Arguments (all optional - interactive prompts if not provided):
#   receiver_principal - Optional: Principal to receive the minted ICP
#                        If not provided, prompts interactively
#   amount_e8s        - Optional: Amount to mint in e8s
#                       If not provided, prompts interactively
#
# Interactive flow:
#   1. Enter receiver principal (if not provided)
#   2. Enter amount in e8s (if not provided)
#
# Example:
#   bash scripts/mint_icp.sh
#   bash scripts/mint_icp.sh 2laou-ygqmf-... 100000000
#   bash scripts/mint_icp.sh 2laou-ygqmf-... 100000000000

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

print_header "Mint ICP Tokens"

# Build command arguments - pass through whatever was provided
CMD_ARGS=("mint-icp")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

