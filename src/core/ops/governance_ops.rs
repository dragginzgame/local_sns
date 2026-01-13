// ICP Governance operations

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use ic_agent::Agent;

use super::super::declarations::icp_governance::{
    AddHotKey, By, ClaimOrRefresh, ClaimOrRefreshResponse, Command1, Configure,
    IncreaseDissolveDelay, MakeProposalRequest, MakeProposalResponse, ManageNeuronCommandRequest,
    ManageNeuronRequest, ManageNeuronResponse, NeuronId, Operation, ProposalActionRequest,
    ProposalId, SetVisibility,
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
    use crate::core::utils::constants::{ICP_TRANSFER_FEE, LEDGER_CANISTER};

    // Load minting identity
    let identity = load_minting_identity().context("Failed to load minting identity")?;

    // Create authenticated agent with minting identity
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with minting identity")?;

    let ledger_canister =
        Principal::from_text(LEDGER_CANISTER).context("Failed to parse ICP Ledger canister ID")?;

    // Transfer ICP (amount includes fee)
    let transfer_amount = amount_e8s + ICP_TRANSFER_FEE;
    let block_height = transfer_icp(
        &agent,
        ledger_canister,
        receiver_principal,
        transfer_amount,
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
) -> Result<u64> {
    use super::identity::{create_agent, load_dfx_identity};
    use super::ledger_ops::{generate_subaccount_by_nonce, transfer_icp};
    use crate::core::utils::constants::{GOVERNANCE_CANISTER, ICP_TRANSFER_FEE, LEDGER_CANISTER};

    // Load identity for the principal
    let identity = load_dfx_identity(None).context("Failed to load dfx identity")?;

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
