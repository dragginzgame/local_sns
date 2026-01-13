// SNS Ledger canister Candid type definitions
// Generated from Candid, with serde_bytes::ByteBuf replaced with Vec<u8>

#![allow(dead_code, unused_imports, unused_variables)]

use candid::{CandidType, Deserialize, Nat, Principal};

#[derive(CandidType, Deserialize)]
pub struct ChangeArchiveOptions {
    pub num_blocks_to_archive: Option<u64>,
    pub max_transactions_per_response: Option<u64>,
    pub trigger_threshold: Option<u64>,
    pub more_controller_ids: Option<std::vec::Vec<Principal>>,
    pub max_message_size_bytes: Option<u64>,
    pub cycles_for_archive_creation: Option<u64>,
    pub node_max_memory_size_bytes: Option<u64>,
    pub controller_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub enum MetadataValue {
    Int(candid::Int),
    Nat(candid::Nat),
    Blob(std::vec::Vec<u8>),
    Text(String),
}

#[derive(CandidType, Deserialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<std::vec::Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub enum ChangeFeeCollector {
    SetTo(Account),
    Unset,
}

#[derive(CandidType, Deserialize)]
pub struct FeatureFlags {
    pub icrc2: bool,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeArgs {
    pub change_archive_options: Option<ChangeArchiveOptions>,
    pub token_symbol: Option<String>,
    pub transfer_fee: Option<Nat>,
    pub metadata: Option<std::vec::Vec<(String, MetadataValue)>>,
    pub change_fee_collector: Option<ChangeFeeCollector>,
    pub max_memo_length: Option<u16>,
    pub index_principal: Option<Principal>,
    pub token_name: Option<String>,
    pub feature_flags: Option<FeatureFlags>,
}

#[derive(CandidType, Deserialize)]
pub struct ArchiveOptions {
    pub num_blocks_to_archive: u64,
    pub max_transactions_per_response: Option<u64>,
    pub trigger_threshold: u64,
    pub more_controller_ids: Option<std::vec::Vec<Principal>>,
    pub max_message_size_bytes: Option<u64>,
    pub cycles_for_archive_creation: Option<u64>,
    pub node_max_memory_size_bytes: Option<u64>,
    pub controller_id: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub decimals: Option<u8>,
    pub token_symbol: String,
    pub transfer_fee: Nat,
    pub metadata: std::vec::Vec<(String, MetadataValue)>,
    pub minting_account: Account,
    pub initial_balances: std::vec::Vec<(Account, Nat)>,
    pub fee_collector_account: Option<Account>,
    pub archive_options: ArchiveOptions,
    pub max_memo_length: Option<u16>,
    pub index_principal: Option<Principal>,
    pub token_name: String,
    pub feature_flags: Option<FeatureFlags>,
}

#[derive(CandidType, Deserialize)]
pub enum LedgerArgument {
    Upgrade(Option<UpgradeArgs>),
    Init(InitArgs),
}

#[derive(CandidType, Deserialize)]
pub struct ArchiveInfo {
    pub block_range_end: Nat,
    pub canister_id: Principal,
    pub block_range_start: Nat,
}

#[derive(CandidType, Deserialize)]
pub struct GetBlocksRequest {
    pub start: Nat,
    pub length: Nat,
}

#[derive(CandidType, Deserialize)]
pub enum VecItem {
    Int(candid::Int),
    Map(std::vec::Vec<(String, Box<Value>)>),
    Nat(candid::Nat),
    Nat64(u64),
    Blob(std::vec::Vec<u8>),
    Text(String),
    Array(Box<ValueVec>),
}

#[derive(CandidType, Deserialize)]
pub struct ValueVec(pub std::vec::Vec<VecItem>);

#[derive(CandidType, Deserialize)]
pub enum Value {
    Int(candid::Int),
    Map(std::vec::Vec<(String, Box<Value>)>),
    Nat(candid::Nat),
    Nat64(u64),
    Blob(std::vec::Vec<u8>),
    Text(String),
    Array(Box<ValueVec>),
}

#[derive(CandidType, Deserialize)]
pub struct BlockRange {
    pub blocks: std::vec::Vec<Box<Value>>,
}

#[derive(CandidType, Deserialize)]
pub struct ArchivedRange {
    pub callback: ArchivedRangeCallback,
    pub start: Nat,
    pub length: Nat,
}

candid::define_function!(pub ArchivedRangeCallback : (GetBlocksRequest) -> (
    BlockRange,
  ) query);

#[derive(CandidType, Deserialize)]
pub struct GetBlocksResponse {
    pub certificate: Option<std::vec::Vec<u8>>,
    pub first_index: Nat,
    pub blocks: std::vec::Vec<Box<Value>>,
    pub chain_length: u64,
    pub archived_blocks: std::vec::Vec<ArchivedRange>,
}

#[derive(CandidType, Deserialize)]
pub struct DataCertificate {
    pub certificate: Option<std::vec::Vec<u8>>,
    pub hash_tree: std::vec::Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct Burn {
    pub from: Account,
    pub memo: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
    pub spender: Option<Account>,
}

#[derive(CandidType, Deserialize)]
pub struct Mint {
    pub to: Account,
    pub memo: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize)]
pub struct Approve {
    pub fee: Option<Nat>,
    pub from: Account,
    pub memo: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub spender: Account,
}

#[derive(CandidType, Deserialize)]
pub struct Transfer {
    pub to: Account,
    pub fee: Option<Nat>,
    pub from: Account,
    pub memo: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
    pub spender: Option<Account>,
}

#[derive(CandidType, Deserialize)]
pub struct Transaction {
    pub burn: Option<Burn>,
    pub kind: String,
    pub mint: Option<Mint>,
    pub approve: Option<Approve>,
    pub timestamp: u64,
    pub transfer: Option<Transfer>,
}

#[derive(CandidType, Deserialize)]
pub struct TransactionRange {
    pub transactions: std::vec::Vec<Transaction>,
}

candid::define_function!(pub ArchivedRange1Callback : (GetBlocksRequest) -> (
    TransactionRange,
  ) query);

#[derive(CandidType, Deserialize)]
pub struct ArchivedRange1 {
    pub callback: ArchivedRange1Callback,
    pub start: Nat,
    pub length: Nat,
}

#[derive(CandidType, Deserialize)]
pub struct GetTransactionsResponse {
    pub first_index: Nat,
    pub log_length: Nat,
    pub transactions: std::vec::Vec<Transaction>,
    pub archived_transactions: std::vec::Vec<ArchivedRange1>,
}

#[derive(CandidType, Deserialize)]
pub struct GetAllowancesArgs {
    pub take: Option<Nat>,
    pub prev_spender: Option<Account>,
    pub from_account: Option<Account>,
}

#[derive(CandidType, Deserialize)]
pub struct Allowance {
    pub from_account: Account,
    pub to_spender: Account,
    pub allowance: Nat,
    pub expires_at: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum GetAllowancesError {
    GenericError { message: String, error_code: Nat },
    AccessDenied { reason: String },
}

#[derive(CandidType, Deserialize)]
pub enum Result_ {
    Ok(std::vec::Vec<Allowance>),
    Err(GetAllowancesError),
}

#[derive(CandidType, Deserialize)]
pub enum Icrc106Error {
    GenericError {
        description: String,
        error_code: Nat,
    },
    IndexPrincipalNotSet,
}

#[derive(CandidType, Deserialize)]
pub enum Result1 {
    Ok(Principal),
    Err(Icrc106Error),
}

#[derive(CandidType, Deserialize)]
pub struct StandardRecord {
    pub url: String,
    pub name: String,
}

#[derive(CandidType, Deserialize)]
pub struct TransferArg {
    pub to: Account,
    pub fee: Option<Nat>,
    pub memo: Option<std::vec::Vec<u8>>,
    pub from_subaccount: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    BadBurn { min_burn_amount: Nat },
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    InsufficientFunds { balance: Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result2 {
    Ok(Nat),
    Err(TransferError),
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageMetadata {
    pub utc_offset_minutes: Option<i16>,
    pub language: String,
}

#[derive(CandidType, Deserialize)]
pub enum DisplayMessageType {
    GenericDisplay,
    FieldsDisplay,
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageSpec {
    pub metadata: ConsentMessageMetadata,
    pub device_spec: Option<DisplayMessageType>,
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageRequest {
    pub arg: std::vec::Vec<u8>,
    pub method: String,
    pub user_preferences: ConsentMessageSpec,
}

#[derive(CandidType, Deserialize)]
pub enum Value1 {
    Text {
        content: String,
    },
    TokenAmount {
        decimals: u8,
        amount: u64,
        symbol: String,
    },
    TimestampSeconds {
        amount: u64,
    },
    DurationSeconds {
        amount: u64,
    },
}

#[derive(CandidType, Deserialize)]
pub struct FieldsDisplay {
    pub fields: std::vec::Vec<(String, Value1)>,
    pub intent: String,
}

#[derive(CandidType, Deserialize)]
pub enum ConsentMessage {
    FieldsDisplayMessage(FieldsDisplay),
    GenericDisplayMessage(String),
}

#[derive(CandidType, Deserialize)]
pub struct ConsentInfo {
    pub metadata: ConsentMessageMetadata,
    pub consent_message: ConsentMessage,
}

#[derive(CandidType, Deserialize)]
pub struct ErrorInfo {
    pub description: String,
}

#[derive(CandidType, Deserialize)]
pub enum Icrc21Error {
    GenericError {
        description: String,
        error_code: Nat,
    },
    InsufficientPayment(ErrorInfo),
    UnsupportedCanisterCall(ErrorInfo),
    ConsentMessageUnavailable(ErrorInfo),
}

#[derive(CandidType, Deserialize)]
pub enum Result3 {
    Ok(ConsentInfo),
    Err(Icrc21Error),
}

#[derive(CandidType, Deserialize)]
pub struct AllowanceArgs {
    pub account: Account,
    pub spender: Account,
}

#[derive(CandidType, Deserialize)]
pub struct Allowance1 {
    pub allowance: Nat,
    pub expires_at: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct ApproveArgs {
    pub fee: Option<Nat>,
    pub memo: Option<std::vec::Vec<u8>>,
    pub from_subaccount: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub spender: Account,
}

#[derive(CandidType, Deserialize)]
pub enum ApproveError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    AllowanceChanged { current_allowance: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    Expired { ledger_time: u64 },
    InsufficientFunds { balance: Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result4 {
    Ok(Nat),
    Err(ApproveError),
}

#[derive(CandidType, Deserialize)]
pub struct TransferFromArgs {
    pub to: Account,
    pub fee: Option<Nat>,
    pub spender_subaccount: Option<std::vec::Vec<u8>>,
    pub from: Account,
    pub memo: Option<std::vec::Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize)]
pub enum TransferFromError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    InsufficientAllowance { allowance: Nat },
    BadBurn { min_burn_amount: Nat },
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    InsufficientFunds { balance: Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result5 {
    Ok(Nat),
    Err(TransferFromError),
}

#[derive(CandidType, Deserialize)]
pub struct GetArchivesArgs {
    pub from: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct Icrc3ArchiveInfo {
    pub end: Nat,
    pub canister_id: Principal,
    pub start: Nat,
}

#[derive(CandidType, Deserialize)]
pub enum Icrc3Value {
    Int(candid::Int),
    Map(std::vec::Vec<(String, Box<Icrc3Value>)>),
    Nat(candid::Nat),
    Blob(std::vec::Vec<u8>),
    Text(String),
    Array(std::vec::Vec<Box<Icrc3Value>>),
}

#[derive(CandidType, Deserialize)]
pub struct BlockWithId {
    pub id: Nat,
    pub block: Box<Icrc3Value>,
}

#[derive(CandidType, Deserialize)]
pub struct ArchivedBlocks {
    pub args: std::vec::Vec<GetBlocksRequest>,
    pub callback: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct GetBlocksResult {
    pub log_length: Nat,
    pub blocks: std::vec::Vec<BlockWithId>,
    pub archived_blocks: std::vec::Vec<ArchivedBlocks>,
}

#[derive(CandidType, Deserialize)]
pub struct Icrc3DataCertificate {
    pub certificate: std::vec::Vec<u8>,
    pub hash_tree: std::vec::Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct SupportedBlockType {
    pub url: String,
    pub block_type: String,
}
