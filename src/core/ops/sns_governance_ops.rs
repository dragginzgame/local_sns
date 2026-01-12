// SNS Governance canister operations

use anyhow::{Context, Result};
use candid::{Decode, Principal};
use ic_agent::Agent;
use std::path::PathBuf;

use super::super::declarations::sns_governance::{
    Account, AddNeuronPermissions, Command, Disburse, ListNeurons, ListNeuronsResponse,
    ManageNeuron, ManageNeuronResponse, Neuron, NeuronId, NeuronPermissionList,
};

/// Get the neuron ID for a participant principal
pub async fn get_participant_neuron_id(
    agent: &Agent,
    governance_canister: Principal,
    participant_principal: Principal,
) -> Result<Option<NeuronId>> {
    let request = ListNeurons {
        of_principal: Some(participant_principal),
        limit: 100,
        start_page_at: None,
    };
    let args = candid::encode_args((request,))?;

    let response = agent
        .query(&governance_canister, "list_neurons")
        .with_arg(args)
        .call()
        .await
        .context("Failed to call list_neurons")?;

    let result: ListNeuronsResponse = Decode!(&response, ListNeuronsResponse)?;

    // Find the first neuron owned by this principal
    for neuron in result.neurons {
        if let Some(id) = neuron.id {
            return Ok(Some(id));
        }
    }

    Ok(None)
}

/// List all neurons for a given principal
pub async fn list_neurons_for_principal(
    agent: &Agent,
    governance_canister: Principal,
    principal: Principal,
) -> Result<Vec<Neuron>> {
    let request = ListNeurons {
        of_principal: Some(principal),
        limit: 100,
        start_page_at: None,
    };
    let args = candid::encode_args((request,))?;

    let response = agent
        .query(&governance_canister, "list_neurons")
        .with_arg(args)
        .call()
        .await
        .context("Failed to call list_neurons")?;

    let result: ListNeuronsResponse = Decode!(&response, ListNeuronsResponse)?;
    Ok(result.neurons)
}

/// High-level function to list neurons for a principal
/// This reads deployment data and lists neurons using an anonymous agent
pub async fn list_neurons_for_principal_default_path(principal: Principal) -> Result<Vec<Neuron>> {
    use super::identity::create_agent;

    // Read deployment data
    let deployment_path = crate::core::utils::data_output::get_output_path();
    let data_content = std::fs::read_to_string(&deployment_path)
        .with_context(|| format!("Failed to read deployment data from: {:?}", deployment_path))?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    // Get governance canister ID
    let governance_canister_id = deployment_data
        .deployed_sns
        .governance_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse governance canister ID from deployment data")?;

    // Create anonymous agent (query doesn't need authentication)
    let anonymous_identity = ic_agent::identity::AnonymousIdentity;
    let agent = create_agent(Box::new(anonymous_identity)).await?;

    // List neurons
    list_neurons_for_principal(&agent, governance_canister_id, principal).await
}

/// Add a hotkey to a neuron
pub async fn add_hotkey_to_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_subaccount: Vec<u8>,
    hotkey_principal: Principal,
    permission_types: Vec<i32>,
) -> Result<()> {
    let command = Command::AddNeuronPermissions(AddNeuronPermissions {
        permissions_to_add: Some(NeuronPermissionList {
            permissions: permission_types,
        }),
        principal_id: Some(hotkey_principal),
    });

    let request = ManageNeuron {
        subaccount: neuron_subaccount,
        command: Some(command),
    };
    let args = candid::encode_args((request,))?;

    let response = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(args)
        .call_and_wait()
        .await
        .context("Failed to call manage_neuron")?;

    let result: ManageNeuronResponse = Decode!(&response, ManageNeuronResponse)?;

    // Check for errors
    if let Some(cmd) = result.command {
        match cmd {
            super::super::declarations::sns_governance::Command1::Error(e) => {
                anyhow::bail!(
                    "Governance error: {} (type: {})",
                    e.error_message,
                    e.error_type
                );
            }
            super::super::declarations::sns_governance::Command1::AddNeuronPermission {} => {
                // Success
            }
            _ => {
                // Other command responses are success cases we don't need to handle specifically
            }
        }
    }

    Ok(())
}

/// High-level function to add a hotkey to a participant's neuron
/// This reads deployment data, loads the participant identity, and adds the hotkey
pub async fn add_hotkey_to_participant_neuron(
    deployment_data_path: &std::path::Path,
    participant_principal: Principal,
    hotkey_principal: Principal,
    permission_types: Option<Vec<i32>>,
) -> Result<()> {
    use super::identity::{create_agent, load_identity_from_seed_file};

    // Read deployment data
    let data_content = std::fs::read_to_string(deployment_data_path).with_context(|| {
        format!(
            "Failed to read deployment data from: {:?}",
            deployment_data_path
        )
    })?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    // Find participant seed file
    let participant_data = deployment_data
        .participants
        .iter()
        .find(|p| p.principal == participant_principal.to_string())
        .with_context(|| {
            format!(
                "Participant principal {} not found in deployment data",
                participant_principal
            )
        })?;

    // Load participant identity from seed file
    let seed_path = PathBuf::from(&participant_data.seed_file);
    let identity = load_identity_from_seed_file(&seed_path)
        .with_context(|| format!("Failed to load identity from: {}", seed_path.display()))?;

    // Create authenticated agent
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with participant identity")?;

    // Get governance canister ID
    let governance_canister = deployment_data
        .deployed_sns
        .governance_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse governance canister ID from deployment data")?;

    // Get participant neuron
    let neuron_id = get_participant_neuron_id(&agent, governance_canister, participant_principal)
        .await
        .context("Failed to get participant neuron ID")?
        .context("Participant has no neurons. Make sure the SNS swap has been finalized.")?;

    // Use default permissions if not specified (SubmitProposal=3 + Vote=4)
    let permissions = permission_types.unwrap_or(vec![
        super::super::declarations::sns_governance::PERMISSION_TYPE_SUBMIT_PROPOSAL, // 3
        super::super::declarations::sns_governance::PERMISSION_TYPE_VOTE,            // 4
    ]);

    // Add hotkey
    add_hotkey_to_neuron(
        &agent,
        governance_canister,
        neuron_id.id,
        hotkey_principal,
        permissions,
    )
    .await
    .context("Failed to add hotkey to neuron")?;

    Ok(())
}

/// Convenience function that reads deployment data from the default location
pub async fn add_hotkey_to_participant_neuron_default_path(
    participant_principal: Principal,
    hotkey_principal: Principal,
    permission_types: Option<Vec<i32>>,
) -> Result<()> {
    let deployment_path = crate::core::utils::data_output::get_output_path();
    add_hotkey_to_participant_neuron(
        &deployment_path,
        participant_principal,
        hotkey_principal,
        permission_types,
    )
    .await
}

/// Disburse a neuron to a specific principal
/// This disburses the full amount of the neuron
pub async fn disburse_neuron(
    agent: &Agent,
    governance_canister: Principal,
    neuron_subaccount: Vec<u8>,
    receiver_principal: Principal,
) -> Result<u64> {
    let command = Command::Disburse(Disburse {
        to_account: Some(Account {
            owner: Some(receiver_principal),
            subaccount: None,
        }),
        amount: None, // None means disburse full amount
    });

    let request = ManageNeuron {
        subaccount: neuron_subaccount.clone(),
        command: Some(command),
    };
    let args = candid::encode_args((request,))?;

    let response = agent
        .update(&governance_canister, "manage_neuron")
        .with_arg(args)
        .call_and_wait()
        .await
        .context("Failed to call manage_neuron")?;

    let result: ManageNeuronResponse = Decode!(&response, ManageNeuronResponse)?;

    // Check for errors
    if let Some(cmd) = result.command {
        match cmd {
            super::super::declarations::sns_governance::Command1::Error(e) => {
                anyhow::bail!(
                    "Governance error: {} (type: {})",
                    e.error_message,
                    e.error_type
                );
            }
            super::super::declarations::sns_governance::Command1::Disburse(response) => {
                Ok(response.transfer_block_height)
            }
            _ => {
                anyhow::bail!("Unexpected response type from manage_neuron")
            }
        }
    } else {
        anyhow::bail!("No response from manage_neuron")
    }
}

/// High-level function to disburse a participant's neuron to a receiver
/// This reads deployment data, loads the participant identity, and disburses the neuron
pub async fn disburse_participant_neuron(
    deployment_data_path: &std::path::Path,
    participant_principal: Principal,
    receiver_principal: Principal,
) -> Result<u64> {
    use super::identity::{create_agent, load_identity_from_seed_file};

    // Read deployment data
    let data_content = std::fs::read_to_string(deployment_data_path).with_context(|| {
        format!(
            "Failed to read deployment data from: {:?}",
            deployment_data_path
        )
    })?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    // Find participant seed file
    let participant_data = deployment_data
        .participants
        .iter()
        .find(|p| p.principal == participant_principal.to_string())
        .with_context(|| {
            format!(
                "Participant principal {} not found in deployment data",
                participant_principal
            )
        })?;

    // Load participant identity from seed file
    let seed_path = PathBuf::from(&participant_data.seed_file);
    let identity = load_identity_from_seed_file(&seed_path)
        .with_context(|| format!("Failed to load identity from: {}", seed_path.display()))?;

    // Create authenticated agent
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with participant identity")?;

    // Get governance canister ID
    let governance_canister = deployment_data
        .deployed_sns
        .governance_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse governance canister ID from deployment data")?;

    // Get participant neuron
    let neuron_id = get_participant_neuron_id(&agent, governance_canister, participant_principal)
        .await
        .context("Failed to get participant neuron ID")?
        .context("Participant has no neurons. Make sure the SNS swap has been finalized.")?;

    // Disburse neuron
    let block_height = disburse_neuron(
        &agent,
        governance_canister,
        neuron_id.id,
        receiver_principal,
    )
    .await
    .context("Failed to disburse neuron")?;

    Ok(block_height)
}

/// Convenience function that reads deployment data from the default location
pub async fn disburse_participant_neuron_default_path(
    participant_principal: Principal,
    receiver_principal: Principal,
) -> Result<u64> {
    let deployment_path = crate::core::utils::data_output::get_output_path();
    disburse_participant_neuron(&deployment_path, participant_principal, receiver_principal).await
}
