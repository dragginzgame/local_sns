mod core;

use anyhow::Result;

use core::commands::{
    handle_add_hotkey, handle_get_icp_neuron, handle_list_neurons, handle_set_icp_visibility,
};
use core::deployment::deploy_sns;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Handle CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "deploy-sns" => return deploy_sns().await,
            "add-hotkey" => return handle_add_hotkey(&args).await,
            "list-neurons" => return handle_list_neurons(&args).await,
            "set-icp-visibility" => return handle_set_icp_visibility(&args).await,
            "get-icp-neuron" => return handle_get_icp_neuron(&args).await,
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("\nAvailable commands:");
                eprintln!("  deploy-sns          - Deploy a new SNS on local dfx network");
                eprintln!("  add-hotkey          - Add a hotkey to an SNS or ICP neuron");
                eprintln!("  list-neurons        - List SNS neurons for a principal");
                eprintln!("  set-icp-visibility  - Set ICP neuron visibility");
                eprintln!("  get-icp-neuron      - Get ICP neuron information");
                std::process::exit(1);
            }
        }
    }

    // Default behavior: deploy SNS if no arguments
    deploy_sns().await
}
