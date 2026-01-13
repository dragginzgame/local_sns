#!/bin/bash
# Script to deploy a local SNS using the local_sns Rust binary
#
# Usage:
#   bash scripts/deploy_local_sns.sh
#
# Prerequisites:
#   - dfx start --clean --system-canisters
#   - Rust toolchain installed
#   - Sufficient ICP balance for owner (via minting account)

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

print_header "Local SNS Deployment"

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    print_error "dfx is not running. Start it with: dfx start --clean --system-canisters"
    exit 1
fi

print_success "dfx is running"

# Run the deployment
print_header "Starting SNS Deployment"
print_info "This will create an SNS on your local dfx network..."
echo ""

cargo run --bin local_sns -- deploy-sns

DEPLOYMENT_DATA="$LOCAL_SNS_ROOT/generated/sns_deployment_data.json"

if [ -f "$DEPLOYMENT_DATA" ]; then
    print_header "Deployment Complete"
    print_success "SNS has been deployed successfully!"
    print_info "Deployment data saved to: $DEPLOYMENT_DATA"
    echo ""
    print_info "Deployed SNS Canisters:"
    cat "$DEPLOYMENT_DATA" | grep -A 5 "deployed_sns" | grep -E "(governance|ledger|swap|root|index)" | sed 's/^/  /'
    echo ""
    print_info "Participants:"
    cat "$DEPLOYMENT_DATA" | grep -A 10 "participants" | grep "principal" | head -5 | sed 's/^/  /'
    echo ""
    print_info "To add a hotkey to a participant's neuron, use:"
    echo "  bash scripts/add_sns_hotkey.sh <participant_principal> <hotkey_principal>"
else
    print_error "Deployment data file not found. Deployment may have failed."
    exit 1
fi
