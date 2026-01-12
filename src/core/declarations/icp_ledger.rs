// ICP Ledger canister Candid type definitions
// Generated from Candid, with serde_bytes::ByteBuf replaced with Vec<u8>

#![allow(dead_code, unused_imports, unused_variables)]

use candid::{CandidType, Deserialize, Nat, Principal};

#[derive(CandidType, Deserialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct TransferArg {
    pub to: Account,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
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
pub enum TransferResult {
    #[serde(rename = "Ok")]
    Ok(Nat),
    #[serde(rename = "Err")]
    Err(TransferError),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Tokens {
    pub e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct TransferArgs {
    pub to: Vec<u8>,
    pub fee: Tokens,
    pub memo: u64,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<TimeStamp>,
    pub amount: Tokens,
}

#[derive(CandidType, Deserialize)]
pub struct TimeStamp {
    pub timestamp_nanos: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError1 {
    TxTooOld { allowed_window_nanos: u64 },
    BadFee { expected_fee: Tokens },
    TxDuplicate { duplicate_of: u64 },
    TxCreatedInFuture,
    InsufficientFunds { balance: Tokens },
}

#[derive(CandidType, Deserialize)]
pub enum TransferResult1 {
    #[serde(rename = "Ok")]
    Ok(u64),
    #[serde(rename = "Err")]
    Err(TransferError1),
}
