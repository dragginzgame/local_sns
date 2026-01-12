// CLI command handlers

use anyhow::{Context, Result};
use candid::Principal;
use hex;

use crate::core::ops::governance_ops::{
    add_hotkey_to_icp_neuron_default_path, get_icp_neuron_default_path,
    set_icp_neuron_visibility_default_path,
};
use crate::core::ops::sns_governance_ops::{
    add_hotkey_to_participant_neuron_default_path, list_neurons_for_principal_default_path,
};
use crate::core::utils::{print_header, print_info, print_success, print_warning};

/// Handle add-hotkey command
pub async fn handle_add_hotkey(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        print_add_hotkey_usage(&args[0]);
        std::process::exit(1);
    }

    let neuron_type = &args[2];

    match neuron_type.as_str() {
        "sns" => {
            if args.len() < 5 {
                eprintln!(
                    "Error: For SNS neurons, owner_principal and hotkey_principal are required"
                );
                eprintln!(
                    "Usage: {} add-hotkey sns <owner_principal> <hotkey_principal> [permissions]",
                    args[0]
                );
                std::process::exit(1);
            }
            let owner_principal =
                Principal::from_text(&args[3]).context("Failed to parse owner principal")?;
            let hotkey_principal =
                Principal::from_text(&args[4]).context("Failed to parse hotkey principal")?;
            let permissions = if args.len() > 5 {
                let perm_str = &args[5];
                Some(
                    perm_str
                        .split(',')
                        .map(|s| {
                            s.trim()
                                .parse::<i32>()
                                .context("Failed to parse permission type")
                        })
                        .collect::<Result<Vec<_>>>()?,
                )
            } else {
                None
            };

            print_header("Adding Hotkey to SNS Neuron");
            print_info(&format!("Participant: {}", owner_principal));
            print_info(&format!("Hotkey: {}", hotkey_principal));

            add_hotkey_to_participant_neuron_default_path(
                owner_principal,
                hotkey_principal,
                permissions,
            )
            .await
            .context("Failed to add hotkey to SNS neuron")?;

            print_success("Hotkey added successfully!");
            Ok(())
        }
        "icp" => {
            if args.len() < 4 {
                eprintln!("Error: For ICP neurons, hotkey_principal is required");
                eprintln!("Usage: {} add-hotkey icp <hotkey_principal>", args[0]);
                std::process::exit(1);
            }
            let hotkey_principal =
                Principal::from_text(&args[3]).context("Failed to parse hotkey principal")?;

            print_header("Adding Hotkey to ICP Neuron");
            print_info(&format!("Hotkey: {}", hotkey_principal));
            print_info("Using ICP neuron from SNS deployment data");

            add_hotkey_to_icp_neuron_default_path(hotkey_principal)
                .await
                .context("Failed to add hotkey to ICP neuron")?;

            print_success("Hotkey added successfully!");
            Ok(())
        }
        _ => {
            eprintln!("Unknown neuron type: {}. Use 'sns' or 'icp'", neuron_type);
            std::process::exit(1);
        }
    }
}

/// Handle list-neurons command
pub async fn handle_list_neurons(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: {} list-neurons <principal>", args[0]);
        eprintln!("\nArguments:");
        eprintln!("  principal - Principal to query neurons for");
        eprintln!("\nExample:");
        eprintln!(
            "  {} list-neurons qc2qr-5u5mz-3ny2c-rzvkj-3z2lh-4uawd-5ggw7-pfwno-ghsmf-gqfau-oqe",
            args[0]
        );
        std::process::exit(1);
    }

    let principal = Principal::from_text(&args[2]).context("Failed to parse principal")?;

    print_header("Listing SNS Neurons");
    print_info(&format!("Principal: {}", principal));

    let neurons = list_neurons_for_principal_default_path(principal)
        .await
        .context("Failed to list neurons")?;

    if neurons.is_empty() {
        print_warning("No neurons found for this principal");
        return Ok(());
    }

    print_success(&format!("Found {} neuron(s)", neurons.len()));
    println!();

    for (i, neuron) in neurons.iter().enumerate() {
        println!("{}Neuron #{}", "━".repeat(60), i + 1);

        if let Some(id) = &neuron.id {
            let hex_id = hex::encode(&id.id);
            println!("  ID (hex): {}", hex_id);

            // Show shortened version if it's 32 bytes
            if id.id.len() == 32 {
                let first = &hex_id[..8];
                let last = &hex_id[hex_id.len() - 8..];
                println!("  ID (short): {}...{}", first, last);
            }
        } else {
            println!("  ID: <none>");
        }

        if !neuron.permissions.is_empty() {
            println!("  Permissions:");
            for perm in &neuron.permissions {
                if let Some(perm_principal) = &perm.principal {
                    println!("    - Principal: {}", perm_principal);
                    if !perm.permission_type.is_empty() {
                        let perm_types: Vec<String> = perm
                            .permission_type
                            .iter()
                            .map(|p| match *p {
                                0 => "Unspecified".to_string(),
                                1 => "ConfigureDissolveState".to_string(),
                                2 => "ManagePrincipals".to_string(),
                                3 => "SubmitProposal".to_string(),
                                4 => "Vote".to_string(),
                                5 => "Disburse".to_string(),
                                6 => "Split".to_string(),
                                7 => "MergeMaturity".to_string(),
                                8 => "DisburseMaturity".to_string(),
                                9 => "StakeMaturity".to_string(),
                                10 => "ManageVotingPermission".to_string(),
                                n => format!("Unknown({})", n),
                            })
                            .collect();
                        println!("      Types: {}", perm_types.join(", "));
                    }
                }
            }
        }
        println!();
    }

    Ok(())
}

/// Handle set-icp-visibility command
pub async fn handle_set_icp_visibility(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: {} set-icp-visibility <true|false>", args[0]);
        eprintln!("\nArguments:");
        eprintln!("  true  - Set neuron to public (visible to everyone)");
        eprintln!("  false - Set neuron to private (only visible to controller)");
        eprintln!("\nNote: Uses ICP neuron from SNS deployment data");
        eprintln!("\nExample:");
        eprintln!("  {} set-icp-visibility true", args[0]);
        eprintln!("  {} set-icp-visibility false", args[0]);
        std::process::exit(1);
    }

    let visibility_str = &args[2].to_lowercase();
    let is_public = match visibility_str.as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => {
            eprintln!("Error: Invalid visibility value: {}", args[2]);
            eprintln!("Use 'true' or 'false'");
            std::process::exit(1);
        }
    };

    print_header("Setting ICP Neuron Visibility");
    print_info(&format!(
        "Visibility: {} (value: {}) (from deployment data)",
        if is_public { "Public" } else { "Private" },
        if is_public { 2 } else { 1 }
    ));

    set_icp_neuron_visibility_default_path(is_public)
        .await
        .context("Failed to set neuron visibility")?;

    print_success("Visibility updated successfully!");
    Ok(())
}

/// Handle get-icp-neuron command
pub async fn handle_get_icp_neuron(args: &[String]) -> Result<()> {
    let neuron_id = if args.len() > 2 {
        Some(
            args[2]
                .parse::<u64>()
                .context("Failed to parse neuron ID")?,
        )
    } else {
        None
    };

    print_header("Getting ICP Neuron Information");
    if let Some(id) = neuron_id {
        print_info(&format!("Neuron ID: {}", id));
    } else {
        print_info("Using neuron ID from deployment data");
    }

    let neuron = get_icp_neuron_default_path(neuron_id)
        .await
        .context("Failed to get neuron")?;

    // Output full response as JSON
    let json =
        serde_json::to_string_pretty(&neuron).context("Failed to serialize neuron to JSON")?;
    println!("{}", json);

    Ok(())
}

fn print_add_hotkey_usage(program_name: &str) {
    eprintln!("Usage: {} add-hotkey <neuron_type> <...>", program_name);
    eprintln!("\nNeuron types:");
    eprintln!("  sns  - SNS neuron (from deployed SNS)");
    eprintln!("  icp  - ICP neuron (ICP Governance)");
    eprintln!("\nFor SNS neurons:");
    eprintln!(
        "  Usage: {} add-hotkey sns <owner_principal> <hotkey_principal> [permissions]",
        program_name
    );
    eprintln!("  owner_principal - Principal of the participant who owns the neuron");
    eprintln!(
        "  permissions    - Optional: comma-separated permission types (default: 3,4 = SubmitProposal + Vote)"
    );
    eprintln!(
        "                   Permission types: 2=ManagePrincipals, 3=SubmitProposal, 4=Vote, etc."
    );
    eprintln!("\nFor ICP neurons:");
    eprintln!(
        "  Usage: {} add-hotkey icp <hotkey_principal>",
        program_name
    );
    eprintln!("  Uses ICP neuron from SNS deployment data");
    eprintln!("  permissions    - Not applicable (ICP neurons don't use permission types)");
    eprintln!("\nExamples:");
    eprintln!(
        "  {} add-hotkey sns <participant_principal> <hotkey_principal>",
        program_name
    );
    eprintln!(
        "  {} add-hotkey sns <participant_principal> <hotkey_principal> 3,4",
        program_name
    );
    eprintln!("  {} add-hotkey icp <hotkey_principal>", program_name);
}
