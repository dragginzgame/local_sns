// SNS Governance canister operations

use anyhow::{Context, Result};
use candid::{Decode, Principal};
use ic_agent::Agent;
use std::path::PathBuf;

use super::super::declarations::sns_governance::{
    AddNeuronPermissions, Command, ListNeurons, ListNeuronsResponse, ManageNeuron,
    ManageNeuronResponse, MintSnsTokens, Neuron, NeuronId, NeuronPermissionList, Proposal,
    ProposalAction, ProposalId, ProposalResponse, RegisterVote, VOTE_YES,
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
            super::super::declarations::sns_governance::Command1::MakeProposal(_) => {
                // Success (handled separately)
            }
            super::super::declarations::sns_governance::Command1::RegisterVote {} => {
                // Success
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

/// Create a proposal to mint SNS tokens
pub async fn create_mint_proposal(
    agent: &Agent,
    governance_canister: Principal,
    neuron_subaccount: Vec<u8>,
    to_principal: Principal,
    amount_e8s: u64,
    title: String,
    summary: String,
) -> Result<ProposalId> {
    let proposal = Proposal {
        title: Some(title),
        summary,
        url: "".to_string(),
        action: Some(ProposalAction::MintSnsTokens(MintSnsTokens {
            to: Some(to_principal),
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
        .context("Failed to call manage_neuron for proposal creation")?;

    let result: ManageNeuronResponse = Decode!(&response, ManageNeuronResponse)?;

    match result.command {
        Some(crate::core::declarations::sns_governance::Command1::MakeProposal(
            ProposalResponse {
                proposal_id: Some(id),
            },
        )) => Ok(id),
        Some(crate::core::declarations::sns_governance::Command1::Error(e)) => {
            anyhow::bail!(
                "Failed to create proposal: {} (type: {})",
                e.error_message,
                e.error_type
            );
        }
        _ => anyhow::bail!("Unexpected response from make_proposal"),
    }
}

/// Vote on an SNS proposal
pub async fn vote_on_proposal(
    agent: &Agent,
    governance_canister: Principal,
    neuron_subaccount: Vec<u8>,
    proposal_id: ProposalId,
    vote_yes: bool,
) -> Result<()> {
    let vote_value = if vote_yes { VOTE_YES } else { 2 }; // 1 = Yes, 2 = No

    let command = Command::RegisterVote(RegisterVote {
        vote: vote_value,
        proposal: Some(proposal_id),
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
        .context("Failed to call manage_neuron for voting")?;

    let result: ManageNeuronResponse = Decode!(&response, ManageNeuronResponse)?;

    match result.command {
        Some(crate::core::declarations::sns_governance::Command1::RegisterVote {}) => Ok(()),
        Some(crate::core::declarations::sns_governance::Command1::Error(e)) => {
            anyhow::bail!(
                "Failed to vote: {} (type: {})",
                e.error_message,
                e.error_type
            );
        }
        _ => anyhow::bail!("Unexpected response from register_vote"),
    }
}

/// High-level function to create a mint proposal and have all participants vote yes
pub async fn mint_tokens_and_vote(
    proposer_principal: Principal,
    to_principal: Principal,
    amount_e8s: u64,
) -> Result<ProposalId> {
    use super::identity::{create_agent, load_identity_from_seed_file};

    // Read deployment data
    let deployment_path = crate::core::utils::data_output::get_output_path();
    let data_content = std::fs::read_to_string(&deployment_path)
        .with_context(|| format!("Failed to read deployment data from: {:?}", deployment_path))?;
    let deployment_data: crate::core::utils::data_output::SnsCreationData =
        serde_json::from_str(&data_content).context("Failed to parse deployment data JSON")?;

    // Get governance canister ID
    let governance_canister = deployment_data
        .deployed_sns
        .governance_canister_id
        .as_ref()
        .and_then(|s| Principal::from_text(s).ok())
        .context("Failed to parse governance canister ID from deployment data")?;

    // Find proposer in participants
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

    // Load proposer identity
    let seed_path = std::path::PathBuf::from(&proposer_data.seed_file);
    let identity = load_identity_from_seed_file(&seed_path)
        .with_context(|| format!("Failed to load identity from: {}", seed_path.display()))?;
    let agent = create_agent(identity)
        .await
        .context("Failed to create agent with proposer identity")?;

    // Get proposer neuron
    let neuron_id = get_participant_neuron_id(&agent, governance_canister, proposer_principal)
        .await
        .context("Failed to get proposer neuron ID")?
        .context("Proposer has no neurons")?;

    // Create proposal
    let proposal_id = create_mint_proposal(
        &agent,
        governance_canister,
        neuron_id.id.clone(),
        to_principal,
        amount_e8s,
        format!("Mint {} tokens to {}", amount_e8s, to_principal),
        format!(
            "Proposal to mint {} e8s tokens to principal {}",
            amount_e8s, to_principal
        ),
    )
    .await
    .context("Failed to create mint proposal")?;

    // Vote with all participants (excluding proposer - proposer doesn't need to vote)
    for participant in &deployment_data.participants {
        let participant_principal = Principal::from_text(&participant.principal)
            .with_context(|| format!("Failed to parse principal: {}", participant.principal))?;

        // Skip the proposer (they don't need to vote)
        if participant_principal == proposer_principal {
            continue;
        }

        // Load participant identity
        let participant_seed_path = std::path::PathBuf::from(&participant.seed_file);
        let participant_identity = load_identity_from_seed_file(&participant_seed_path)
            .with_context(|| {
                format!(
                    "Failed to load participant identity from: {}",
                    participant_seed_path.display()
                )
            })?;
        let participant_agent = create_agent(participant_identity)
            .await
            .context("Failed to create agent with participant identity")?;

        // Get ALL neurons for this participant (participants can have multiple neurons)
        let neurons = list_neurons_for_principal(
            &participant_agent,
            governance_canister,
            participant_principal,
        )
        .await
        .context("Failed to list neurons for participant")?;

        if neurons.is_empty() {
            anyhow::bail!("Participant {} has no neurons", participant.principal);
        }

        // Vote yes with all neurons for this participant
        for neuron in &neurons {
            if let Some(neuron_id) = &neuron.id {
                match vote_on_proposal(
                    &participant_agent,
                    governance_canister,
                    neuron_id.id.clone(),
                    proposal_id.clone(),
                    true,
                )
                .await
                {
                    Ok(_) => {
                        eprintln!("Voted yes with participant {}", participant.principal,);
                    }
                    Err(e) => {
                        // Vote failed - log warning but continue with other neurons
                        // Common reasons: neuron already voted (following another neuron), insufficient permissions, etc.
                        let neuron_id_hex = hex::encode(&neuron_id.id);
                        let short_id = if neuron_id_hex.len() > 16 {
                            format!(
                                "{}...{}",
                                &neuron_id_hex[..8],
                                &neuron_id_hex[neuron_id_hex.len() - 8..]
                            )
                        } else {
                            neuron_id_hex.clone()
                        };
                        eprintln!(
                            "⚠ Failed to vote with participant {} neuron ({}): {}. Continuing with other neurons.",
                            participant.principal, short_id, e
                        );
                    }
                }
            }
        }
    }

    Ok(proposal_id)
}
