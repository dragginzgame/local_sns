// ICP Governance canister Candid type definitions
// Generated from Candid, with serde_bytes::ByteBuf replaced with Vec<u8>

#![allow(dead_code, unused_imports, unused_variables)]

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize)]
pub struct NeuronId {
    pub id: u64,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct Followees {
    pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct DateUtc {
    pub day: u32,
    pub month: u32,
    pub year: u32,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct AccountIdentifier {
    pub hash: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct NodeProvider {
    pub id: Option<Principal>,
    pub reward_account: Option<AccountIdentifier>,
}

#[derive(CandidType, Deserialize)]
pub struct RewardToNeuron {
    pub dissolve_delay_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct RewardToAccount {
    pub to_account: Option<AccountIdentifier>,
}

#[derive(CandidType, Deserialize)]
pub enum RewardMode {
    RewardToNeuron(RewardToNeuron),
    RewardToAccount(RewardToAccount),
}

#[derive(CandidType, Deserialize)]
pub struct RewardNodeProvider {
    pub node_provider: Option<NodeProvider>,
    pub reward_mode: Option<RewardMode>,
    pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct XdrConversionRate {
    pub xdr_permyriad_per_icp: Option<u64>,
    pub timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct MonthlyNodeProviderRewards {
    pub algorithm_version: Option<u32>,
    pub minimum_xdr_permyriad_per_icp: Option<u64>,
    pub end_date: Option<DateUtc>,
    pub registry_version: Option<u64>,
    pub node_providers: Vec<NodeProvider>,
    pub start_date: Option<DateUtc>,
    pub timestamp: u64,
    pub rewards: Vec<RewardNodeProvider>,
    pub xdr_conversion_rate: Option<XdrConversionRate>,
    pub maximum_node_provider_rewards_e8s: Option<u64>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct ProposalId {
    pub id: u64,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct NeuronStakeTransfer {
    pub to_subaccount: Vec<u8>,
    pub neuron_stake_e8s: u64,
    pub from: Option<Principal>,
    pub memo: u64,
    pub from_subaccount: Vec<u8>,
    pub transfer_timestamp: u64,
    pub block_height: u64,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct GovernanceError {
    pub error_message: String,
    pub error_type: i32,
}

#[derive(CandidType, Deserialize)]
pub struct Ballot {
    pub vote: i32,
    pub voting_power: u64,
}

#[derive(CandidType, Deserialize)]
pub struct CanisterStatusResultV2 {
    pub status: Option<i32>,
    pub freezing_threshold: Option<u64>,
    pub controllers: Vec<Principal>,
    pub memory_size: Option<u64>,
    pub cycles: Option<u64>,
    pub idle_cycles_burned_per_day: Option<u64>,
    pub module_hash: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct CanisterSummary {
    pub status: Option<CanisterStatusResultV2>,
    pub canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct Spawn {
    pub percentage_to_spawn: Option<u32>,
    pub new_controller: Option<Principal>,
    pub nonce: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Split {
    pub memo: Option<u64>,
    pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Follow {
    pub topic: i32,
    pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct Account {
    pub owner: Option<Principal>,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturity {
    pub to_account_identifier: Option<AccountIdentifier>,
    pub to_account: Option<Account>,
    pub percentage_to_disburse: u32,
}

#[derive(CandidType, Deserialize)]
pub struct RefreshVotingPower {}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefreshNeuronFromAccount {
    pub controller: Option<Principal>,
    pub memo: u64,
}

#[derive(CandidType, Deserialize)]
pub enum By {
    NeuronIdOrSubaccount {},
    MemoAndController(ClaimOrRefreshNeuronFromAccount),
    Memo(u64),
}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefresh {
    pub by: Option<By>,
}

#[derive(CandidType, Deserialize)]
pub struct RemoveHotKey {
    pub hot_key_to_remove: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct AddHotKey {
    pub new_hot_key: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct ChangeAutoStakeMaturity {
    pub requested_setting_for_auto_stake_maturity: bool,
}

#[derive(CandidType, Deserialize)]
pub struct IncreaseDissolveDelay {
    pub additional_dissolve_delay_seconds: u32,
}

#[derive(CandidType, Deserialize)]
pub struct SetVisibility {
    pub visibility: Option<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct SetDissolveTimestamp {
    pub dissolve_timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub enum Operation {
    RemoveHotKey(RemoveHotKey),
    AddHotKey(AddHotKey),
    ChangeAutoStakeMaturity(ChangeAutoStakeMaturity),
    StopDissolving {},
    StartDissolving {},
    IncreaseDissolveDelay(IncreaseDissolveDelay),
    SetVisibility(SetVisibility),
    JoinCommunityFund {},
    LeaveCommunityFund {},
    SetDissolveTimestamp(SetDissolveTimestamp),
}

#[derive(CandidType, Deserialize)]
pub struct Configure {
    pub operation: Option<Operation>,
}

#[derive(CandidType, Deserialize)]
pub struct RegisterVote {
    pub vote: i32,
    pub proposal: Option<ProposalId>,
}

#[derive(CandidType, Deserialize)]
pub struct Merge {
    pub source_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseToNeuron {
    pub dissolve_delay_seconds: u64,
    pub kyc_verified: bool,
    pub amount_e8s: u64,
    pub new_controller: Option<Principal>,
    pub nonce: u64,
}

#[derive(CandidType, Deserialize)]
pub struct FolloweesForTopic {
    pub topic: Option<i32>,
    pub followees: Option<Vec<NeuronId>>,
}

#[derive(CandidType, Deserialize)]
pub struct SetFollowing {
    pub topic_following: Option<Vec<FolloweesForTopic>>,
}

#[derive(CandidType, Deserialize)]
pub struct StakeMaturity {
    pub percentage_to_stake: Option<u32>,
}

#[derive(CandidType, Deserialize)]
pub struct MergeMaturity {
    pub percentage_to_merge: u32,
}

#[derive(CandidType, Deserialize)]
pub struct Amount {
    pub e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Disburse {
    pub to_account: Option<AccountIdentifier>,
    pub amount: Option<Amount>,
}

#[derive(CandidType, Deserialize)]
pub enum NeuronIdOrSubaccount {
    Subaccount(Vec<u8>),
    NeuronId(NeuronId),
}

#[derive(CandidType, Deserialize)]
pub struct Duration {
    pub seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Tokens {
    pub e8s: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Percentage {
    pub basis_points: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct VotingRewardParameters {
    pub reward_rate_transition_duration: Option<Duration>,
    pub initial_reward_rate: Option<Percentage>,
    pub final_reward_rate: Option<Percentage>,
}

#[derive(CandidType, Deserialize)]
pub struct GovernanceParameters {
    pub neuron_maximum_dissolve_delay_bonus: Option<Percentage>,
    pub neuron_maximum_age_for_age_bonus: Option<Duration>,
    pub neuron_maximum_dissolve_delay: Option<Duration>,
    pub neuron_minimum_dissolve_delay_to_vote: Option<Duration>,
    pub neuron_maximum_age_bonus: Option<Percentage>,
    pub neuron_minimum_stake: Option<Tokens>,
    pub proposal_wait_for_quiet_deadline_increase: Option<Duration>,
    pub proposal_initial_voting_period: Option<Duration>,
    pub proposal_rejection_fee: Option<Tokens>,
    pub voting_reward_parameters: Option<VotingRewardParameters>,
}

#[derive(CandidType, Deserialize)]
pub struct Image {
    pub base64_encoding: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct LedgerParameters {
    pub transaction_fee: Option<Tokens>,
    pub token_symbol: Option<String>,
    pub token_logo: Option<Image>,
    pub token_name: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct Canister {
    pub id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronBasketConstructionParameters {
    pub dissolve_delay_interval: Option<Duration>,
    pub count: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct GlobalTimeOfDay {
    pub seconds_after_utc_midnight: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Countries {
    pub iso_codes: Vec<String>,
}

#[derive(CandidType, Deserialize)]
pub struct SwapParameters {
    pub minimum_participants: Option<u64>,
    pub neurons_fund_participation: Option<bool>,
    pub duration: Option<Duration>,
    pub neuron_basket_construction_parameters: Option<NeuronBasketConstructionParameters>,
    pub confirmation_text: Option<String>,
    pub maximum_participant_icp: Option<Tokens>,
    pub minimum_icp: Option<Tokens>,
    pub minimum_direct_participation_icp: Option<Tokens>,
    pub minimum_participant_icp: Option<Tokens>,
    pub start_time: Option<GlobalTimeOfDay>,
    pub maximum_direct_participation_icp: Option<Tokens>,
    pub maximum_icp: Option<Tokens>,
    pub neurons_fund_investment_icp: Option<Tokens>,
    pub restricted_countries: Option<Countries>,
}

#[derive(CandidType, Deserialize)]
pub struct SwapDistribution {
    pub total: Option<Tokens>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronDistribution {
    pub controller: Option<Principal>,
    pub dissolve_delay: Option<Duration>,
    pub memo: Option<u64>,
    pub vesting_period: Option<Duration>,
    pub stake: Option<Tokens>,
}

#[derive(CandidType, Deserialize)]
pub struct DeveloperDistribution {
    pub developer_neurons: Vec<NeuronDistribution>,
}

#[derive(CandidType, Deserialize)]
pub struct InitialTokenDistribution {
    pub treasury_distribution: Option<SwapDistribution>,
    pub developer_distribution: Option<DeveloperDistribution>,
    pub swap_distribution: Option<SwapDistribution>,
}

#[derive(CandidType, Deserialize)]
pub struct CreateServiceNervousSystem {
    pub url: Option<String>,
    pub governance_parameters: Option<GovernanceParameters>,
    pub fallback_controller_principal_ids: Vec<Principal>,
    pub logo: Option<Image>,
    pub name: Option<String>,
    pub ledger_parameters: Option<LedgerParameters>,
    pub description: Option<String>,
    pub dapp_canisters: Vec<Canister>,
    pub swap_parameters: Option<SwapParameters>,
    pub initial_token_distribution: Option<InitialTokenDistribution>,
}

#[derive(CandidType, Deserialize)]
pub struct ExecuteNnsFunction {
    pub nns_function: i32,
    pub payload: Vec<u8>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct KnownNeuronData {
    pub name: String,
    pub description: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct KnownNeuron {
    pub id: Option<NeuronId>,
    pub known_neuron_data: Option<KnownNeuronData>,
}

#[derive(CandidType, Deserialize)]
pub struct FulfillSubnetRentalRequest {
    pub user: Option<Principal>,
    pub replica_version_id: Option<String>,
    pub node_ids: Option<Vec<Principal>>,
}

#[derive(CandidType, Deserialize)]
pub struct UpdateCanisterSettings {
    pub canister_id: Option<Principal>,
    pub settings: Option<()>, // Simplified
}

#[derive(CandidType, Deserialize)]
pub struct InstallCodeRequest {
    pub arg: Option<Vec<u8>>,
    pub wasm_module: Option<Vec<u8>>,
    pub skip_stopping_before_installing: Option<bool>,
    pub canister_id: Option<Principal>,
    pub install_mode: Option<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct BlessAlternativeGuestOsVersion {
    pub rootfs_hash: Option<String>,
    pub chip_ids: Option<Vec<Vec<u8>>>,
    pub base_guest_launch_measurements: Option<()>, // Simplified
}

#[derive(CandidType, Deserialize)]
pub struct DeregisterKnownNeuron {
    pub id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct StopOrStartCanister {
    pub action: Option<i32>,
    pub canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct NetworkEconomics {
    pub neuron_minimum_stake_e8s: u64,
    pub voting_power_economics: Option<()>, // Simplified
    pub max_proposals_to_keep_per_topic: u32,
    pub neuron_management_fee_per_proposal_e8s: u64,
    pub reject_cost_e8s: u64,
    pub transaction_fee_e8s: u64,
    pub neuron_spawn_dissolve_delay_seconds: u64,
    pub minimum_icp_xdr_rate: u64,
    pub maximum_node_provider_rewards_e8s: u64,
    pub neurons_fund_economics: Option<()>, // Simplified
}

#[derive(CandidType, Deserialize)]
pub struct Principals {
    pub principals: Vec<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct Motion {
    pub motion_text: String,
}

#[derive(CandidType, Deserialize)]
pub struct RewardNodeProviders {
    pub use_registry_derived_rewards: Option<bool>,
    pub rewards: Vec<RewardNodeProvider>,
}

#[derive(CandidType, Deserialize)]
pub struct AddOrRemoveNodeProvider {
    pub change: Option<()>, // Simplified
}

#[derive(CandidType, Deserialize)]
pub enum ProposalActionRequest {
    RegisterKnownNeuron(KnownNeuron),
    FulfillSubnetRentalRequest(FulfillSubnetRentalRequest),
    ManageNeuron(Box<ManageNeuronRequest>),
    BlessAlternativeGuestOsVersion(BlessAlternativeGuestOsVersion),
    UpdateCanisterSettings(UpdateCanisterSettings),
    InstallCode(InstallCodeRequest),
    DeregisterKnownNeuron(DeregisterKnownNeuron),
    StopOrStartCanister(StopOrStartCanister),
    CreateServiceNervousSystem(CreateServiceNervousSystem),
    ExecuteNnsFunction(ExecuteNnsFunction),
    RewardNodeProvider(RewardNodeProvider),
    RewardNodeProviders(RewardNodeProviders),
    ManageNetworkEconomics(NetworkEconomics),
    ApproveGenesisKyc(Principals),
    AddOrRemoveNodeProvider(AddOrRemoveNodeProvider),
    Motion(Motion),
}

#[derive(CandidType, Deserialize)]
pub struct MakeProposalRequest {
    pub url: String,
    pub title: Option<String>,
    pub action: Option<ProposalActionRequest>,
    pub summary: String,
}

#[derive(CandidType, Deserialize)]
pub enum ManageNeuronCommandRequest {
    Spawn(Spawn),
    Split(Split),
    Follow(Follow),
    DisburseMaturity(DisburseMaturity),
    RefreshVotingPower(RefreshVotingPower),
    ClaimOrRefresh(ClaimOrRefresh),
    Configure(Configure),
    RegisterVote(RegisterVote),
    Merge(Merge),
    DisburseToNeuron(DisburseToNeuron),
    SetFollowing(SetFollowing),
    MakeProposal(MakeProposalRequest),
    StakeMaturity(StakeMaturity),
    MergeMaturity(MergeMaturity),
    Disburse(Disburse),
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuronRequest {
    pub id: Option<NeuronId>,
    pub command: Option<ManageNeuronCommandRequest>,
    pub neuron_id_or_subaccount: Option<NeuronIdOrSubaccount>,
}

#[derive(CandidType, Deserialize)]
pub struct SpawnResponse {
    pub created_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturityResponse {
    pub amount_disbursed_e8s: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct RefreshVotingPowerResponse {}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefreshResponse {
    pub refreshed_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct ConfigureResponse {}

#[derive(CandidType, Deserialize)]
pub struct MakeProposalResponse {
    pub message: Option<String>,
    pub proposal_id: Option<ProposalId>,
}

#[derive(CandidType, Deserialize)]
pub enum Command1 {
    Error(GovernanceError),
    Spawn(SpawnResponse),
    Split(SpawnResponse),
    Follow {},
    DisburseMaturity(DisburseMaturityResponse),
    RefreshVotingPower(RefreshVotingPowerResponse),
    ClaimOrRefresh(ClaimOrRefreshResponse),
    Configure {},
    RegisterVote {},
    Merge(MergeResponse),
    DisburseToNeuron(SpawnResponse),
    SetFollowing(SetFollowingResponse),
    MakeProposal(MakeProposalResponse),
    StakeMaturity(StakeMaturityResponse),
    MergeMaturity(MergeMaturityResponse),
    Disburse(DisburseResponse),
}

#[derive(CandidType, Deserialize)]
pub struct MergeResponse {
    pub target_neuron: Option<Neuron>,
    pub source_neuron: Option<Neuron>,
    pub target_neuron_info: Option<NeuronInfo>,
    pub source_neuron_info: Option<NeuronInfo>,
}

#[derive(CandidType, Deserialize)]
pub struct SetFollowingResponse {}

#[derive(CandidType, Deserialize)]
pub struct StakeMaturityResponse {
    pub maturity_e8s: u64,
    pub staked_maturity_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct MergeMaturityResponse {
    pub merged_maturity_e8s: u64,
    pub new_stake_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseResponse {
    pub transfer_block_height: u64,
}

// Placeholder types for full neuron
#[derive(CandidType, Deserialize, Serialize)]
pub struct MaturityDisbursement {
    pub account_identifier_to_disburse_to: Option<AccountIdentifier>,
    pub timestamp_of_disbursement_seconds: Option<u64>,
    pub amount_e8s: Option<u64>,
    pub account_to_disburse_to: Option<Account>,
    pub finalize_disbursement_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronInfo {
    pub id: Option<NeuronId>,
    // Simplified - just need id for now
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct Neuron {
    pub id: Option<NeuronId>,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub controller: Option<Principal>,
    pub recent_ballots: Vec<BallotInfo>,
    pub voting_power_refreshed_timestamp_seconds: Option<u64>,
    pub kyc_verified: bool,
    pub potential_voting_power: Option<u64>,
    pub neuron_type: Option<i32>,
    pub not_for_profit: bool,
    pub maturity_e8s_equivalent: u64,
    pub deciding_voting_power: Option<u64>,
    pub cached_neuron_stake_e8s: u64,
    pub created_timestamp_seconds: u64,
    pub auto_stake_maturity: Option<bool>,
    pub aging_since_timestamp_seconds: u64,
    pub hot_keys: Vec<Principal>,
    pub account: Vec<u8>,
    pub joined_community_fund_timestamp_seconds: Option<u64>,
    pub maturity_disbursements_in_progress: Option<Vec<MaturityDisbursement>>,
    pub dissolve_state: Option<DissolveState>,
    pub followees: Vec<(i32, Followees)>,
    pub neuron_fees_e8s: u64,
    pub visibility: Option<i32>,
    pub transfer: Option<NeuronStakeTransfer>,
    pub known_neuron_data: Option<KnownNeuronData>,
    pub spawn_at_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct BallotInfo {
    pub vote: i32,
    pub proposal_id: Option<ProposalId>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub enum DissolveState {
    DissolveDelaySeconds(u64),
    WhenDissolvedTimestampSeconds(u64),
}

#[derive(CandidType, Deserialize, Serialize)]
pub enum Result2 {
    Ok(Neuron),
    Err(GovernanceError),
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuronResponse {
    pub command: Option<Command1>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronSubaccount {
    pub subaccount: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct ListNeurons {
    pub page_size: Option<u64>,
    pub include_public_neurons_in_full_neurons: Option<bool>,
    pub neuron_ids: Vec<u64>,
    pub page_number: Option<u64>,
    pub include_empty_neurons_readable_by_caller: Option<bool>,
    pub neuron_subaccounts: Option<Vec<NeuronSubaccount>>,
    pub include_neurons_readable_by_caller: bool,
}

#[derive(CandidType, Deserialize)]
pub struct ListNeuronsResponse {
    pub neuron_infos: Vec<(u64, NeuronInfo)>,
    pub full_neurons: Vec<Neuron>,
    pub total_pages_available: Option<u64>,
}
