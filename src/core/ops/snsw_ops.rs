// SNS-W (SNS Wrapper) canister operations

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use ic_agent::Agent;

use super::super::declarations::sns_wasm::{
    DeployedSns, GetDeployedSnsByProposalIdRequest, GetDeployedSnsByProposalIdResponse,
    GetDeployedSnsByProposalIdResult,
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
