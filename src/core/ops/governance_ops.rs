// ICP Governance operations

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use ic_agent::Agent;

use super::super::declarations::icp_governance::{
    AddHotKey, By, ClaimOrRefresh, ClaimOrRefreshResponse, Command1, Configure, Countries,
    CreateServiceNervousSystem, DeveloperDistribution, Duration, GovernanceParameters, Image,
    IncreaseDissolveDelay, InitialTokenDistribution, LedgerParameters, MakeProposalRequest,
    MakeProposalResponse, ManageNeuronCommandRequest, ManageNeuronRequest, ManageNeuronResponse,
    NeuronBasketConstructionParameters, NeuronDistribution, NeuronId, Operation, Percentage,
    ProposalActionRequest, ProposalId, SetVisibility, SwapDistribution, SwapParameters, Tokens,
    VotingRewardParameters,
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
    let logo_base64 = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAFJElEQVR4nG2WT4slZxXGf8+puvd298z0ZDLp9BCTjAlGBnElCAoDLty6cCTgwm8gfoCAe79CNu7cKAiShW6yUBAkKEgEEYOQsZ1BkkxPt9N/771VdR4Xb1Xd23On4HKr3fe9589zznnOq//+8K4YnuDxw+bkuJMAbHZvVW/cnRjAlXmiyQ9uv31YRQBm6Xj/zqMHN48XDkA1zz7xkz9mVL2+hFoYKDaEimqBKT+QleXVoyfjq2RpJSohW4AE";

    let sns_data = CreateServiceNervousSystem {
        name: Some("Toolkit".to_string()),
        description: Some("Toolkit is a versatile suite for managing Service Nervous Systems (SNS) and projects on the Internet Computer. From governance proposals to canister deployment, it empowers users to innovate, collaborate, and decentralize seamlessly.".to_string()),
        url: Some("https://ic-toolkit.app".to_string()),
        logo: Some(Image {
            base64_encoding: Some(logo_base64.to_string()),
        }),
        fallback_controller_principal_ids: vec![owner_principal],
        dapp_canisters: vec![],
        ledger_parameters: Some(LedgerParameters {
            transaction_fee: Some(Tokens { e8s: Some(10_000) }),
            token_symbol: Some("TKT".to_string()),
            token_logo: Some(Image {
                base64_encoding: Some(logo_base64.to_string()),
            }),
            token_name: Some("Toolkit Token".to_string()),
        }),
        governance_parameters: Some(GovernanceParameters {
            neuron_maximum_dissolve_delay_bonus: Some(Percentage { basis_points: Some(10_000) }),
            neuron_maximum_age_bonus: Some(Percentage { basis_points: Some(0) }),
            neuron_minimum_stake: Some(Tokens { e8s: Some(10_000_000) }),
            neuron_maximum_age_for_age_bonus: Some(Duration { seconds: Some(4 * 365 * 24 * 60 * 60) }),
            neuron_maximum_dissolve_delay: Some(Duration { seconds: Some(8 * 365 * 24 * 60 * 60) }),
            neuron_minimum_dissolve_delay_to_vote: Some(Duration { seconds: Some(30 * 24 * 60 * 60) }),
            proposal_initial_voting_period: Some(Duration { seconds: Some(4 * 24 * 60 * 60) }),
            proposal_wait_for_quiet_deadline_increase: Some(Duration { seconds: Some(24 * 60 * 60) }),
            proposal_rejection_fee: Some(Tokens { e8s: Some(11_000_000) }),
            voting_reward_parameters: Some(VotingRewardParameters {
                initial_reward_rate: Some(Percentage { basis_points: Some(0) }),
                final_reward_rate: Some(Percentage { basis_points: Some(0) }),
                reward_rate_transition_duration: Some(Duration { seconds: Some(0) }),
            }),
        }),
        swap_parameters: Some(SwapParameters {
            minimum_participants: Some(5),
            neurons_fund_participation: Some(false),
            minimum_direct_participation_icp: Some(Tokens { e8s: Some(100_000_000 * 5) }),
            maximum_direct_participation_icp: Some(Tokens { e8s: Some(1_000_000_000 * 5) }),
            minimum_participant_icp: Some(Tokens { e8s: Some(100_000_000) }),
            maximum_participant_icp: Some(Tokens { e8s: Some(1_000_000_000) }),
            confirmation_text: None,
            minimum_icp: None,
            maximum_icp: None,
            neurons_fund_investment_icp: None,
            restricted_countries: Some(Countries {
                iso_codes: vec!["AQ".to_string()],
            }),
            start_time: None,
            duration: Some(Duration { seconds: Some(7 * 24 * 60 * 60) }),
            neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
                count: Some(3),
                dissolve_delay_interval: Some(Duration { seconds: Some(30 * 24 * 60 * 60) }),
            }),
        }),
        initial_token_distribution: Some(InitialTokenDistribution {
            treasury_distribution: Some(SwapDistribution {
                total: Some(Tokens { e8s: Some(1_000_000_000) }),
            }),
            developer_distribution: Some(DeveloperDistribution {
                developer_neurons: vec![NeuronDistribution {
                    controller: Some(owner_principal),
                    dissolve_delay: Some(Duration { seconds: Some(2 * 365 * 24 * 60 * 60) }),
                    memo: Some(0),
                    vesting_period: Some(Duration { seconds: Some(4 * 365 * 24 * 60 * 60) }),
                    stake: Some(Tokens { e8s: Some(100_000_000) }),
                }],
            }),
            swap_distribution: Some(SwapDistribution {
                total: Some(Tokens { e8s: Some(2_000_000_000) }),
            }),
        }),
    };

    let proposal = MakeProposalRequest {
        url: "".to_string(),
        title: Some("Deploy Toolkit SNS".to_string()),
        summary: "Deploy Toolkit SNS summary".to_string(),
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
    let deployment_path = super::super::data_output::get_output_path();
    let data_content = std::fs::read_to_string(&deployment_path)
        .with_context(|| format!("Failed to read deployment data from: {:?}", deployment_path))?;
    let deployment_data: super::super::data_output::SnsCreationData =
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

/// High-level function to set visibility for the ICP neuron used for SNS deployment
/// This reads deployment data, loads the owner identity, and sets the visibility
pub async fn set_icp_neuron_visibility_default_path(is_public: bool) -> Result<()> {
    use super::identity::{create_agent, load_dfx_identity};

    // Read deployment data
    let deployment_path = super::super::data_output::get_output_path();
    let data_content = std::fs::read_to_string(&deployment_path)
        .with_context(|| format!("Failed to read deployment data from: {:?}", deployment_path))?;
    let deployment_data: super::super::data_output::SnsCreationData =
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
        let deployment_path = super::super::data_output::get_output_path();
        let data_content = std::fs::read_to_string(&deployment_path).with_context(|| {
            format!("Failed to read deployment data from: {:?}", deployment_path)
        })?;
        let deployment_data: super::super::data_output::SnsCreationData =
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
