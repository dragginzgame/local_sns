mod core;
mod init;

use anyhow::Result;

use core::ops::commands::{
    handle_add_hotkey, handle_check_sns_deployed, handle_create_icp_neuron,
    handle_create_sns_neuron, handle_disburse_sns_neuron, handle_get_icp_balance,
    handle_get_icp_neuron, handle_get_sns_balance, handle_increase_sns_dissolve_delay,
    handle_list_neurons, handle_manage_sns_dissolving, handle_mint_icp, handle_mint_sns_tokens,
    handle_set_icp_visibility,
};
use core::ops::deployment::deploy_sns;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Handle CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "deploy-sns" => return deploy_sns().await,
            "add-hotkey" => return handle_add_hotkey(&args).await,
            "list-sns-neurons" => return handle_list_neurons(&args).await,
            "mint-sns-tokens" => return handle_mint_sns_tokens(&args).await,
            "create-sns-neuron" => return handle_create_sns_neuron(&args).await,
            "disburse-sns-neuron" => return handle_disburse_sns_neuron(&args).await,
            "increase-sns-dissolve-delay" => {
                return handle_increase_sns_dissolve_delay(&args).await;
            }
            "manage-sns-dissolving" => return handle_manage_sns_dissolving(&args).await,
            "set-icp-visibility" => return handle_set_icp_visibility(&args).await,
            "get-icp-neuron" => return handle_get_icp_neuron(&args).await,
            "get-icp-balance" => return handle_get_icp_balance(&args).await,
            "get-sns-balance" => return handle_get_sns_balance(&args).await,
            "mint-icp" => return handle_mint_icp(&args).await,
            "create-icp-neuron" => return handle_create_icp_neuron(&args).await,
            "check-sns-deployed" => return handle_check_sns_deployed(&args).await,
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("\nAvailable commands:");
                eprintln!("  deploy-sns          - Deploy a new SNS on local dfx network");
                eprintln!("  add-hotkey          - Add a hotkey to an SNS or ICP neuron");
                eprintln!("  list-sns-neurons    - List SNS neurons for a principal");
                eprintln!("  mint-sns-tokens     - Create proposal to mint SNS tokens and vote");
                eprintln!("  create-sns-neuron        - Create an SNS neuron by staking tokens");
                eprintln!(
                    "  disburse-sns-neuron      - Disburse an SNS neuron to a receiver principal"
                );
                eprintln!(
                    "  increase-sns-dissolve-delay - Increase dissolve delay for an SNS neuron"
                );
                eprintln!("  manage-sns-dissolving    - Start or stop dissolving an SNS neuron");
                eprintln!("  set-icp-visibility       - Set ICP neuron visibility");
                eprintln!("  get-icp-neuron           - Get ICP neuron information");
                eprintln!("  get-icp-balance          - Get ICP ledger balance for an account");
                eprintln!("  get-sns-balance          - Get SNS ledger balance for an account");
                eprintln!("  mint-icp                 - Mint ICP tokens from minting account");
                eprintln!("  create-icp-neuron        - Create an ICP neuron by staking ICP");
                std::process::exit(1);
            }
        }
    }

    // Default behavior: deploy SNS if no arguments
    deploy_sns().await
}
