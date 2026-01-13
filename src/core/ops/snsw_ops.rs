// SNS-W (SNS Wrapper) canister operations

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use ic_agent::Agent;

use super::super::declarations::sns_wasm::{
    DeployedSns, GetDeployedSnsByProposalIdRequest, GetDeployedSnsByProposalIdResponse,
    GetDeployedSnsByProposalIdResult, ListDeployedSnsesArg, ListDeployedSnsesResponse,
};

/// Get deployed SNS by proposal ID
pub async fn get_deployed_sns(
    agent: &Agent,
    snsw_canister: Principal,
    proposal_id: u64,
) -> Result<DeployedSns> {
    let request = GetDeployedSnsByProposalIdRequest { proposal_id };

    let result_bytes = agent
        .query(&snsw_canister, "get_deployed_sns_by_proposal_id")
        .with_arg(encode_args((request,))?)
        .call()
        .await
        .context("Failed to get deployed SNS")?;

    let response: GetDeployedSnsByProposalIdResponse =
        Decode!(&result_bytes, GetDeployedSnsByProposalIdResponse)
            .context("Failed to decode deployed SNS response")?;

    match response.get_deployed_sns_by_proposal_id_result {
        Some(GetDeployedSnsByProposalIdResult::DeployedSns(sns)) => Ok(sns),
        Some(GetDeployedSnsByProposalIdResult::Error(err)) => {
            anyhow::bail!("SNS-W error: {}", err.message);
        }
        None => anyhow::bail!("No result from get_deployed_sns_by_proposal_id"),
    }
}

/// List all deployed SNSes
pub async fn list_deployed_snses(
    agent: &Agent,
    snsw_canister: Principal,
) -> Result<Vec<DeployedSns>> {
    let request = ListDeployedSnsesArg {};

    let result_bytes = agent
        .query(&snsw_canister, "list_deployed_snses")
        .with_arg(encode_args((request,))?)
        .call()
        .await
        .context("Failed to list deployed SNSes")?;

    let response: ListDeployedSnsesResponse = Decode!(&result_bytes, ListDeployedSnsesResponse)
        .context("Failed to decode list_deployed_snses response")?;

    Ok(response.instances)
}

/// Check if any SNS is deployed
pub async fn check_sns_deployed(agent: &Agent, snsw_canister: Principal) -> Result<bool> {
    let deployed = list_deployed_snses(agent, snsw_canister).await?;
    Ok(!deployed.is_empty())
}

/// High-level function to check if SNS is deployed using default agent and canister
pub async fn check_sns_deployed_default_path() -> Result<bool> {
    use super::identity::create_agent;
    use crate::core::utils::constants::SNSW_CANISTER;

    let anonymous_identity = ic_agent::identity::AnonymousIdentity;
    let agent = create_agent(Box::new(anonymous_identity)).await?;
    let snsw_canister =
        Principal::from_text(SNSW_CANISTER).context("Failed to parse SNS-W canister ID")?;

    check_sns_deployed(&agent, snsw_canister).await
}
