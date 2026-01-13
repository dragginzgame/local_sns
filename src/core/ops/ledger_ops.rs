// ICP Ledger operations

use anyhow::{Context, Result};
use candid::{Decode, Nat, Principal, encode_args};
use ic_agent::Agent;
use ic_ledger_types::Subaccount;
use sha2::{Digest, Sha256};

use super::super::declarations::icp_ledger::{
    Account as LedgerAccount, TransferArg, TransferResult,
};
use super::super::declarations::sns_ledger::{
    Account as SnsLedgerAccount, Result2 as SnsTransferResult, TransferArg as SnsTransferArg,
};

/// Generate neuron subaccount (matches Rust implementation from test code)
pub fn generate_subaccount_by_nonce(nonce: u64, principal: Principal) -> Subaccount {
    let mut hasher = Sha256::new();
    hasher.update([0x0c]);
    hasher.update(b"neuron-stake");
    hasher.update(principal.as_slice());
    hasher.update(nonce.to_be_bytes());
    let hash_result = hasher.finalize();
    let mut subaccount = [0u8; 32];
    subaccount.copy_from_slice(&hash_result[..]);
    Subaccount(subaccount)
}

/// Transfer ICP using icrc1_transfer (for general use)
pub async fn transfer_icp(
    agent: &Agent,
    ledger_canister: Principal,
    to: Principal,
    amount: u64,
    subaccount: Option<Vec<u8>>,
) -> Result<u64> {
    // Use icrc1_transfer with correct types from ICP ledger
    let args = TransferArg {
        to: LedgerAccount {
            owner: to,
            subaccount,
        },
        fee: None,
        memo: None,
        from_subaccount: None,
        created_at_time: None,
        amount: Nat::from(amount),
    };

    let result_bytes = agent
        .update(&ledger_canister, "icrc1_transfer")
        .with_arg(encode_args((args,))?)
        .call_and_wait()
        .await
        .context("Failed to call icrc1_transfer")?;

    let result: TransferResult =
        Decode!(&result_bytes, TransferResult).context("Failed to decode transfer result")?;

    match result {
        TransferResult::Ok(block_height) => {
            // Convert candid::Nat to u64
            // Nat stores as BigUint, convert first digit or 0
            let digits = block_height.0.to_u64_digits();
            Ok(digits.first().copied().unwrap_or(0))
        }
        TransferResult::Err(e) => {
            anyhow::bail!("Transfer failed: {e:?}");
        }
    }
}

/// Get SNS ledger transfer fee
pub async fn get_sns_ledger_fee(agent: &Agent, ledger_canister: Principal) -> Result<u64> {
    let result_bytes = agent
        .query(&ledger_canister, "icrc1_fee")
        .with_arg(encode_args(())?)
        .call()
        .await
        .context("Failed to call icrc1_fee")?;

    let fee: Nat = Decode!(&result_bytes, Nat).context("Failed to decode fee")?;

    // Convert candid::Nat to u64
    let digits = fee.0.to_u64_digits();
    Ok(digits.first().copied().unwrap_or(0))
}

/// Get ICP ledger balance for an account
pub async fn get_icp_ledger_balance(
    agent: &Agent,
    ledger_canister: Principal,
    account: Principal,
    subaccount: Option<Vec<u8>>,
) -> Result<u64> {
    let account = LedgerAccount {
        owner: account,
        subaccount,
    };

    let result_bytes = agent
        .query(&ledger_canister, "icrc1_balance_of")
        .with_arg(encode_args((account,))?)
        .call()
        .await
        .context("Failed to call icrc1_balance_of")?;

    let balance: Nat = Decode!(&result_bytes, Nat).context("Failed to decode balance")?;

    // Convert candid::Nat to u64
    let digits = balance.0.to_u64_digits();
    Ok(digits.first().copied().unwrap_or(0))
}

/// Get SNS ledger balance for an account
pub async fn get_sns_ledger_balance(
    agent: &Agent,
    ledger_canister: Principal,
    account: Principal,
    subaccount: Option<Vec<u8>>,
) -> Result<u64> {
    let account = SnsLedgerAccount {
        owner: account,
        subaccount,
    };

    let result_bytes = agent
        .query(&ledger_canister, "icrc1_balance_of")
        .with_arg(encode_args((account,))?)
        .call()
        .await
        .context("Failed to call icrc1_balance_of")?;

    let balance: Nat = Decode!(&result_bytes, Nat).context("Failed to decode balance")?;

    // Convert candid::Nat to u64
    let digits = balance.0.to_u64_digits();
    Ok(digits.first().copied().unwrap_or(0))
}

/// Transfer SNS tokens using icrc1_transfer
pub async fn transfer_sns_tokens(
    agent: &Agent,
    ledger_canister: Principal,
    to: Principal,
    amount: u64,
    subaccount: Option<Vec<u8>>,
) -> Result<u64> {
    let args = SnsTransferArg {
        to: SnsLedgerAccount {
            owner: to,
            subaccount,
        },
        fee: None,
        memo: None,
        from_subaccount: None,
        created_at_time: None,
        amount: Nat::from(amount),
    };

    let result_bytes = agent
        .update(&ledger_canister, "icrc1_transfer")
        .with_arg(encode_args((args,))?)
        .call_and_wait()
        .await
        .context("Failed to call icrc1_transfer")?;

    let result: SnsTransferResult =
        Decode!(&result_bytes, SnsTransferResult).context("Failed to decode transfer result")?;

    match result {
        SnsTransferResult::Ok(block_height) => {
            // Convert candid::Nat to u64
            let digits = block_height.0.to_u64_digits();
            Ok(digits.first().copied().unwrap_or(0))
        }
        SnsTransferResult::Err(e) => {
            anyhow::bail!("Transfer failed: {e:?}");
        }
    }
}
