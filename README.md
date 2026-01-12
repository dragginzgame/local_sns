# Local SNS Deployment Tool

> requires --system-canisters supported DFX version (dfx 0.30.1 or higher)

Standalone Rust tool for deploying and managing Service Nervous System (SNS) instances on local `dfx` networks.

This directory is self-contained and can be copied to a separate repository. All dependencies are specified in `Cargo.toml`, and all generated files are stored in the `generated/` directory.

## Directory Structure

```
local_sns/
├── Cargo.toml              # Standalone Rust package configuration
├── README.md               # This file
├── .gitignore             # Git ignore rules for generated files
├── src/                   # Rust source code
│   ├── main.rs            # Main entry point
│   ├── commands.rs        # CLI command handlers
│   ├── constants.rs       # Constants and configuration
│   ├── data_output.rs     # Deployment data serialization
│   ├── deployment.rs      # Core SNS deployment logic
│   ├── declarations/      # Candid type definitions
│   │   ├── icp_governance.rs
│   │   ├── icp_ledger.rs
│   │   ├── sns_governance.rs
│   │   ├── sns_swap.rs
│   │   └── sns_wasm.rs
│   ├── ops/               # Operation modules
│   │   ├── governance_ops.rs
│   │   ├── identity.rs
│   │   ├── ledger_ops.rs
│   │   ├── sns_governance_ops.rs
│   │   ├── snsw_ops.rs
│   │   └── swap_ops.rs
│   └── utils/             # Utility functions
│       └── mod.rs
├── scripts/               # Bash wrapper scripts
│   ├── deploy_local_sns.sh
│   ├── add_sns_hotkey.sh
│   ├── add_icp_hotkey.sh
│   ├── set_icp_visibility.sh
│   ├── get_sns_neurons.sh
│   └── get_icp_neuron.sh
└── generated/             # Generated files (git-ignored)
    ├── sns_deployment_data.json
    └── participants/
        └── participant_*.seed
```

## Prerequisites

- **Rust toolchain**: Install from [rustup.rs](https://rustup.rs/)
- **dfx SDK**: Internet Computer SDK installed and configured
- **Local dfx network**: Must be running with system canisters
  ```bash
  dfx start --clean --system-canisters
  ```

## Quick Start

### Using Bash Scripts (Recommended)

```bash
# Deploy a local SNS
bash scripts/deploy_local_sns.sh

# Add a hotkey to an SNS neuron
bash scripts/add_sns_hotkey.sh <participant_principal> <hotkey_principal>

# Query SNS neurons
bash scripts/get_sns_neurons.sh <principal>
```

### Using Rust Binary Directly

```bash
# Build the binary
cargo build --release --bin local_sns

# Deploy SNS (no arguments = full deployment)
cargo run --bin local_sns

# Add hotkey to SNS neuron
cargo run --bin local_sns -- add-hotkey sns <participant_principal> <hotkey_principal> [permissions]

# Add hotkey to ICP neuron
cargo run --bin local_sns -- add-hotkey icp <hotkey_principal>

# List SNS neurons for a principal
cargo run --bin local_sns -- list-neurons <principal>

# Set ICP neuron visibility
cargo run --bin local_sns -- set-icp-visibility <true|false>

# Get ICP neuron information
cargo run --bin local_sns -- get-icp-neuron [neuron_id]
```

## Generated Files

All generated files are stored in the `generated/` directory (relative to the `local_sns/` root):

- **`generated/sns_deployment_data.json`**: Deployment metadata including:

  - ICP neuron ID used for proposal
  - Proposal ID
  - Owner principal
  - Deployed SNS canister IDs (governance, ledger, swap, etc.)
  - Participant principals and their seed file paths

- **`generated/participants/participant_*.seed`**: Seed files for participant identities (hex-encoded 32-byte Ed25519 seeds)

These files are git-ignored and overwritten on each deployment.

## How SNS Deployment Works

The deployment process follows these steps:

1. **Create ICP Neuron**: Creates and configures an ICP neuron with maximum dissolve delay
2. **Fund Owner**: Transfers ICP from minting account to owner account
3. **Create SNS Proposal**: Submits a `CreateServiceNervousSystem` proposal to ICP Governance
4. **Wait for Execution**: Polls SNS-W canister until proposal executes and SNS canisters are deployed
5. **Prepare Participants**: Creates 5 deterministic participant identities and funds them
6. **Wait for Swap to Open**: Blocks until swap reaches lifecycle 2 (Open state)
7. **Participate in Swap**: Each participant transfers ICP and creates sale tickets
8. **Finalize Swap**: Finalizes the swap when participation thresholds are met
9. **Save Deployment Data**: Writes all metadata to `generated/sns_deployment_data.json`

For detailed information about each step, see the inline documentation in the source files.

## CLI Commands

### `add-hotkey`

Add a hotkey to an SNS or ICP neuron.

**Usage:**

```bash
cargo run --bin local_sns -- add-hotkey <sns|icp> <owner_principal> <hotkey_principal> [permissions]
```

**Arguments:**

- `sns|icp`: Neuron type
- `owner_principal`: For SNS, the participant principal. For ICP, ignored (uses default dfx identity)
- `hotkey_principal`: Principal to add as hotkey
- `permissions` (SNS only): Comma-separated permission types (default: `3,4`)

### `list-neurons`

List all SNS neurons owned by a principal.

**Usage:**

```bash
cargo run --bin local_sns -- list-neurons <principal>
```

### `set-icp-visibility`

Set the visibility of the ICP neuron (public/private).

**Usage:**

```bash
cargo run --bin local_sns -- set-icp-visibility <true|false>
```

### `get-icp-neuron`

Get full information about an ICP neuron.

**Usage:**

```bash
cargo run --bin local_sns -- get-icp-neuron [neuron_id]
```

If `neuron_id` is not provided, uses the neuron ID from deployment data.

## Canister IDs

Uses standard NNS canister IDs for local development:

- **ICP Governance**: `rrkah-fqaaa-aaaaa-aaaaq-cai`
- **ICP Ledger**: `ryjl3-tyaaa-aaaaa-aaaba-cai`
- **SNS-W (Wrapper)**: `qaa6y-5yaaa-aaaaa-aaafa-cai`

## Identity Management

- **Owner Identity**: Loaded from `~/.config/dfx/identity/default/identity.pem`
- **Minting Identity**: Hardcoded PEM in `src/ops/identity.rs` (used for funding operations)
- **Participant Identities**: Deterministic seeds saved to `generated/participants/` for reuse

## Building

```bash
# Debug build
cargo build --bin local_sns

# Release build
cargo build --release --bin local_sns
```

## Testing

Run the deployment script on a local dfx network:

```bash
# Start local network
dfx start --clean --system-canisters

# In another terminal, run deployment
bash scripts/deploy_local_sns.sh
```

## Copying to Another Repository

This entire `local_sns/` directory is self-contained. To use it in another project:

1. Copy the entire `local_sns/` directory to your repository
2. Update paths if necessary (all paths are relative to the `local_sns/` root)
3. Run `cargo build` to verify it compiles
4. Ensure `dfx` is configured with system canisters

All generated files will be created in the `local_sns/generated/` directory.

## License

Same as the parent project.
