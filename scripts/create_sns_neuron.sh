#!/bin/bash
# Script to create an SNS neuron by staking tokens from the SNS ledger
#
# Usage:
#   bash scripts/create_sns_neuron.sh [principal] [amount_e8s] [memo]
#
# Arguments (all optional - interactive prompts if not provided):
#   principal   - Optional: Principal to create the neuron for
#                If not provided, shows participant selection menu
#   amount_e8s  - Optional: Amount of tokens to stake in e8s
#                If not provided, stakes all available balance
#   memo        - Optional: Memo to use for neuron creation (default: 1)
#
# Interactive flow:
#   1. Select participant/principal (if not provided)
#   2. Enter amount (if not provided, uses all available balance)
#   3. Enter memo (if not provided, uses default: 1)
#
# Example:
#   bash scripts/create_sns_neuron.sh
#   bash scripts/create_sns_neuron.sh 2laou-ygqmf-...
#   bash scripts/create_sns_neuron.sh 2laou-ygqmf-... 10000000000
#   bash scripts/create_sns_neuron.sh 2laou-ygqmf-... 10000000000 1

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

print_header "Create SNS Neuron"

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

# Create neuron - Rust code will handle interactive flow
print_header "Creating SNS Neuron"
echo ""

# Build command arguments - pass through whatever was provided
CMD_ARGS=("create-sns-neuron")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

print_success "SNS neuron created successfully!"

