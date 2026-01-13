#!/bin/bash
# Script to add a hotkey to an SNS neuron
#
# Usage:
#   bash scripts/add_sns_hotkey.sh [participant_principal] [neuron_id_hex|hotkey_principal] [hotkey_principal|permissions] [permissions]
#
# Arguments (all optional - interactive prompts if not provided):
#   participant_principal - Optional: Principal of the participant who owns the neuron
#                          If not provided, shows participant selection menu
#   neuron_id_hex        - Optional: Neuron ID in hex format
#                          If not provided, shows neuron selection menu
#   hotkey_principal     - Optional: Principal to add as a hotkey
#                          If not provided, prompts interactively
#   permissions          - Optional: comma-separated permission types (default: 3,4)
#                          Permission types: 2=ManagePrincipals, 3=SubmitProposal, 4=Vote
#
# Interactive flow:
#   1. Select participant (if not provided)
#   2. Select neuron (if not provided)
#   3. Enter hotkey principal (if not provided)
#
# Example:
#   bash scripts/add_sns_hotkey.sh
#   bash scripts/add_sns_hotkey.sh 2laou-ygqmf-... your-hotkey-principal
#   bash scripts/add_sns_hotkey.sh 2laou-ygqmf-... your-hotkey-principal 2,3,4
#   bash scripts/add_sns_hotkey.sh 2laou-ygqmf-... 0xabcd1234... your-hotkey-principal 2,3,4

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

print_header "Add SNS Neuron Hotkey"

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

# Add hotkey - Rust code will handle interactive flow
print_header "Adding Hotkey"
echo ""

# Build command arguments - pass through whatever was provided
CMD_ARGS=("add-hotkey" "sns")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

print_success "Hotkey added successfully!"
