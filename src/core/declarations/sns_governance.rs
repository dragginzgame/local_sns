// SNS Governance canister Candid types
// Based on SNS Governance canister IDL

#![allow(dead_code, unused_imports, unused_variables)]
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct NeuronId {
    pub id: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronPermissionList {
    pub permissions: Vec<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct AddNeuronPermissions {
    pub permissions_to_add: Option<NeuronPermissionList>,
    pub principal_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct RemoveNeuronPermissions {
    pub permissions_to_remove: Option<NeuronPermissionList>,
    pub principal_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub enum Command {
    AddNeuronPermissions(AddNeuronPermissions),
    RemoveNeuronPermissions(RemoveNeuronPermissions),
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuron {
    pub subaccount: Vec<u8>,
    pub command: Option<Command>,
}

#[derive(CandidType, Deserialize)]
pub struct GovernanceError {
    pub error_message: String,
    pub error_type: i32,
}

#[derive(CandidType, Deserialize)]
pub enum Command1 {
    Error(GovernanceError),
    AddNeuronPermission {},
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuronResponse {
    pub command: Option<Command1>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct NeuronPermission {
    pub principal: Option<Principal>,
    pub permission_type: Vec<i32>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct Neuron {
    pub id: Option<NeuronId>,
    pub permissions: Vec<NeuronPermission>,
    // ... other fields omitted for brevity
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
