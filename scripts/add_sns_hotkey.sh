#!/bin/bash
# Script to add a hotkey to an SNS neuron
#
# Usage:
#   bash scripts/add_sns_hotkey.sh <participant_principal> <hotkey_principal> [permissions]
#
# Arguments:
#   participant_principal - Principal of the participant who owns the neuron
#   hotkey_principal      - Principal to add as a hotkey
#   permissions          - Optional: comma-separated permission types (default: 3,4)
#                         Permission types: 1=ManagePrincipals, 2=SubmitProposal, 3=Vote, 4=ManageNeuron
#
# Example:
#   bash scripts/add_sns_hotkey.sh 2laou-ygqmf-... your-hotkey-principal
#   bash scripts/add_sns_hotkey.sh 2laou-ygqmf-... your-hotkey-principal 2,3,4

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

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <participant_principal> <hotkey_principal> [permissions]"
    echo ""
    echo "Arguments:"
    echo "  participant_principal - Principal of the participant who owns the neuron"
    echo "  hotkey_principal      - Principal to add as a hotkey"
    echo "  permissions          - Optional: comma-separated permission types (default: 3,4)"
    echo ""
    echo "Permission types:"
    echo "  1 = ManagePrincipals"
    echo "  2 = SubmitProposal"
    echo "  3 = Vote"
    echo "  4 = ManageNeuron"
    exit 1
fi

PARTICIPANT_PRINCIPAL="$1"
HOTKEY_PRINCIPAL="$2"
PERMISSIONS="${3:-3,4}"

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

# Verify participant principal is in deployment data
if ! grep -q "$PARTICIPANT_PRINCIPAL" "$DEPLOYMENT_DATA"; then
    print_warning "Participant principal not found in deployment data"
    print_info "Available participants:"
    grep -A 2 '"participants"' "$DEPLOYMENT_DATA" | grep '"principal"' | head -5 | sed 's/^/  /'
fi

# Check if Rust toolchain is available
if ! command -v cargo &> /dev/null; then
    print_error "cargo is not installed. Please install Rust toolchain."
    exit 1
fi

# Build the binary if needed
print_info "Building local_sns binary..."
if cargo build --bin local_sns --release 2>/dev/null; then
    print_success "Binary built successfully"
else
    print_warning "Release build failed, trying dev build..."
    cargo build --bin local_sns
fi

# Add hotkey
print_header "Adding Hotkey"
print_info "Participant: $PARTICIPANT_PRINCIPAL"
print_info "Hotkey: $HOTKEY_PRINCIPAL"
print_info "Permissions: $PERMISSIONS"
echo ""

if [ "$PERMISSIONS" != "" ]; then
    cargo run --bin local_sns -- add-hotkey sns "$PARTICIPANT_PRINCIPAL" "$HOTKEY_PRINCIPAL" "$PERMISSIONS"
else
    cargo run --bin local_sns -- add-hotkey sns "$PARTICIPANT_PRINCIPAL" "$HOTKEY_PRINCIPAL"
fi

print_success "Hotkey added successfully!"
