// ICP Governance operations

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use ic_agent::Agent;

use super::super::declarations::icp_governance::{
    AccountIdentifier, AddHotKey, Amount, By, ClaimOrRefresh, ClaimOrRefreshResponse, Command1,
    Configure, Disburse, DisburseResponse, IncreaseDissolveDelay, MakeProposalRequest,
    MakeProposalResponse, ManageNeuronCommandRequest, ManageNeuronRequest, ManageNeuronResponse,
    NeuronId, Operation, ProposalActionRequest, ProposalId, SetVisibility,
};

/// Claim neuron using manage_neuron
pub async fn claim_neuron(agent: &Agent, governance_canister: Principal, memo: u64) -> Result<u64> {
    let request = ManageNeuronRequest {
        id: None,
        command: Some(ManageNeuronCommandRequest::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::Memo(memo)),
        })),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to call manage_neuron")?;

    let response: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)
        .context("Failed to decode manage_neuron response")?;

    match response.command {
        Some(Command1::ClaimOrRefresh(ClaimOrRefreshResponse {
            refreshed_neuron_id: Some(NeuronId { id }),
        })) => Ok(id),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to claim neuron: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from manage_neuron"),
    }
}

/// Set dissolve delay for neuron
pub async fn set_dissolve_delay(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
    dissolve_delay: u64,
) -> Result<()> {
    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds: dissolve_delay as u32,
            })),
        })),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to set dissolve delay")?;

    let response: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)
        .context("Failed to decode configure response")?;

    match response.command {
        Some(Command1::Configure {}) => Ok(()),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to set dissolve delay: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from configure"),
    }
}

/// Create SNS proposal
pub async fn create_sns_proposal(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
    owner_principal: Principal,
) -> Result<u64> {
    // Build SNS configuration from sns_config.rs
    let sns_data = crate::init::sns_config::build_sns_config(owner_principal);

    let proposal = MakeProposalRequest {
        url: "".to_string(),
        title: Some(crate::init::sns_config::default_proposal_title()),
        summary: crate::init::sns_config::default_proposal_summary(),
        action: Some(ProposalActionRequest::CreateServiceNervousSystem(sns_data)),
    };

    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::MakeProposal(proposal)),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to create SNS proposal")?;

    let response: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)
        .context("Failed to decode make_proposal response")?;

    match response.command {
        Some(Command1::MakeProposal(MakeProposalResponse {
            proposal_id: Some(ProposalId { id }),
            ..
        })) => Ok(id),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to create proposal: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from make_proposal"),
    }
}

/// Add a hotkey to an ICP neuron
///
/// Note: ICP neurons use a simpler API than SNS neurons - they don't have permission types,
/// just add/remove hotkeys. The hotkey can perform any operation the controller can do.
pub async fn add_hotkey_to_icp_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
    hotkey_principal: Principal,
) -> Result<()> {
    let operation = Operation::AddHotKey(AddHotKey {
        new_hot_key: Some(hotkey_principal),
    });

    let configure = Configure {
        operation: Some(operation),
    };

    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::Configure(configure)),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to call manage_neuron for adding hotkey")?;

    let result: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)?;

    match result.command {
        Some(Command1::Configure {}) => Ok(()),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to add hotkey: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from manage_neuron"),
    }
}

/// High-level function to add a hotkey to the ICP neuron used for SNS deployment
/// This reads deployment data, loads the owner identity, and adds the hotkey
pub async fn add_hotkey_to_icp_neuron_default_path(hotkey_principal: Principal) -> Result<()> {
    use super::identity::{create_agent, load_dfx_identity};

    // Read deployment data
    let deployment_path = crate::core::utils::data_output::get_output_path();
    let data_content = std::fs::read_to_string(&deployment_path)
        .with_context(|| format!("Failed to read deployment data from: {:?}", deployment_path))?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    // Get ICP neuron ID
    let neuron_id = deployment_data.icp_neuron_id;

    // Load owner identity (default dfx identity)
    let identity = load_dfx_identity(None).context("Failed to load owner dfx identity")?;

    // Create authenticated agent
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with owner identity")?;

    // ICP Governance canister (standard NNS canister ID for local development)
    let governance_canister = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")
        .context("Failed to parse ICP Governance canister ID")?;

    // Add hotkey
    add_hotkey_to_icp_neuron(&agent, governance_canister, neuron_id, hotkey_principal)
        .await
        .context("Failed to add hotkey to ICP neuron")?;

    Ok(())
}

/// Set neuron visibility (public/private)
/// visibility: true = public (2), false = private (1)
pub async fn set_neuron_visibility(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
    is_public: bool,
) -> Result<()> {
    let visibility_value = if is_public { 2 } else { 1 };
    let operation = Operation::SetVisibility(SetVisibility {
        visibility: Some(visibility_value),
    });

    let configure = Configure {
        operation: Some(operation),
    };

    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::Configure(configure)),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to call manage_neuron for setting visibility")?;

    let result: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)?;

    match result.command {
        Some(Command1::Configure {}) => Ok(()),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to set visibility: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from manage_neuron"),
    }
}

/// Get minting account balance
pub async fn get_minting_account_balance() -> Result<u64> {
    use super::identity::{create_agent, load_minting_identity};
    use super::ledger_ops::get_icp_ledger_balance;
    use crate::core::utils::constants::LEDGER_CANISTER;

    // Load minting identity
    let identity = load_minting_identity().context("Failed to load minting identity")?;

    // Create authenticated agent with minting identity
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with minting identity")?;

    let ledger_canister =
        Principal::from_text(LEDGER_CANISTER).context("Failed to parse ICP Ledger canister ID")?;

    // Get minting account principal
    let minting_principal = agent
        .get_principal()
        .map_err(|e| anyhow::anyhow!("Failed to get minting account principal: {}", e))?;

    // Get balance
    let balance = get_icp_ledger_balance(&agent, ledger_canister, minting_principal, None)
        .await
        .context("Failed to get minting account balance")?;

    Ok(balance)
}

/// Mint ICP tokens by transferring from minting account to a receiver
pub async fn mint_icp_default_path(receiver_principal: Principal, amount_e8s: u64) -> Result<u64> {
    use super::identity::{create_agent, load_minting_identity};
    use super::ledger_ops::transfer_icp;
    use crate::core::utils::constants::LEDGER_CANISTER;

    // Load minting identity
    let identity = load_minting_identity().context("Failed to load minting identity")?;

    // Create authenticated agent with minting identity
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with minting identity")?;

    let ledger_canister =
        Principal::from_text(LEDGER_CANISTER).context("Failed to parse ICP Ledger canister ID")?;

    // Transfer ICP (minting doesn't require fee - fee is deducted from minting account automatically)
    let block_height = transfer_icp(
        &agent,
        ledger_canister,
        receiver_principal,
        amount_e8s,
        None,
    )
    .await
    .context("Failed to transfer ICP")?;

    Ok(block_height)
}

/// Create an ICP neuron by transferring ICP and claiming it
pub async fn create_icp_neuron_default_path(
    principal: Principal,
    amount_e8s: u64,
    memo: Option<u64>,
    dissolve_delay_seconds: Option<u64>,
) -> Result<u64> {
    use super::identity::{create_agent, load_dfx_identity, load_identity_from_seed_file};
    use super::ledger_ops::{generate_subaccount_by_nonce, transfer_icp};
    use crate::core::utils::constants::{GOVERNANCE_CANISTER, ICP_TRANSFER_FEE, LEDGER_CANISTER};
    use crate::core::utils::data_output;
    use std::path::PathBuf;

    // Try to load participant identity from deployment data, fallback to dfx identity
    let identity = {
        let deployment_path = data_output::get_output_path();
        if deployment_path.exists() {
            if let Ok(data_content) = std::fs::read_to_string(&deployment_path) {
                if let Ok(deployment_data) =
                    serde_json::from_str::<data_output::SnsCreationData>(&data_content)
                {
                    // Check if principal is the owner
                    if deployment_data.owner_principal == principal.to_string() {
                        // Owner uses dfx identity
                        load_dfx_identity(None).context("Failed to load dfx identity")?
                    } else if let Some(participant_data) = deployment_data
                        .participants
                        .iter()
                        .find(|p| p.principal == principal.to_string())
                    {
                        // Load participant identity from seed file
                        let seed_path = PathBuf::from(&participant_data.seed_file);
                        if let Ok(participant_identity) = load_identity_from_seed_file(&seed_path) {
                            participant_identity
                        } else {
                            // Fallback to dfx identity
                            load_dfx_identity(None).context("Failed to load dfx identity")?
                        }
                    } else {
                        // Principal not found in participants or owner, use dfx identity
                        load_dfx_identity(None).context("Failed to load dfx identity")?
                    }
                } else {
                    // Failed to parse deployment data, use dfx identity
                    load_dfx_identity(None).context("Failed to load dfx identity")?
                }
            } else {
                // Failed to read deployment data, use dfx identity
                load_dfx_identity(None).context("Failed to load dfx identity")?
            }
        } else {
            // No deployment data, use dfx identity
            load_dfx_identity(None).context("Failed to load dfx identity")?
        }
    };

    // Create authenticated agent
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent")?;

    let ledger_canister =
        Principal::from_text(LEDGER_CANISTER).context("Failed to parse ICP Ledger canister ID")?;
    let governance_canister = Principal::from_text(GOVERNANCE_CANISTER)
        .context("Failed to parse ICP Governance canister ID")?;

    // Use provided memo or default to 1
    let memo_value = memo.unwrap_or(1);

    // Generate subaccount for neuron
    let subaccount = generate_subaccount_by_nonce(memo_value, principal);

    // Transfer ICP to governance subaccount (amount should include fee)
    let transfer_amount = amount_e8s + ICP_TRANSFER_FEE;
    transfer_icp(
        &agent,
        ledger_canister,
        governance_canister,
        transfer_amount,
        Some(subaccount.0.to_vec()),
    )
    .await
    .context("Failed to transfer ICP to governance subaccount")?;

    // Wait a bit for the transfer to settle
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Claim neuron
    let neuron_id = claim_neuron(&agent, governance_canister, memo_value)
        .await
        .context("Failed to claim ICP neuron")?;

    // Set dissolve delay if specified
    if let Some(dissolve_delay) = dissolve_delay_seconds {
        if dissolve_delay > 0 {
            set_dissolve_delay(&agent, governance_canister, neuron_id, dissolve_delay)
                .await
                .context("Failed to set dissolve delay")?;
        }
    }

    Ok(neuron_id)
}

/// High-level function to set visibility for the ICP neuron used for SNS deployment
/// This reads deployment data, loads the owner identity, and sets the visibility
pub async fn set_icp_neuron_visibility_default_path(is_public: bool) -> Result<()> {
    use super::identity::{create_agent, load_dfx_identity};

    // Read deployment data
    let deployment_path = crate::core::utils::data_output::get_output_path();
    let data_content = std::fs::read_to_string(&deployment_path)
        .with_context(|| format!("Failed to read deployment data from: {:?}", deployment_path))?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    // Get ICP neuron ID
    let neuron_id = deployment_data.icp_neuron_id;

    // Load owner identity (default dfx identity)
    let identity = load_dfx_identity(None).context("Failed to load owner dfx identity")?;

    // Create authenticated agent
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with owner identity")?;

    // ICP Governance canister (standard NNS canister ID for local development)
    let governance_canister = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")
        .context("Failed to parse ICP Governance canister ID")?;

    // Set visibility
    set_neuron_visibility(&agent, governance_canister, neuron_id, is_public)
        .await
        .context("Failed to set neuron visibility")?;

    Ok(())
}

/// List all ICP neurons for a given principal, sorted by dissolve delay (lowest first) and cached stake (highest first)
/// Note: ICP neurons are protected and require authentication (the agent must be authenticated as the principal)
/// The principal parameter is used for documentation - the actual neurons returned are those readable by the authenticated caller
pub async fn list_icp_neurons_for_principal(
    agent: &Agent,
    governance_canister: Principal,
    _principal: Principal,
) -> Result<Vec<super::super::declarations::icp_governance::Neuron>> {
    use super::super::declarations::icp_governance::{
        DissolveState, ListNeurons, ListNeuronsResponse,
    };

    // Use the new ListNeurons interface - include_neurons_readable_by_caller will return neurons
    // that the authenticated caller (principal) can read
    let request = ListNeurons {
        page_size: Some(100),
        include_public_neurons_in_full_neurons: Some(false),
        neuron_ids: Vec::new(),
        page_number: Some(0),
        include_empty_neurons_readable_by_caller: Some(false),
        neuron_subaccounts: None,
        include_neurons_readable_by_caller: true,
    };
    let args = candid::encode_args((request,))?;

    let response = agent
        .query(&governance_canister, "list_neurons")
        .with_arg(args)
        .call()
        .await
        .context("Failed to call list_neurons")?;

    let result: ListNeuronsResponse = Decode!(&response, ListNeuronsResponse)?;

    // Use full_neurons from the response
    // Sort neurons by dissolve delay (lowest first), then by cached stake (highest first)
    let mut neurons = result.full_neurons;
    neurons.sort_by(|a, b| {
        let a_delay = match &a.dissolve_state {
            Some(DissolveState::DissolveDelaySeconds(seconds)) => *seconds,
            Some(DissolveState::WhenDissolvedTimestampSeconds(_)) => 0, // Dissolving = 0 delay
            None => u64::MAX, // No state = highest (sort last)
        };
        let b_delay = match &b.dissolve_state {
            Some(DissolveState::DissolveDelaySeconds(seconds)) => *seconds,
            Some(DissolveState::WhenDissolvedTimestampSeconds(_)) => 0, // Dissolving = 0 delay
            None => u64::MAX, // No state = highest (sort last)
        };

        // First sort by dissolve delay (ascending - lowest first)
        match a_delay.cmp(&b_delay) {
            std::cmp::Ordering::Equal => {
                // If dissolve delays are equal, sort by cached stake (descending - highest first)
                b.cached_neuron_stake_e8s.cmp(&a.cached_neuron_stake_e8s)
            }
            other => other,
        }
    });

    Ok(neurons)
}

/// High-level function to list ICP neurons for a principal
/// This loads the identity for the principal (from deployment data if available, otherwise dfx identity)
/// ICP neurons are protected and require authentication
pub async fn list_icp_neurons_for_principal_default_path(
    principal: Principal,
) -> Result<Vec<super::super::declarations::icp_governance::Neuron>> {
    use super::identity::{create_agent, load_dfx_identity, load_identity_from_seed_file};
    use crate::core::utils::constants::GOVERNANCE_CANISTER;
    use crate::core::utils::data_output;
    use std::path::PathBuf;

    // Try to load participant identity from deployment data, fallback to dfx identity
    let identity = {
        let deployment_path = data_output::get_output_path();
        if deployment_path.exists() {
            if let Ok(data_content) = std::fs::read_to_string(&deployment_path) {
                if let Ok(deployment_data) =
                    serde_json::from_str::<data_output::SnsCreationData>(&data_content)
                {
                    // Check if principal is the owner
                    if deployment_data.owner_principal == principal.to_string() {
                        // Owner uses dfx identity
                        load_dfx_identity(None).context("Failed to load dfx identity")?
                    } else if let Some(participant_data) = deployment_data
                        .participants
                        .iter()
                        .find(|p| p.principal == principal.to_string())
                    {
                        // Load participant identity from seed file
                        let seed_path = PathBuf::from(&participant_data.seed_file);
                        if let Ok(participant_identity) = load_identity_from_seed_file(&seed_path) {
                            participant_identity
                        } else {
                            // Fallback to dfx identity
                            load_dfx_identity(None).context("Failed to load dfx identity")?
                        }
                    } else {
                        // Principal not found in participants or owner, use dfx identity
                        load_dfx_identity(None).context("Failed to load dfx identity")?
                    }
                } else {
                    // Failed to parse deployment data, use dfx identity
                    load_dfx_identity(None).context("Failed to load dfx identity")?
                }
            } else {
                // Failed to read deployment data, use dfx identity
                load_dfx_identity(None).context("Failed to load dfx identity")?
            }
        } else {
            // No deployment data, use dfx identity
            load_dfx_identity(None).context("Failed to load dfx identity")?
        }
    };

    // Create authenticated agent with the principal's identity
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent")?;

    let governance_canister = Principal::from_text(GOVERNANCE_CANISTER)
        .context("Failed to parse ICP Governance canister ID")?;

    // List neurons (requires authentication for ICP neurons)
    list_icp_neurons_for_principal(&agent, governance_canister, principal).await
}

/// Get full neuron information by neuron ID
pub async fn get_icp_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
) -> Result<super::super::declarations::icp_governance::Neuron> {
    use super::super::declarations::icp_governance::Result2;

    let args = candid::encode_args((neuron_id,))?;

    let response = agent
        .query(&governance_canister, "get_full_neuron")
        .with_arg(args)
        .call()
        .await
        .context("Failed to call get_full_neuron")?;

    let result: Result2 = Decode!(&response, Result2)?;

    match result {
        Result2::Ok(neuron) => Ok(neuron),
        Result2::Err(e) => {
            anyhow::bail!(
                "Failed to get neuron: {} (type: {})",
                e.error_message,
                e.error_type
            );
        }
    }
}

/// High-level function to get ICP neuron information
/// This reads deployment data and queries the neuron using the owner's identity
pub async fn get_icp_neuron_default_path(
    neuron_id: Option<u64>,
) -> Result<super::super::declarations::icp_governance::Neuron> {
    use super::identity::{create_agent, load_dfx_identity};

    let id = if let Some(id) = neuron_id {
        id
    } else {
        // Read deployment data
        let deployment_path = crate::core::utils::data_output::get_output_path();
        let data_content = std::fs::read_to_string(&deployment_path).with_context(|| {
            format!("Failed to read deployment data from: {:?}", deployment_path)
        })?;
        let deployment_data: crate::core::utils::data_output::SnsCreationData =
            serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;
        deployment_data.icp_neuron_id
    };

    // Load owner identity (default dfx identity) - get_full_neuron requires authentication
    let identity = load_dfx_identity(None).context("Failed to load owner dfx identity")?;

    // Create authenticated agent
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with owner identity")?;

    // ICP Governance canister (standard NNS canister ID for local development)
    let governance_canister = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")
        .context("Failed to parse ICP Governance canister ID")?;

    // Get neuron
    get_icp_neuron(&agent, governance_canister, id).await
}

/// Disburse an ICP neuron to a receiver account
pub async fn disburse_icp_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
    receiver_principal: Principal,
    amount_e8s: Option<u64>,
) -> Result<u64> {
    use ic_ledger_types::AccountIdentifier as LedgerAccountIdentifier;

    // Convert principal to AccountIdentifier using ic_ledger_types
    let ledger_account_id =
        LedgerAccountIdentifier::new(&receiver_principal, &ic_ledger_types::Subaccount([0u8; 32]));
    // Convert to governance AccountIdentifier (hash is Vec<u8>)
    // AccountIdentifier from ic_ledger_types is a tuple struct, convert to bytes
    let account_identifier = AccountIdentifier {
        hash: ledger_account_id.as_ref().to_vec(),
    };

    let disburse = Disburse {
        to_account: Some(account_identifier),
        amount: amount_e8s.map(|e8s| Amount { e8s }),
    };

    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::Disburse(disburse)),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to disburse neuron")?;

    let response: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)
        .context("Failed to decode disburse response")?;

    match response.command {
        Some(Command1::Disburse(DisburseResponse {
            transfer_block_height,
        })) => Ok(transfer_block_height),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to disburse neuron: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from disburse"),
    }
}

/// Start dissolving an ICP neuron
pub async fn start_dissolving_icp_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
) -> Result<()> {
    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::Configure(Configure {
            operation: Some(Operation::StartDissolving {}),
        })),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to start dissolving")?;

    let response: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)
        .context("Failed to decode configure response")?;

    match response.command {
        Some(Command1::Configure {}) => Ok(()),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to start dissolving: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from configure"),
    }
}

/// Stop dissolving an ICP neuron
pub async fn stop_dissolving_icp_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
) -> Result<()> {
    let request = ManageNeuronRequest {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(ManageNeuronCommandRequest::Configure(Configure {
            operation: Some(Operation::StopDissolving {}),
        })),
        neuron_id_or_subaccount: None,
    };

    let result_bytes = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to stop dissolving")?;

    let response: ManageNeuronResponse = Decode!(&result_bytes, ManageNeuronResponse)
        .context("Failed to decode configure response")?;

    match response.command {
        Some(Command1::Configure {}) => Ok(()),
        Some(Command1::Error(e)) => {
            anyhow::bail!("Failed to stop dissolving: {}", e.error_message);
        }
        _ => anyhow::bail!("Unexpected response from configure"),
    }
}

/// Increase dissolve delay for an ICP neuron (wrapper around set_dissolve_delay)
pub async fn increase_icp_dissolve_delay(
    agent: &Agent,
    governance_canister: Principal,
    neuron_id: u64,
    additional_dissolve_delay_seconds: u64,
) -> Result<()> {
    set_dissolve_delay(
        agent,
        governance_canister,
        neuron_id,
        additional_dissolve_delay_seconds,
    )
    .await
}

/// High-level function to disburse an ICP neuron for a principal
/// This reads deployment data, loads the participant identity (or dfx), and disburses the neuron
pub async fn disburse_icp_neuron_for_principal_default_path(
    principal: Principal,
    receiver_principal: Principal,
    neuron_id: Option<u64>,
    amount_e8s: Option<u64>,
) -> Result<u64> {
    use super::identity::{create_agent, load_dfx_identity, load_identity_from_seed_file};
    use crate::core::utils::{constants::GOVERNANCE_CANISTER, data_output::get_output_path};
    use std::fs;

    // Try to load participant identity from deployment data
    let deployment_path = get_output_path();
    let identity = if deployment_path.exists() {
        let data_content =
            fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
        let deployment_data: crate::core::utils::data_output::SnsCreationData =
            serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

        // Try to find matching participant
        let mut found_identity = None;
        for participant in &deployment_data.participants {
            let participant_principal = Principal::from_text(&participant.principal)
                .context("Failed to parse participant principal")?;
            if participant_principal == principal {
                let seed_path = std::path::PathBuf::from(&participant.seed_file);
                if let Ok(participant_identity) = load_identity_from_seed_file(&seed_path) {
                    found_identity = Some(participant_identity);
                    break;
                }
            }
        }
        found_identity.unwrap_or_else(|| {
            load_dfx_identity(None)
                .context("Failed to load dfx identity")
                .expect("Failed to load dfx identity as fallback")
        })
    } else {
        load_dfx_identity(None).context("Failed to load dfx identity")?
    };

    let agent = create_agent(identity)
        .await
        .context("Failed to create agent")?;

    let governance_canister = Principal::from_text(GOVERNANCE_CANISTER)
        .context("Failed to parse ICP Governance canister ID")?;

    let final_neuron_id = if let Some(id) = neuron_id {
        id
    } else {
        // Get neurons and select the one with lowest dissolve delay
        let neurons = list_icp_neurons_for_principal(&agent, governance_canister, principal)
            .await
            .context("Failed to list neurons")?;

        neurons
            .first()
            .and_then(|n| n.id.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!("Principal has no neurons. Make sure you have created neurons.")
            })?
            .id
    };

    disburse_icp_neuron(
        &agent,
        governance_canister,
        final_neuron_id,
        receiver_principal,
        amount_e8s,
    )
    .await
}

/// High-level function to increase dissolve delay for an ICP neuron
pub async fn increase_icp_dissolve_delay_for_principal_default_path(
    principal: Principal,
    neuron_id: Option<u64>,
    additional_dissolve_delay_seconds: u64,
) -> Result<()> {
    use super::identity::{create_agent, load_dfx_identity, load_identity_from_seed_file};
    use crate::core::utils::{constants::GOVERNANCE_CANISTER, data_output::get_output_path};
    use std::fs;

    // Try to load participant identity from deployment data
    let deployment_path = get_output_path();
    let identity = if deployment_path.exists() {
        let data_content =
            fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
        let deployment_data: crate::core::utils::data_output::SnsCreationData =
            serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

        // Try to find matching participant
        let mut found_identity = None;
        for participant in &deployment_data.participants {
            let participant_principal = Principal::from_text(&participant.principal)
                .context("Failed to parse participant principal")?;
            if participant_principal == principal {
                let seed_path = std::path::PathBuf::from(&participant.seed_file);
                if let Ok(participant_identity) = load_identity_from_seed_file(&seed_path) {
                    found_identity = Some(participant_identity);
                    break;
                }
            }
        }
        found_identity.unwrap_or_else(|| {
            load_dfx_identity(None)
                .context("Failed to load dfx identity")
                .expect("Failed to load dfx identity as fallback")
        })
    } else {
        load_dfx_identity(None).context("Failed to load dfx identity")?
    };

    let agent = create_agent(identity)
        .await
        .context("Failed to create agent")?;

    let governance_canister = Principal::from_text(GOVERNANCE_CANISTER)
        .context("Failed to parse ICP Governance canister ID")?;

    let final_neuron_id = if let Some(id) = neuron_id {
        id
    } else {
        // Get neurons and select the one with lowest dissolve delay
        let neurons = list_icp_neurons_for_principal(&agent, governance_canister, principal)
            .await
            .context("Failed to list neurons")?;

        neurons
            .first()
            .and_then(|n| n.id.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!("Principal has no neurons. Make sure you have created neurons.")
            })?
            .id
    };

    increase_icp_dissolve_delay(
        &agent,
        governance_canister,
        final_neuron_id,
        additional_dissolve_delay_seconds,
    )
    .await
}

/// High-level function to manage dissolving state for an ICP neuron
pub async fn manage_icp_dissolving_state_for_principal_default_path(
    principal: Principal,
    neuron_id: Option<u64>,
    start_dissolving: bool,
) -> Result<()> {
    use super::identity::{create_agent, load_dfx_identity, load_identity_from_seed_file};
    use crate::core::utils::{constants::GOVERNANCE_CANISTER, data_output::get_output_path};
    use std::fs;

    // Try to load participant identity from deployment data
    let deployment_path = get_output_path();
    let identity = if deployment_path.exists() {
        let data_content =
            fs::read_to_string(&deployment_path).context("Failed to read deployment data")?;
        let deployment_data: crate::core::utils::data_output::SnsCreationData =
            serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

        // Try to find matching participant
        let mut found_identity = None;
        for participant in &deployment_data.participants {
            let participant_principal = Principal::from_text(&participant.principal)
                .context("Failed to parse participant principal")?;
            if participant_principal == principal {
                let seed_path = std::path::PathBuf::from(&participant.seed_file);
                if let Ok(participant_identity) = load_identity_from_seed_file(&seed_path) {
                    found_identity = Some(participant_identity);
                    break;
                }
            }
        }
        found_identity.unwrap_or_else(|| {
            load_dfx_identity(None)
                .context("Failed to load dfx identity")
                .expect("Failed to load dfx identity as fallback")
        })
    } else {
        load_dfx_identity(None).context("Failed to load dfx identity")?
    };

    let agent = create_agent(identity)
        .await
        .context("Failed to create agent")?;

    let governance_canister = Principal::from_text(GOVERNANCE_CANISTER)
        .context("Failed to parse ICP Governance canister ID")?;

    let final_neuron_id = if let Some(id) = neuron_id {
        id
    } else {
        // Get neurons and select the one with lowest dissolve delay
        let neurons = list_icp_neurons_for_principal(&agent, governance_canister, principal)
            .await
            .context("Failed to list neurons")?;

        neurons
            .first()
            .and_then(|n| n.id.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!("Principal has no neurons. Make sure you have created neurons.")
            })?
            .id
    };

    if start_dissolving {
        start_dissolving_icp_neuron(&agent, governance_canister, final_neuron_id).await
    } else {
        stop_dissolving_icp_neuron(&agent, governance_canister, final_neuron_id).await
    }
}
