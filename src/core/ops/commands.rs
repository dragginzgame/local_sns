// CLI command handlers

use anyhow::{Context, Result};
use candid::Principal;
use hex;

use crate::core::ops::governance_ops::{
    add_hotkey_to_icp_neuron_default_path, create_icp_neuron_default_path,
    get_icp_neuron_default_path, list_icp_neurons_for_principal_default_path,
    mint_icp_default_path, set_icp_neuron_visibility_default_path,
};
use crate::core::ops::ledger_ops::{get_icp_ledger_balance, get_sns_ledger_balance};
use crate::core::ops::sns_governance_ops::{
    add_hotkey_to_participant_neuron_default_path, create_sns_neuron_default_path,
    disburse_participant_neuron_default_path,
    increase_dissolve_delay_participant_neuron_default_path,
    list_neurons_for_principal_default_path,
    manage_dissolving_state_participant_neuron_default_path,
    mint_sns_tokens_with_all_votes_default_path,
};
use crate::core::ops::snsw_ops::check_sns_deployed_default_path;
use crate::core::utils::{print_header, print_info, print_success, print_warning};

/// Select participant OR enter custom principal
/// Shows participants (1-N) OR allows entering a custom principal
fn select_participant_or_custom() -> Result<Principal> {
    select_participant_or_custom_with_label(None)
}

/// Select participant OR enter custom principal with optional label
/// Shows participants (1-N) OR allows entering a custom principal
fn select_participant_or_custom_with_label(label: Option<&str>) -> Result<Principal> {
    use crate::core::utils::data_output::SnsCreationData;
    use std::io::{self, Write};

    // Try to read deployment data
    let deployment_path = crate::core::utils::data_output::get_output_path();

    if deployment_path.exists() {
        if let Ok(data_content) = std::fs::read_to_string(&deployment_path) {
            if let Ok(deployment_data) = serde_json::from_str::<SnsCreationData>(&data_content) {
                let owner_option = deployment_data.participants.len() + 1;
                let custom_option = owner_option + 1;

                if let Some(lbl) = label {
                    println!("{}", lbl);
                    println!();
                }
                println!("Available options:");
                println!();
                // Show participants first
                for (i, participant) in deployment_data.participants.iter().enumerate() {
                    println!("  [{}] {}", i + 1, participant.principal);
                }
                // Show owner before custom principal
                println!(
                    "  [{}] {} (SNS proposer)",
                    owner_option, deployment_data.owner_principal
                );
                println!("  [{}] Enter custom principal", custom_option);
                println!();
                print!(
                    "Select option number (1-{}) or enter principal: ",
                    custom_option
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input_trimmed = input.trim();

                // If input is empty, treat as invalid
                if input_trimmed.is_empty() {
                    anyhow::bail!("No input provided. Please enter a number or principal.");
                }

                // Check if input looks like a principal (contains dashes, typical format)
                // Principals typically have 5 dashes and are 63 characters long
                if input_trimmed.contains('-') && input_trimmed.len() > 20 {
                    // Try to parse as principal directly
                    match Principal::from_text(input_trimmed) {
                        Ok(principal) => return Ok(principal),
                        Err(e) => {
                            // If principal parsing fails, check if it's a number
                            // Otherwise return the error
                            if input_trimmed.parse::<usize>().is_ok() {
                                // It's actually a number, continue to number parsing below
                            } else {
                                return Err(anyhow::anyhow!("Failed to parse principal: {}", e));
                            }
                        }
                    }
                }

                // Try to parse as number
                match input_trimmed.parse::<usize>() {
                    Ok(selection) => {
                        if selection < 1 || selection > custom_option {
                            anyhow::bail!(
                                "Invalid selection. Please choose a number between 1 and {}",
                                custom_option
                            );
                        }

                        if selection == custom_option {
                            // Custom principal option
                            print!("Enter principal: ");
                            io::stdout().flush()?;
                            let mut principal_input = String::new();
                            io::stdin().read_line(&mut principal_input)?;
                            let principal_trimmed = principal_input.trim();
                            if principal_trimmed.is_empty() {
                                anyhow::bail!("No principal provided.");
                            }
                            Principal::from_text(principal_trimmed)
                                .context("Failed to parse principal")
                        } else if selection == owner_option {
                            // Owner (SNS proposer)
                            Principal::from_text(&deployment_data.owner_principal)
                                .context("Failed to parse owner principal")
                        } else {
                            // Participant (selection is 1-based, participants array is 0-based)
                            Principal::from_text(
                                &deployment_data.participants[selection - 1].principal,
                            )
                            .context("Failed to parse selected participant principal")
                        }
                    }
                    Err(_) => {
                        // Not a number, try to parse as principal anyway
                        Principal::from_text(input_trimmed).context("Failed to parse principal")
                    }
                }
            } else {
                // Deployment data exists but can't parse - fall back to custom input
                if let Some(lbl) = label {
                    println!("{}", lbl);
                } else {
                    print_header("Select Principal");
                }
                print!("Enter principal: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                Principal::from_text(input.trim()).context("Failed to parse principal")
            }
        } else {
            // Can't read deployment data - fall back to custom input
            if let Some(lbl) = label {
                println!("{}", lbl);
            } else {
                print_header("Select Principal");
            }
            print!("Enter principal: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Principal::from_text(input.trim()).context("Failed to parse principal")
        }
    } else {
        // No deployment data - fall back to custom input
        if let Some(lbl) = label {
            println!("{}", lbl);
        } else {
            print_header("Select Principal");
        }
        print!("Enter principal: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Principal::from_text(input.trim()).context("Failed to parse principal")
    }
}

/// Helper function to select a neuron interactively for a given principal
async fn select_neuron(principal: Principal) -> Result<Vec<u8>> {
    use crate::core::ops::sns_governance_ops::list_neurons_for_principal_default_path;
    use std::io::{self, Write};

    print_header("Select Neuron");

    let neurons = list_neurons_for_principal_default_path(principal)
        .await
        .context("Failed to list neurons")?;

    if neurons.is_empty() {
        anyhow::bail!("No neurons found for this principal");
    }

    println!("Available neurons:");
    println!();
    for (i, neuron) in neurons.iter().enumerate() {
        let neuron_id_display = if let Some(id) = &neuron.id {
            let hex_id = hex::encode(&id.id);
            if hex_id.len() >= 15 {
                format!("{}...{}", &hex_id[..7], &hex_id[hex_id.len() - 8..])
            } else {
                hex_id
            }
        } else {
            "<none>".to_string()
        };

        let stake = neuron.cached_neuron_stake_e8s;
        let dissolve_info = match &neuron.dissolve_state {
            Some(crate::core::declarations::sns_governance::DissolveState::DissolveDelaySeconds(seconds)) => {
                let days = *seconds / 86400;
                format!("{} days", days)
            }
            Some(crate::core::declarations::sns_governance::DissolveState::WhenDissolvedTimestampSeconds(_)) => {
                "Dissolving".to_string()
            }
            None => "No state".to_string(),
        };

        println!(
            "  [{}] {} - Stake: {} e8s - {}",
            i + 1,
            neuron_id_display,
            stake,
            dissolve_info
        );
    }
    println!();
    print!("Select neuron number (1-{}): ", neurons.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse().context("Invalid selection")?;

    if selection < 1 || selection > neurons.len() {
        anyhow::bail!(
            "Invalid selection. Please choose a number between 1 and {}",
            neurons.len()
        );
    }

    let selected_neuron = &neurons[selection - 1];
    if let Some(id) = &selected_neuron.id {
        Ok(id.id.clone())
    } else {
        anyhow::bail!("Selected neuron has no ID")
    }
}

/// Handle add-hotkey command
pub async fn handle_add_hotkey(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    if args.len() < 3 {
        print_add_hotkey_usage(&args[0]);
        std::process::exit(1);
    }

    let neuron_type = &args[2];

    match neuron_type.as_str() {
        "sns" => {
            // Step 1: Get owner principal (select if not provided)
            let owner_principal = if args.len() >= 4 {
                Principal::from_text(&args[3]).context("Failed to parse owner principal")?
            } else {
                select_participant_or_custom()?
            };

            // Step 2: Get neuron_id and hotkey_principal
            let (neuron_id, hotkey_principal, permissions) = if args.len() >= 5 {
                let arg4 = &args[4];

                // Check if arg4 looks like a neuron_id (hex string)
                let looks_like_neuron_id = (arg4.starts_with("0x") && arg4.len() > 10)
                    || (!arg4.contains(',')
                        && !arg4.contains('-') // Principal contains dashes
                        && arg4.chars().all(|c| c.is_ascii_hexdigit())
                        && arg4.len() > 8);

                if looks_like_neuron_id {
                    // arg4 is neuron_id
                    let hex_str = arg4.strip_prefix("0x").unwrap_or(arg4);
                    let neuron_id_val =
                        Some(hex::decode(hex_str).context("Failed to decode neuron_id from hex")?);

                    // Get hotkey_principal from next arg
                    let hotkey = if args.len() >= 6 {
                        Principal::from_text(&args[5])
                            .context("Failed to parse hotkey principal")?
                    } else {
                        print!("Enter hotkey principal: ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        Principal::from_text(input.trim())
                            .context("Failed to parse hotkey principal")?
                    };

                    // Check for permissions
                    let perms = if args.len() >= 7 {
                        let perm_str = &args[6];
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

                    (neuron_id_val, hotkey, perms)
                } else {
                    // arg4 is hotkey_principal, need to select neuron
                    let hotkey =
                        Principal::from_text(arg4).context("Failed to parse hotkey principal")?;
                    let neuron_id_val = select_neuron(owner_principal).await?;

                    let perms = if args.len() >= 6 {
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

                    (Some(neuron_id_val), hotkey, perms)
                }
            } else {
                // Need to select neuron and get hotkey interactively
                let neuron_id_val = select_neuron(owner_principal).await?;

                print!("Enter hotkey principal: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let hotkey = Principal::from_text(input.trim())
                    .context("Failed to parse hotkey principal")?;

                (Some(neuron_id_val), hotkey, None)
            };

            print_header("Adding Hotkey to SNS Neuron");
            print_info(&format!("Participant: {}", owner_principal));
            print_info(&format!("Hotkey: {}", hotkey_principal));
            if let Some(ref id) = neuron_id {
                let hex_id = hex::encode(id);
                if hex_id.len() >= 15 {
                    print_info(&format!(
                        "Neuron ID: {}...{}",
                        &hex_id[..7],
                        &hex_id[hex_id.len() - 8..]
                    ));
                } else {
                    print_info(&format!("Neuron ID: {}", hex_id));
                }
            } else {
                print_info("Neuron ID: Auto-selecting (longest dissolve delay)");
            }

            add_hotkey_to_participant_neuron_default_path(
                owner_principal,
                hotkey_principal,
                permissions,
                neuron_id,
            )
            .await
            .context("Failed to add hotkey to SNS neuron")?;

            print_success("Hotkey added successfully!");
            Ok(())
        }
        "icp" => {
            // Step 1: Get hotkey principal (interactive if not provided)
            let hotkey_principal = if args.len() >= 4 {
                Principal::from_text(&args[3]).context("Failed to parse hotkey principal")?
            } else {
                print_header("Add Hotkey to ICP Neuron");
                print_info("Using ICP neuron from SNS deployment data");
                print!("Enter hotkey principal: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                Principal::from_text(input.trim()).context("Failed to parse hotkey principal")?
            };

            print_header("Adding Hotkey to ICP Neuron");
            print_info(&format!("Hotkey: {}", hotkey_principal));
            print_info("Using ICP neuron from SNS deployment data");
            print_info("Note: ICP neurons don't use permission types - hotkeys have full control");

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

/// Handle list-sns-neurons command
pub async fn handle_list_neurons(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    let principal = if args.len() < 3 {
        // No principal provided - show participant selection or custom (includes owner)
        select_participant_or_custom()?
    } else {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    };

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

    // Print table header
    println!("{:-<100}", "");
    println!(
        "{:<5} {:<20} {:<20} {:<25} {:<30}",
        "#", "Neuron ID", "Stake (e8s)", "Dissolve Delay", "Permissions"
    );
    println!("{:-<100}", "");

    for (index, neuron) in neurons.iter().enumerate() {
        // Neuron ID (hex) - use short format like e35f1b8...518559ea
        let neuron_id_display = if let Some(id) = &neuron.id {
            let hex_id = hex::encode(&id.id);
            if hex_id.len() >= 15 {
                // Show first 7 chars + ... + last 8 chars
                format!("{}...{}", &hex_id[..7], &hex_id[hex_id.len() - 8..])
            } else {
                hex_id
            }
        } else {
            "<none>".to_string()
        };

        // Stake
        let stake_str = format!("{}", neuron.cached_neuron_stake_e8s);

        // Dissolve delay
        let dissolve_delay_str = match &neuron.dissolve_state {
            Some(super::super::declarations::sns_governance::DissolveState::DissolveDelaySeconds(seconds)) => {
                let days = *seconds / 86400;
                format!("{} days ({}s)", days, seconds)
            }
            Some(super::super::declarations::sns_governance::DissolveState::WhenDissolvedTimestampSeconds(timestamp)) => {
                format!("Dissolving (dissolves at {})", timestamp)
            }
            None => "No state".to_string(),
        };

        // Permissions - summarize all permission types across all principals, use numeric values
        let mut all_permissions: Vec<i32> = Vec::new();
        for perm in &neuron.permissions {
            all_permissions.extend(&perm.permission_type);
        }
        all_permissions.sort();
        all_permissions.dedup();
        let perm_str = if all_permissions.is_empty() {
            "None".to_string()
        } else {
            all_permissions
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        };

        // Truncate dissolve delay if too long for table formatting
        let dissolve_delay_display = if dissolve_delay_str.len() > 18 {
            format!("{}...", &dissolve_delay_str[..18])
        } else {
            dissolve_delay_str
        };

        println!(
            "{:<5} {:<20} {:<20} {:<25} {:<30}",
            index + 1,
            neuron_id_display,
            stake_str,
            dissolve_delay_display,
            perm_str
        );
    }

    println!("{:-<100}", "");
    println!();

    // Ask if user wants to see details for a specific neuron
    if neurons.len() > 0 {
        println!();
        print!(
            "Enter neuron number to see full details (1-{}) or press Enter to skip: ",
            neurons.len()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection = input.trim();

        if !selection.is_empty() {
            let selection_num: usize = selection
                .parse()
                .context("Invalid selection - must be a number")?;
            if selection_num < 1 || selection_num > neurons.len() {
                eprintln!(
                    "Invalid selection. Please choose a number between 1 and {}",
                    neurons.len()
                );
                return Ok(());
            }

            let selected_neuron = &neurons[selection_num - 1];
            display_neuron_details(selected_neuron);
        }
    }

    Ok(())
}

/// Display full details for a single neuron
fn display_neuron_details(neuron: &crate::core::declarations::sns_governance::Neuron) {
    use crate::core::declarations::sns_governance::DissolveState;

    print_header("Neuron Details");

    // Neuron ID
    if let Some(id) = &neuron.id {
        let hex_id = hex::encode(&id.id);
        print_info(&format!("Neuron ID: {}", hex_id));
    } else {
        print_info("Neuron ID: <none>");
    }

    // Stake information
    println!();
    print_info("Stake Information:");
    println!("  Cached Stake: {} e8s", neuron.cached_neuron_stake_e8s);
    if let Some(staked_maturity) = neuron.staked_maturity_e8s_equivalent {
        println!("  Staked Maturity: {} e8s", staked_maturity);
    }
    println!("  Maturity: {} e8s", neuron.maturity_e8s_equivalent);

    // Dissolve state
    println!();
    print_info("Dissolve State:");
    match &neuron.dissolve_state {
        Some(DissolveState::DissolveDelaySeconds(seconds)) => {
            let days = *seconds / 86400;
            let hours = (*seconds % 86400) / 3600;
            println!("  Type: Dissolve Delay");
            println!(
                "  Delay: {} seconds ({} days, {} hours)",
                seconds, days, hours
            );
        }
        Some(DissolveState::WhenDissolvedTimestampSeconds(timestamp)) => {
            println!("  Type: Dissolving");
            println!("  Dissolves at timestamp: {}", timestamp);
        }
        None => {
            println!("  Type: None");
        }
    }

    // Aging
    println!();
    print_info("Aging:");
    println!(
        "  Aging since timestamp: {}",
        neuron.aging_since_timestamp_seconds
    );
    println!("  Created timestamp: {}", neuron.created_timestamp_seconds);

    // Voting power
    println!();
    print_info(&format!(
        "Voting Power Multiplier: {}%",
        neuron.voting_power_percentage_multiplier
    ));

    // Permissions
    println!();
    print_info("Permissions:");
    if neuron.permissions.is_empty() {
        println!("  None");
    } else {
        for perm in &neuron.permissions {
            if let Some(principal) = &perm.principal {
                println!("  Principal: {}", principal);
                println!("    Permission Types: {:?}", perm.permission_type);
            } else {
                println!("  Unknown Principal:");
                println!("    Permission Types: {:?}", perm.permission_type);
            }
        }
    }

    // Auto stake maturity
    if let Some(auto_stake) = neuron.auto_stake_maturity {
        println!();
        print_info(&format!("Auto Stake Maturity: {}", auto_stake));
    }

    // Vesting
    if let Some(vesting) = neuron.vesting_period_seconds {
        println!();
        print_info(&format!("Vesting Period: {} seconds", vesting));
    }

    // Disburse maturity in progress
    if !neuron.disburse_maturity_in_progress.is_empty() {
        println!();
        print_info("Disburse Maturity In Progress:");
        for disburse in &neuron.disburse_maturity_in_progress {
            println!("  Amount: {} e8s", disburse.amount_e8s);
            println!(
                "  Timestamp: {}",
                disburse.timestamp_of_disbursement_seconds
            );
            if let Some(account) = &disburse.account_to_disburse_to {
                if let Some(owner) = &account.owner {
                    println!("  Account Owner: {}", owner);
                }
            }
        }
    }

    // Followees
    if !neuron.followees.is_empty() {
        println!();
        print_info("Followees:");
        for (function_id, followees) in &neuron.followees {
            println!(
                "  Function ID {}: {} followee(s)",
                function_id,
                followees.followees.len()
            );
        }
    }

    // Topic followees
    if let Some(topic_followees) = &neuron.topic_followees {
        if !topic_followees.topic_id_to_followees.is_empty() {
            println!();
            print_info("Topic Followees:");
            for (topic_id, topic_data) in &topic_followees.topic_id_to_followees {
                if let Some(topic) = &topic_data.topic {
                    // Match on the Topic enum variant
                    use crate::core::declarations::sns_governance::Topic;
                    let topic_str = match topic {
                        Topic::DappCanisterManagement => "DappCanisterManagement",
                        Topic::DaoCommunitySettings => "DaoCommunitySettings",
                        Topic::ApplicationBusinessLogic => "ApplicationBusinessLogic",
                        Topic::CriticalDappOperations => "CriticalDappOperations",
                        Topic::TreasuryAssetManagement => "TreasuryAssetManagement",
                        Topic::Governance => "Governance",
                        Topic::SnsFrameworkManagement => "SnsFrameworkManagement",
                    };
                    println!(
                        "  Topic {} (ID {}): {} followee(s)",
                        topic_str,
                        topic_id,
                        topic_data.followees.len()
                    );
                } else {
                    println!(
                        "  Topic ID {}: {} followee(s)",
                        topic_id,
                        topic_data.followees.len()
                    );
                }
            }
        }
    }

    println!();
}

/// Handle set-icp-visibility command
pub async fn handle_set_icp_visibility(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Get visibility (interactive if not provided)
    let is_public = if args.len() >= 3 {
        let visibility_str = &args[2].to_lowercase();
        match visibility_str.as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                eprintln!("Error: Invalid visibility value: {}", args[2]);
                eprintln!("Use 'true' or 'false'");
                std::process::exit(1);
            }
        }
    } else {
        // Interactive prompt
        print_header("Set ICP Neuron Visibility");
        print_info("Using ICP neuron from SNS deployment data");
        println!();
        println!("Visibility options:");
        println!("  [1] Public (visible to everyone)");
        println!("  [2] Private (only visible to controller)");
        println!();
        print!("Select option (1 or 2, default: 2): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "1" | "public" | "true" | "yes" => true,
            "2" | "private" | "false" | "no" | "" => false,
            _ => {
                anyhow::bail!("Invalid selection. Use 1 for public or 2 for private.");
            }
        }
    };

    print_header("Setting ICP Neuron Visibility");
    print_info(&format!(
        "Visibility: {} (value: {})",
        if is_public { "Public" } else { "Private" },
        if is_public { 2 } else { 1 }
    ));
    print_info("Using ICP neuron from SNS deployment data");

    set_icp_neuron_visibility_default_path(is_public)
        .await
        .context("Failed to set neuron visibility")?;

    print_success("Visibility updated successfully!");
    Ok(())
}

/// Handle get-icp-neuron command
pub async fn handle_get_icp_neuron(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Get neuron ID (interactive if not provided)
    let neuron_id = if args.len() > 2 {
        Some(
            args[2]
                .parse::<u64>()
                .context("Failed to parse neuron ID")?,
        )
    } else {
        // Try to get from deployment data, or prompt
        let deployment_path = crate::core::utils::data_output::get_output_path();
        if deployment_path.exists() {
            let data_content = std::fs::read_to_string(&deployment_path)
                .context("Failed to read deployment data")?;
            let deployment_data: crate::core::utils::data_output::SnsCreationData =
                serde_json::from_str(&data_content)
                    .context("Failed to parse deployment data JSON")?;

            if deployment_data.icp_neuron_id > 0 {
                None // Will use from deployment data
            } else {
                // No neuron ID in deployment data, prompt for it
                print_header("Get ICP Neuron Information");
                print_info("No neuron ID found in deployment data");
                print!("Enter neuron ID (or press Enter to exit): ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                if input.is_empty() {
                    anyhow::bail!("No neuron ID provided");
                }
                Some(
                    input
                        .parse::<u64>()
                        .context("Failed to parse neuron ID - must be a number")?,
                )
            }
        } else {
            // No deployment data, must provide neuron ID
            print_header("Get ICP Neuron Information");
            print_info("No deployment data found");
            print!("Enter neuron ID: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Some(
                input
                    .trim()
                    .parse::<u64>()
                    .context("Failed to parse neuron ID - must be a number")?,
            )
        }
    };

    print_header("Getting ICP Neuron Information");
    if let Some(id) = neuron_id {
        print_info(&format!("Neuron ID: {} (specified)", id));
    } else {
        let deployment_path = crate::core::utils::data_output::get_output_path();
        let data_content =
            std::fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
        let deployment_data: crate::core::utils::data_output::SnsCreationData =
            serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;
        print_info(&format!(
            "Neuron ID: {} (from deployment data)",
            deployment_data.icp_neuron_id
        ));
    }

    let neuron = get_icp_neuron_default_path(neuron_id)
        .await
        .context("Failed to get neuron")?;

    // Output full response as JSON
    let json =
        serde_json::to_string_pretty(&neuron).context("Failed to serialize neuron to JSON")?;
    println!();
    println!("{}", json);

    Ok(())
}

/// Handle mint-icp command
pub async fn handle_mint_icp(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get receiver principal (select participant or custom if not provided)
    let receiver_principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse receiver principal")?
    } else {
        select_participant_or_custom()?
    };

    // Get minting account balance to show user
    use crate::core::ops::governance_ops::get_minting_account_balance;
    let minting_balance = get_minting_account_balance()
        .await
        .context("Failed to get minting account balance")?;
    let minting_balance_icp = minting_balance as f64 / 100_000_000.0;

    // Step 2: Get amount (interactive if not provided)
    let amount_e8s = if args.len() >= 4 {
        args[3]
            .parse::<u64>()
            .context("Failed to parse amount_e8s")?
    } else {
        print_header("Mint ICP");
        print_info(&format!("Receiver: {}", receiver_principal));
        print_info(&format!(
            "Available balance: {} e8s ({:.8} ICP)",
            minting_balance, minting_balance_icp
        ));
        println!();
        print!("Enter amount in e8s (e.g., 100000000 for 1 ICP): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input
            .trim()
            .parse::<u64>()
            .context("Failed to parse amount - must be a number")?
    };

    print_header("Minting ICP");
    print_info(&format!("Receiver: {}", receiver_principal));
    print_info(&format!(
        "Available balance: {} e8s ({:.8} ICP)",
        minting_balance, minting_balance_icp
    ));
    let icp_amount = amount_e8s as f64 / 100_000_000.0;
    print_info(&format!(
        "Amount: {} e8s ({:.8} ICP)",
        amount_e8s, icp_amount
    ));

    let block_height = mint_icp_default_path(receiver_principal, amount_e8s)
        .await
        .context("Failed to mint ICP")?;

    print_success(&format!(
        "ICP minted successfully! Transfer block height: {}",
        block_height
    ));
    Ok(())
}

/// Handle create-icp-neuron command
pub async fn handle_create_icp_neuron(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get principal (select participant or custom if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Get ICP balance for the principal to show available amount
    use crate::core::ops::identity::create_agent;
    use crate::core::utils::constants::LEDGER_CANISTER;
    let anonymous_identity = ic_agent::identity::AnonymousIdentity;
    let agent_for_balance = create_agent(Box::new(anonymous_identity))
        .await
        .context("Failed to create agent for balance query")?;

    let ledger_canister =
        Principal::from_text(LEDGER_CANISTER).context("Failed to parse ICP Ledger canister ID")?;

    let icp_balance = get_icp_ledger_balance(&agent_for_balance, ledger_canister, principal, None)
        .await
        .context("Failed to get ICP balance")?;
    let icp_balance_display = icp_balance as f64 / 100_000_000.0;

    use crate::core::utils::constants::ICP_TRANSFER_FEE;
    let available_after_fee = if icp_balance > ICP_TRANSFER_FEE {
        icp_balance - ICP_TRANSFER_FEE
    } else {
        0
    };
    let available_after_fee_display = available_after_fee as f64 / 100_000_000.0;

    // Step 2: Get amount (interactive if not provided)
    let amount_e8s = if args.len() >= 4 {
        args[3]
            .parse::<u64>()
            .context("Failed to parse amount_e8s")?
    } else {
        print_header("Create ICP Neuron");
        print_info(&format!("Principal: {}", principal));
        print_info(&format!(
            "Available balance: {} e8s ({:.8} ICP)",
            icp_balance, icp_balance_display
        ));
        print_info(&format!(
            "Transfer fee: {} e8s ({:.8} ICP)",
            ICP_TRANSFER_FEE,
            ICP_TRANSFER_FEE as f64 / 100_000_000.0
        ));
        if available_after_fee > 0 {
            print_info(&format!(
                "Available after fee: {} e8s ({:.8} ICP)",
                available_after_fee, available_after_fee_display
            ));
        }
        println!();
        print!(
            "Enter amount in e8s to stake (e.g., 100000000 for 1 ICP, or press Enter to use all available): "
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            // Use all available balance after fee
            if available_after_fee == 0 {
                anyhow::bail!(
                    "Insufficient balance. Need at least {} e8s (transfer fee) + amount to stake",
                    ICP_TRANSFER_FEE
                );
            }
            available_after_fee
        } else {
            input
                .parse::<u64>()
                .context("Failed to parse amount - must be a number")?
        }
    };

    // Step 3: Get optional memo
    let memo = if args.len() >= 5 {
        Some(args[4].parse::<u64>().context("Failed to parse memo")?)
    } else {
        None
    };

    // Step 4: Get optional dissolve delay (interactive if not provided)
    let dissolve_delay_seconds = if args.len() >= 6 {
        let delay = args[5]
            .parse::<u64>()
            .context("Failed to parse dissolve_delay_seconds")?;
        if delay > 0 { Some(delay) } else { None }
    } else {
        // Interactive prompt for dissolve delay
        println!();
        print!("Enter dissolve delay in seconds (or press Enter to skip, default: 0): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            None // No dissolve delay
        } else {
            let delay: u64 = input
                .parse()
                .context("Failed to parse dissolve delay - must be a number")?;
            if delay > 0 { Some(delay) } else { None }
        }
    };

    // Get existing neuron count to show what memo will be used
    let existing_neurons = list_icp_neurons_for_principal_default_path(principal)
        .await
        .context("Failed to list existing neurons")?;
    let neuron_count = existing_neurons.len();
    let auto_memo = (neuron_count + 1) as u64;

    if args.len() >= 4 {
        // Show header if amount was provided via args
        print_header("Creating ICP Neuron");
        print_info(&format!("Principal: {}", principal));
        print_info(&format!("Existing neurons: {}", neuron_count));
        let icp_amount = amount_e8s as f64 / 100_000_000.0;
        print_info(&format!(
            "Amount: {} e8s ({:.8} ICP)",
            amount_e8s, icp_amount
        ));
        if let Some(m) = memo {
            print_info(&format!("Memo: {} (specified)", m));
        } else {
            print_info(&format!("Memo: {} (auto: neuron count + 1)", auto_memo));
        }
        if let Some(delay) = dissolve_delay_seconds {
            print_info(&format!("Dissolve delay: {} seconds", delay));
        } else {
            print_info("Dissolve delay: 0 seconds (none)");
        }
    } else {
        // Amount was entered interactively, show memo and dissolve delay info
        print_info(&format!("Existing neurons: {}", neuron_count));
        if let Some(m) = memo {
            print_info(&format!("Memo: {} (specified)", m));
        } else {
            print_info(&format!("Memo: {} (auto: neuron count + 1)", auto_memo));
        }
        if let Some(delay) = dissolve_delay_seconds {
            print_info(&format!("Dissolve delay: {} seconds", delay));
        } else {
            print_info("Dissolve delay: 0 seconds (none)");
        }
    }

    // Use auto-assigned memo if not specified
    let final_memo = memo.unwrap_or(auto_memo);

    let neuron_id = create_icp_neuron_default_path(
        principal,
        amount_e8s,
        Some(final_memo),
        dissolve_delay_seconds,
    )
    .await
    .context("Failed to create ICP neuron")?;

    print_success(&format!(
        "ICP neuron created successfully! Neuron ID: {}",
        neuron_id
    ));
    Ok(())
}

/// Handle list-icp-neurons command
pub async fn handle_list_icp_neurons(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get principal (select participant or custom if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    print_header("Listing ICP Neurons");
    print_info(&format!("Principal: {}", principal));

    let neurons = list_icp_neurons_for_principal_default_path(principal)
        .await
        .context("Failed to list ICP neurons")?;

    if neurons.is_empty() {
        print_warning("No neurons found for this principal");
        return Ok(());
    }

    print_success(&format!("Found {} neuron(s)", neurons.len()));
    println!();

    // Print table header
    println!("{:-<100}", "");
    println!(
        "{:<5} {:<20} {:<20} {:<25} {:<30}",
        "#", "Neuron ID", "Stake (e8s)", "Dissolve Delay", "Hotkeys"
    );
    println!("{:-<100}", "");

    for (index, neuron) in neurons.iter().enumerate() {
        // Neuron ID - ICP uses u64 IDs
        let neuron_id_display = if let Some(id) = &neuron.id {
            id.id.to_string()
        } else {
            "<none>".to_string()
        };

        // Stake
        let stake_str = format!("{}", neuron.cached_neuron_stake_e8s);

        // Dissolve delay
        let dissolve_delay_str = match &neuron.dissolve_state {
            Some(super::super::declarations::icp_governance::DissolveState::DissolveDelaySeconds(seconds)) => {
                let days = *seconds / 86400;
                format!("{} days ({}s)", days, seconds)
            }
            Some(super::super::declarations::icp_governance::DissolveState::WhenDissolvedTimestampSeconds(timestamp)) => {
                format!("Dissolving (dissolves at {})", timestamp)
            }
            None => "No state".to_string(),
        };

        // Hotkeys
        let hotkeys_str = if neuron.hot_keys.is_empty() {
            "None".to_string()
        } else {
            format!("{} hotkey(s)", neuron.hot_keys.len())
        };

        // Truncate dissolve delay if too long for table formatting
        let dissolve_delay_display = if dissolve_delay_str.len() > 18 {
            format!("{}...", &dissolve_delay_str[..18])
        } else {
            dissolve_delay_str
        };

        println!(
            "{:<5} {:<20} {:<20} {:<25} {:<30}",
            index + 1,
            neuron_id_display,
            stake_str,
            dissolve_delay_display,
            hotkeys_str
        );
    }

    println!("{:-<100}", "");
    println!();

    // Ask if user wants to see details for a specific neuron
    if neurons.len() > 0 {
        println!();
        print!(
            "Enter neuron number to see full details (1-{}) or press Enter to skip: ",
            neurons.len()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection = input.trim();

        if !selection.is_empty() {
            let selection_num: usize = selection
                .parse()
                .context("Invalid selection - must be a number")?;
            if selection_num < 1 || selection_num > neurons.len() {
                eprintln!(
                    "Invalid selection. Please choose a number between 1 and {}",
                    neurons.len()
                );
                return Ok(());
            }

            let selected_neuron = &neurons[selection_num - 1];
            display_icp_neuron_details(selected_neuron);
        }
    }

    Ok(())
}

/// Display full details for a single ICP neuron
fn display_icp_neuron_details(neuron: &crate::core::declarations::icp_governance::Neuron) {
    use crate::core::declarations::icp_governance::DissolveState;

    print_header("ICP Neuron Details");

    // Neuron ID
    if let Some(id) = &neuron.id {
        print_info(&format!("Neuron ID: {}", id.id));
    } else {
        print_info("Neuron ID: <none>");
    }

    // Controller
    if let Some(controller) = &neuron.controller {
        print_info(&format!("Controller: {}", controller));
    }

    // Stake information
    println!();
    print_info("Stake Information:");
    println!("  Cached Stake: {} e8s", neuron.cached_neuron_stake_e8s);
    if let Some(staked_maturity) = neuron.staked_maturity_e8s_equivalent {
        println!("  Staked Maturity: {} e8s", staked_maturity);
    }
    println!("  Maturity: {} e8s", neuron.maturity_e8s_equivalent);

    // Dissolve state
    println!();
    print_info("Dissolve State:");
    match &neuron.dissolve_state {
        Some(DissolveState::DissolveDelaySeconds(seconds)) => {
            let days = *seconds / 86400;
            let hours = (*seconds % 86400) / 3600;
            println!("  Type: Dissolve Delay");
            println!(
                "  Delay: {} seconds ({} days, {} hours)",
                seconds, days, hours
            );
        }
        Some(DissolveState::WhenDissolvedTimestampSeconds(timestamp)) => {
            println!("  Type: Dissolving");
            println!("  Dissolves at timestamp: {}", timestamp);
        }
        None => {
            println!("  Type: None");
        }
    }

    // Aging
    println!();
    print_info("Aging:");
    println!(
        "  Aging since timestamp: {}",
        neuron.aging_since_timestamp_seconds
    );
    println!("  Created timestamp: {}", neuron.created_timestamp_seconds);

    // Voting power
    println!();
    if let Some(voting_power) = neuron.deciding_voting_power {
        print_info(&format!("Deciding Voting Power: {} e8s", voting_power));
    }
    if let Some(potential_power) = neuron.potential_voting_power {
        print_info(&format!("Potential Voting Power: {} e8s", potential_power));
    }

    // Hotkeys
    println!();
    print_info("Hotkeys:");
    if neuron.hot_keys.is_empty() {
        println!("  None");
    } else {
        for (i, hotkey) in neuron.hot_keys.iter().enumerate() {
            println!("  [{}] {}", i + 1, hotkey);
        }
    }

    // Visibility
    if let Some(visibility) = neuron.visibility {
        println!();
        print_info(&format!(
            "Visibility: {}",
            if visibility == 0 { "Public" } else { "Private" }
        ));
    }

    // KYC
    println!();
    print_info(&format!("KYC Verified: {}", neuron.kyc_verified));

    // Auto stake maturity
    if let Some(auto_stake) = neuron.auto_stake_maturity {
        println!();
        print_info(&format!("Auto Stake Maturity: {}", auto_stake));
    }

    println!();
}

/// Handle get-icp-balance command
pub async fn handle_get_icp_balance(args: &[String]) -> Result<()> {
    use crate::core::ops::identity::create_agent;
    use crate::core::utils::constants::LEDGER_CANISTER;

    // Step 1: Get principal (select participant or custom if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2: Get subaccount (optional)
    let subaccount = if args.len() >= 4 {
        let hex_str = args[3].strip_prefix("0x").unwrap_or(&args[3]);
        Some(hex::decode(hex_str).context("Failed to decode subaccount from hex")?)
    } else {
        None
    };

    print_header("Get ICP Balance");
    print_info(&format!("Principal: {}", principal));
    if let Some(ref sub) = subaccount {
        let hex_sub = hex::encode(sub);
        if hex_sub.len() >= 15 {
            print_info(&format!(
                "Subaccount: {}...{}",
                &hex_sub[..7],
                &hex_sub[hex_sub.len() - 8..]
            ));
        } else {
            print_info(&format!("Subaccount: {}", hex_sub));
        }
    } else {
        print_info("Subaccount: None (default account)");
    }

    // Create anonymous agent for query
    let anonymous_identity = ic_agent::identity::AnonymousIdentity;
    let agent = create_agent(Box::new(anonymous_identity))
        .await
        .context("Failed to create agent")?;

    let ledger_canister =
        Principal::from_text(LEDGER_CANISTER).context("Failed to parse ICP Ledger canister ID")?;

    let balance = get_icp_ledger_balance(&agent, ledger_canister, principal, subaccount)
        .await
        .context("Failed to get ICP balance")?;

    let icp_amount = balance as f64 / 100_000_000.0;
    println!();
    print_success(&format!("Balance: {} e8s ({:.8} ICP)", balance, icp_amount));
    Ok(())
}

/// Handle get-sns-balance command
pub async fn handle_get_sns_balance(args: &[String]) -> Result<()> {
    use crate::core::ops::identity::create_agent;
    use crate::core::utils::data_output;

    // Read deployment data to get ledger canister ID
    let deployment_path = data_output::get_output_path();
    let data_content =
        std::fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
    let deployment_data: data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    let ledger_canister = deployment_data
        .deployed_sns
        .ledger_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse ledger canister ID from deployment data")?;

    // Step 1: Get principal (select participant or custom if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2: Get subaccount (optional)
    let subaccount = if args.len() >= 4 {
        let hex_str = args[3].strip_prefix("0x").unwrap_or(&args[3]);
        Some(hex::decode(hex_str).context("Failed to decode subaccount from hex")?)
    } else {
        None
    };

    print_header("Get SNS Balance");
    print_info(&format!("Ledger Canister: {}", ledger_canister));
    print_info(&format!("Principal: {}", principal));
    if let Some(ref sub) = subaccount {
        let hex_sub = hex::encode(sub);
        if hex_sub.len() >= 15 {
            print_info(&format!(
                "Subaccount: {}...{}",
                &hex_sub[..7],
                &hex_sub[hex_sub.len() - 8..]
            ));
        } else {
            print_info(&format!("Subaccount: {}", hex_sub));
        }
    } else {
        print_info("Subaccount: None (default account)");
    }

    // Create anonymous agent for query
    let anonymous_identity = ic_agent::identity::AnonymousIdentity;
    let agent = create_agent(Box::new(anonymous_identity))
        .await
        .context("Failed to create agent")?;

    let balance = get_sns_ledger_balance(&agent, ledger_canister, principal, subaccount)
        .await
        .context("Failed to get SNS balance")?;

    // Convert to token amount (assuming 8 decimals like ICP)
    let token_amount = balance as f64 / 100_000_000.0;
    println!();
    print_success(&format!(
        "Balance: {} e8s ({:.8} tokens)",
        balance, token_amount
    ));
    Ok(())
}

/// Handle mint-sns-tokens command
pub async fn handle_mint_sns_tokens(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get proposer principal (select participant or custom if not provided)
    let proposer_principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse proposer principal")?
    } else {
        select_participant_or_custom_with_label(Some("Select Proposer Principal:"))?
    };

    // Step 2: Get receiver_principal (select participant or custom if not provided)
    let receiver_principal = if args.len() >= 4 {
        Principal::from_text(&args[3]).context("Failed to parse receiver principal")?
    } else {
        select_participant_or_custom_with_label(Some("Select Receiver Principal:"))?
    };

    // Step 3: Get amount_e8s
    let amount_e8s = if args.len() >= 5 {
        args[4]
            .parse::<u64>()
            .context("Failed to parse amount_e8s")?
    } else {
        print!("Enter amount to mint (in e8s, e.g., 100000000 = 1 token): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input
            .trim()
            .parse::<u64>()
            .context("Failed to parse amount_e8s")?
    };

    print_header("Minting SNS Tokens");
    print_info(&format!("Proposer: {}", proposer_principal));
    print_info(&format!("Receiver: {}", receiver_principal));
    print_info(&format!("Amount: {} e8s", amount_e8s));
    print_info("Creating proposal and getting all neurons to vote...");

    let proposal_id = mint_sns_tokens_with_all_votes_default_path(
        proposer_principal,
        receiver_principal,
        amount_e8s,
    )
    .await
    .context("Failed to mint tokens")?;

    print_success(&format!(
        "Proposal created successfully! Proposal ID: {}",
        proposal_id
    ));
    print_info("All participant neurons have voted on the proposal.");
    Ok(())
}

/// Handle create-sns-neuron command
pub async fn handle_create_sns_neuron(args: &[String]) -> Result<()> {
    use crate::core::ops::identity::create_agent;
    use crate::core::ops::sns_governance_ops::get_neuron_minimum_stake;
    use crate::core::utils::data_output::get_output_path;
    use std::fs;

    // Read deployment data to get governance canister ID
    let deployment_path = get_output_path();
    let data_content =
        fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    let governance_canister = deployment_data
        .deployed_sns
        .governance_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse governance canister ID from deployment data")?;

    // Get minimum stake (using anonymous identity for query)
    let anonymous_identity = ic_agent::identity::AnonymousIdentity;
    let agent = create_agent(Box::new(anonymous_identity))
        .await
        .context("Failed to create agent")?;
    let minimum_stake = get_neuron_minimum_stake(&agent, governance_canister)
        .await
        .context("Failed to get minimum stake")?;

    // Step 1: Get principal (select participant if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Get balance and fee to show user options
    use crate::core::ops::ledger_ops::{get_sns_ledger_balance, get_sns_ledger_fee};
    let ledger_canister = deployment_data
        .deployed_sns
        .ledger_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse ledger canister ID from deployment data")?;

    let balance = get_sns_ledger_balance(&agent, ledger_canister, principal, None)
        .await
        .context("Failed to get SNS ledger balance")?;
    let transfer_fee = get_sns_ledger_fee(&agent, ledger_canister)
        .await
        .context("Failed to get SNS ledger transfer fee")?;

    // Step 2: Get optional amount (interactive if not provided)
    use std::io::{self, Write};
    let amount_e8s = if args.len() >= 4 {
        Some(
            args[3]
                .parse::<u64>()
                .context("Failed to parse amount_e8s")?,
        )
    } else {
        // Interactive prompt for amount
        print_header("Creating SNS Neuron");
        print_info(&format!("Principal: {}", principal));
        print_info(&format!("Available balance: {} e8s", balance));
        print_info(&format!("Transfer fee: {} e8s", transfer_fee));
        print_info(&format!("Minimum stake required: {} e8s", minimum_stake));
        let max_available = if balance > transfer_fee {
            balance - transfer_fee
        } else {
            0
        };
        if max_available >= minimum_stake {
            print_info(&format!(
                "Maximum stakeable (balance - fee): {} e8s",
                max_available
            ));
        }
        println!();
        print!(
            "Enter amount to stake in e8s (or press Enter to use maximum: {} e8s): ",
            max_available
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            // Use maximum available
            if max_available < minimum_stake {
                anyhow::bail!(
                    "Insufficient balance: maximum available {} e8s is less than minimum stake {} e8s",
                    max_available,
                    minimum_stake
                );
            }
            None // Will be handled in create_sns_neuron as "all available"
        } else {
            let amount: u64 = input
                .parse()
                .context("Failed to parse amount - must be a number")?;
            Some(amount)
        }
    };

    // Step 3: Get optional memo
    let memo = if args.len() >= 5 {
        Some(args[4].parse::<u64>().context("Failed to parse memo")?)
    } else {
        None
    };

    // Step 4: Get optional dissolve delay (interactive if not provided)
    let dissolve_delay_seconds = if args.len() >= 6 {
        let delay = args[5]
            .parse::<u64>()
            .context("Failed to parse dissolve_delay_seconds")?;
        if delay > 0 { Some(delay) } else { None }
    } else {
        // Interactive prompt for dissolve delay
        println!();
        print!("Enter dissolve delay in seconds (or press Enter to skip, default: 0): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            None // No dissolve delay
        } else {
            let delay: u64 = input
                .parse()
                .context("Failed to parse dissolve delay - must be a number")?;
            if delay > 0 { Some(delay) } else { None }
        }
    };

    // Get existing neuron count to show what memo will be used
    let existing_neurons = list_neurons_for_principal_default_path(principal)
        .await
        .context("Failed to list existing neurons")?;
    let neuron_count = existing_neurons.len();
    let auto_memo = neuron_count + 1;

    if args.len() >= 4 {
        // Show header if amount was provided via args
        print_header("Creating SNS Neuron");
        print_info(&format!("Principal: {}", principal));
        print_info(&format!("Existing neurons: {}", neuron_count));
        print_info(&format!("Minimum stake required: {} e8s", minimum_stake));
        if let Some(amount) = amount_e8s {
            print_info(&format!("Amount: {} e8s", amount));
        }
        if let Some(m) = memo {
            print_info(&format!("Memo: {} (specified)", m));
        } else {
            print_info(&format!("Memo: {} (auto: neuron count + 1)", auto_memo));
        }
        if let Some(delay) = dissolve_delay_seconds {
            print_info(&format!("Dissolve delay: {} seconds", delay));
        } else {
            print_info("Dissolve delay: 0 seconds (none)");
        }
    } else {
        // Amount was entered interactively, show memo and dissolve delay info
        print_info(&format!("Existing neurons: {}", neuron_count));
        if let Some(m) = memo {
            print_info(&format!("Memo: {} (specified)", m));
        } else {
            print_info(&format!("Memo: {} (auto: neuron count + 1)", auto_memo));
        }
        if let Some(delay) = dissolve_delay_seconds {
            print_info(&format!("Dissolve delay: {} seconds", delay));
        } else {
            print_info("Dissolve delay: 0 seconds (none)");
        }
    }

    let neuron_id =
        create_sns_neuron_default_path(principal, amount_e8s, memo, dissolve_delay_seconds)
            .await
            .context("Failed to create SNS neuron")?;

    let hex_id = hex::encode(&neuron_id);
    print_success(&format!(
        "SNS neuron created successfully! Neuron ID: {}",
        hex_id
    ));
    Ok(())
}

/// Handle disburse-sns-neuron command
pub async fn handle_disburse_sns_neuron(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get participant principal (select if not provided)
    let participant_principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse participant principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2 & 3: Get neuron_id and receiver_principal
    let (neuron_id, receiver_principal) = if args.len() >= 4 {
        let arg3 = &args[3];
        // Check if arg3 looks like a neuron_id (hex string)
        let looks_like_neuron_id = (arg3.starts_with("0x") && arg3.len() > 10)
            || (!arg3.contains('-') // Principal contains dashes
                && arg3.chars().all(|c| c.is_ascii_hexdigit())
                && arg3.len() > 8);

        if looks_like_neuron_id {
            // arg3 is neuron_id
            let hex_str = arg3.strip_prefix("0x").unwrap_or(arg3);
            let neuron_id_val =
                Some(hex::decode(hex_str).context("Failed to decode neuron_id from hex")?);

            // Get receiver_principal from next arg
            let receiver = if args.len() >= 5 {
                Principal::from_text(&args[4]).context("Failed to parse receiver principal")?
            } else {
                print!("Enter receiver principal: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                Principal::from_text(input.trim()).context("Failed to parse receiver principal")?
            };

            (neuron_id_val, receiver)
        } else {
            // arg3 is receiver_principal, need to select neuron
            let receiver =
                Principal::from_text(arg3).context("Failed to parse receiver principal")?;
            let neuron_id_val = select_neuron(participant_principal).await?;
            (Some(neuron_id_val), receiver)
        }
    } else {
        // Need to select neuron and get receiver interactively
        let neuron_id_val = select_neuron(participant_principal).await?;

        print!("Enter receiver principal: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let receiver =
            Principal::from_text(input.trim()).context("Failed to parse receiver principal")?;

        (Some(neuron_id_val), receiver)
    };

    print_header("Disbursing SNS Neuron");
    print_info(&format!("Participant: {}", participant_principal));
    print_info(&format!("Receiver: {}", receiver_principal));
    if let Some(id) = &neuron_id {
        let hex_id = hex::encode(id);
        if hex_id.len() >= 15 {
            print_info(&format!(
                "Neuron ID: {}...{}",
                &hex_id[..7],
                &hex_id[hex_id.len() - 8..]
            ));
        } else {
            print_info(&format!("Neuron ID: {}", hex_id));
        }
    } else {
        print_info("Neuron ID: Auto-selecting (lowest dissolve delay)");
    }
    print_info("Amount: Full neuron stake");

    let block_height = disburse_participant_neuron_default_path(
        participant_principal,
        receiver_principal,
        neuron_id,
    )
    .await
    .context("Failed to disburse neuron")?;

    print_success(&format!(
        "Neuron disbursed successfully! Transfer block height: {}",
        block_height
    ));
    Ok(())
}

fn print_add_hotkey_usage(program_name: &str) {
    eprintln!("Usage: {} add-hotkey <neuron_type> <...>", program_name);
    eprintln!("\nNeuron types:");
    eprintln!("  sns  - SNS neuron (from deployed SNS)");
    eprintln!("  icp  - ICP neuron (ICP Governance)");
    eprintln!("\nFor SNS neurons:");
    eprintln!(
        "  Usage: {} add-hotkey sns [owner_principal] [neuron_id_hex|hotkey_principal] [hotkey_principal|permissions] [permissions]",
        program_name
    );
    eprintln!("  owner_principal - Optional: Principal of the participant who owns the neuron");
    eprintln!("                    If not provided, shows participant selection menu");
    eprintln!("  neuron_id_hex   - Optional: Neuron ID in hex format");
    eprintln!("                    If not provided, shows neuron selection menu");
    eprintln!("  hotkey_principal - Required: Principal to add as a hotkey");
    eprintln!("                     If not provided as argument, prompts interactively");
    eprintln!(
        "  permissions    - Optional: comma-separated permission types (default: 3,4 = SubmitProposal + Vote)"
    );
    eprintln!(
        "                   Permission types: 2=ManagePrincipals, 3=SubmitProposal, 4=Vote, etc."
    );
    eprintln!("\nInteractive flow:");
    eprintln!("  1. Select participant (if owner_principal not provided)");
    eprintln!("  2. Select neuron (if neuron_id_hex not provided)");
    eprintln!("  3. Enter hotkey principal (if not provided as argument)");
    eprintln!("\nFor ICP neurons:");
    eprintln!(
        "  Usage: {} add-hotkey icp [hotkey_principal]",
        program_name
    );
    eprintln!("  hotkey_principal - Optional: Principal to add as a hotkey");
    eprintln!("                     If not provided, prompts interactively");
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

/// Handle increase-sns-dissolve-delay command
pub async fn handle_increase_sns_dissolve_delay(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get participant principal (select participant or custom if not provided)
    let participant_principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse participant principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2: Get neuron ID (select if not provided)
    let neuron_id = if args.len() >= 4 {
        let hex_str = args[3].strip_prefix("0x").unwrap_or(&args[3]);
        Some(hex::decode(hex_str).context("Failed to decode neuron_id from hex")?)
    } else {
        // Interactive neuron selection
        Some(select_neuron(participant_principal).await?)
    };

    // Step 3: Get additional dissolve delay (interactive if not provided)
    let additional_dissolve_delay_seconds = if args.len() >= 5 {
        args[4]
            .parse::<u64>()
            .context("Failed to parse additional_dissolve_delay_seconds")?
    } else {
        // Interactive prompt
        print_header("Increase SNS Neuron Dissolve Delay");
        print_info(&format!("Participant: {}", participant_principal));
        if let Some(ref id) = neuron_id {
            let hex_id = hex::encode(id);
            if hex_id.len() >= 15 {
                print_info(&format!(
                    "Neuron ID: {}...{}",
                    &hex_id[..7],
                    &hex_id[hex_id.len() - 8..]
                ));
            } else {
                print_info(&format!("Neuron ID: {}", hex_id));
            }
        }
        println!();
        print!("Enter additional dissolve delay in seconds (e.g., 2592000 for 30 days): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input
            .trim()
            .parse::<u64>()
            .context("Failed to parse dissolve delay - must be a number")?
    };

    print_header("Increasing Dissolve Delay");
    print_info(&format!("Participant: {}", participant_principal));
    if let Some(ref id) = neuron_id {
        let hex_id = hex::encode(id);
        if hex_id.len() >= 15 {
            print_info(&format!(
                "Neuron ID: {}...{}",
                &hex_id[..7],
                &hex_id[hex_id.len() - 8..]
            ));
        } else {
            print_info(&format!("Neuron ID: {}", hex_id));
        }
    }
    let days = additional_dissolve_delay_seconds / 86400;
    let hours = (additional_dissolve_delay_seconds % 86400) / 3600;
    print_info(&format!(
        "Additional Delay: {} seconds ({} days, {} hours)",
        additional_dissolve_delay_seconds, days, hours
    ));

    increase_dissolve_delay_participant_neuron_default_path(
        participant_principal,
        additional_dissolve_delay_seconds,
        neuron_id,
    )
    .await
    .context("Failed to increase dissolve delay")?;

    print_success("Dissolve delay increased successfully!");
    Ok(())
}

/// Handle manage-sns-dissolving command
pub async fn handle_manage_sns_dissolving(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get participant principal (select if not provided)
    let participant_principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse participant principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2: Get action (start/stop) - interactive if not provided
    let start_dissolving = if args.len() >= 4 {
        match args[3].to_lowercase().as_str() {
            "start" | "true" | "1" => true,
            "stop" | "false" | "0" => false,
            _ => {
                anyhow::bail!("Invalid action. Use 'start' or 'stop'");
            }
        }
    } else {
        // Interactive prompt
        print_header("Manage SNS Neuron Dissolving State");
        print_info(&format!("Participant: {}", participant_principal));
        println!();
        println!("Actions:");
        println!("  [1] Start Dissolving");
        println!("  [2] Stop Dissolving");
        println!();
        print!("Select action (1 or 2): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection = input.trim().to_lowercase();

        match selection.as_str() {
            "1" | "start" => true,
            "2" | "stop" => false,
            _ => {
                anyhow::bail!("Invalid selection. Use 1 to start or 2 to stop dissolving.");
            }
        }
    };

    // Step 3: Get neuron ID (select if not provided)
    let neuron_id = if args.len() >= 5 {
        let hex_str = args[4].strip_prefix("0x").unwrap_or(&args[4]);
        Some(hex::decode(hex_str).context("Failed to decode neuron_id from hex")?)
    } else {
        // Interactive neuron selection
        Some(select_neuron(participant_principal).await?)
    };

    print_header(if start_dissolving {
        "Starting Dissolving"
    } else {
        "Stopping Dissolving"
    });
    print_info(&format!("Participant: {}", participant_principal));
    if let Some(ref id) = neuron_id {
        let hex_id = hex::encode(id);
        if hex_id.len() >= 15 {
            print_info(&format!(
                "Neuron ID: {}...{}",
                &hex_id[..7],
                &hex_id[hex_id.len() - 8..]
            ));
        } else {
            print_info(&format!("Neuron ID: {}", hex_id));
        }
    }

    manage_dissolving_state_participant_neuron_default_path(
        participant_principal,
        start_dissolving,
        neuron_id,
    )
    .await
    .context("Failed to manage dissolving state")?;

    print_success(if start_dissolving {
        "Neuron is now dissolving!"
    } else {
        "Neuron dissolving stopped!"
    });
    Ok(())
}

/// Handle check-sns-deployed command
/// Returns exit code 0 if deployed, 1 if not deployed
pub async fn handle_check_sns_deployed(_args: &[String]) -> Result<()> {
    let deployed = check_sns_deployed_default_path()
        .await
        .context("Failed to check SNS deployment status")?;

    if deployed {
        // Exit with 0 if deployed
        std::process::exit(0);
    } else {
        // Exit with 1 if not deployed
        std::process::exit(1);
    }
}

/// Select an ICP neuron interactively from a list
async fn select_icp_neuron(principal: Principal) -> Result<u64> {
    use crate::core::ops::governance_ops::list_icp_neurons_for_principal_default_path;
    use std::io::{self, Write};

    let neurons = list_icp_neurons_for_principal_default_path(principal)
        .await
        .context("Failed to list ICP neurons")?;

    if neurons.is_empty() {
        anyhow::bail!("No ICP neurons found for principal {}", principal);
    }

    println!("\nAvailable ICP Neurons:");
    for (idx, neuron) in neurons.iter().enumerate() {
        let neuron_id = neuron.id.as_ref().map(|n| n.id).unwrap_or(0);
        let stake = neuron.cached_neuron_stake_e8s;
        let dissolve_delay = match &neuron.dissolve_state {
            Some(crate::core::declarations::icp_governance::DissolveState::DissolveDelaySeconds(
                seconds,
            )) => *seconds,
            Some(crate::core::declarations::icp_governance::DissolveState::WhenDissolvedTimestampSeconds(_)) => {
                0 // Dissolving
            }
            None => u64::MAX, // No state
        };

        let days = dissolve_delay / 86400;
        let hours = (dissolve_delay % 86400) / 3600;
        let dissolve_str = if dissolve_delay == 0 {
            "Dissolving".to_string()
        } else if dissolve_delay == u64::MAX {
            "No delay".to_string()
        } else {
            format!("{}d {}h", days, hours)
        };

        println!(
            "  {}: Neuron ID: {}, Stake: {} e8s, Dissolve Delay: {}",
            idx + 1,
            neuron_id,
            stake,
            dissolve_str
        );
    }

    print!("\nSelect neuron number (1-{}): ", neurons.len());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input
        .trim()
        .parse()
        .context("Invalid selection - must be a number")?;

    if selection < 1 || selection > neurons.len() {
        anyhow::bail!(
            "Invalid selection - must be between 1 and {}",
            neurons.len()
        );
    }

    let selected_neuron = &neurons[selection - 1];
    selected_neuron
        .id
        .as_ref()
        .map(|n| n.id)
        .ok_or_else(|| anyhow::anyhow!("Selected neuron has no ID"))
}

/// Handle disburse-icp-neuron command
pub async fn handle_disburse_icp_neuron(args: &[String]) -> Result<()> {
    use crate::core::ops::governance_ops::disburse_icp_neuron_for_principal_default_path;
    use std::io::{self, Write};

    // Step 1: Get principal (select participant or custom if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2 & 3: Get neuron_id and receiver_principal
    let (neuron_id, receiver_principal) = if args.len() >= 4 {
        let arg3 = &args[3];
        // Check if arg3 looks like a neuron_id (number)
        if let Ok(id) = arg3.parse::<u64>() {
            // arg3 is neuron_id
            let neuron_id_val = Some(id);

            // Get receiver_principal from next arg
            let receiver = if args.len() >= 5 {
                Principal::from_text(&args[4]).context("Failed to parse receiver principal")?
            } else {
                print!("Enter receiver principal: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                Principal::from_text(input.trim()).context("Failed to parse receiver principal")?
            };

            (neuron_id_val, receiver)
        } else {
            // arg3 is receiver_principal, need to select neuron
            let receiver =
                Principal::from_text(arg3).context("Failed to parse receiver principal")?;
            let neuron_id_val = Some(select_icp_neuron(principal).await?);
            (neuron_id_val, receiver)
        }
    } else {
        // Need to select neuron and get receiver interactively
        let neuron_id_val = Some(select_icp_neuron(principal).await?);

        print!("Enter receiver principal: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let receiver =
            Principal::from_text(input.trim()).context("Failed to parse receiver principal")?;

        (neuron_id_val, receiver)
    };

    // Step 4: Get amount (optional)
    let amount_e8s = if args.len() >= 6 {
        Some(
            args[5]
                .parse::<u64>()
                .context("Failed to parse amount_e8s")?,
        )
    } else {
        None // Full disbursement
    };

    print_header("Disbursing ICP Neuron");
    print_info(&format!("Principal: {}", principal));
    print_info(&format!("Receiver: {}", receiver_principal));
    if let Some(id) = neuron_id {
        print_info(&format!("Neuron ID: {}", id));
    }
    if let Some(amount) = amount_e8s {
        print_info(&format!("Amount: {} e8s", amount));
    } else {
        print_info("Amount: Full neuron stake");
    }

    let block_height = disburse_icp_neuron_for_principal_default_path(
        principal,
        receiver_principal,
        neuron_id,
        amount_e8s,
    )
    .await
    .context("Failed to disburse neuron")?;

    print_success(&format!(
        "Neuron disbursed successfully! Transfer block height: {}",
        block_height
    ));
    Ok(())
}

/// Handle increase-icp-dissolve-delay command
pub async fn handle_increase_icp_dissolve_delay(args: &[String]) -> Result<()> {
    use crate::core::ops::governance_ops::increase_icp_dissolve_delay_for_principal_default_path;
    use std::io::{self, Write};

    // Step 1: Get principal (select participant if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2: Get neuron ID (select if not provided)
    let neuron_id = if args.len() >= 4 {
        Some(
            args[3]
                .parse::<u64>()
                .context("Failed to parse neuron_id")?,
        )
    } else {
        // Interactive neuron selection
        Some(select_icp_neuron(principal).await?)
    };

    // Step 3: Get additional dissolve delay (interactive if not provided)
    let additional_dissolve_delay_seconds = if args.len() >= 5 {
        args[4]
            .parse::<u64>()
            .context("Failed to parse additional_dissolve_delay_seconds")?
    } else {
        // Interactive prompt
        print_header("Increase ICP Neuron Dissolve Delay");
        print_info(&format!("Principal: {}", principal));
        if let Some(id) = neuron_id {
            print_info(&format!("Neuron ID: {}", id));
        }
        println!();
        print!("Enter additional dissolve delay in seconds (e.g., 2592000 for 30 days): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input
            .trim()
            .parse::<u64>()
            .context("Failed to parse dissolve delay - must be a number")?
    };

    print_header("Increasing Dissolve Delay");
    print_info(&format!("Principal: {}", principal));
    if let Some(id) = neuron_id {
        print_info(&format!("Neuron ID: {}", id));
    }
    let days = additional_dissolve_delay_seconds / 86400;
    let hours = (additional_dissolve_delay_seconds % 86400) / 3600;
    print_info(&format!(
        "Additional Delay: {} seconds ({} days, {} hours)",
        additional_dissolve_delay_seconds, days, hours
    ));

    increase_icp_dissolve_delay_for_principal_default_path(
        principal,
        neuron_id,
        additional_dissolve_delay_seconds,
    )
    .await
    .context("Failed to increase dissolve delay")?;

    print_success("Dissolve delay increased successfully!");
    Ok(())
}

/// Handle manage-icp-dissolving command
pub async fn handle_manage_icp_dissolving(args: &[String]) -> Result<()> {
    use crate::core::ops::governance_ops::manage_icp_dissolving_state_for_principal_default_path;
    use std::io::{self, Write};

    // Step 1: Get principal (select participant if not provided)
    let principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse principal")?
    } else {
        select_participant_or_custom()?
    };

    // Step 2: Get action (start/stop) - interactive if not provided
    let start_dissolving = if args.len() >= 4 {
        match args[3].to_lowercase().as_str() {
            "start" | "true" | "1" => true,
            "stop" | "false" | "0" => false,
            _ => {
                anyhow::bail!("Invalid action. Use 'start' or 'stop'");
            }
        }
    } else {
        // Interactive prompt
        print_header("Manage ICP Neuron Dissolving State");
        print_info(&format!("Principal: {}", principal));
        println!();
        println!("  [1] Start Dissolving");
        println!("  [2] Stop Dissolving");
        print!("Select action [1-2]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => true,
            "2" => false,
            _ => anyhow::bail!("Invalid selection - must be 1 or 2"),
        }
    };

    // Step 3: Get neuron ID (select if not provided)
    let neuron_id = if args.len() >= 5 {
        Some(
            args[4]
                .parse::<u64>()
                .context("Failed to parse neuron_id")?,
        )
    } else {
        // Interactive neuron selection
        Some(select_icp_neuron(principal).await?)
    };

    print_header(if start_dissolving {
        "Starting Dissolving"
    } else {
        "Stopping Dissolving"
    });
    print_info(&format!("Principal: {}", principal));
    if let Some(id) = neuron_id {
        print_info(&format!("Neuron ID: {}", id));
    }

    manage_icp_dissolving_state_for_principal_default_path(principal, neuron_id, start_dissolving)
        .await
        .context("Failed to manage dissolving state")?;

    print_success(if start_dissolving {
        "Dissolving started successfully!"
    } else {
        "Dissolving stopped successfully!"
    });
    Ok(())
}
