// Output data structure for SNS creation results

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantData {
    pub principal: String,
    pub seed_file: String, // Path to the seed file
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnsCreationData {
    pub icp_neuron_id: u64,
    pub proposal_id: u64,
    pub owner_principal: String,
    pub deployed_sns: DeployedSnsData,
    pub participants: Vec<ParticipantData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployedSnsData {
    pub root_canister_id: Option<String>,
    pub governance_canister_id: Option<String>,
    pub index_canister_id: Option<String>,
    pub swap_canister_id: Option<String>,
    pub ledger_canister_id: Option<String>,
}

impl From<&super::declarations::sns_wasm::DeployedSns> for DeployedSnsData {
    fn from(sns: &super::declarations::sns_wasm::DeployedSns) -> Self {
        DeployedSnsData {
            root_canister_id: sns.root_canister_id.map(|p| p.to_string()),
            governance_canister_id: sns.governance_canister_id.map(|p| p.to_string()),
            index_canister_id: sns.index_canister_id.map(|p| p.to_string()),
            swap_canister_id: sns.swap_canister_id.map(|p| p.to_string()),
            ledger_canister_id: sns.ledger_canister_id.map(|p| p.to_string()),
        }
    }
}

const OUTPUT_DIR: &str = "generated";
const OUTPUT_FILE: &str = "sns_deployment_data.json";

pub fn get_output_dir() -> PathBuf {
    PathBuf::from(OUTPUT_DIR)
}

pub fn get_output_path() -> PathBuf {
    get_output_dir().join(OUTPUT_FILE)
}

/// Ensure the output directory exists
pub fn ensure_output_dir() -> anyhow::Result<()> {
    let dir = get_output_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create output directory: {}", dir.display()))?;
    Ok(())
}

pub fn write_data(data: &SnsCreationData) -> anyhow::Result<()> {
    ensure_output_dir()?;
    let path = get_output_path();
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, json)?;
    Ok(())
}
