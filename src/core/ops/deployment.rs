// Deployment orchestration functions

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use hex;
use ic_agent::Agent;
use sha2::Digest;
use std::time::Duration as StdDuration;

use crate::core::declarations::icp_ledger::Account as LedgerAccount;
use crate::core::declarations::sns_swap::GetLifecycleResponse;
use crate::core::ops::governance_ops::{claim_neuron, create_sns_proposal, set_dissolve_delay};
use crate::core::ops::identity::{create_agent, load_dfx_identity, load_minting_identity};
use crate::core::ops::ledger_ops::{generate_subaccount_by_nonce, transfer_icp};
use crate::core::ops::snsw_ops::get_deployed_sns;
use crate::core::ops::swap_ops::{
    create_sale_ticket, finalize_swap, generate_participant_subaccount, get_derived_state,
    get_swap_lifecycle, refresh_buyer_tokens,
};
use crate::core::utils::{print_header, print_info, print_step, print_success, print_warning};

use crate::core::utils::constants::*;

pub struct DeploymentContext {
    pub agent: Agent,
    pub minting_agent: Agent,
    pub owner_principal: Principal,
    pub governance_canister: Principal,
    pub ledger_canister: Principal,
    pub snsw_canister: Principal,
}

/// Initialize deployment context (load identities, create agents, parse canisters)
pub async fn initialize_deployment_context() -> Result<DeploymentContext> {
    print_step("Loading dfx identity...");
    let identity = load_dfx_identity(None)
        .context("Failed to load dfx identity. Make sure dfx is configured.")?;
    print_success("Dfx identity loaded");

    print_step("Creating agent...");
    let agent = create_agent(identity).await?;
    print_success("Agent created");

    print_step("Parsing canister principals...");
    let governance_canister = Principal::from_text(GOVERNANCE_CANISTER)
        .context("Failed to parse GOVERNANCE_CANISTER principal")?;
    let ledger_canister = Principal::from_text(LEDGER_CANISTER)
        .context("Failed to parse LEDGER_CANISTER principal")?;
    let snsw_canister =
        Principal::from_text(SNSW_CANISTER).context("Failed to parse SNSW_CANISTER principal")?;
    print_success("Canister principals parsed");

    print_step("Getting owner principal...");
    let owner_principal = agent
        .get_principal()
        .map_err(|e| anyhow::anyhow!("Failed to get principal: {e}"))?;
    print_info(&format!("Owner principal: {}", owner_principal));

    // Load minting identity
    print_header("Setting Up Minting Account");
    let minting_identity = load_minting_identity().context("Failed to load minting identity")?;
    let minting_agent = create_agent(minting_identity).await?;
    let minting_principal = minting_agent
        .get_principal()
        .map_err(|e| anyhow::anyhow!("Failed to get minting principal: {e}"))?;
    print_info(&format!("Minting principal: {}", minting_principal));

    Ok(DeploymentContext {
        agent,
        minting_agent,
        owner_principal,
        governance_canister,
        ledger_canister,
        snsw_canister,
    })
}

/// Setup minting account - transfer ICP from minting account to owner
pub async fn setup_minting_account(ctx: &DeploymentContext) -> Result<()> {
    print_step("Transferring ICP from minting account to developer...");
    let developer_icp_with_fee = DEVELOPER_ICP + ICP_TRANSFER_FEE;
    transfer_icp(
        &ctx.minting_agent,
        ctx.ledger_canister,
        ctx.owner_principal,
        developer_icp_with_fee,
        None,
    )
    .await
    .context("Failed to transfer ICP to developer")?;
    print_success("ICP transferred to developer");
    Ok(())
}

/// Create ICP neuron - transfer ICP, claim neuron
pub async fn create_icp_neuron(ctx: &DeploymentContext) -> Result<u64> {
    print_header("Creating ICP Neuron");
    print_step("Transferring ICP to governance subaccount...");

    let subaccount = generate_subaccount_by_nonce(MEMO, ctx.owner_principal);
    print_info(&format!(
        "Calculated subaccount: {}",
        hex::encode(subaccount.0)
    ));

    // Transfer ICP to governance subaccount
    transfer_icp(
        &ctx.agent,
        ctx.ledger_canister,
        ctx.governance_canister,
        DEVELOPER_ICP,
        Some(subaccount.0.to_vec()),
    )
    .await
    .context("Failed to transfer ICP to governance subaccount")?;
    print_success("ICP transferred to governance subaccount");

    // Wait a bit for the transfer to settle
    tokio::time::sleep(StdDuration::from_secs(2)).await;

    // Claim neuron
    print_step("Claiming neuron...");
    let neuron_id = claim_neuron(&ctx.agent, ctx.governance_canister, MEMO)
        .await
        .context("Failed to claim neuron")?;
    print_success(&format!("ICP neuron created with ID: {neuron_id}"));

    Ok(neuron_id)
}

/// Configure neuron - set dissolve delay
pub async fn configure_neuron(ctx: &DeploymentContext, neuron_id: u64) -> Result<()> {
    print_header("Setting Max Dissolve Delay");
    print_step("Configuring neuron dissolve delay to 8 years...");
    set_dissolve_delay(
        &ctx.agent,
        ctx.governance_canister,
        neuron_id,
        DISSOLVE_DELAY,
    )
    .await
    .context("Failed to set dissolve delay")?;
    print_success("Dissolve delay set");
    Ok(())
}

/// Create and wait for SNS proposal execution
pub async fn create_and_wait_for_proposal(
    ctx: &DeploymentContext,
    neuron_id: u64,
) -> Result<(u64, crate::core::declarations::sns_wasm::DeployedSns)> {
    // Create SNS Proposal
    print_header("Creating SNS Proposal");
    print_step("Creating SNS proposal...");
    let proposal_id = create_sns_proposal(
        &ctx.agent,
        ctx.governance_canister,
        neuron_id,
        ctx.owner_principal,
    )
    .await
    .context("Failed to create SNS proposal")?;
    print_success(&format!("Proposal created with ID: {proposal_id}"));

    // Wait for Proposal Execution
    print_header("Waiting for Proposal Execution");
    print_step(&format!("Waiting for proposal {proposal_id} to execute..."));
    print_warning(
        "Proposal execution may take time. In local dfx, you may need to manually advance time.",
    );

    // Poll for proposal execution
    let mut executed = false;
    for i in 0..60 {
        tokio::time::sleep(StdDuration::from_secs(10)).await;

        // Try to get deployed SNS
        match get_deployed_sns(&ctx.agent, ctx.snsw_canister, proposal_id).await {
            Ok(_) => {
                executed = true;
                break;
            }
            Err(_) => {
                if i % 6 == 0 {
                    print_info(&format!("Still waiting... (attempt {}/60)", i + 1));
                }
            }
        }
    }

    if !executed {
        print_warning("Proposal may not have executed automatically. Check manually.");
    } else {
        print_success("Proposal executed");
    }

    // Get Deployed SNS
    print_header("Getting Deployed SNS");
    print_step("Fetching deployed SNS canisters...");
    let deployed_sns = get_deployed_sns(&ctx.agent, ctx.snsw_canister, proposal_id)
        .await
        .context("Failed to get deployed SNS")?;

    let governance_sns = deployed_sns
        .governance_canister_id
        .context("Missing governance canister ID")?;
    let ledger_sns = deployed_sns
        .ledger_canister_id
        .context("Missing ledger canister ID")?;
    let swap_sns = deployed_sns
        .swap_canister_id
        .context("Missing swap canister ID")?;

    print_success("SNS deployed:");
    print_info(&format!("  Governance: {governance_sns}"));
    print_info(&format!("  Ledger: {ledger_sns}"));
    print_info(&format!("  Swap: {swap_sns}"));

    Ok((proposal_id, deployed_sns))
}

/// Wait for swap to reach Open state (lifecycle 2) - blocking operation
pub async fn wait_for_swap_to_open(ctx: &DeploymentContext, swap_sns: Principal) -> Result<()> {
    print_header("Waiting for SNS Swap to Open");
    print_step("Checking swap lifecycle...");

    let mut current_lifecycle = get_swap_lifecycle(&ctx.agent, swap_sns)
        .await
        .context("Failed to get swap lifecycle")?;

    print_info(&format!("Current swap lifecycle: {current_lifecycle}"));

    // Get lifecycle details to show open timestamp if available
    let lifecycle_response = ctx
        .agent
        .query(&swap_sns, "get_lifecycle")
        .with_arg(encode_args((
            crate::core::declarations::sns_swap::GetLifecycleArg {},
        ))?)
        .call()
        .await
        .ok();

    if let Some(bytes) = lifecycle_response
        && let Ok(lifecycle) = Decode!(&bytes, GetLifecycleResponse)
        && let Some(open_timestamp) = lifecycle.decentralization_sale_open_timestamp_seconds
    {
        print_info(&format!("Swap open timestamp: {} seconds", open_timestamp));
    }

    // Block until lifecycle reaches 2 (Open) - this is REQUIRED before participation
    if current_lifecycle != 2 {
        print_step("Waiting for swap to reach Open state (lifecycle 2)...");
        print_info(
            "This is a blocking operation - participation cannot proceed until swap is Open",
        );

        let mut attempts = 0;
        let max_attempts = 300; // 5 minutes max wait (300 seconds)
        let check_interval = 2; // Check every 2 seconds

        loop {
            attempts += 1;

            // Check lifecycle
            current_lifecycle = get_swap_lifecycle(&ctx.agent, swap_sns).await.unwrap_or(0);

            if current_lifecycle == 2 {
                print_success(&format!(
                    "✓ Swap is now Open (lifecycle 2) after {} seconds",
                    attempts * check_interval
                ));
                break;
            }

            if attempts >= max_attempts {
                anyhow::bail!(
                    "Swap did not reach Open state (lifecycle 2) after {} seconds. Current lifecycle: {}. Cannot proceed with participation.",
                    attempts * check_interval,
                    current_lifecycle
                );
            }

            // Print status every 10 seconds (every 5 checks)
            if attempts % 5 == 0 {
                print_info(&format!(
                    "Still waiting... (lifecycle: {}, attempt {}/{}, {} seconds elapsed)",
                    current_lifecycle,
                    attempts,
                    max_attempts,
                    attempts * check_interval
                ));
            }

            tokio::time::sleep(StdDuration::from_secs(check_interval)).await;
        }
    } else {
        print_success("Swap is already Open (lifecycle 2)");
    }

    // Final verification - this should always be 2 at this point, but double-check
    let final_lifecycle = get_swap_lifecycle(&ctx.agent, swap_sns)
        .await
        .context("Failed to verify final swap lifecycle")?;

    if final_lifecycle != 2 {
        anyhow::bail!(
            "Swap lifecycle verification failed. Expected 2 (Open), got {}. Cannot proceed.",
            final_lifecycle
        );
    }

    print_success("✓ Swap confirmed Open (lifecycle 2) - ready for participation");
    Ok(())
}

/// Create a single participant and have them participate in the swap
pub async fn create_and_participate_participant(
    ctx: &DeploymentContext,
    participant_num: usize,
    swap_sns: Principal,
) -> Result<Principal> {
    print_step(&format!("Participant {participant_num}/5"));

    // Generate a deterministic Ed25519 identity for participant
    let participant_seed = format!("sns-participant-{participant_num}");
    let mut seed = [0u8; 32];
    let seed_bytes = sha2::Sha256::digest(participant_seed.as_bytes());
    seed.copy_from_slice(&seed_bytes[..32]);

    // Save participant seed to file for later use
    let seed_path = crate::core::utils::data_output::get_output_dir()
        .join("participants")
        .join(format!("participant_{}.seed", participant_num));
        crate::core::ops::identity::save_seed_to_file(&seed, &seed_path)
        .with_context(|| format!("Failed to save participant {participant_num} seed"))?;
    print_info(&format!(
        "  Saved participant identity: {}",
        seed_path.display()
    ));

    // Create identity from the seed (Ed25519 key)
    let participant_identity = ic_agent::identity::BasicIdentity::from_raw_key(&seed);

    // Create the agent first, then get the principal from it
    let participant_agent = create_agent(Box::new(participant_identity))
        .await
        .with_context(|| format!("Failed to create agent for participant {participant_num}"))?;

    // Get the principal from the agent (properly derived from identity)
    let participant_principal = participant_agent
        .get_principal()
        .map_err(|e| anyhow::anyhow!("Failed to get participant principal: {e}"))?;
    print_info(&format!("  Participant principal: {participant_principal}"));

    // Mint ICP for participant using minting account
    let participant_icp_amount = PARTICIPANT_ICP + 1_000_000_000 + ICP_TRANSFER_FEE;
    print_info(&format!("  Minting ICP for participant..."));

    transfer_icp(
        &ctx.minting_agent,
        ctx.ledger_canister,
        participant_principal,
        participant_icp_amount,
        None,
    )
    .await
    .with_context(|| format!("Failed to mint ICP for participant {participant_num}"))?;

    tokio::time::sleep(StdDuration::from_secs(1)).await;

    // Generate subaccount for participant
    let participant_subaccount = generate_participant_subaccount(participant_principal);

    // Create sale ticket first
    print_info("  Creating sale ticket...");
    const MAX_SALE_TICKET_AMOUNT: u64 = 1_000_000_000; // 10 ICP in e8s
    let sale_ticket_amount = std::cmp::min(PARTICIPANT_ICP, MAX_SALE_TICKET_AMOUNT);

    let sale_ticket_created = create_sale_ticket(
        &participant_agent,
        swap_sns,
        sale_ticket_amount,
        Some(participant_subaccount.0.to_vec()),
    )
    .await
    .unwrap_or(false);

    if sale_ticket_created {
        print_info("  ✓ Sale ticket created");
    } else {
        print_warning("  Sale ticket creation failed or not supported (continuing)");
    }

    tokio::time::sleep(StdDuration::from_secs(1)).await;

    // Transfer ICP to swap canister WITH subaccount derived from participant principal
    print_info("  Transferring ICP to swap canister (with subaccount)...");
    let transfer_amount = PARTICIPANT_ICP + ICP_TRANSFER_FEE;

    transfer_icp(
        &participant_agent,
        ctx.ledger_canister,
        swap_sns,
        transfer_amount,
        Some(participant_subaccount.0.to_vec()),
    )
    .await
    .with_context(|| format!("Failed to transfer ICP for participant {participant_num}"))?;

    tokio::time::sleep(StdDuration::from_secs(2)).await;

    // Verify balance at the swap's subaccount for this participant
    print_info("  Verifying ICP balance at swap subaccount...");
    let balance_args = LedgerAccount {
        owner: swap_sns,
        subaccount: Some(participant_subaccount.0.to_vec()),
    };

    let balance_bytes = ctx
        .agent
        .query(&ctx.ledger_canister, "icrc1_balance_of")
        .with_arg(encode_args((balance_args,))?)
        .call()
        .await
        .context("Failed to check balance")?;

    let balance: candid::Nat =
        Decode!(&balance_bytes, candid::Nat).context("Failed to decode balance")?;

    let balance_u64 = balance.0.to_u64_digits().first().copied().unwrap_or(0);
    print_info(&format!(
        "  Balance at swap subaccount (participant {}): {} e8s (transferred: {} e8s, expected after fee: {} e8s)",
        participant_principal, balance_u64, transfer_amount, PARTICIPANT_ICP
    ));

    if balance_u64 < PARTICIPANT_ICP {
        print_warning(&format!(
            "  ⚠ WARNING: Balance at subaccount ({}) is less than expected participation amount ({})",
            balance_u64, PARTICIPANT_ICP
        ));
        print_warning("  This may cause 'Amount transferred: 0' error during refresh_buyer_tokens");
    } else {
        print_success("  ✓ ICP balance verified at swap subaccount");
    }

    // Refresh buyer tokens - this is CRITICAL as it registers the participation in the swap
    print_info("  Refreshing buyer tokens (this registers participation)...");

    let mut refresh_success = false;

    for retry in 0..3 {
        match refresh_buyer_tokens(&participant_agent, swap_sns, participant_principal).await {
            Ok(response) => {
                if response.icp_accepted_participation_e8s > 0 {
                    print_info("  ✓ Buyer tokens refreshed - participation registered!");
                    refresh_success = true;
                    break;
                } else {
                    let swap_balance_seen = response.icp_ledger_account_balance_e8s;
                    print_warning(&format!(
                        "  ⚠ Swap accepted 0 e8s (balance it saw: {} e8s)",
                        swap_balance_seen
                    ));
                    if swap_balance_seen == 0 {
                        print_warning(
                            "  ⚠ Swap checked a different account/subaccount than where we sent funds!",
                        );
                        print_warning(&format!(
                            "  ⚠ We sent to swap + subaccount (balance: {}), but swap saw: {}",
                            balance_u64, swap_balance_seen
                        ));
                    }
                    if retry < 2 {
                        tokio::time::sleep(StdDuration::from_secs(3)).await;
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("{e}");
                if error_msg.contains("Amount transferred: 0") {
                    print_warning("  ⚠ Swap panicked: 'Amount transferred: 0'");
                    print_warning("  This means the swap checked a different account/subaccount");
                    print_warning(&format!(
                        "  We sent {} e8s to swap + subaccount, balance we see: {} e8s",
                        transfer_amount, balance_u64
                    ));
                    print_warning("  The swap's subaccount derivation likely doesn't match ours!");
                }
                if retry < 2 {
                    print_warning(&format!(
                        "  Refresh failed, retrying ({}/3): {e}",
                        retry + 2
                    ));
                    tokio::time::sleep(StdDuration::from_secs(2)).await;
                } else {
                    print_warning(&format!("  ⚠ Refresh failed after retries: {e}"));
                    print_warning("  ⚠ Participant may not be registered - check swap state");
                }
            }
        }
    }

    if !refresh_success {
        print_warning(
            "  ⚠ WARNING: Buyer tokens refresh failed - participation may not be registered!",
        );
    }

    print_success(&format!("Participant {participant_num} configured"));

    Ok(participant_principal)
}

/// Participate in SNS sale - create participants and have them participate
pub async fn participate_in_swap(
    ctx: &DeploymentContext,
    swap_sns: Principal,
) -> Result<Vec<Principal>> {
    print_header("Participating in SNS Sale");
    const NUM_PARTICIPANTS: usize = 5;
    print_step(&format!("Creating {NUM_PARTICIPANTS} participants..."));

    let mut participant_principals = Vec::new();

    for i in 1..=NUM_PARTICIPANTS {
        let principal = create_and_participate_participant(ctx, i, swap_sns).await?;
        participant_principals.push(principal);
    }

    Ok(participant_principals)
}

/// Finalize SNS sale - check thresholds and finalize swap
pub async fn finalize_sns_sale(ctx: &DeploymentContext, swap_sns: Principal) -> Result<()> {
    print_header("Finalizing SNS Sale");

    // Check participation thresholds
    print_step("Checking participation thresholds...");
    let derived_state = get_derived_state(&ctx.agent, swap_sns)
        .await
        .context("Failed to get derived state")?;

    let direct_participants = derived_state.direct_participant_count.unwrap_or(0);
    let direct_participation_icp = derived_state.direct_participation_icp_e8s.unwrap_or(0);
    let min_participants = 5;
    let min_direct_participation_icp = 100_000_000 * 5; // 5 ICP in e8s

    print_info(&format!(
        "Direct participants: {direct_participants} (minimum: {min_participants})"
    ));
    print_info(&format!(
        "Direct participation ICP: {} e8s (minimum: {} e8s)",
        direct_participation_icp, min_direct_participation_icp
    ));

    let thresholds_met = direct_participants >= min_participants
        && direct_participation_icp >= min_direct_participation_icp;

    if thresholds_met {
        print_success(
            "Participation thresholds met! Swap should auto-commit (no time advance needed in local replica)...",
        );
        print_info("Waiting for swap to transition to committed state (lifecycle 2 -> 3)...");
    } else {
        print_warning(&format!(
            "Participation thresholds not yet met (participants: {direct_participants}/{min_participants}, ICP: {direct_participation_icp}/{min_direct_participation_icp})"
        ));
    }

    // Wait for lifecycle 3 (Committed)
    print_step("Checking swap lifecycle...");
    let mut lifecycle = 0;
    let mut attempts = 0;
    let max_attempts = 30;

    while lifecycle != 3 && attempts < max_attempts {
        attempts += 1;
        tokio::time::sleep(StdDuration::from_secs(1)).await;

        match get_swap_lifecycle(&ctx.agent, swap_sns).await {
            Ok(l) => {
                lifecycle = l;
                if lifecycle == 3 {
                    print_success("Swap committed! (lifecycle 3)");
                    break;
                }

                // Periodically re-check participation state
                if lifecycle == 2 {
                    if let Ok(updated_state) = get_derived_state(&ctx.agent, swap_sns).await {
                        let updated_participants =
                            updated_state.direct_participant_count.unwrap_or(0);
                        let updated_icp = updated_state.direct_participation_icp_e8s.unwrap_or(0);
                        if updated_participants >= min_participants
                            && updated_icp >= min_direct_participation_icp
                        {
                            print_info(&format!(
                                "Thresholds met (participants: {updated_participants}, ICP: {updated_icp} e8s), waiting for auto-commit..."
                            ));
                        } else {
                            print_info(&format!(
                                "Lifecycle: {lifecycle}, participants: {updated_participants}, ICP: {updated_icp} e8s"
                            ));
                        }
                    }
                } else {
                    print_info(&format!("Lifecycle: {lifecycle}"));
                }
            }
            Err(e) => {
                print_warning(&format!("Failed to get lifecycle: {e}"));
            }
        }
    }

    if lifecycle == 3 {
        print_step("Finalizing swap...");
        match finalize_swap(&ctx.agent, swap_sns).await {
            Ok(_) => print_success("Swap finalized"),
            Err(e) => print_warning(&format!("Failed to finalize swap: {e}")),
        }
    } else {
        print_warning(&format!(
            "Swap not in finalizable state (lifecycle: {lifecycle})"
        ));
        print_info("You may need to manually advance time or finalize the swap");

        // Try finalizing anyway - sometimes lifecycle check is delayed
        if direct_participants >= min_participants
            && direct_participation_icp >= min_direct_participation_icp
        {
            print_info("Attempting to finalize swap despite lifecycle state...");
            match finalize_swap(&ctx.agent, swap_sns).await {
                Ok(_) => print_success("Swap finalized"),
                Err(e) => print_warning(&format!("Failed to finalize swap: {e}")),
            }
        }
    }

    Ok(())
}

/// Write deployment data to JSON file
pub async fn write_deployment_data(
    neuron_id: u64,
    proposal_id: u64,
    owner_principal: Principal,
    deployed_sns: &crate::core::declarations::sns_wasm::DeployedSns,
    participant_principals: &[Principal],
) -> Result<()> {
    print_header("Writing Deployment Data");
    let deployment_data = crate::core::utils::data_output::SnsCreationData {
        icp_neuron_id: neuron_id,
        proposal_id,
        owner_principal: owner_principal.to_string(),
        deployed_sns: crate::core::utils::data_output::DeployedSnsData::from(deployed_sns),
        participants: participant_principals
            .iter()
            .enumerate()
            .map(|(i, p)| crate::core::utils::data_output::ParticipantData {
                principal: p.to_string(),
                seed_file: format!("generated/participants/participant_{}.seed", i + 1),
            })
            .collect(),
    };

    crate::core::utils::data_output::write_data(&deployment_data).context("Failed to write deployment data file")?;

    let output_path = crate::core::utils::data_output::get_output_path();
    print_success(&format!(
        "Deployment data written to: {}",
        output_path.display()
    ));

    Ok(())
}

/// Main SNS deployment function - orchestrates the complete deployment flow
pub async fn deploy_sns() -> Result<()> {
    // Main SNS deployment flow
    println!("🚀 Starting SNS creation on local dfx network\n");

    // Initialize deployment context
    let ctx = initialize_deployment_context().await?;

    // Setup minting account
    setup_minting_account(&ctx).await?;

    // Create and configure ICP neuron
    let neuron_id = create_icp_neuron(&ctx).await?;
    configure_neuron(&ctx, neuron_id).await?;

    // Update SNS Subnet List (skipped for local)
    print_header("Updating SNS Subnet List");
        crate::core::utils::print_warning(
        "Subnet update skipped - may need manual configuration for local setup",
    );

    // Create proposal and wait for execution
    let (proposal_id, deployed_sns) = create_and_wait_for_proposal(&ctx, neuron_id).await?;

    let swap_sns = deployed_sns
        .swap_canister_id
        .ok_or_else(|| anyhow::anyhow!("Missing swap canister ID"))?;
    let governance_sns = deployed_sns
        .governance_canister_id
        .ok_or_else(|| anyhow::anyhow!("Missing governance canister ID"))?;
    let ledger_sns = deployed_sns
        .ledger_canister_id
        .ok_or_else(|| anyhow::anyhow!("Missing ledger canister ID"))?;

    // Wait for swap to open
    wait_for_swap_to_open(&ctx, swap_sns).await?;

    // Participate in swap
    let participant_principals = participate_in_swap(&ctx, swap_sns).await?;

    // Finalize swap
    finalize_sns_sale(&ctx, swap_sns).await?;

    // Write deployment data
    write_deployment_data(
        neuron_id,
        proposal_id,
        ctx.owner_principal,
        &deployed_sns,
        &participant_principals,
    )
    .await?;

    // Final Summary
    print_header("SNS Creation Complete");
    print_success("SNS has been created and deployed!");
    print_info(&format!("Governance Canister: {governance_sns}"));
    print_info(&format!("Ledger Canister: {ledger_sns}"));
    print_info(&format!("Swap Canister: {swap_sns}"));
    print_info(&format!("ICP Neuron ID: {neuron_id}"));
    print_info(&format!("Proposal ID: {proposal_id}"));

    let output_path = crate::core::utils::data_output::get_output_path();
    println!("\n💡 You can now interact with the SNS using these canister IDs");
    println!(
        "💡 Deployment data has been saved to: {}",
        output_path.display()
    );

    Ok(())
}
