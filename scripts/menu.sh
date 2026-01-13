#!/bin/bash
# Main interactive menu script for Local SNS management
#
# Usage:
#   bash scripts/menu.sh
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

# Show menu
show_menu() {
    clear
    print_header "Local SNS Management Menu"
    echo ""
    echo -e "${CYAN}Available Operations:${NC}"
    echo ""
    echo -e "  ${GREEN}1)${NC} Deploy Local SNS"
    echo -e "     Create a new SNS on your local dfx network based on init/sns_config.rs"
    echo ""
    echo -e "  ${GREEN}2)${NC} Add SNS Neuron Hotkey"
    echo -e "     Add a hotkey to an SNS participant neuron (interactive)"
    echo ""
    echo -e "  ${GREEN}3)${NC} Add ICP Neuron Hotkey"
    echo -e "     Add a hotkey to the ICP neuron used for SNS deployment"
    echo ""
    echo -e "  ${GREEN}4)${NC} List SNS Neurons"
    echo -e "     Query and display SNS neurons for a principal (interactive)"
    echo ""
    echo -e "  ${GREEN}5)${NC} Get ICP Neuron Info"
    echo -e "     Get detailed information about the ICP neuron"
    echo ""
    echo -e "  ${GREEN}6)${NC} Set ICP Neuron Visibility"
    echo -e "     Set the ICP neuron to public or private"
    echo ""
    echo -e "  ${GREEN}7)${NC} Disburse Neuron"
    echo -e "     Disburse tokens from an SNS neuron"
    echo ""
    echo -e "  ${GREEN}8)${NC} Mint SNS Tokens"
    echo -e "     Mint additional tokens to an account"
    echo ""
    echo -e "  ${GREEN}0)${NC} Exit"
    echo ""
    echo -n -e "${CYAN}Select an option [0-8]: ${NC}"
}

# Run selected script
run_script() {
    local choice="$1"
    local script_name=""
    local script_args=()
    
    case "$choice" in
        1)
            script_name="deploy_local_sns.sh"
            ;;
        2)
            script_name="add_sns_hotkey.sh"
            # Pass through any additional arguments
            shift
            script_args=("$@")
            ;;
        3)
            script_name="add_icp_hotkey.sh"
            shift
            script_args=("$@")
            ;;
        4)
            script_name="get_sns_neurons.sh"
            shift
            script_args=("$@")
            ;;
        5)
            script_name="get_icp_neuron.sh"
            shift
            script_args=("$@")
            ;;
        6)
            script_name="set_icp_visibility.sh"
            shift
            script_args=("$@")
            ;;
        7)
            script_name="disburse_neuron.sh"
            shift
            script_args=("$@")
            ;;
        8)
            script_name="mint_sns_tokens.sh"
            shift
            script_args=("$@")
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
    
    # Interactive menu loop
    while true; do
        show_menu
        read -r choice
        
        case "$choice" in
            0)
                print_info "Exiting..."
                exit 0
                ;;
            [1-8])
                run_script "$choice"
                echo ""
                echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                read -r
                ;;
            *)
                print_error "Invalid option. Please select 0-8."
                sleep 1
                ;;
        esac
    done
}

# Run main function
main "$@"

