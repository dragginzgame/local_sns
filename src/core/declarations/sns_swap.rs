#![allow(dead_code, unused_imports, unused_variables)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};

#[derive(CandidType, Deserialize, Debug)]
pub struct NewSaleTicketRequest {
    pub subaccount: Option<Vec<u8>>,
    pub amount_icp_e8s: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Ticket {
    pub creation_time: u64,
    pub ticket_id: u64,
    pub account: Option<Icrc1Account>,
    pub amount_icp_e8s: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Icrc1Account {
    pub owner: Option<Principal>,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Ok2 {
    pub ticket: Option<Ticket>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct InvalidUserAmount {
    pub min_amount_icp_e8s_included: u64,
    pub max_amount_icp_e8s_included: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Err2 {
    pub invalid_user_amount: Option<InvalidUserAmount>,
    pub existing_ticket: Option<Ticket>,
    pub error_type: i32,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum Result2 {
    #[serde(rename = "Ok")]
    Ok(Ok2),
    #[serde(rename = "Err")]
    Err(Err2),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct NewSaleTicketResponse {
    pub result: Option<Result2>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RefreshBuyerTokensRequest {
    pub confirmation_text: Option<String>,
    pub buyer: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RefreshBuyerTokensResponse {
    pub icp_accepted_participation_e8s: u64,
    pub icp_ledger_account_balance_e8s: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetLifecycleArg {}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetLifecycleResponse {
    pub decentralization_sale_open_timestamp_seconds: Option<u64>,
    pub lifecycle: Option<i32>,
    pub decentralization_swap_termination_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct OpenRequest {}

// Open doesn't return anything - it's just an update call

#[derive(CandidType, Deserialize, Debug)]
pub struct FinalizeSwapArg {}

#[derive(CandidType, Deserialize, Debug)]
pub struct FinalizeSwapResponse {
    pub error_message: Option<String>,
    // Note: Other fields are complex nested types we don't need to decode
    // The error_message is sufficient for basic error checking
}
