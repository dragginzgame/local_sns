// CLI command handlers

use anyhow::{Context, Result};
use candid::Principal;
use hex;

use crate::core::ops::governance_ops::{
    add_hotkey_to_icp_neuron_default_path, get_icp_neuron_default_path,
    set_icp_neuron_visibility_default_path,
};
use crate::core::ops::sns_governance_ops::{
    add_hotkey_to_participant_neuron_default_path, create_sns_neuron_default_path,
    disburse_participant_neuron_default_path, list_neurons_for_principal_default_path,
    mint_sns_tokens_with_all_votes_default_path,
};
use crate::core::ops::snsw_ops::check_sns_deployed_default_path;
use crate::core::utils::{print_header, print_info, print_success, print_warning};

/// Helper function to select a participant interactively
fn select_participant() -> Result<Principal> {
    use crate::core::utils::data_output::SnsCreationData;
    use std::io::{self, Write};

    print_header("Select Participant");

    // Read deployment data
    let deployment_path = crate::core::utils::data_output::get_output_path();
    let data_content =
        std::fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
    let deployment_data: SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    if deployment_data.participants.is_empty() {
        anyhow::bail!("No participants found in deployment data");
    }

    println!("Available participants:");
    println!();
    for (i, participant) in deployment_data.participants.iter().enumerate() {
        println!("  [{}] {}", i + 1, participant.principal);
    }
    println!();
    print!(
        "Select participant number (1-{}): ",
        deployment_data.participants.len()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse().context("Invalid selection")?;

    if selection < 1 || selection > deployment_data.participants.len() {
        anyhow::bail!(
            "Invalid selection. Please choose a number between 1 and {}",
            deployment_data.participants.len()
        );
    }

    Principal::from_text(&deployment_data.participants[selection - 1].principal)
        .context("Failed to parse selected participant principal")
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
                select_participant()?
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

/// Handle list-sns-neurons command
pub async fn handle_list_neurons(args: &[String]) -> Result<()> {
    use crate::core::utils::data_output::SnsCreationData;
    use std::io::{self, Write};

    let principal = if args.len() < 3 {
        // No principal provided - show participant selection
        print_header("Select Participant");

        // Read deployment data
        let deployment_path = crate::core::utils::data_output::get_output_path();
        let data_content =
            std::fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
        let deployment_data: SnsCreationData =
            serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

        if deployment_data.participants.is_empty() {
            eprintln!("No participants found in deployment data");
            std::process::exit(1);
        }

        println!("Available participants:");
        println!();
        for (i, participant) in deployment_data.participants.iter().enumerate() {
            println!("  [{}] {}", i + 1, participant.principal);
        }
        println!();
        print!(
            "Select participant number (1-{}): ",
            deployment_data.participants.len()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection: usize = input.trim().parse().context("Invalid selection")?;

        if selection < 1 || selection > deployment_data.participants.len() {
            eprintln!(
                "Invalid selection. Please choose a number between 1 and {}",
                deployment_data.participants.len()
            );
            std::process::exit(1);
        }

        Principal::from_text(&deployment_data.participants[selection - 1].principal)
            .context("Failed to parse selected participant principal")?
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
    println!("{:-<90}", "");
    println!(
        "{:<20} {:<20} {:<20} {:<30}",
        "Neuron ID", "Stake (e8s)", "Dissolve Delay", "Permissions"
    );
    println!("{:-<90}", "");

    for neuron in &neurons {
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
            "{:<20} {:<20} {:<20} {:<30}",
            neuron_id_display, stake_str, dissolve_delay_display, perm_str
        );
    }

    println!("{:-<90}", "");
    println!();

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

/// Handle mint-sns-tokens command
pub async fn handle_mint_sns_tokens(args: &[String]) -> Result<()> {
    use std::io::{self, Write};

    // Step 1: Get proposer principal (select if not provided)
    let proposer_principal = if args.len() >= 3 {
        Principal::from_text(&args[2]).context("Failed to parse proposer principal")?
    } else {
        select_participant()?
    };

    // Step 2: Get receiver_principal
    let receiver_principal = if args.len() >= 4 {
        Principal::from_text(&args[3]).context("Failed to parse receiver principal")?
    } else {
        print!("Enter receiver principal: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Principal::from_text(input.trim()).context("Failed to parse receiver principal")?
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
        select_participant()?
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
    } else {
        // Amount was entered interactively, show memo info
        print_info(&format!("Existing neurons: {}", neuron_count));
        if let Some(m) = memo {
            print_info(&format!("Memo: {} (specified)", m));
        } else {
            print_info(&format!("Memo: {} (auto: neuron count + 1)", auto_memo));
        }
    }

    let neuron_id = create_sns_neuron_default_path(principal, amount_e8s, memo)
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
        select_participant()?
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
