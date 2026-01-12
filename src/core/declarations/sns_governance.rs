// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(
    dead_code,
    unused_imports,
    renamed_and_removed_lints,
    large_enum_variant,
    clippy::vec_box
)]
use candid::{self, CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize)]
pub struct Timers {
    pub last_spawned_timestamp_seconds: Option<u64>,
    pub last_reset_timestamp_seconds: Option<u64>,
    pub requires_periodic_tasks: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub struct Version {
    pub archive_wasm_hash: Vec<u8>,
    pub root_wasm_hash: Vec<u8>,
    pub swap_wasm_hash: Vec<u8>,
    pub ledger_wasm_hash: Vec<u8>,
    pub governance_wasm_hash: Vec<u8>,
    pub index_wasm_hash: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct Versions {
    pub versions: Vec<Version>,
}

#[derive(CandidType, Deserialize)]
pub struct CachedUpgradeSteps {
    pub upgrade_steps: Option<Versions>,
    pub response_timestamp_seconds: Option<u64>,
    pub requested_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum Topic {
    DappCanisterManagement,
    DaoCommunitySettings,
    ApplicationBusinessLogic,
    CriticalDappOperations,
    TreasuryAssetManagement,
    Governance,
    SnsFrameworkManagement,
}

#[derive(CandidType, Deserialize)]
pub struct GenericNervousSystemFunction {
    pub topic: Option<Topic>,
    pub validator_canister_id: Option<Principal>,
    pub target_canister_id: Option<Principal>,
    pub validator_method_name: Option<String>,
    pub target_method_name: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub enum FunctionType {
    NativeNervousSystemFunction {},
    GenericNervousSystemFunction(GenericNervousSystemFunction),
}

#[derive(CandidType, Deserialize)]
pub struct NervousSystemFunction {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub function_type: Option<FunctionType>,
}

#[derive(CandidType, Deserialize)]
pub struct Subaccount {
    pub subaccount: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct Account {
    pub owner: Option<Principal>,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize)]
pub struct TreasuryMetrics {
    pub name: Option<String>,
    pub original_amount_e8s: Option<u64>,
    pub amount_e8s: Option<u64>,
    pub account: Option<Account>,
    pub ledger_canister_id: Option<Principal>,
    pub treasury: i32,
    pub timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct VotingPowerMetrics {
    pub governance_total_potential_voting_power: Option<u64>,
    pub timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct GovernanceCachedMetrics {
    pub treasury_metrics: Vec<TreasuryMetrics>,
    pub not_dissolving_neurons_e8s_buckets: Vec<(u64, f64)>,
    pub garbage_collectable_neurons_count: u64,
    pub neurons_with_invalid_stake_count: u64,
    pub not_dissolving_neurons_count_buckets: Vec<(u64, u64)>,
    pub neurons_with_less_than_6_months_dissolve_delay_count: u64,
    pub dissolved_neurons_count: u64,
    pub total_staked_e8s: u64,
    pub total_supply_governance_tokens: u64,
    pub voting_power_metrics: Option<VotingPowerMetrics>,
    pub not_dissolving_neurons_count: u64,
    pub dissolved_neurons_e8s: u64,
    pub neurons_with_less_than_6_months_dissolve_delay_e8s: u64,
    pub dissolving_neurons_count_buckets: Vec<(u64, u64)>,
    pub dissolving_neurons_count: u64,
    pub dissolving_neurons_e8s_buckets: Vec<(u64, f64)>,
    pub timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct MaturityModulation {
    pub current_basis_points: Option<i32>,
    pub updated_at_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct TargetVersionSet {
    pub old_target_version: Option<Version>,
    pub new_target_version: Option<Version>,
    pub is_advanced_automatically: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeStepsReset {
    pub human_readable: Option<String>,
    pub upgrade_steps: Option<Versions>,
}

#[derive(CandidType, Deserialize)]
pub enum UpgradeOutcomeStatusInner {
    Success {},
    Timeout {},
    ExternalFailure {},
    InvalidState { version: Option<Version> },
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeOutcome {
    pub status: Option<UpgradeOutcomeStatusInner>,
    pub human_readable: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct ProposalId {
    pub id: u64,
}

#[derive(CandidType, Deserialize)]
pub enum UpgradeStartedReasonInner {
    UpgradeSnsToNextVersionProposal(ProposalId),
    BehindTargetVersion {},
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeStarted {
    pub current_version: Option<Version>,
    pub expected_version: Option<Version>,
    pub reason: Option<UpgradeStartedReasonInner>,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeStepsRefreshed {
    pub upgrade_steps: Option<Versions>,
}

#[derive(CandidType, Deserialize)]
pub struct TargetVersionReset {
    pub human_readable: Option<String>,
    pub old_target_version: Option<Version>,
    pub new_target_version: Option<Version>,
}

#[derive(CandidType, Deserialize)]
pub enum UpgradeJournalEntryEventInner {
    TargetVersionSet(TargetVersionSet),
    UpgradeStepsReset(UpgradeStepsReset),
    UpgradeOutcome(UpgradeOutcome),
    UpgradeStarted(UpgradeStarted),
    UpgradeStepsRefreshed(UpgradeStepsRefreshed),
    TargetVersionReset(TargetVersionReset),
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeJournalEntry {
    pub event: Option<UpgradeJournalEntryEventInner>,
    pub timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeJournal {
    pub entries: Vec<UpgradeJournalEntry>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronId {
    pub id: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct Followees {
    pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct DefaultFollowees {
    pub followees: Vec<(u64, Followees)>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronPermissionList {
    pub permissions: Vec<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct VotingRewardsParameters {
    pub final_reward_rate_basis_points: Option<u64>,
    pub initial_reward_rate_basis_points: Option<u64>,
    pub reward_rate_transition_duration_seconds: Option<u64>,
    pub round_duration_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct NervousSystemParameters {
    pub default_followees: Option<DefaultFollowees>,
    pub max_dissolve_delay_seconds: Option<u64>,
    pub max_dissolve_delay_bonus_percentage: Option<u64>,
    pub max_followees_per_function: Option<u64>,
    pub automatically_advance_target_version: Option<bool>,
    pub neuron_claimer_permissions: Option<NeuronPermissionList>,
    pub neuron_minimum_stake_e8s: Option<u64>,
    pub max_neuron_age_for_age_bonus: Option<u64>,
    pub initial_voting_period_seconds: Option<u64>,
    pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
    pub reject_cost_e8s: Option<u64>,
    pub max_proposals_to_keep_per_action: Option<u32>,
    pub wait_for_quiet_deadline_increase_seconds: Option<u64>,
    pub max_number_of_neurons: Option<u64>,
    pub transaction_fee_e8s: Option<u64>,
    pub max_number_of_proposals_with_ballots: Option<u64>,
    pub max_age_bonus_percentage: Option<u64>,
    pub neuron_grantable_permissions: Option<NeuronPermissionList>,
    pub voting_rewards_parameters: Option<VotingRewardsParameters>,
    pub maturity_modulation_disabled: Option<bool>,
    pub max_number_of_principals_per_neuron: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct RewardEvent {
    pub rounds_since_last_distribution: Option<u64>,
    pub actual_timestamp_seconds: u64,
    pub end_timestamp_seconds: Option<u64>,
    pub total_available_e8s_equivalent: Option<u64>,
    pub distributed_e8s_equivalent: u64,
    pub round: u64,
    pub settled_proposals: Vec<ProposalId>,
}

#[derive(CandidType, Deserialize)]
pub struct PendingVersion {
    pub mark_failed_at_seconds: u64,
    pub checking_upgrade_lock: u64,
    pub proposal_id: Option<u64>,
    pub target_version: Option<Version>,
}

#[derive(CandidType, Deserialize)]
pub struct GovernanceError {
    pub error_message: String,
    pub error_type: i32,
}

#[derive(CandidType, Deserialize)]
pub struct Decimal {
    pub human_readable: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct Tokens {
    pub e8s: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct ValuationFactors {
    pub xdrs_per_icp: Option<Decimal>,
    pub icps_per_token: Option<Decimal>,
    pub tokens: Option<Tokens>,
}

#[derive(CandidType, Deserialize)]
pub struct Valuation {
    pub token: Option<i32>,
    pub account: Option<Account>,
    pub valuation_factors: Option<ValuationFactors>,
    pub timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct MintSnsTokensActionAuxiliary {
    pub valuation: Option<Valuation>,
}

#[derive(CandidType, Deserialize)]
pub struct SnsVersion {
    pub archive_wasm_hash: Option<Vec<u8>>,
    pub root_wasm_hash: Option<Vec<u8>>,
    pub swap_wasm_hash: Option<Vec<u8>>,
    pub ledger_wasm_hash: Option<Vec<u8>>,
    pub governance_wasm_hash: Option<Vec<u8>>,
    pub index_wasm_hash: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct AdvanceSnsTargetVersionActionAuxiliary {
    pub target_version: Option<SnsVersion>,
}

#[derive(CandidType, Deserialize)]
pub enum ActionAuxiliary {
    TransferSnsTreasuryFunds(MintSnsTokensActionAuxiliary),
    MintSnsTokens(MintSnsTokensActionAuxiliary),
    AdvanceSnsTargetVersion(AdvanceSnsTargetVersionActionAuxiliary),
}

#[derive(CandidType, Deserialize)]
pub struct Ballot {
    pub vote: i32,
    pub cast_timestamp_seconds: u64,
    pub voting_power: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Percentage {
    pub basis_points: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Tally {
    pub no: u64,
    pub yes: u64,
    pub total: u64,
    pub timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct ManageDappCanisterSettings {
    pub freezing_threshold: Option<u64>,
    pub wasm_memory_threshold: Option<u64>,
    pub canister_ids: Vec<Principal>,
    pub reserved_cycles_limit: Option<u64>,
    pub log_visibility: Option<i32>,
    pub wasm_memory_limit: Option<u64>,
    pub memory_allocation: Option<u64>,
    pub compute_allocation: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum PreciseValue {
    Int(i64),
    Map(Vec<(String, Box<Self>)>),
    Nat(u64),
    Blob(Vec<u8>),
    Bool(bool),
    Text(String),
    Array(Vec<Box<Self>>),
}

#[derive(CandidType, Deserialize)]
pub struct ExtensionOperationArg {
    pub value: Option<Box<PreciseValue>>,
}

#[derive(CandidType, Deserialize)]
pub struct ExecuteExtensionOperation {
    pub extension_canister_id: Option<Principal>,
    pub operation_name: Option<String>,
    pub operation_arg: Option<ExtensionOperationArg>,
}

#[derive(CandidType, Deserialize)]
pub struct ChunkedCanisterWasm {
    pub wasm_module_hash: Vec<u8>,
    pub chunk_hashes_list: Vec<Vec<u8>>,
    pub store_canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub enum Wasm {
    Chunked(ChunkedCanisterWasm),
    Bytes(Vec<u8>),
}

#[derive(CandidType, Deserialize)]
pub struct ExtensionUpgradeArg {
    pub value: Option<Box<PreciseValue>>,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeExtension {
    pub extension_canister_id: Option<Principal>,
    pub wasm: Option<Wasm>,
    pub canister_upgrade_arg: Option<ExtensionUpgradeArg>,
}

#[derive(CandidType, Deserialize)]
pub struct SetTopicsForCustomProposals {
    pub custom_function_id_to_topic: Vec<(u64, Topic)>,
}

#[derive(CandidType, Deserialize)]
pub struct ExtensionInit {
    pub value: Option<Box<PreciseValue>>,
}

#[derive(CandidType, Deserialize)]
pub struct RegisterExtension {
    pub chunked_canister_wasm: Option<ChunkedCanisterWasm>,
    pub extension_init: Option<ExtensionInit>,
}

#[derive(CandidType, Deserialize)]
pub struct RegisterDappCanisters {
    pub canister_ids: Vec<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct TransferSnsTreasuryFunds {
    pub from_treasury: i32,
    pub to_principal: Option<Principal>,
    pub to_subaccount: Option<Subaccount>,
    pub memo: Option<u64>,
    pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeSnsControlledCanister {
    pub new_canister_wasm: Vec<u8>,
    pub mode: Option<i32>,
    pub canister_id: Option<Principal>,
    pub chunked_canister_wasm: Option<ChunkedCanisterWasm>,
    pub canister_upgrade_arg: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct DeregisterDappCanisters {
    pub canister_ids: Vec<Principal>,
    pub new_controllers: Vec<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct MintSnsTokens {
    pub to_principal: Option<Principal>,
    pub to_subaccount: Option<Subaccount>,
    pub memo: Option<u64>,
    pub amount_e8s: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct AdvanceSnsTargetVersion {
    pub new_target: Option<SnsVersion>,
}

#[derive(CandidType, Deserialize)]
pub struct ManageSnsMetadata {
    pub url: Option<String>,
    pub logo: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct ExecuteGenericNervousSystemFunction {
    pub function_id: u64,
    pub payload: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct ManageLedgerParameters {
    pub token_symbol: Option<String>,
    pub transfer_fee: Option<u64>,
    pub token_logo: Option<String>,
    pub token_name: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct Motion {
    pub motion_text: String,
}

#[derive(CandidType, Deserialize)]
pub enum Action {
    ManageNervousSystemParameters(NervousSystemParameters),
    AddGenericNervousSystemFunction(NervousSystemFunction),
    ManageDappCanisterSettings(ManageDappCanisterSettings),
    ExecuteExtensionOperation(ExecuteExtensionOperation),
    UpgradeExtension(UpgradeExtension),
    RemoveGenericNervousSystemFunction(u64),
    SetTopicsForCustomProposals(SetTopicsForCustomProposals),
    RegisterExtension(RegisterExtension),
    UpgradeSnsToNextVersion {},
    RegisterDappCanisters(RegisterDappCanisters),
    TransferSnsTreasuryFunds(TransferSnsTreasuryFunds),
    UpgradeSnsControlledCanister(UpgradeSnsControlledCanister),
    DeregisterDappCanisters(DeregisterDappCanisters),
    MintSnsTokens(MintSnsTokens),
    AdvanceSnsTargetVersion(AdvanceSnsTargetVersion),
    Unspecified {},
    ManageSnsMetadata(ManageSnsMetadata),
    ExecuteGenericNervousSystemFunction(ExecuteGenericNervousSystemFunction),
    ManageLedgerParameters(ManageLedgerParameters),
    Motion(Motion),
}

#[derive(CandidType, Deserialize)]
pub struct Proposal {
    pub url: String,
    pub title: String,
    pub action: Option<Action>,
    pub summary: String,
}

#[derive(CandidType, Deserialize)]
pub struct WaitForQuietState {
    pub current_deadline_timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct ProposalData {
    pub id: Option<ProposalId>,
    pub payload_text_rendering: Option<String>,
    pub topic: Option<Topic>,
    pub action: u64,
    pub failure_reason: Option<GovernanceError>,
    pub action_auxiliary: Option<ActionAuxiliary>,
    pub ballots: Vec<(String, Ballot)>,
    pub minimum_yes_proportion_of_total: Option<Percentage>,
    pub reward_event_round: u64,
    pub failed_timestamp_seconds: u64,
    pub reward_event_end_timestamp_seconds: Option<u64>,
    pub proposal_creation_timestamp_seconds: u64,
    pub initial_voting_period_seconds: u64,
    pub reject_cost_e8s: u64,
    pub latest_tally: Option<Tally>,
    pub wait_for_quiet_deadline_increase_seconds: u64,
    pub decided_timestamp_seconds: u64,
    pub proposal: Option<Proposal>,
    pub proposer: Option<NeuronId>,
    pub wait_for_quiet_state: Option<WaitForQuietState>,
    pub minimum_yes_proportion_of_exercised: Option<Percentage>,
    pub is_eligible_for_rewards: bool,
    pub executed_timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Split {
    pub memo: u64,
    pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Follow {
    pub function_id: u64,
    pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturity {
    pub to_account: Option<Account>,
    pub percentage_to_disburse: u32,
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
pub struct SetDissolveTimestamp {
    pub dissolve_timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub enum Operation {
    ChangeAutoStakeMaturity(ChangeAutoStakeMaturity),
    StopDissolving {},
    StartDissolving {},
    IncreaseDissolveDelay(IncreaseDissolveDelay),
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
pub struct Followee {
    pub alias: Option<String>,
    pub neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct FolloweesForTopic {
    pub topic: Option<Topic>,
    pub followees: Vec<Followee>,
}

#[derive(CandidType, Deserialize)]
pub struct SetFollowing {
    pub topic_following: Vec<FolloweesForTopic>,
}

#[derive(CandidType, Deserialize)]
pub struct FinalizeDisburseMaturity {
    pub amount_to_be_disbursed_e8s: u64,
    pub to_account: Option<Account>,
}

#[derive(CandidType, Deserialize)]
pub struct MemoAndController {
    pub controller: Option<Principal>,
    pub memo: u64,
}

#[derive(CandidType, Deserialize)]
pub enum By {
    MemoAndController(MemoAndController),
    NeuronId {},
}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefresh {
    pub by: Option<By>,
}

#[derive(CandidType, Deserialize)]
pub struct RemoveNeuronPermissions {
    pub permissions_to_remove: Option<NeuronPermissionList>,
    pub principal_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct AddNeuronPermissions {
    pub permissions_to_add: Option<NeuronPermissionList>,
    pub principal_id: Option<Principal>,
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
    pub to_account: Option<Account>,
    pub amount: Option<Amount>,
}

#[derive(CandidType, Deserialize)]
pub enum Command2 {
    Split(Split),
    Follow(Follow),
    DisburseMaturity(DisburseMaturity),
    Configure(Configure),
    RegisterVote(RegisterVote),
    SetFollowing(SetFollowing),
    SyncCommand {},
    MakeProposal(Proposal),
    FinalizeDisburseMaturity(FinalizeDisburseMaturity),
    ClaimOrRefreshNeuron(ClaimOrRefresh),
    RemoveNeuronPermissions(RemoveNeuronPermissions),
    AddNeuronPermissions(AddNeuronPermissions),
    MergeMaturity(MergeMaturity),
    Disburse(Disburse),
}

#[derive(CandidType, Deserialize)]
pub struct NeuronInFlightCommand {
    pub command: Option<Command2>,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronPermission {
    pub principal: Option<Principal>,
    pub permission_type: Vec<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronTopicFolloweesInner {
    pub topic_id_to_followees: Vec<(i32, FolloweesForTopic)>,
}

#[derive(CandidType, Deserialize)]
pub enum DissolveState {
    DissolveDelaySeconds(u64),
    WhenDissolvedTimestampSeconds(u64),
}

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturityInProgress {
    pub timestamp_of_disbursement_seconds: u64,
    pub amount_e8s: u64,
    pub account_to_disburse_to: Option<Account>,
    pub finalize_disbursement_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Neuron {
    pub id: Option<NeuronId>,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub permissions: Vec<NeuronPermission>,
    pub maturity_e8s_equivalent: u64,
    pub cached_neuron_stake_e8s: u64,
    pub created_timestamp_seconds: u64,
    pub topic_followees: Option<NeuronTopicFolloweesInner>,
    pub source_nns_neuron_id: Option<u64>,
    pub auto_stake_maturity: Option<bool>,
    pub aging_since_timestamp_seconds: u64,
    pub dissolve_state: Option<DissolveState>,
    pub voting_power_percentage_multiplier: u64,
    pub vesting_period_seconds: Option<u64>,
    pub disburse_maturity_in_progress: Vec<DisburseMaturityInProgress>,
    pub followees: Vec<(u64, Followees)>,
    pub neuron_fees_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Governance {
    pub root_canister_id: Option<Principal>,
    pub timers: Option<Timers>,
    pub cached_upgrade_steps: Option<CachedUpgradeSteps>,
    pub id_to_nervous_system_functions: Vec<(u64, NervousSystemFunction)>,
    pub metrics: Option<GovernanceCachedMetrics>,
    pub maturity_modulation: Option<MaturityModulation>,
    pub upgrade_journal: Option<UpgradeJournal>,
    pub mode: i32,
    pub parameters: Option<NervousSystemParameters>,
    pub is_finalizing_disburse_maturity: Option<bool>,
    pub deployed_version: Option<Version>,
    pub sns_initialization_parameters: String,
    pub latest_reward_event: Option<RewardEvent>,
    pub pending_version: Option<PendingVersion>,
    pub swap_canister_id: Option<Principal>,
    pub ledger_canister_id: Option<Principal>,
    pub proposals: Vec<(u64, ProposalData)>,
    pub in_flight_commands: Vec<(String, NeuronInFlightCommand)>,
    pub sns_metadata: Option<ManageSnsMetadata>,
    pub neurons: Vec<(String, Neuron)>,
    pub target_version: Option<Version>,
    pub genesis_timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Principals {
    pub principals: Vec<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronsFund {
    pub nns_neuron_hotkeys: Option<Principals>,
    pub nns_neuron_controller: Option<Principal>,
    pub nns_neuron_id: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum Participant {
    NeuronsFund(NeuronsFund),
    Direct {},
}

#[derive(CandidType, Deserialize)]
pub struct NeuronIds {
    pub neuron_ids: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronRecipe {
    pub controller: Option<Principal>,
    pub dissolve_delay_seconds: Option<u64>,
    pub participant: Option<Participant>,
    pub stake_e8s: Option<u64>,
    pub followees: Option<NeuronIds>,
    pub neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronRecipes {
    pub neuron_recipes: Vec<NeuronRecipe>,
}

#[derive(CandidType, Deserialize)]
pub struct ClaimSwapNeuronsRequest {
    pub neuron_recipes: Option<NeuronRecipes>,
}

#[derive(CandidType, Deserialize)]
pub struct SwapNeuron {
    pub id: Option<NeuronId>,
    pub status: i32,
}

#[derive(CandidType, Deserialize)]
pub struct ClaimedSwapNeurons {
    pub swap_neurons: Vec<SwapNeuron>,
}

#[derive(CandidType, Deserialize)]
pub enum ClaimSwapNeuronsResult {
    Ok(ClaimedSwapNeurons),
    Err(i32),
}

#[derive(CandidType, Deserialize)]
pub struct ClaimSwapNeuronsResponse {
    pub claim_swap_neurons_result: Option<ClaimSwapNeuronsResult>,
}

#[derive(CandidType, Deserialize)]
pub struct FailStuckUpgradeInProgressArg {}

#[derive(CandidType, Deserialize)]
pub struct FailStuckUpgradeInProgressRet {}

#[derive(CandidType, Deserialize)]
pub struct GetMaturityModulationArg {}

#[derive(CandidType, Deserialize)]
pub struct GetMaturityModulationResponse {
    pub maturity_modulation: Option<MaturityModulation>,
}

#[derive(CandidType, Deserialize)]
pub struct GetMetadataArg {}

#[derive(CandidType, Deserialize)]
pub struct GetMetadataResponse {
    pub url: Option<String>,
    pub logo: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct GetMetricsRequest {
    pub time_window_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Metrics {
    pub treasury_metrics: Option<Vec<TreasuryMetrics>>,
    pub voting_power_metrics: Option<VotingPowerMetrics>,
    pub last_ledger_block_timestamp: Option<u64>,
    pub num_recently_executed_proposals: Option<u64>,
    pub num_recently_submitted_proposals: Option<u64>,
    pub genesis_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum GetMetricsResult {
    Ok(Metrics),
    Err(GovernanceError),
}

#[derive(CandidType, Deserialize)]
pub struct GetMetricsResponse {
    pub get_metrics_result: Option<GetMetricsResult>,
}

#[derive(CandidType, Deserialize)]
pub struct GetModeArg {}

#[derive(CandidType, Deserialize)]
pub struct GetModeResponse {
    pub mode: Option<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct GetNeuron {
    pub neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub enum Result_ {
    Error(GovernanceError),
    Neuron(Neuron),
}

#[derive(CandidType, Deserialize)]
pub struct GetNeuronResponse {
    pub result: Option<Result_>,
}

#[derive(CandidType, Deserialize)]
pub struct GetProposal {
    pub proposal_id: Option<ProposalId>,
}

#[derive(CandidType, Deserialize)]
pub enum Result1 {
    Error(GovernanceError),
    Proposal(ProposalData),
}

#[derive(CandidType, Deserialize)]
pub struct GetProposalResponse {
    pub result: Option<Result1>,
}

#[derive(CandidType, Deserialize)]
pub struct MemoryMetrics {
    pub wasm_binary_size: Option<candid::Nat>,
    pub wasm_chunk_store_size: Option<candid::Nat>,
    pub canister_history_size: Option<candid::Nat>,
    pub stable_memory_size: Option<candid::Nat>,
    pub snapshots_size: Option<candid::Nat>,
    pub wasm_memory_size: Option<candid::Nat>,
    pub global_memory_size: Option<candid::Nat>,
    pub custom_sections_size: Option<candid::Nat>,
}

#[derive(CandidType, Deserialize)]
pub enum CanisterStatusType {
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "running")]
    Running,
}

#[derive(CandidType, Deserialize)]
pub struct DefiniteCanisterSettingsArgs {
    pub freezing_threshold: candid::Nat,
    pub wasm_memory_threshold: Option<candid::Nat>,
    pub controllers: Vec<Principal>,
    pub wasm_memory_limit: Option<candid::Nat>,
    pub memory_allocation: candid::Nat,
    pub compute_allocation: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub struct QueryStats {
    pub response_payload_bytes_total: Option<candid::Nat>,
    pub num_instructions_total: Option<candid::Nat>,
    pub num_calls_total: Option<candid::Nat>,
    pub request_payload_bytes_total: Option<candid::Nat>,
}

#[derive(CandidType, Deserialize)]
pub struct CanisterStatusResultV2 {
    pub memory_metrics: Option<MemoryMetrics>,
    pub status: CanisterStatusType,
    pub memory_size: candid::Nat,
    pub cycles: candid::Nat,
    pub settings: DefiniteCanisterSettingsArgs,
    pub query_stats: Option<QueryStats>,
    pub idle_cycles_burned_per_day: candid::Nat,
    pub module_hash: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct GetRunningSnsVersionArg {}

#[derive(CandidType, Deserialize)]
pub struct GetRunningSnsVersionResponsePendingVersionInner {
    pub mark_failed_at_seconds: u64,
    pub checking_upgrade_lock: u64,
    pub proposal_id: u64,
    pub target_version: Option<Version>,
}

#[derive(CandidType, Deserialize)]
pub struct GetRunningSnsVersionResponse {
    pub deployed_version: Option<Version>,
    pub pending_version: Option<GetRunningSnsVersionResponsePendingVersionInner>,
}

#[derive(CandidType, Deserialize)]
pub struct GetSnsInitializationParametersArg {}

#[derive(CandidType, Deserialize)]
pub struct GetSnsInitializationParametersResponse {
    pub sns_initialization_parameters: String,
}

#[derive(CandidType, Deserialize)]
pub struct GetTimersArg {}

#[derive(CandidType, Deserialize)]
pub struct GetTimersResponse {
    pub timers: Option<Timers>,
}

#[derive(CandidType, Deserialize)]
pub struct GetUpgradeJournalRequest {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct GetUpgradeJournalResponse {
    pub upgrade_journal: Option<UpgradeJournal>,
    pub upgrade_steps: Option<Versions>,
    pub response_timestamp_seconds: Option<u64>,
    pub deployed_version: Option<Version>,
    pub target_version: Option<Version>,
    pub upgrade_journal_entry_count: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct ListNervousSystemFunctionsResponse {
    pub reserved_ids: Vec<u64>,
    pub functions: Vec<NervousSystemFunction>,
}

#[derive(CandidType, Deserialize)]
pub struct ListNeurons {
    pub of_principal: Option<Principal>,
    pub limit: u32,
    pub start_page_at: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct ListNeuronsResponse {
    pub neurons: Vec<Neuron>,
}

#[derive(CandidType, Deserialize)]
pub struct TopicSelector {
    pub topic: Option<Topic>,
}

#[derive(CandidType, Deserialize)]
pub struct ListProposals {
    pub include_reward_status: Vec<i32>,
    pub before_proposal: Option<ProposalId>,
    pub limit: u32,
    pub exclude_type: Vec<u64>,
    pub include_topics: Option<Vec<TopicSelector>>,
    pub include_status: Vec<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct ListProposalsResponse {
    pub include_ballots_by_caller: Option<bool>,
    pub proposals: Vec<ProposalData>,
    pub include_topic_filtering: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub struct ListTopicsRequest {}

#[derive(CandidType, Deserialize)]
pub enum ExtensionOperationType {
    TreasuryManagerWithdraw,
    TreasuryManagerDeposit,
}

#[derive(CandidType, Deserialize)]
pub enum ExtensionType {
    TreasuryManager,
}

#[derive(CandidType, Deserialize)]
pub struct ExtensionOperationSpec {
    pub topic: Option<Topic>,
    pub operation_type: Option<ExtensionOperationType>,
    pub description: Option<String>,
    pub extension_type: Option<ExtensionType>,
}

#[derive(CandidType, Deserialize)]
pub struct RegisteredExtensionOperationSpec {
    pub spec: Option<ExtensionOperationSpec>,
    pub canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct TopicInfo {
    pub extension_operations: Option<Vec<RegisteredExtensionOperationSpec>>,
    pub native_functions: Option<Vec<NervousSystemFunction>>,
    pub topic: Option<Topic>,
    pub is_critical: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub custom_functions: Option<Vec<NervousSystemFunction>>,
}

#[derive(CandidType, Deserialize)]
pub struct ListTopicsResponse {
    pub uncategorized_functions: Option<Vec<NervousSystemFunction>>,
    pub topics: Option<Vec<TopicInfo>>,
}

#[derive(CandidType, Deserialize)]
pub struct StakeMaturity {
    pub percentage_to_stake: Option<u32>,
}

#[derive(CandidType, Deserialize)]
pub enum Command {
    Split(Split),
    Follow(Follow),
    DisburseMaturity(DisburseMaturity),
    ClaimOrRefresh(ClaimOrRefresh),
    Configure(Configure),
    RegisterVote(RegisterVote),
    SetFollowing(SetFollowing),
    MakeProposal(Proposal),
    StakeMaturity(StakeMaturity),
    RemoveNeuronPermissions(RemoveNeuronPermissions),
    AddNeuronPermissions(AddNeuronPermissions),
    MergeMaturity(MergeMaturity),
    Disburse(Disburse),
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuron {
    pub subaccount: Vec<u8>,
    pub command: Option<Command>,
}

#[derive(CandidType, Deserialize)]
pub struct SplitResponse {
    pub created_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturityResponse {
    pub amount_disbursed_e8s: u64,
    pub amount_deducted_e8s: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefreshResponse {
    pub refreshed_neuron_id: Option<NeuronId>,
}

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

#[derive(CandidType, Deserialize)]
pub enum Command1 {
    Error(GovernanceError),
    Split(SplitResponse),
    Follow {},
    DisburseMaturity(DisburseMaturityResponse),
    ClaimOrRefresh(ClaimOrRefreshResponse),
    Configure {},
    RegisterVote {},
    SetFollowing {},
    MakeProposal(GetProposal),
    RemoveNeuronPermission {},
    StakeMaturity(StakeMaturityResponse),
    MergeMaturity(MergeMaturityResponse),
    Disburse(DisburseResponse),
    AddNeuronPermission {},
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuronResponse {
    pub command: Option<Command1>,
}

#[derive(CandidType, Deserialize)]
pub struct ResetTimersArg {}

#[derive(CandidType, Deserialize)]
pub struct ResetTimersRet {}

#[derive(CandidType, Deserialize)]
pub struct SetMode {
    pub mode: i32,
}

#[derive(CandidType, Deserialize)]
pub struct SetModeRet {}

// Permission type constants
// Based on the NeuronPermissionType enum from SNS Governance
pub const PERMISSION_TYPE_UNSPECIFIED: i32 = 0;
pub const PERMISSION_TYPE_CONFIGURE_DISSOLVE_STATE: i32 = 1;
pub const PERMISSION_TYPE_MANAGE_PRINCIPALS: i32 = 2;
pub const PERMISSION_TYPE_SUBMIT_PROPOSAL: i32 = 3;
pub const PERMISSION_TYPE_VOTE: i32 = 4;
pub const PERMISSION_TYPE_DISBURSE: i32 = 5;
pub const PERMISSION_TYPE_SPLIT: i32 = 6;
pub const PERMISSION_TYPE_MERGE_MATURITY: i32 = 7;
pub const PERMISSION_TYPE_DISBURSE_MATURITY: i32 = 8;
pub const PERMISSION_TYPE_STAKE_MATURITY: i32 = 9;
pub const PERMISSION_TYPE_MANAGE_VOTING_PERMISSION: i32 = 10;
