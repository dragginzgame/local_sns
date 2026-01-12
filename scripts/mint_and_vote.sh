#!/bin/bash
# Script to mint SNS tokens and have all participants vote yes
#
# Usage:
#   bash scripts/mint_and_vote.sh <to_principal> <amount_e8s>
#
# Arguments:
#   to_principal - Principal to mint tokens to
#   amount_e8s   - Amount of tokens to mint in e8s (1 token = 100_000_000 e8s)
#
# Example:
#   bash scripts/mint_and_vote.sh 2laou-ygqmf-... 1000000000000
#   (This mints 10,000 tokens to the specified principal)

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

print_header "Mint SNS Tokens and Vote"

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <to_principal> <amount_e8s>"
    echo ""
    echo "Arguments:"
    echo "  to_principal - Principal to mint tokens to"
    echo "  amount_e8s   - Amount of tokens to mint in e8s (1 token = 100_000_000 e8s)"
    echo ""
    echo "Example:"
    echo "  $0 2laou-ygqmf-... 1000000000000"
    echo "  (This mints 10,000 tokens to the specified principal)"
    exit 1
fi

TO_PRINCIPAL="$1"
AMOUNT_E8S="$2"

# Validate amount is a number
if ! [[ "$AMOUNT_E8S" =~ ^[0-9]+$ ]]; then
    print_error "amount_e8s must be a number"
    exit 1
fi

# Calculate token amount for display (1 token = 100,000,000 e8s)
TOKENS=$((AMOUNT_E8S / 100000000))
print_info "Amount: $AMOUNT_E8S e8s (${TOKENS} tokens)"

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

# Mint tokens and vote
print_header "Creating Mint Proposal and Voting"
print_info "To principal: $TO_PRINCIPAL"
print_info "Amount: $AMOUNT_E8S e8s (${TOKENS} tokens)"
echo ""

cargo run --bin local_sns -- mint-and-vote "$TO_PRINCIPAL" "$AMOUNT_E8S"

print_success "Mint proposal created and all participants voted successfully!"

