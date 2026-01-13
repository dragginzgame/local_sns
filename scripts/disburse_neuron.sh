#!/bin/bash
# Script to disburse the disburseable SNS neuron to a receiver principal
#
# Usage:
#   bash scripts/disburse_neuron.sh [participant_principal] [neuron_id_hex|receiver_principal] [receiver_principal]
#
# Arguments (all optional - interactive prompts if not provided):
#   participant_principal - Optional: Principal of the participant who owns the neuron
#                          If not provided, shows participant selection menu
#   neuron_id_hex        - Optional: Neuron ID in hex format
#                          If not provided, shows neuron selection menu
#   receiver_principal   - Optional: Principal to receive the disbursed tokens
#                          If not provided, prompts interactively
#
# Interactive flow:
#   1. Select participant (if not provided)
#   2. Select neuron (if not provided)
#   3. Enter receiver principal (if not provided)
#
# Example:
#   bash scripts/disburse_neuron.sh
#   bash scripts/disburse_neuron.sh 2laou-ygqmf-... receiver-principal-...
#   bash scripts/disburse_neuron.sh 2laou-ygqmf-... 0xabcd1234... receiver-principal-...

# to check the balance of a principal, use the following command:
# dfx canister call LEDGER_CANISTER_ID icrc1_balance_of '(record {owner=principal "USER_PRINCIPAL"; subaccount=null})'

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

print_header "Disburse SNS Neuron"

# All arguments are optional - Rust code handles interactive flow

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    print_error "dfx is not running. Start it with: dfx start --clean --system-canisters"
    exit 1
fi

# Check if deployment data exists
DEPLOYMENT_DATA="generated/sns_deployment_data.json"
if [ ! -f "$DEPLOYMENT_DATA" ]; then
    print_error "Deployment data file not found: $DEPLOYMENT_DATA"
    print_info "Please run deploy_local_sns.sh first to create an SNS"
    exit 1
fi

# Disburse neuron - Rust code will handle interactive flow
print_header "Disbursing Neuron"
echo ""

# Build command arguments - pass through whatever was provided
CMD_ARGS=("disburse-sns-neuron")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

print_success "Neuron disbursed successfully!"

