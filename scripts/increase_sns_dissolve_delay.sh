#!/bin/bash
# Script to increase dissolve delay for an SNS neuron
#
# Usage:
#   bash scripts/increase_sns_dissolve_delay.sh [participant_principal] [neuron_id_hex] [additional_dissolve_delay_seconds]
#
# Arguments (all optional - interactive prompts if not provided):
#   participant_principal           - Optional: Principal of the participant who owns the neuron
#                                    If not provided, shows participant selection menu
#   neuron_id_hex                  - Optional: Neuron ID in hex format
#                                    If not provided, auto-selects neuron with longest dissolve delay
#   additional_dissolve_delay_seconds - Optional: Additional dissolve delay in seconds
#                                      If not provided, prompts interactively
#
# Interactive flow:
#   1. Select participant (if not provided)
#   2. Select neuron (if not provided)
#   3. Enter additional dissolve delay in seconds (if not provided)
#
# Example:
#   bash scripts/increase_sns_dissolve_delay.sh
#   bash scripts/increase_sns_dissolve_delay.sh 2laou-ygqmf-... 2592000
#   bash scripts/increase_sns_dissolve_delay.sh 2laou-ygqmf-... 0xabcd1234... 2592000

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

# Check if deployment data exists
DEPLOYMENT_DATA="$LOCAL_SNS_ROOT/generated/sns_deployment_data.json"
if [ ! -f "$DEPLOYMENT_DATA" ]; then
    print_error "Deployment data file not found: $DEPLOYMENT_DATA"
    print_info "Please deploy an SNS first (option 9 in menu, or run deploy_local_sns.sh)"
    exit 1
fi

print_header "Increase SNS Neuron Dissolve Delay"

# Build command arguments - pass through whatever was provided
CMD_ARGS=("increase-sns-dissolve-delay")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

