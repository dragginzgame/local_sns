// Constants for SNS deployment

// Standard NNS canister IDs for local development
pub const GOVERNANCE_CANISTER: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
pub const LEDGER_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const SNSW_CANISTER: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

// Amounts in e8s (1 ICP = 100_000_000 e8s)
pub const DEVELOPER_ICP: u64 = 100_000_000_000_000; // 1M ICP in e8s
pub const PARTICIPANT_ICP: u64 = 100_000_000_000; // 1000 ICP in e8s
pub const ICP_TRANSFER_FEE: u64 = 10_000; // ICP transfer fee in e8s (0.0001 ICP)

// Neuron configuration
pub const MEMO: u64 = 1;
pub const DISSOLVE_DELAY: u64 = 252460800; // 8 years in seconds
