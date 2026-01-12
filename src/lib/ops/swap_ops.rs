// SNS Swap canister operations

use anyhow::{Context, Result};
use candid::{Decode, Principal, encode_args};
use ic_agent::Agent;
use ic_ledger_types::Subaccount;

use super::super::declarations::sns_swap::{
    FinalizeSwapArg, FinalizeSwapResponse, GetLifecycleArg, GetLifecycleResponse,
    NewSaleTicketRequest, NewSaleTicketResponse, RefreshBuyerTokensRequest,
    RefreshBuyerTokensResponse, Result2,
};
use super::super::utils::{print_info, print_warning};

#[derive(candid::CandidType, candid::Deserialize, Debug)]
struct GetDerivedStateArg {}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub struct GetDerivedStateResponse {
    pub direct_participant_count: Option<u64>,
    pub direct_participation_icp_e8s: Option<u64>,
    pub cf_participant_count: Option<u64>,
    pub cf_participation_icp_e8s: Option<u64>,
    pub buyer_total_icp_e8s: Option<u64>,
    pub sns_tokens_per_icp: Option<u64>,
}

/// Generate participant subaccount from principal using ic_ledger_types::Subaccount
///
/// This matches the implementation in ic-ledger-types::Subaccount::from(Principal)
/// which uses a length prefix: [length_byte, principal_bytes..., 0...]
pub fn generate_participant_subaccount(principal: Principal) -> Subaccount {
    let mut subaccount = [0u8; 32];
    let principal_bytes = principal.as_slice();
    subaccount[0] = principal_bytes.len().try_into().unwrap();
    subaccount[1..1 + principal_bytes.len()].copy_from_slice(principal_bytes);
    Subaccount(subaccount)
}

/// Create new sale ticket
/// Returns true if successful, false otherwise
/// Note: Sale ticket creation is optional - the bash script ignores failures
pub async fn create_sale_ticket(
    agent: &Agent,
    swap_canister: Principal,
    amount_icp_e8s: u64,
    subaccount: Option<Vec<u8>>,
) -> Result<bool> {
    let request = NewSaleTicketRequest {
        amount_icp_e8s,
        subaccount: subaccount.map(|v| v.to_vec()),
    };

    let result_bytes = match agent
        .update(&swap_canister, "new_sale_ticket")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
    {
        Ok(bytes) => bytes,
        Err(_e) => {
            // Call failed - sale ticket creation is optional
            return Ok(false);
        }
    };

    // Decode using the correct structure from sns_swap.rs
    match Decode!(&result_bytes, NewSaleTicketResponse) {
        Ok(response) => {
            if let Some(result) = response.result {
                match result {
                    Result2::Ok(ok) => {
                        if let Some(ticket) = ok.ticket {
                            print_info(&format!(
                                "  ✓ Sale ticket created (ID: {})",
                                ticket.ticket_id
                            ));
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                    Result2::Err(err) => {
                        let msg = if let Some(existing) = err.existing_ticket {
                            format!("Existing ticket found (ID: {})", existing.ticket_id)
                        } else if let Some(invalid) = err.invalid_user_amount {
                            format!(
                                "Invalid amount (min: {}, max: {})",
                                invalid.min_amount_icp_e8s_included,
                                invalid.max_amount_icp_e8s_included
                            )
                        } else {
                            format!("Error type: {}", err.error_type)
                        };
                        print_warning(&format!("  Sale ticket error: {msg}"));
                        Ok(false)
                    }
                }
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            // Decode failed - might be different format, but continue anyway
            // The bash script ignores sale ticket failures
            print_warning(&format!(
                "  Could not decode sale ticket response (continuing): {e}"
            ));
            Ok(false)
        }
    }
}

/// Refresh buyer tokens
pub async fn refresh_buyer_tokens(
    agent: &Agent,
    swap_canister: Principal,
    buyer: Principal,
) -> Result<RefreshBuyerTokensResponse> {
    let request = RefreshBuyerTokensRequest {
        confirmation_text: None,
        buyer: buyer.to_string(),
    };

    let result_bytes = agent
        .update(&swap_canister, "refresh_buyer_tokens")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to refresh buyer tokens")?;

    let response: RefreshBuyerTokensResponse =
        Decode!(&result_bytes, RefreshBuyerTokensResponse)
            .context("Failed to decode refresh_buyer_tokens response")?;

    // RefreshBuyerTokensResponse shows what the swap actually saw
    print_info(&format!(
        "  Swap reports: ICP accepted: {} e8s, Ledger balance checked: {} e8s",
        response.icp_accepted_participation_e8s, response.icp_ledger_account_balance_e8s
    ));

    Ok(response)
}

/// Get swap lifecycle
pub async fn get_swap_lifecycle(agent: &Agent, swap_canister: Principal) -> Result<i32> {
    let request = GetLifecycleArg {};

    let result_bytes = agent
        .query(&swap_canister, "get_lifecycle")
        .with_arg(encode_args((request,))?)
        .call()
        .await
        .context("Failed to get swap lifecycle")?;

    let response: GetLifecycleResponse = Decode!(&result_bytes, GetLifecycleResponse)
        .context("Failed to decode get_lifecycle response")?;

    Ok(response.lifecycle.unwrap_or(0))
}

/// Get swap derived state to check participation
pub async fn get_derived_state(
    agent: &Agent,
    swap_canister: Principal,
) -> Result<GetDerivedStateResponse> {
    let request = GetDerivedStateArg {};

    let result_bytes = agent
        .query(&swap_canister, "get_derived_state")
        .with_arg(encode_args((request,))?)
        .call()
        .await
        .context("Failed to get derived state")?;

    let response: GetDerivedStateResponse = Decode!(&result_bytes, GetDerivedStateResponse)
        .context("Failed to decode get_derived_state response")?;

    Ok(response)
}

/// Finalize swap
pub async fn finalize_swap(agent: &Agent, swap_canister: Principal) -> Result<()> {
    let request = FinalizeSwapArg {};

    let result_bytes = agent
        .update(&swap_canister, "finalize_swap")
        .with_arg(encode_args((request,))?)
        .call_and_wait()
        .await
        .context("Failed to finalize swap")?;

    let response: FinalizeSwapResponse = Decode!(&result_bytes, FinalizeSwapResponse)
        .context("Failed to decode finalize_swap response")?;

    // FinalizeSwapResponse is a struct with optional fields
    if let Some(error_msg) = response.error_message {
        print_warning(&format!("Finalize swap warning: {error_msg}"));
    }

    Ok(())
}
