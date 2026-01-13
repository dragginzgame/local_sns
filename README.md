# Local SNS Deployment Tool

> **Important**: Requires dfx version 0.30.1 or higher that supports the `--system-canisters` flag

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
│   ├── init/              # SNS initialization configuration
│   │   ├── mod.rs
│   │   ├── sns_config.rs  # SNS parameters and configuration
│   │   └── logo.png       # Logo file (PNG format)
│   ├── core/
│   │   ├── declarations/  # Candid type definitions
│   │   │   ├── icp_governance.rs
│   │   │   ├── icp_ledger.rs
│   │   │   ├── sns_governance.rs
│   │   │   ├── sns_swap.rs
│   │   │   └── sns_wasm.rs
│   │   ├── ops/           # Operation modules
│   │   │   ├── commands.rs        # CLI command handlers
│   │   │   ├── deployment.rs      # Core SNS deployment logic
│   │   │   ├── governance_ops.rs
│   │   │   ├── identity.rs
│   │   │   ├── ledger_ops.rs
│   │   │   ├── sns_governance_ops.rs
│   │   │   ├── snsw_ops.rs
│   │   │   └── swap_ops.rs
│   │   └── utils/         # Utility functions
│   │       ├── mod.rs
│   │       ├── constants.rs       # Constants and configuration
│   │       └── data_output.rs     # Deployment data serialization
├── scripts/               # Bash wrapper scripts
│   ├── start.sh                   # Interactive menu (main entry point)
│   ├── build.sh                   # Build the local_sns binary
│   ├── deploy_local_sns.sh        # Deploy a new SNS
│   ├── add_sns_hotkey.sh          # Add hotkey to SNS neuron (interactive)
│   ├── add_icp_hotkey.sh          # Add hotkey to ICP neuron
│   ├── get_sns_neurons.sh         # List SNS neurons (interactive)
│   ├── get_icp_neuron.sh          # Get ICP neuron information
│   ├── set_icp_visibility.sh      # Set ICP neuron visibility
│   ├── disburse_neuron.sh         # Disburse tokens from SNS neuron (interactive)
│   └── mint_sns_tokens.sh         # Mint SNS tokens via proposal (interactive)
└── generated/             # Generated files (git-ignored)
    ├── sns_deployment_data.json
    └── participants/
        └── participant_*.seed
```

## Prerequisites

- **Rust toolchain**: Install from [rustup.rs](https://rustup.rs/)
- **dfx SDK**: Internet Computer SDK version **0.30.1 or higher** that supports the `--system-canisters` flag
- **Local dfx network**: Must be running with system canisters

  ```bash
  dfx start --clean --system-canisters
  ```

  > **Note**: The `--system-canisters` flag is required. Older versions of dfx do not support this flag.

## Quick Start

### Using Interactive Menu (Recommended)

```bash
# Launch the interactive menu to select operations
bash scripts/start.sh
```

The menu provides easy access to all available operations:

- **Deploy Local SNS** - Create a new SNS on your local dfx network
- **Add SNS Neuron Hotkey** - Add a hotkey to an SNS participant neuron (interactive)
- **Add ICP Neuron Hotkey** - Add a hotkey to the ICP neuron used for SNS deployment
- **List SNS Neurons** - Query and display SNS neurons for a principal (interactive)
- **Get ICP Neuron Info** - Get detailed information about the ICP neuron
- **Set ICP Neuron Visibility** - Set the ICP neuron to public or private
- **Disburse Neuron** - Disburse tokens from an SNS neuron to a receiver (interactive)
- **Mint SNS Tokens** - Mint additional tokens by creating a proposal (interactive)

### Using Bash Scripts Directly

All scripts support interactive prompts for missing arguments:

```bash
# Deploy a local SNS
bash scripts/deploy_local_sns.sh

# Add a hotkey to an SNS neuron (interactive - prompts for participant, neuron, hotkey)
bash scripts/add_sns_hotkey.sh
# Or with arguments:
bash scripts/add_sns_hotkey.sh <participant_principal> <hotkey_principal>

# Add a hotkey to ICP neuron
bash scripts/add_icp_hotkey.sh <hotkey_principal>

# Query SNS neurons (interactive - shows participant menu if no principal provided)
bash scripts/get_sns_neurons.sh
# Or with principal:
bash scripts/get_sns_neurons.sh <principal>

# Get ICP neuron information
bash scripts/get_icp_neuron.sh [neuron_id]

# Set ICP neuron visibility
bash scripts/set_icp_visibility.sh <true|false>

# Disburse neuron tokens (interactive - prompts for participant, neuron, receiver)
bash scripts/disburse_neuron.sh

# Mint SNS tokens (interactive - prompts for proposer, receiver, amount)
bash scripts/mint_sns_tokens.sh
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

## SNS Configuration

The SNS deployment parameters can be customized in `src/init/sns_config.rs`. This file contains all the configuration for your SNS including:

- **Basic Information**: Name, description, and URL
- **Ledger Parameters**: Token symbol, name, and transaction fees
- **Governance Parameters**: Voting periods, dissolve delays, and neuron configuration
- **Swap Parameters**: Participation requirements, minimum/maximum ICP amounts, and duration
- **Token Distribution**: Treasury, developer, and swap allocations

### Logo Configuration

The tool automatically loads a logo from `src/init/logo.png` (PNG format). The logo is:

- Converted to base64 format automatically
- Used for both the SNS logo and token logo
- Falls back to a default logo if the file is not found

To customize your SNS logo:

1. Place a PNG image file named `logo.png` in the `src/init/` directory
2. The logo will be automatically loaded and used during deployment
3. If no logo is found, the deployment will use a default logo and print an info message

> **Note**: The logo file must be in PNG format. The tool will automatically handle the base64 encoding.

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

### `list-sns-neurons`

List all SNS neurons owned by a principal.

**Usage:**

```bash
cargo run --bin local_sns -- list-sns-neurons <principal>
```

### `create-sns-neuron`

Create an SNS neuron by staking tokens from the SNS ledger balance.

**Usage:**

```bash
cargo run --bin local_sns -- create-sns-neuron [principal] [amount_e8s] [memo]
```

**Arguments:**

- `principal`: Optional. Principal to create the neuron for. If not provided, shows participant selection menu.
- `amount_e8s`: Optional. Amount of tokens to stake in e8s. If not provided, stakes all available balance.
- `memo`: Optional. Memo to use for neuron creation (default: 1).

The command will:

1. Check the SNS ledger balance for the principal
2. Verify the balance meets the minimum stake requirement (0.1 tokens = 10,000,000 e8s)
3. Transfer tokens to the governance canister subaccount
4. Claim the neuron

### `disburse-sns-neuron`

Disburse tokens from an SNS neuron to a receiver principal.

**Usage:**

```bash
cargo run --bin local_sns -- disburse-sns-neuron [participant_principal] [neuron_id_hex|receiver_principal] [receiver_principal]
```

**Arguments:**

- `participant_principal`: Optional. Principal of the participant who owns the neuron.
- `neuron_id_hex`: Optional. Neuron ID in hex format.
- `receiver_principal`: Optional. Principal to receive the disbursed tokens.

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
- **Minting Identity**: Hardcoded PEM in `src/core/ops/identity.rs` (used for funding operations)
- **Participant Identities**: Deterministic seeds saved to `generated/participants/` for reuse

## Building

The binary is automatically built when you run `start.sh`. You can also build it manually:

```bash
# Build using the build script (tries release, falls back to dev)
bash scripts/build.sh

# Or build directly with cargo
cargo build --bin local_sns              # Debug build
cargo build --release --bin local_sns    # Release build
```

> **Note**: Individual scripts no longer rebuild the binary. The binary is built once when you start the menu, or you can run `build.sh` manually before running individual scripts.

## Testing

Run the deployment script on a local dfx network:

```bash
# Start local network
dfx start --clean --system-canisters

# In another terminal, launch the interactive menu
bash scripts/start.sh

# Or run deployment directly
bash scripts/deploy_local_sns.sh
```

## Available Scripts

All scripts are located in the `scripts/` directory:

- **`start.sh`** - Interactive menu for all operations (recommended entry point)
- **`deploy_local_sns.sh`** - Full SNS deployment workflow
- **`add_sns_hotkey.sh`** - Add hotkey to SNS neuron (interactive participant/neuron selection)
- **`add_icp_hotkey.sh`** - Add hotkey to ICP neuron
- **`get_sns_neurons.sh`** - List all SNS neurons for a principal (interactive participant selection)
- **`get_icp_neuron.sh`** - Get detailed ICP neuron information
- **`set_icp_visibility.sh`** - Set ICP neuron visibility (public/private)
- **`create_sns_neuron.sh`** - Create an SNS neuron by staking tokens from ledger balance
- **`disburse_sns_neuron.sh`** - Disburse tokens from SNS neuron (interactive selection)
- **`mint_sns_tokens.sh`** - Mint tokens via governance proposal (interactive prompts)

Most scripts provide interactive prompts when arguments are omitted, making them easy to use without remembering exact command syntax.

## Copying to Another Repository

This entire `local_sns/` directory is self-contained. To use it in another project:

1. Copy the entire `local_sns/` directory to your repository
2. Update paths if necessary (all paths are relative to the `local_sns/` root)
3. Run `cargo build` to verify it compiles
4. Ensure `dfx` is configured with system canisters

All generated files will be created in the `local_sns/generated/` directory.

## License

Same as the parent project.
