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
    local category="$1"
    local operation="$2"
    
    # Utils operations never require SNS
    if [ "$category" = "utils" ]; then
        return 1  # Doesn't require SNS
    fi
    
    # ICP operations don't require SNS deployment
    if [ "$category" = "icp" ]; then
        return 1  # Doesn't require SNS
    fi
    
    # SNS operations require SNS deployment
    # Exception: "Get SNS Balance" (operation 8) requires SNS, but it's already in SNS category
    if [ "$category" = "sns" ]; then
        return 0  # Requires SNS
    fi
    
    return 1
}

# Show main menu
show_main_menu() {
    clear
    print_header "Local SNS Management Menu"
    echo ""
    echo -e "${CYAN}Select Category:${NC}"
    echo ""
    echo -e "  ${GREEN}1${NC} / [${GREEN}I${NC}]  ICP Operations"
    echo -e "     Manage ICP neurons, balances, and tokens"
    echo ""
    echo -e "  ${GREEN}2${NC} / [${GREEN}S${NC}]  SNS Operations"
    echo -e "     Manage SNS neurons, tokens, and governance"
    echo ""
    echo -e "  ${GREEN}3${NC} / [${GREEN}U${NC}]  Utils"
    echo -e "     Deploy SNS and rebuild binary"
    echo ""
    echo -e "  ${GREEN}0${NC}         Exit"
    echo ""
    echo -n -e "${CYAN}Select category [0-3, I, S, U]: ${NC}"
}

# Show ICP submenu (when SNS is deployed)
show_icp_menu() {
    clear
    print_header "ICP Operations"
    echo ""
    echo -e "${CYAN}Available Operations:${NC}"
    echo ""
    echo -e "  ${GREEN}1${NC} / [${GREEN}L${NC}]  List ICP Neurons"
    echo -e "     List all ICP neurons for a principal (includes detailed view)"
    echo ""
    echo -e "  ${GREEN}2${NC} / [${GREEN}C${NC}]  Create ICP Neuron"
    echo -e "     Create an ICP neuron by staking ICP tokens"
    echo ""
    echo -e "  ${GREEN}3${NC} / [${GREEN}D${NC}]  Disburse ICP Neuron"
    echo -e "     Disburse tokens from an ICP neuron"
    echo ""
    echo -e "  ${GREEN}4${NC} / [${GREEN}M${NC}]  Mint ICP Tokens"
    echo -e "     Mint ICP tokens from minting account to a receiver"
    echo ""
    echo -e "  ${GREEN}5${NC} / [${GREEN}H${NC}]  Add ICP Neuron Hotkey"
    echo -e "     Add a hotkey to an ICP neuron"
    echo ""
    echo -e "  ${GREEN}6${NC} / [${GREEN}I${NC}]  Increase ICP Neuron Dissolve Delay"
    echo -e "     Add dissolve delay to an ICP neuron"
    echo ""
    echo -e "  ${GREEN}7${NC} / [${GREEN}DD${NC}] Dissolve ICP Neuron"
    echo -e "     Start or stop dissolving for an ICP neuron"
    echo ""
    echo -e "  ${GREEN}8${NC} / [${GREEN}B${NC}]  Get ICP Balance"
    echo -e "     Get ICP ledger balance for an account"
    echo ""
    echo -e "  ${GREEN}0${NC} / ${CYAN}Enter${NC}  Back to Main Menu"
    echo ""
    echo -n -e "${CYAN}Select operation [0-8, L, C, D, M, H, I, DD, B, or Enter]: ${NC}"
}

# Show ICP submenu (when SNS is NOT deployed)
show_icp_menu_no_sns() {
    clear
    print_header "ICP Operations (No SNS Deployed)"
    echo ""
    echo -e "${YELLOW}No SNS deployment found on the network.${NC}"
    echo ""
    echo -e "${CYAN}Available Operations:${NC}"
    echo ""
    echo -e "  ${GREEN}1${NC} / [${GREEN}M${NC}]  Mint ICP Tokens"
    echo -e "     Mint ICP tokens from minting account to a receiver"
    echo ""
    echo -e "  ${GREEN}2${NC} / [${GREEN}B${NC}]  Get ICP Balance"
    echo -e "     Get ICP ledger balance for an account"
    echo ""
    echo -e "  ${GREEN}3${NC} / [${GREEN}D${NC}]  Deploy SNS"
    echo -e "     Deploy a new SNS instance"
    echo ""
    echo -e "  ${GREEN}0${NC} / ${CYAN}Enter${NC}  Back to Main Menu"
    echo ""
    echo -n -e "${CYAN}Select operation [0-3, M, B, D, or Enter]: ${NC}"
}

# Show SNS submenu
show_sns_menu() {
    clear
    print_header "SNS Operations"
    echo ""
    echo -e "${CYAN}Available Operations:${NC}"
    echo ""
    echo -e "  ${GREEN}1${NC} / [${GREEN}L${NC}]  List SNS Neurons"
    echo -e "     Query and display SNS neurons for a principal (interactive)"
    echo ""
    echo -e "  ${GREEN}2${NC} / [${GREEN}C${NC}]  Create SNS Neuron"
    echo -e "     Create an SNS neuron by staking tokens from ledger balance"
    echo ""
    echo -e "  ${GREEN}3${NC} / [${GREEN}D${NC}]  Disburse SNS Neuron"
    echo -e "     Disburse tokens from an SNS neuron"
    echo ""
    echo -e "  ${GREEN}4${NC} / [${GREEN}M${NC}]  Mint SNS Tokens"
    echo -e "     Mint additional tokens to an account"
    echo ""
    echo -e "  ${GREEN}5${NC} / [${GREEN}H${NC}]  Add SNS Neuron Hotkey"
    echo -e "     Add a hotkey to an SNS participant neuron (interactive)"
    echo ""
    echo -e "  ${GREEN}6${NC} / [${GREEN}I${NC}]  Increase SNS Neuron Dissolve Delay"
    echo -e "     Add dissolve delay to an SNS neuron"
    echo ""
    echo -e "  ${GREEN}7${NC} / [${GREEN}DD${NC}] Dissolve SNS Neuron"
    echo -e "     Start or stop dissolving for an SNS neuron"
    echo ""
    echo -e "  ${GREEN}8${NC} / [${GREEN}B${NC}]  Get SNS Balance"
    echo -e "     Get SNS ledger balance for an account"
    echo ""
    echo -e "  ${GREEN}0${NC} / ${CYAN}Enter${NC}  Back to Main Menu"
    echo ""
    echo -n -e "${CYAN}Select operation [0-8, L, C, D, M, H, I, DD, B, or Enter]: ${NC}"
}

# Show Utils submenu
show_utils_menu() {
    clear
    print_header "Utils"
    echo ""
    echo -e "${CYAN}Available Operations:${NC}"
    echo ""
    echo -e "  ${GREEN}1${NC} / [${GREEN}D${NC}]  Deploy New SNS"
    echo -e "     Create a new SNS instance (creates a separate SNS, does not replace existing)"
    echo ""
    echo -e "  ${GREEN}2${NC} / [${GREEN}R${NC}]  Rebuild Binary"
    echo -e "     Rebuild the Rust binary (useful after code changes)"
    echo ""
    echo -e "  ${GREEN}0${NC} / ${CYAN}Enter${NC}  Back to Main Menu"
    echo ""
    echo -n -e "${CYAN}Select operation [0-2, D, R, or Enter]: ${NC}"
}

# Run selected script
run_script() {
    local category="$1"
    local operation="$2"
    shift 2 || true
    # Initialize script_args array - use explicit empty array if no args to avoid unbound variable errors
    local script_args=()
    if [ $# -gt 0 ]; then
        script_args=("$@")
    fi
    local script_name=""
    
    # Check if this operation requires SNS deployment
    if requires_sns_deployment "$category" "$operation"; then
        if ! check_sns_deployed; then
            print_error "No SNS deployment found on the network!"
            echo ""
            print_info "Please deploy an SNS first using Utils > Deploy New SNS."
            print_info "Configuration can be modified in src/init/sns_config.rs"
            echo ""
            return 1
        fi
    fi
    
    case "$category" in
        icp)
            case "$operation" in
                1|l|L)
                    script_name="get_icp_neurons.sh"
                    ;;
                2|c|C)
                    script_name="create_icp_neuron.sh"
                    ;;
                3|d|D)
                    script_name="disburse_icp_neuron.sh"
                    ;;
                4|m|M)
                    script_name="mint_icp.sh"
                    ;;
                5|h|H)
                    script_name="add_icp_hotkey.sh"
                    ;;
                6|i|I)
                    script_name="increase_icp_dissolve_delay.sh"
                    ;;
                7|dd|DD)
                    script_name="manage_icp_dissolving.sh"
                    ;;
                8|b|B)
                    script_name="get_icp_balance.sh"
                    ;;
                *)
                    print_error "Invalid ICP operation: $operation"
                    return 1
                    ;;
            esac
            ;;
        sns)
            case "$operation" in
                1|l|L)
                    script_name="get_sns_neurons.sh"
                    ;;
                2|c|C)
                    script_name="create_sns_neuron.sh"
                    ;;
                3|d|D)
                    script_name="disburse_sns_neuron.sh"
                    ;;
                4|m|M)
                    script_name="mint_sns_tokens.sh"
                    ;;
                5|h|H)
                    script_name="add_sns_hotkey.sh"
                    ;;
                6|i|I)
                    script_name="increase_sns_dissolve_delay.sh"
                    ;;
                7|dd|DD)
                    script_name="manage_sns_dissolving.sh"
                    ;;
                8|b|B)
                    script_name="get_sns_balance.sh"
                    ;;
                *)
                    print_error "Invalid SNS operation: $operation"
                    return 1
                    ;;
            esac
            ;;
        utils)
            case "$operation" in
                1|d|D)
                    script_name="deploy_local_sns.sh"
                    ;;
                2|r|R)
                    print_header "Rebuilding Binary"
                    bash "$SCRIPT_DIR/build.sh"
                    print_success "Binary rebuilt successfully!"
                    return 0
                    ;;
                *)
                    print_error "Invalid Utils operation: $operation"
                    return 1
                    ;;
            esac
            ;;
        *)
            print_error "Invalid category: $category"
            return 1
            ;;
    esac
    
    if [ -n "$script_name" ]; then
        local script_path="$SCRIPT_DIR/$script_name"
        
        if [ ! -f "$script_path" ]; then
            print_error "Script not found: $script_name"
            return 1
        fi
        
        # Make sure script is executable
        chmod +x "$script_path"
        
        # Run the script with any passed arguments
        print_header "Running: $script_name"
        # script_args is always initialized, but check length to avoid issues with set -u
        if [ ${#script_args[@]} -eq 0 ]; then
            bash "$script_path"
        else
            bash "$script_path" "${script_args[@]}"
        fi
    fi
    
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
    # Format: start.sh <category> <operation> [args...]
    if [ $# -gt 0 ]; then
        if [ $# -lt 2 ]; then
            print_error "Usage: $0 <category> <operation> [args...]"
            print_info "Categories: icp, sns, utils"
            exit 1
        fi
        run_script "$1" "$2" "${@:3}"
        exit $?
    fi
    
    # Interactive menu loop
    while true; do
        # Check if SNS is deployed - if not, show simplified menu directly
        if ! check_sns_deployed; then
            # No SNS deployed - show simplified menu
            show_icp_menu_no_sns
            read -r operation_choice
            
            # Handle empty input (Enter pressed) - exit
            if [ -z "$operation_choice" ]; then
                print_info "Exiting..."
                exit 0
            fi
            
            case "$operation_choice" in
                0)
                    print_info "Exiting..."
                    exit 0
                    ;;
                1|m|M)
                    # Mint ICP
                    run_script "icp" "4"
                    echo ""
                    echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                    read -r
                    ;;
                2|b|B)
                    # Get ICP Balance
                    run_script "icp" "8"
                    echo ""
                    echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                    read -r
                    ;;
                3|d|D)
                    # Deploy SNS
                    run_script "utils" "1"
                    echo ""
                    echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                    read -r
                    ;;
                *)
                    print_error "Invalid option. Please see menu for available options."
                    sleep 1
                    ;;
            esac
        else
            # SNS is deployed - show main menu
            show_main_menu
            read -r category_choice
            
            case "$category_choice" in
                0)
                    print_info "Exiting..."
                    exit 0
                    ;;
                1|i|I)
                    # ICP submenu
                    while true; do
                        show_icp_menu
                        read -r operation_choice
                        
                        # Handle empty input (Enter pressed) - go back to main menu
                        if [ -z "$operation_choice" ]; then
                            break
                        fi
                        
                        case "$operation_choice" in
                            0)
                                break  # Back to main menu
                                ;;
                            [1-8]|[lL]|[cC]|[dD]|[mM]|[hH]|[iI]|dd|DD|[bB])
                                run_script "icp" "$operation_choice"
                                echo ""
                                echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                                read -r
                                ;;
                            *)
                                print_error "Invalid option. Please see menu for available options."
                                sleep 1
                                ;;
                        esac
                    done
                    ;;
                2|s|S)
                    # SNS submenu
                    while true; do
                        show_sns_menu
                        read -r operation_choice
                        
                        # Handle empty input (Enter pressed) - go back to main menu
                        if [ -z "$operation_choice" ]; then
                            break
                        fi
                        
                        case "$operation_choice" in
                            0)
                                break  # Back to main menu
                                ;;
                            [1-8]|[lL]|[cC]|[dD]|[mM]|[hH]|[iI]|dd|DD|[bB])
                                run_script "sns" "$operation_choice"
                                echo ""
                                echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                                read -r
                                ;;
                            *)
                                print_error "Invalid option. Please see menu for available options."
                                sleep 1
                                ;;
                        esac
                    done
                    ;;
                3|u|U)
                    # Utils submenu
                    while true; do
                        show_utils_menu
                        read -r operation_choice
                        
                        # Handle empty input (Enter pressed) - go back to main menu
                        if [ -z "$operation_choice" ]; then
                            break
                        fi
                        
                        case "$operation_choice" in
                            0)
                                break  # Back to main menu
                                ;;
                            [1-2]|[dD]|[rR])
                                run_script "utils" "$operation_choice"
                                echo ""
                                echo -n -e "${CYAN}Press Enter to return to menu...${NC}"
                                read -r
                                ;;
                            *)
                                print_error "Invalid option. Please see menu for available options."
                                sleep 1
                                ;;
                        esac
                    done
                    ;;
                *)
                    print_error "Invalid option. Please see menu for available options."
                    sleep 1
                    ;;
            esac
        fi
    done
}

# Run main function
main "$@"


