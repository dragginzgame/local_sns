// SNS Governance canister operations

use anyhow::{Context, Result};
use candid::{Decode, Principal};
use ic_agent::Agent;
use std::path::PathBuf;

#[allow(unused_imports)]
use super::super::declarations::sns_governance::{
    Account, Action, AddNeuronPermissions, Command, Disburse, DissolveState, GetProposal,
    ListNeurons, ListNeuronsResponse, ManageNeuron, ManageNeuronResponse, MintSnsTokens, Neuron,
    NeuronId, NeuronPermissionList, Proposal, ProposalId, RegisterVote,
};

/// List all neurons for a given principal, sorted by dissolve delay (lowest first) and cached stake (highest first)
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

    // Sort neurons by dissolve delay (lowest first), then by cached stake (highest first)
    let mut neurons = result.neurons;
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
/// If neuron_id is None, automatically finds the neuron with longest dissolve delay
pub async fn add_hotkey_to_participant_neuron(
    deployment_data_path: &std::path::Path,
    participant_principal: Principal,
    hotkey_principal: Principal,
    permission_types: Option<Vec<i32>>,
    neuron_id: Option<Vec<u8>>,
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

    // Use neuron_id if provided, otherwise find it automatically
    let neuron_subaccount = if let Some(id) = neuron_id {
        id
    } else {
        // Get neurons (sorted by dissolve delay, then by cached stake)
        let neurons =
            list_neurons_for_principal(&agent, governance_canister, participant_principal)
                .await
                .context("Failed to list neurons")?;

        // Get the neuron with the longest dissolve delay (last in sorted list, skipping dissolving/None)
        // Filter out dissolving neurons and ones with no state for this use case
        neurons
            .iter()
            .rev()
            .find(|n| {
                matches!(
                    n.dissolve_state,
                    Some(DissolveState::DissolveDelaySeconds(_))
                )
            })
            .and_then(|n| n.id.as_ref())
            .or_else(|| neurons.last().and_then(|n| n.id.as_ref()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Participant has no neurons. Make sure the SNS swap has been finalized."
                )
            })?
            .id
            .clone()
    };

    // Use default permissions if not specified (SubmitProposal=3 + Vote=4)
    let permissions = permission_types.unwrap_or(vec![
        super::super::declarations::sns_governance::PERMISSION_TYPE_SUBMIT_PROPOSAL, // 3
        super::super::declarations::sns_governance::PERMISSION_TYPE_VOTE,            // 4
    ]);

    // Add hotkey
    add_hotkey_to_neuron(
        &agent,
        governance_canister,
        neuron_subaccount,
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
    neuron_id: Option<Vec<u8>>,
) -> Result<()> {
    let deployment_path = crate::core::utils::data_output::get_output_path();
    add_hotkey_to_participant_neuron(
        &deployment_path,
        participant_principal,
        hotkey_principal,
        permission_types,
        neuron_id,
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
/// If neuron_id is None, automatically finds the neuron with lowest dissolve delay
pub async fn disburse_participant_neuron(
    deployment_data_path: &std::path::Path,
    participant_principal: Principal,
    receiver_principal: Principal,
    neuron_id: Option<Vec<u8>>,
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

    // Use neuron_id if provided, otherwise find it automatically
    let neuron_subaccount = if let Some(id) = neuron_id {
        id
    } else {
        // Get neurons (sorted by dissolve delay, then by cached stake)
        let neurons =
            list_neurons_for_principal(&agent, governance_canister, participant_principal)
                .await
                .context("Failed to list neurons")?;

        // Get the neuron with the lowest dissolve delay (first in sorted list)
        neurons
            .first()
            .and_then(|n| n.id.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Participant has no neurons. Make sure the SNS swap has been finalized."
                )
            })?
            .id
            .clone()
    };

    // Disburse neuron
    let block_height = disburse_neuron(
        &agent,
        governance_canister,
        neuron_subaccount,
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
    neuron_id: Option<Vec<u8>>,
) -> Result<u64> {
    let deployment_path = crate::core::utils::data_output::get_output_path();
    disburse_participant_neuron(
        &deployment_path,
        participant_principal,
        receiver_principal,
        neuron_id,
    )
    .await
}

/// Create a proposal to mint SNS tokens
pub async fn make_mint_tokens_proposal(
    agent: &Agent,
    governance_canister: Principal,
    neuron_subaccount: Vec<u8>,
    receiver_principal: Principal,
    amount_e8s: u64,
) -> Result<u64> {
    let proposal = Proposal {
        url: "".to_string(),
        title: format!("Mint {} tokens to {}", amount_e8s, receiver_principal),
        summary: format!(
            "Proposal to mint {} e8s tokens to principal {}",
            amount_e8s, receiver_principal
        ),
        action: Some(Action::MintSnsTokens(MintSnsTokens {
            to_principal: Some(receiver_principal),
            to_subaccount: None,
            memo: None,
            amount_e8s: Some(amount_e8s),
        })),
    };

    let command = Command::MakeProposal(proposal);

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
        .context("Failed to call manage_neuron to create proposal")?;

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
            super::super::declarations::sns_governance::Command1::MakeProposal(get_proposal) => {
                // GetProposal contains proposal_id
                if let Some(proposal_id) = get_proposal.proposal_id {
                    Ok(proposal_id.id)
                } else {
                    anyhow::bail!("Proposal created but no proposal ID returned")
                }
            }
            _ => {
                anyhow::bail!("Unexpected response type from make_proposal")
            }
        }
    } else {
        anyhow::bail!("No response from manage_neuron")
    }
}

/// Vote on a proposal with a neuron
pub async fn vote_on_proposal(
    agent: &Agent,
    governance_canister: Principal,
    neuron_subaccount: Vec<u8>,
    proposal_id: u64,
    vote: i32, // 1 = Yes, 2 = No
) -> Result<()> {
    let command = Command::RegisterVote(RegisterVote {
        vote,
        proposal: Some(ProposalId { id: proposal_id }),
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
        .context("Failed to call manage_neuron to vote")?;

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
            super::super::declarations::sns_governance::Command1::RegisterVote {} => {
                // Success
                Ok(())
            }
            _ => {
                anyhow::bail!("Unexpected response type from register_vote")
            }
        }
    } else {
        anyhow::bail!("No response from manage_neuron")
    }
}

/// High-level function to mint SNS tokens by creating a proposal and getting all neurons to vote
pub async fn mint_sns_tokens_with_all_votes(
    deployment_data_path: &std::path::Path,
    proposer_principal: Principal,
    receiver_principal: Principal,
    amount_e8s: u64,
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

    // Find proposer seed file
    let proposer_data = deployment_data
        .participants
        .iter()
        .find(|p| p.principal == proposer_principal.to_string())
        .with_context(|| {
            format!(
                "Proposer principal {} not found in deployment data",
                proposer_principal
            )
        })?;

    // Load proposer identity from seed file
    let seed_path = PathBuf::from(&proposer_data.seed_file);
    let proposer_identity = load_identity_from_seed_file(&seed_path)
        .with_context(|| format!("Failed to load identity from: {}", seed_path.display()))?;

    // Create authenticated agent for proposer
    let proposer_agent = create_agent(proposer_identity)
        .await
        .context("Failed to create agent with proposer identity")?;

    // Get governance canister ID
    let governance_canister = deployment_data
        .deployed_sns
        .governance_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse governance canister ID from deployment data")?;

    // Get proposer neurons (sorted by dissolve delay, then by cached stake)
    let proposer_neurons =
        list_neurons_for_principal(&proposer_agent, governance_canister, proposer_principal)
            .await
            .context("Failed to list proposer neurons")?;

    // Get the neuron with the longest dissolve delay (last in sorted list, skipping dissolving/None)
    let proposer_neuron_id = proposer_neurons
        .iter()
        .rev()
        .find(|n| {
            matches!(
                n.dissolve_state,
                Some(DissolveState::DissolveDelaySeconds(_))
            )
        })
        .and_then(|n| n.id.as_ref())
        .or_else(|| proposer_neurons.last().and_then(|n| n.id.as_ref()))
        .ok_or_else(|| {
            anyhow::anyhow!("Proposer has no neurons. Make sure the SNS swap has been finalized.")
        })?;

    // Create the proposal
    let proposal_id = make_mint_tokens_proposal(
        &proposer_agent,
        governance_canister,
        proposer_neuron_id.id.clone(),
        receiver_principal,
        amount_e8s,
    )
    .await
    .context("Failed to create mint tokens proposal")?;

    // Now get the main neuron for each participant and have them vote
    // (other neurons follow the main one, so we only need the main one to vote)
    for participant in &deployment_data.participants {
        let participant_principal = Principal::from_text(&participant.principal)
            .context("Failed to parse participant principal")?;

        // Skip the proposer since they already created the proposal
        if participant_principal == proposer_principal {
            continue;
        }

        // Load participant identity
        let participant_seed_path = PathBuf::from(&participant.seed_file);
        let participant_identity = load_identity_from_seed_file(&participant_seed_path)
            .with_context(|| {
                format!(
                    "Failed to load identity from: {}",
                    participant_seed_path.display()
                )
            })?;

        // Create authenticated agent for participant
        let participant_agent = create_agent(participant_identity)
            .await
            .context("Failed to create agent with participant identity")?;

        // Get neurons for this participant (already sorted by dissolve delay, then by cached stake)
        let neurons = list_neurons_for_principal(
            &participant_agent,
            governance_canister,
            participant_principal,
        )
        .await
        .context("Failed to list neurons for participant")?;

        // Find the main neuron - the one with the longest dissolve delay (last in sorted list, skipping dissolving/None)
        // This is typically the neuron with highest stake that others follow
        let main_neuron = neurons
            .iter()
            .rev()
            .find(|n| {
                matches!(
                    n.dissolve_state,
                    Some(DissolveState::DissolveDelaySeconds(_))
                )
            })
            .and_then(|n| n.id.as_ref())
            .or_else(|| neurons.last().and_then(|n| n.id.as_ref()));

        if let Some(main_neuron_id) = main_neuron {
            // Vote yes on the proposal with the main neuron
            vote_on_proposal(
                &participant_agent,
                governance_canister,
                main_neuron_id.id.clone(),
                proposal_id,
                1, // Yes
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to vote with main neuron for participant {}",
                    participant_principal
                )
            })?;
        } else {
            anyhow::bail!("No neurons found for participant {}", participant_principal);
        }
    }

    Ok(proposal_id)
}

/// Convenience function that reads deployment data from the default location
pub async fn mint_sns_tokens_with_all_votes_default_path(
    proposer_principal: Principal,
    receiver_principal: Principal,
    amount_e8s: u64,
) -> Result<u64> {
    let deployment_path = crate::core::utils::data_output::get_output_path();
    mint_sns_tokens_with_all_votes(
        &deployment_path,
        proposer_principal,
        receiver_principal,
        amount_e8s,
    )
    .await
}
