#!/bin/bash
# Main interactive menu script for Local SNS management
#
# Usage:
#   bash scripts/start.sh
#
# This script provides an interactive menu to select and run various SNS management operations.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

# Check if SNS is deployed via network call
check_sns_deployed() {
    # Use Rust binary to check via network (suppress all output)
    # Exit code 0 = deployed, 1 = not deployed
    if cargo run --quiet --bin local_sns -- check-sns-deployed >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

# Check if a menu option requires SNS to be deployed
requires_sns_deployment() {
    local choice="$1"
    # Options 9 (Redeploy SNS) and b (Rebuild Binary) don't require SNS to be deployed
    case "$choice" in
        9|b|B)
            return 1  # Doesn't require SNS
            ;;
        *)
            return 0  # Requires SNS
            ;;
    esac
}

# Show menu
show_menu() {
    clear
    print_header "Local SNS Management Menu"
    echo ""
    echo -e "${CYAN}Available Operations:${NC}"
    echo ""
    echo -e "  ${GREEN}1)${NC} Add SNS Neuron Hotkey"
    echo -e "     Add a hotkey to an SNS participant neuron (interactive)"
    echo ""
    echo -e "  ${GREEN}2)${NC} Add ICP Neuron Hotkey"
    echo -e "     Add a hotkey to the ICP neuron used for SNS deployment"
    echo ""
    echo -e "  ${GREEN}3)${NC} List SNS Neurons"
    echo -e "     Query and display SNS neurons for a principal (interactive)"
    echo ""
    echo -e "  ${GREEN}4)${NC} Get ICP Neuron Info"
    echo -e "     Get detailed information about the ICP neuron"
    echo ""
    echo -e "  ${GREEN}5)${NC} Set ICP Neuron Visibility"
    echo -e "     Set the ICP neuron to public or private"
    echo ""
    echo -e "  ${GREEN}6)${NC} Create SNS Neuron"
    echo -e "     Create an SNS neuron by staking tokens from ledger balance"
    echo ""
    echo -e "  ${GREEN}7)${NC} Disburse SNS Neuron"
    echo -e "     Disburse tokens from an SNS neuron"
    echo ""
    echo -e "  ${GREEN}8)${NC} Mint SNS Tokens"
    echo -e "     Mint additional tokens to an account"
    echo ""
    echo -e "  ${GREEN}9)${NC} Deploy New SNS"
    echo -e "     Create a new SNS instance (creates a separate SNS, does not replace existing)"
    echo ""
    echo -e "  ${GREEN}b)${NC} Rebuild Binary"
    echo -e "     Rebuild the Rust binary (useful after code changes)"
    echo ""
    echo -e "  ${GREEN}0)${NC} Exit"
    echo ""
    echo -n -e "${CYAN}Select an option [0-9, b]: ${NC}"
}

# Run selected script
run_script() {
    local choice="$1"
    local script_name=""
    local script_args=()
    
    # Check if this option requires SNS deployment
    if requires_sns_deployment "$choice"; then
        if ! check_sns_deployed; then
            print_error "No SNS deployment found on the network!"
            echo ""
            print_info "Please deploy an SNS first using option 9 (Deploy New SNS)."
            print_info "Configuration can be modified in src/init/sns_config.rs"
            echo ""
            return 1
        fi
    fi
    
    case "$choice" in
        1)
            script_name="add_sns_hotkey.sh"
            # Pass through any additional arguments
            shift
            script_args=("$@")
            ;;
        2)
            script_name="add_icp_hotkey.sh"
            shift
            script_args=("$@")
            ;;
        3)
            script_name="get_sns_neurons.sh"
            shift
            script_args=("$@")
            ;;
        4)
            script_name="get_icp_neuron.sh"
            shift
            script_args=("$@")
            ;;
        5)
            script_name="set_icp_visibility.sh"
            shift
            script_args=("$@")
            ;;
        6)
            script_name="create_sns_neuron.sh"
            shift
            script_args=("$@")
            ;;
        7)
            script_name="disburse_sns_neuron.sh"
            shift
            script_args=("$@")
            ;;
        8)
            script_name="mint_sns_tokens.sh"
            shift
            script_args=("$@")
            ;;
        9)
            script_name="deploy_local_sns.sh"
            shift
            script_args=("$@")
            ;;
        b|B)
            print_header "Rebuilding Binary"
            bash "$SCRIPT_DIR/build.sh"
            print_success "Binary rebuilt successfully!"
            return 0
            ;;
        0)
            print_info "Exiting..."
            exit 0
            ;;
        *)
            print_error "Invalid option: $choice"
            return 1
            ;;
    esac
    
    local script_path="$SCRIPT_DIR/$script_name"
    
    if [ ! -f "$script_path" ]; then
        print_error "Script not found: $script_name"
        return 1
    fi
    
    # Make sure script is executable
    chmod +x "$script_path"
    
    # Run the script with any passed arguments
    print_header "Running: $script_name"
    bash "$script_path" "${script_args[@]}"
    
    return $?
}

# Main loop
main() {
    # Check if binary exists, build if it doesn't (for fresh clones)
    local binary_path="target/release/local_sns"
    if [ ! -f "$LOCAL_SNS_ROOT/$binary_path" ]; then
        # Try debug build if release doesn't exist
        binary_path="target/debug/local_sns"
        if [ ! -f "$LOCAL_SNS_ROOT/$binary_path" ]; then
            print_info "Binary not found. Building for the first time..."
            bash "$SCRIPT_DIR/build.sh"
            echo ""
        fi
    fi
    
    # Check if dfx is running (unless we're being called from another script)
    if [ "${CHECK_DFX:-true}" = "true" ]; then
        if ! dfx ping >/dev/null 2>&1; then
            print_error "dfx is not running. Start it with: dfx start --clean --system-canisters"
            exit 1
        fi
    fi
    
    # If arguments are provided, run script directly (non-interactive mode)
    if [ $# -gt 0 ]; then
        run_script "$@"
        exit $?
    fi
    
    # Check SNS deployment status and show message if not deployed
    if ! check_sns_deployed; then
        echo ""
        print_info "Press Enter to deploy an SNS."
        print_info "Configuration can be modified in src/init/sns_config.rs"
        echo ""
        read -r
        # Automatically deploy SNS (option 9)
        run_script 9
        echo ""
        echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
        read -r
    fi
    
    # Interactive menu loop
    while true; do
        show_menu
        read -r choice
        
        case "$choice" in
            0)
                print_info "Exiting..."
                exit 0
                ;;
            [1-9])
                run_script "$choice"
                echo ""
                echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                read -r
                ;;
            b|B)
                run_script "$choice"
                echo ""
                echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                read -r
                ;;
            *)
                print_error "Invalid option. Please select 0-9 or b."
                sleep 1
                ;;
        esac
    done
}

# Run main function
main "$@"

