#!/bin/bash
# Script to mint SNS tokens by creating a proposal and getting all neurons to vote
#
# Usage:
#   bash scripts/mint_sns_tokens.sh [proposer_principal] [receiver_principal] [amount_e8s]
#
# Arguments (all optional - interactive prompts if not provided):
#   proposer_principal - Optional: Principal of the participant who will create the proposal
#                        If not provided, shows participant selection menu
#   receiver_principal - Optional: Principal to receive the minted tokens
#                        If not provided, prompts interactively
#   amount_e8s        - Optional: Amount of tokens to mint (in e8s, e.g., 100000000 = 1 token)
#                        If not provided, prompts interactively
#
# Interactive flow:
#   1. Select proposer participant (if not provided)
#   2. Enter receiver principal (if not provided)
#   3. Enter amount to mint (if not provided)
#
# Example:
#   bash scripts/mint_sns_tokens.sh
#   bash scripts/mint_sns_tokens.sh 2laou-ygqmf-... receiver-principal-... 100000000000

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

print_header "Mint SNS Tokens"

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

# Verify deployment data is valid JSON (basic check)
if ! python3 -m json.tool "$DEPLOYMENT_DATA" >/dev/null 2>&1; then
    print_warning "Deployment data file may be invalid JSON"
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

# Mint tokens - Rust code will handle interactive flow
print_header "Creating Proposal and Voting"
echo ""

# Build command arguments - pass through whatever was provided
CMD_ARGS=("mint-sns-tokens")
# Pass all provided arguments through - Rust code will handle interactive prompts for missing ones
for arg in "$@"; do
    CMD_ARGS+=("$arg")
done

cargo run --bin local_sns -- "${CMD_ARGS[@]}"

print_success "Tokens minting proposal created and voted on successfully!"

