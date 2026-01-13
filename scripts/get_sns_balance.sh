#!/bin/bash
# Script to get SNS ledger balance for an account
#
# Usage:
#   bash scripts/get_sns_balance.sh [principal] [subaccount_hex]
#
# Arguments (all optional - interactive prompts if not provided):
#   principal       - Optional: Principal to query balance for
#                     If not provided, shows participant selection menu or uses dfx identity
#   subaccount_hex  - Optional: Subaccount in hex format
#                     If not provided, queries default account (no subaccount)
#
# Interactive flow:
#   1. Select participant or enter principal (if not provided)
#   2. Enter subaccount (if not provided, uses default account)
#
# Example:
#   bash scripts/get_sns_balance.sh
#   bash scripts/get_sns_balance.sh 2laou-ygqmf-...
#   bash scripts/get_sns_balance.sh 2laou-ygqmf-... 0xabcd1234...

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

# Check if deployment data exists (needed to get ledger canister ID)
DEPLOYMENT_DATA="$LOCAL_SNS_ROOT/generated/sns_deployment_data.json"
if [ ! -f "$DEPLOYMENT_DATA" ]; then
    print_error "Deployment data file not found: $DEPLOYMENT_DATA"
    print_info "Please deploy an SNS first (option 12 in menu, or run deploy_local_sns.sh)"
    exit 1
fi

print_header "Get SNS Balance"

# Build command arguments - pass through whatever was provided
CMD_ARGS=("get-sns-balance")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

