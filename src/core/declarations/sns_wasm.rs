// SNS-W (SNS Wrapper) canister Candid type definitions
// Generated from Candid, with serde_bytes::ByteBuf replaced with Vec<u8>

#![allow(dead_code, unused_imports, unused_variables)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsWasmCanisterInitPayload {
    pub allowed_principals: Vec<Principal>,
    pub access_controls_enabled: bool,
    pub sns_subnet_ids: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsWasm {
    pub wasm: Vec<u8>, // Changed from serde_bytes::ByteBuf
    pub proposal_id: Option<u64>,
    pub canister_type: i32,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct AddWasmRequest {
    pub hash: Vec<u8>, // Changed from serde_bytes::ByteBuf
    pub wasm: Option<SnsWasm>,
    pub skip_update_latest_version: Option<bool>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsWasmError {
    pub message: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum Result_ {
    Error(SnsWasmError),
    Hash(Vec<u8>), // Changed from serde_bytes::ByteBuf
}

#[derive(CandidType, Deserialize, Debug)]
pub struct AddWasmResponse {
    pub result: Option<Result_>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct NeuronBasketConstructionParameters {
    pub dissolve_delay_interval_seconds: u64,
    pub count: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Canister {
    pub id: Option<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct DappCanisters {
    pub canisters: Vec<Canister>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct LinearScalingCoefficient {
    pub slope_numerator: Option<u64>,
    pub intercept_icp_e8s: Option<u64>,
    pub from_direct_participation_icp_e8s: Option<u64>,
    pub slope_denominator: Option<u64>,
    pub to_direct_participation_icp_e8s: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct IdealMatchedParticipationFunction {
    pub serialized_representation: Option<String>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct NeuronsFundParticipationConstraints {
    pub coefficient_intervals: Vec<LinearScalingCoefficient>,
    pub max_neurons_fund_participation_icp_e8s: Option<u64>,
    pub min_direct_participation_threshold_icp_e8s: Option<u64>,
    pub ideal_matched_participation_function: Option<IdealMatchedParticipationFunction>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct TreasuryDistribution {
    pub total_e8s: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct NeuronDistribution {
    pub controller: Option<Principal>,
    pub dissolve_delay_seconds: u64,
    pub memo: u64,
    pub stake_e8s: u64,
    pub vesting_period_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct DeveloperDistribution {
    pub developer_neurons: Vec<NeuronDistribution>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SwapDistribution {
    pub total_e8s: u64,
    pub initial_swap_amount_e8s: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct FractionalDeveloperVotingPower {
    pub treasury_distribution: Option<TreasuryDistribution>,
    pub developer_distribution: Option<DeveloperDistribution>,
    pub swap_distribution: Option<SwapDistribution>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum InitialTokenDistribution {
    FractionalDeveloperVotingPower(FractionalDeveloperVotingPower),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Countries {
    pub iso_codes: Vec<String>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsInitPayload {
    pub url: Option<String>,
    pub max_dissolve_delay_seconds: Option<u64>,
    pub max_dissolve_delay_bonus_percentage: Option<u64>,
    pub nns_proposal_id: Option<u64>,
    pub neurons_fund_participation: Option<bool>,
    pub min_participant_icp_e8s: Option<u64>,
    pub neuron_basket_construction_parameters: Option<NeuronBasketConstructionParameters>,
    pub fallback_controller_principal_ids: Vec<String>,
    pub token_symbol: Option<String>,
    pub final_reward_rate_basis_points: Option<u64>,
    pub max_icp_e8s: Option<u64>,
    pub neuron_minimum_stake_e8s: Option<u64>,
    pub confirmation_text: Option<String>,
    pub logo: Option<String>,
    pub name: Option<String>,
    pub swap_start_timestamp_seconds: Option<u64>,
    pub swap_due_timestamp_seconds: Option<u64>,
    pub initial_voting_period_seconds: Option<u64>,
    pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
    pub description: Option<String>,
    pub max_neuron_age_seconds_for_age_bonus: Option<u64>,
    pub min_participants: Option<u64>,
    pub initial_reward_rate_basis_points: Option<u64>,
    pub wait_for_quiet_deadline_increase_seconds: Option<u64>,
    pub transaction_fee_e8s: Option<u64>,
    pub dapp_canisters: Option<DappCanisters>,
    pub neurons_fund_participation_constraints: Option<NeuronsFundParticipationConstraints>,
    pub max_age_bonus_percentage: Option<u64>,
    pub initial_token_distribution: Option<InitialTokenDistribution>,
    pub reward_rate_transition_duration_seconds: Option<u64>,
    pub token_logo: Option<String>,
    pub token_name: Option<String>,
    pub max_participant_icp_e8s: Option<u64>,
    pub min_direct_participation_icp_e8s: Option<u64>,
    pub proposal_reject_cost_e8s: Option<u64>,
    pub restricted_countries: Option<Countries>,
    pub min_icp_e8s: Option<u64>,
    pub max_direct_participation_icp_e8s: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct DeployNewSnsRequest {
    pub sns_init_payload: Option<SnsInitPayload>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct DappCanistersTransferResult {
    pub restored_dapp_canisters: Vec<Canister>,
    pub nns_controlled_dapp_canisters: Vec<Canister>,
    pub sns_controlled_dapp_canisters: Vec<Canister>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsCanisterIds {
    pub root: Option<Principal>,
    pub swap: Option<Principal>,
    pub ledger: Option<Principal>,
    pub index: Option<Principal>,
    pub governance: Option<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct DeployNewSnsResponse {
    pub dapp_canisters_transfer_result: Option<DappCanistersTransferResult>,
    pub subnet_id: Option<Principal>,
    pub error: Option<SnsWasmError>,
    pub canisters: Option<SnsCanisterIds>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetAllowedPrincipalsArg {}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetAllowedPrincipalsResponse {
    pub allowed_principals: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetDeployedSnsByProposalIdRequest {
    pub proposal_id: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct DeployedSns {
    pub root_canister_id: Option<Principal>,
    pub governance_canister_id: Option<Principal>,
    pub index_canister_id: Option<Principal>,
    pub swap_canister_id: Option<Principal>,
    pub ledger_canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum GetDeployedSnsByProposalIdResult {
    Error(SnsWasmError),
    #[serde(rename = "DeployedSns")]
    DeployedSns(DeployedSns),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetDeployedSnsByProposalIdResponse {
    pub get_deployed_sns_by_proposal_id_result: Option<GetDeployedSnsByProposalIdResult>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsVersion {
    pub archive_wasm_hash: Vec<u8>,    // Changed from serde_bytes::ByteBuf
    pub root_wasm_hash: Vec<u8>,       // Changed from serde_bytes::ByteBuf
    pub swap_wasm_hash: Vec<u8>,       // Changed from serde_bytes::ByteBuf
    pub ledger_wasm_hash: Vec<u8>,     // Changed from serde_bytes::ByteBuf
    pub governance_wasm_hash: Vec<u8>, // Changed from serde_bytes::ByteBuf
    pub index_wasm_hash: Vec<u8>,      // Changed from serde_bytes::ByteBuf
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetNextSnsVersionRequest {
    pub governance_canister_id: Option<Principal>,
    pub current_version: Option<SnsVersion>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetNextSnsVersionResponse {
    pub next_version: Option<SnsVersion>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetProposalIdThatAddedWasmRequest {
    pub hash: Vec<u8>, // Changed from serde_bytes::ByteBuf
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetProposalIdThatAddedWasmResponse {
    pub proposal_id: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetSnsSubnetIdsArg {}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetSnsSubnetIdsResponse {
    pub sns_subnet_ids: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetWasmRequest {
    pub hash: Vec<u8>, // Changed from serde_bytes::ByteBuf
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetWasmResponse {
    pub wasm: Option<SnsWasm>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetWasmMetadataRequest {
    pub hash: Option<Vec<u8>>, // Changed from serde_bytes::ByteBuf
}

#[derive(CandidType, Deserialize, Debug)]
pub struct MetadataSection {
    pub contents: Option<Vec<u8>>, // Changed from serde_bytes::ByteBuf
    pub name: Option<String>,
    pub visibility: Option<String>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Ok {
    pub sections: Vec<MetadataSection>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum Result1 {
    Ok(Ok),
    Error(SnsWasmError),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetWasmMetadataResponse {
    pub result: Option<Result1>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SnsUpgrade {
    pub next_version: Option<SnsVersion>,
    pub current_version: Option<SnsVersion>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct InsertUpgradePathEntriesRequest {
    pub upgrade_path: Vec<SnsUpgrade>,
    pub sns_governance_canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct InsertUpgradePathEntriesResponse {
    pub error: Option<SnsWasmError>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ListDeployedSnsesArg {}

#[derive(CandidType, Deserialize, Debug)]
pub struct ListDeployedSnsesResponse {
    pub instances: Vec<DeployedSns>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ListUpgradeStepsRequest {
    pub limit: u32,
    pub starting_at: Option<SnsVersion>,
    pub sns_governance_canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct PrettySnsVersion {
    pub archive_wasm_hash: String,
    pub root_wasm_hash: String,
    pub swap_wasm_hash: String,
    pub ledger_wasm_hash: String,
    pub governance_wasm_hash: String,
    pub index_wasm_hash: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ListUpgradeStep {
    pub pretty_version: Option<PrettySnsVersion>,
    pub version: Option<SnsVersion>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ListUpgradeStepsResponse {
    pub steps: Vec<ListUpgradeStep>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UpdateAllowedPrincipalsRequest {
    pub added_principals: Vec<Principal>,
    pub removed_principals: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum UpdateAllowedPrincipalsResult {
    Error(SnsWasmError),
    AllowedPrincipals(GetAllowedPrincipalsResponse),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UpdateAllowedPrincipalsResponse {
    pub update_allowed_principals_result: Option<UpdateAllowedPrincipalsResult>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UpdateSnsSubnetListRequest {
    pub sns_subnet_ids_to_add: Vec<Principal>,
    pub sns_subnet_ids_to_remove: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UpdateSnsSubnetListResponse {
    pub error: Option<SnsWasmError>,
}
