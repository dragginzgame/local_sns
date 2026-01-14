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
│   ├── add_sns_hotkey.sh          # Add hotkey to SNS neuron
│   ├── add_icp_hotkey.sh          # Add hotkey to ICP neuron
│   ├── get_sns_neurons.sh         # List SNS neurons
│   ├── get_icp_neurons.sh         # List ICP neurons
│   ├── get_icp_neuron.sh          # Get ICP neuron information
│   ├── set_icp_visibility.sh      # Set ICP neuron visibility
│   ├── create_sns_neuron.sh       # Create SNS neuron by staking tokens
│   ├── create_icp_neuron.sh       # Create ICP neuron by staking ICP
│   ├── disburse_sns_neuron.sh     # Disburse tokens from SNS neuron
│   ├── disburse_icp_neuron.sh     # Disburse tokens from ICP neuron
│   ├── mint_sns_tokens.sh         # Mint SNS tokens via proposal
│   ├── mint_icp.sh                # Mint ICP tokens
│   ├── increase_sns_dissolve_delay.sh  # Increase dissolve delay for SNS neuron
│   ├── increase_icp_dissolve_delay.sh  # Increase dissolve delay for ICP neuron
│   ├── manage_sns_dissolving.sh   # Start or stop dissolving SNS neuron
│   ├── manage_icp_dissolving.sh   # Start or stop dissolving ICP neuron
│   ├── get_sns_balance.sh         # Get SNS ledger balance
│   └── get_icp_balance.sh         # Get ICP ledger balance
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

The menu automatically checks if an SNS is deployed and shows different options accordingly:

#### When No SNS is Deployed

If no SNS is detected on the network, the menu shows a simplified interface with 3 options:

- **1 / [M]** Mint ICP Tokens - Mint ICP tokens from minting account to a receiver
- **2 / [B]** Get ICP Balance - Get ICP ledger balance for an account
- **3 / [D]** Deploy SNS - Deploy a new SNS instance
- **0 / Enter** Exit

#### When SNS is Deployed

Once an SNS is deployed, the menu provides a hierarchical structure with three main categories:

**Main Menu:**

- **1 / [I]** ICP Operations
- **2 / [S]** SNS Operations
- **3 / [U]** Utils
- **0** Exit

**ICP Operations Submenu:**

- **1 / [L]** List ICP Neurons - List all ICP neurons for a principal (includes detailed view)
- **2 / [C]** Create ICP Neuron - Create an ICP neuron by staking ICP tokens
- **3 / [D]** Disburse ICP Neuron - Disburse tokens from an ICP neuron
- **4 / [M]** Mint ICP Tokens - Mint ICP tokens from minting account to a receiver
- **5 / [H]** Add ICP Neuron Hotkey - Add a hotkey to an ICP neuron
- **6 / [I]** Increase ICP Neuron Dissolve Delay - Add dissolve delay to an ICP neuron
- **7 / [DD]** Dissolve ICP Neuron - Start or stop dissolving for an ICP neuron
- **8 / [B]** Get ICP Balance - Get ICP ledger balance for an account
- **0 / Enter** Back to Main Menu

**SNS Operations Submenu:**

- **1 / [L]** List SNS Neurons - Query and display SNS neurons for a principal (interactive)
- **2 / [C]** Create SNS Neuron - Create an SNS neuron by staking tokens from ledger balance
- **3 / [D]** Disburse SNS Neuron - Disburse tokens from an SNS neuron
- **4 / [M]** Mint SNS Tokens - Mint additional tokens to an account
- **5 / [H]** Add SNS Neuron Hotkey - Add a hotkey to an SNS participant neuron (interactive)
- **6 / [I]** Increase SNS Neuron Dissolve Delay - Add dissolve delay to an SNS neuron
- **7 / [DD]** Dissolve SNS Neuron - Start or stop dissolving for an SNS neuron
- **8 / [B]** Get SNS Balance - Get SNS ledger balance for an account
- **0 / Enter** Back to Main Menu

**Utils Submenu:**

- **1 / [D]** Deploy New SNS - Create a new SNS instance (creates a separate SNS, does not replace existing)
- **2 / [R]** Rebuild Binary - Rebuild the Rust binary (useful after code changes)
- **0 / Enter** Back to Main Menu

After deploying an SNS (option 3 in the simplified menu), the menu automatically refreshes and shows the full main menu on the next iteration. You can navigate back to the main menu by pressing **0** or **Enter** in any submenu.

### Using Bash Scripts Directly

**All scripts are fully interactive** - run them without arguments and they will prompt for all required inputs. You can also provide arguments to skip prompts.

```bash
# Deploy a local SNS (fully automated)
bash scripts/deploy_local_sns.sh

# Add a hotkey to an SNS neuron (interactive - prompts for participant, neuron, hotkey, permissions)
bash scripts/add_sns_hotkey.sh
# Or with arguments:
bash scripts/add_sns_hotkey.sh <participant_principal> <hotkey_principal> [permissions]

# Add a hotkey to ICP neuron (interactive - prompts for hotkey if not provided)
bash scripts/add_icp_hotkey.sh
# Or with argument:
bash scripts/add_icp_hotkey.sh <hotkey_principal>

# Query SNS neurons (interactive - shows participant menu if no principal provided)
bash scripts/get_sns_neurons.sh
# Or with principal:
bash scripts/get_sns_neurons.sh <principal>

# List ICP neurons (interactive - shows participant menu if no principal provided)
bash scripts/get_icp_neurons.sh
# Or with principal:
bash scripts/get_icp_neurons.sh <principal>

# Get ICP neuron information (interactive - uses deployment data or prompts for neuron ID)
bash scripts/get_icp_neuron.sh
# Or with neuron ID:
bash scripts/get_icp_neuron.sh <neuron_id>

# Set ICP neuron visibility (interactive - shows menu if not provided)
bash scripts/set_icp_visibility.sh
# Or with argument:
bash scripts/set_icp_visibility.sh <true|false>

# Create ICP neuron (interactive - prompts for principal, amount, memo, dissolve delay)
bash scripts/create_icp_neuron.sh
# Or with arguments:
bash scripts/create_icp_neuron.sh <principal> <amount_e8s> [memo] [dissolve_delay_seconds]

# Disburse ICP neuron tokens (interactive - prompts for principal, neuron, receiver)
bash scripts/disburse_icp_neuron.sh
# Or with arguments:
bash scripts/disburse_icp_neuron.sh <principal> [neuron_id|receiver_principal] [receiver_principal] [amount_e8s]

# Increase ICP neuron dissolve delay (interactive - prompts for principal, neuron, delay)
bash scripts/increase_icp_dissolve_delay.sh
# Or with arguments:
bash scripts/increase_icp_dissolve_delay.sh <principal> [neuron_id] [additional_dissolve_delay_seconds]

# Manage ICP neuron dissolving state (interactive - start or stop dissolving)
bash scripts/manage_icp_dissolving.sh
# Or with arguments:
bash scripts/manage_icp_dissolving.sh <principal> [start|stop] [neuron_id]

# Mint ICP tokens (interactive - prompts for receiver and amount)
bash scripts/mint_icp.sh
# Or with arguments:
bash scripts/mint_icp.sh <receiver_principal> <amount_e8s>

# Get ICP balance (interactive - prompts for principal and optional subaccount)
bash scripts/get_icp_balance.sh
# Or with arguments:
bash scripts/get_icp_balance.sh <principal> [subaccount_hex]

# Get SNS balance (interactive - prompts for principal and optional subaccount)
bash scripts/get_sns_balance.sh
# Or with arguments:
bash scripts/get_sns_balance.sh <principal> [subaccount_hex]

# Create SNS neuron (interactive - prompts for principal, amount, memo, dissolve delay)
bash scripts/create_sns_neuron.sh
# Or with arguments:
bash scripts/create_sns_neuron.sh <principal> <amount_e8s> [memo] [dissolve_delay_seconds]

# Disburse SNS neuron tokens (interactive - prompts for participant, neuron, receiver)
bash scripts/disburse_sns_neuron.sh
# Or with arguments:
bash scripts/disburse_sns_neuron.sh <participant_principal> [neuron_id_hex|receiver_principal] [receiver_principal]

# Increase SNS neuron dissolve delay (interactive - prompts for participant, neuron, delay)
bash scripts/increase_sns_dissolve_delay.sh
# Or with arguments:
bash scripts/increase_sns_dissolve_delay.sh <principal> [neuron_id_hex] [additional_dissolve_delay_seconds]

# Manage SNS neuron dissolving state (interactive - start or stop dissolving)
bash scripts/manage_sns_dissolving.sh
# Or with arguments:
bash scripts/manage_sns_dissolving.sh <principal> [start|stop] [neuron_id_hex]

# Mint SNS tokens (interactive - prompts for proposer, receiver, amount)
bash scripts/mint_sns_tokens.sh
# Or with arguments:
bash scripts/mint_sns_tokens.sh <proposer_principal> <receiver_principal> <amount_e8s>

# Build the binary
bash scripts/build.sh
```

### Using Rust Binary Directly

All commands support interactive prompts when arguments are omitted:

```bash
# Build the binary (or use build.sh)
bash scripts/build.sh
# Or manually:
cargo build --release --bin local_sns

# Deploy SNS (fully automated)
cargo run --bin local_sns -- deploy-sns

# Add hotkey to SNS neuron (interactive)
cargo run --bin local_sns -- add-hotkey sns [participant_principal] [neuron_id_hex|hotkey_principal] [hotkey_principal|permissions] [permissions]

# Add hotkey to ICP neuron (interactive)
cargo run --bin local_sns -- add-hotkey icp [hotkey_principal]

# List SNS neurons (interactive - shows participant menu if no principal)
cargo run --bin local_sns -- list-sns-neurons [principal]

# List ICP neurons (interactive - shows participant menu if no principal)
cargo run --bin local_sns -- list-icp-neurons [principal]

# Create SNS neuron (interactive)
cargo run --bin local_sns -- create-sns-neuron [principal] [amount_e8s] [memo] [dissolve_delay_seconds]

# Create ICP neuron (interactive)
cargo run --bin local_sns -- create-icp-neuron [principal] [amount_e8s] [memo] [dissolve_delay_seconds]

# Disburse SNS neuron (interactive)
cargo run --bin local_sns -- disburse-sns-neuron [participant_principal] [neuron_id_hex|receiver_principal] [receiver_principal]

# Disburse ICP neuron (interactive)
cargo run --bin local_sns -- disburse-icp-neuron [principal] [neuron_id|receiver_principal] [receiver_principal] [amount_e8s]

# Increase SNS neuron dissolve delay (interactive)
cargo run --bin local_sns -- increase-sns-dissolve-delay [participant_principal] [neuron_id_hex] [additional_dissolve_delay_seconds]

# Increase ICP neuron dissolve delay (interactive)
cargo run --bin local_sns -- increase-icp-dissolve-delay [principal] [neuron_id] [additional_dissolve_delay_seconds]

# Manage SNS neuron dissolving state (interactive)
cargo run --bin local_sns -- manage-sns-dissolving [participant_principal] [start|stop] [neuron_id_hex]

# Manage ICP neuron dissolving state (interactive)
cargo run --bin local_sns -- manage-icp-dissolving [principal] [start|stop] [neuron_id]

# Mint SNS tokens (interactive)
cargo run --bin local_sns -- mint-sns-tokens [proposer_principal] [receiver_principal] [amount_e8s]

# Mint ICP tokens (interactive)
cargo run --bin local_sns -- mint-icp [receiver_principal] [amount_e8s]

# Set ICP neuron visibility (interactive - shows menu if not provided)
cargo run --bin local_sns -- set-icp-visibility [true|false]

# Get ICP neuron information (interactive)
cargo run --bin local_sns -- get-icp-neuron [neuron_id]

# Get ICP balance (interactive)
cargo run --bin local_sns -- get-icp-balance [principal] [subaccount_hex]

# Get SNS balance (interactive)
cargo run --bin local_sns -- get-sns-balance [principal] [subaccount_hex]

# Check if SNS is deployed
cargo run --bin local_sns -- check-sns-deployed
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
cargo run --bin local_sns -- add-hotkey <sns|icp> [owner_principal] [neuron_id_hex|hotkey_principal] [hotkey_principal|permissions] [permissions]
```

**Arguments (all optional - interactive prompts if omitted):**

- `sns|icp`: Neuron type (required)
- **For SNS neurons:**
  - `owner_principal`: Optional. Participant principal who owns the neuron. If not provided, shows participant selection menu
  - `neuron_id_hex|hotkey_principal`: Optional. Either neuron ID in hex format or hotkey principal. If not provided, shows neuron selection menu
  - `hotkey_principal`: Optional. Principal to add as hotkey. Prompts if not provided
  - `permissions`: Optional. Comma-separated permission types (default: `3,4` = SubmitProposal + Vote)
- **For ICP neurons:**
  - `hotkey_principal`: Optional. Principal to add as hotkey. Prompts if not provided
  - Note: ICP neurons don't use permission types - hotkeys have full control like the owner

### `list-sns-neurons`

List all SNS neurons owned by a principal.

**Usage:**

```bash
cargo run --bin local_sns -- list-sns-neurons [principal]
```

**Arguments:**

- `principal`: Optional. Principal to query neurons for. If not provided, shows participant selection menu.

The output displays a formatted table showing neuron ID, stake, dissolve delay, and permissions.

### `create-sns-neuron`

Create an SNS neuron by staking tokens from the SNS ledger balance.

> **Note**: You must have tokens in your ledger balance before creating a neuron. Use `mint-sns-tokens` first to mint tokens to your account.

**Usage:**

```bash
cargo run --bin local_sns -- create-sns-neuron [principal] [amount_e8s] [memo] [dissolve_delay_seconds]
```

**Arguments (all optional - interactive prompts if omitted):**

- `principal`: Optional. Principal to create the neuron for. If not provided, shows participant selection menu.
- `amount_e8s`: Optional. Amount of tokens to stake in e8s. If not provided, stakes all available balance (after deducting transfer fee).
- `memo`: Optional. Memo to use for neuron creation. If not provided, auto-generated based on neuron count (neuron_count + 1).
- `dissolve_delay_seconds`: Optional. Dissolve delay in seconds. If not provided or 0, no dissolve delay is set.

The command will:

1. Check the SNS ledger balance for the principal
2. Display available balance, transfer fee, and minimum stake requirement
3. Verify the balance meets the minimum stake requirement (fetched from governance canister)
4. Transfer tokens to the governance canister subaccount
5. Claim the neuron
6. Optionally set dissolve delay if specified

### `disburse-sns-neuron`

Disburse tokens from an SNS neuron to a receiver principal.

**Usage:**

```bash
cargo run --bin local_sns -- disburse-sns-neuron [participant_principal] [neuron_id_hex|receiver_principal] [receiver_principal]
```

**Arguments (all optional - interactive prompts if omitted):**

- `participant_principal`: Optional. Principal of the participant who owns the neuron. If not provided, shows participant selection menu.
- `neuron_id_hex`: Optional. Neuron ID in hex format. If not provided and receiver is not provided, shows neuron selection menu.
- `receiver_principal`: Optional. Principal to receive the disbursed tokens. Prompts if not provided.

The command disburses the full neuron stake to the receiver.

### `mint-sns-tokens`

Create a governance proposal to mint tokens and get all neurons to vote on it.

> **Note**: This should be done before creating neurons. You need tokens in your ledger balance to stake them into neurons.

**Usage:**

```bash
cargo run --bin local_sns -- mint-sns-tokens [proposer_principal] [receiver_principal] [amount_e8s]
```

**Arguments (all optional - interactive prompts if omitted):**

- `proposer_principal`: Optional. Principal of the participant who will create the proposal. If not provided, shows participant selection menu.
- `receiver_principal`: Optional. Principal to receive the minted tokens. Prompts if not provided.
- `amount_e8s`: Optional. Amount of tokens to mint in e8s. Prompts if not provided.

### `set-icp-visibility`

Set the visibility of the ICP neuron (public/private).

**Usage:**

```bash
cargo run --bin local_sns -- set-icp-visibility [true|false]
```

**Arguments:**

- `true|false`: Optional. Visibility setting. If not provided, shows interactive menu:
  - `[1] Public (visible to everyone)`
  - `[2] Private (only visible to controller)` (default)

Uses the ICP neuron from SNS deployment data.

### `list-icp-neurons`

List all ICP neurons owned by a principal.

**Usage:**

```bash
cargo run --bin local_sns -- list-icp-neurons [principal]
```

**Arguments:**

- `principal`: Optional. Principal to query neurons for. If not provided, shows participant selection menu.

The output displays a formatted table showing neuron ID, stake, dissolve delay, and hotkeys. You can select a neuron to view full details.

### `create-icp-neuron`

Create an ICP neuron by staking ICP tokens.

> **Note**: You must have ICP in your ledger balance before creating a neuron. Use `mint-icp` first to mint ICP to your account.

**Usage:**

```bash
cargo run --bin local_sns -- create-icp-neuron [principal] [amount_e8s] [memo] [dissolve_delay_seconds]
```

**Arguments (all optional - interactive prompts if omitted):**

- `principal`: Optional. Principal to create the neuron for. If not provided, shows participant selection menu.
- `amount_e8s`: Optional. Amount of ICP to stake in e8s. If not provided, stakes all available balance (after deducting transfer fee).
- `memo`: Optional. Memo to use for neuron creation. If not provided, auto-generated based on neuron count (neuron_count + 1).
- `dissolve_delay_seconds`: Optional. Dissolve delay in seconds. If not provided, prompts interactively.

The command will:

1. Check the ICP ledger balance for the principal
2. Display available balance and transfer fee
3. Transfer ICP to the governance canister subaccount
4. Claim the neuron
5. Optionally set dissolve delay if specified

### `disburse-icp-neuron`

Disburse tokens from an ICP neuron to a receiver account.

**Usage:**

```bash
cargo run --bin local_sns -- disburse-icp-neuron [principal] [neuron_id|receiver_principal] [receiver_principal] [amount_e8s]
```

**Arguments (all optional - interactive prompts if omitted):**

- `principal`: Optional. Principal of the owner of the neuron. If not provided, shows participant selection menu.
- `neuron_id`: Optional. Neuron ID (number). If not provided and receiver is not provided, shows neuron selection menu.
- `receiver_principal`: Optional. Principal to receive the disbursed tokens. Prompts if not provided.
- `amount_e8s`: Optional. Amount to disburse in e8s. If not provided, full disbursement.

### `increase-icp-dissolve-delay`

Increase the dissolve delay for an ICP neuron by adding additional seconds.

**Usage:**

```bash
cargo run --bin local_sns -- increase-icp-dissolve-delay [principal] [neuron_id] [additional_dissolve_delay_seconds]
```

**Arguments (all optional - interactive prompts if omitted):**

- `principal`: Optional. Principal of the owner of the neuron. If not provided, shows participant selection menu.
- `neuron_id`: Optional. Neuron ID (number). If not provided, shows neuron selection menu.
- `additional_dissolve_delay_seconds`: Optional. Additional dissolve delay in seconds to add. If not provided, prompts interactively.

The command will show available neurons and allow you to select which one to modify.

### `manage-icp-dissolving`

Start or stop dissolving for an ICP neuron.

**Usage:**

```bash
cargo run --bin local_sns -- manage-icp-dissolving [principal] [start|stop] [neuron_id]
```

**Arguments (all optional - interactive prompts if omitted):**

- `principal`: Optional. Principal of the owner of the neuron. If not provided, shows participant selection menu.
- `start|stop`: Optional. Action to perform. If not provided, shows interactive menu:
  - `[1] Start Dissolving`
  - `[2] Stop Dissolving`
- `neuron_id`: Optional. Neuron ID (number). If not provided, shows neuron selection menu.

### `mint-icp`

Mint ICP tokens from the minting account to a receiver.

**Usage:**

```bash
cargo run --bin local_sns -- mint-icp [receiver_principal] [amount_e8s]
```

**Arguments (all optional - interactive prompts if omitted):**

- `receiver_principal`: Optional. Principal to receive the minted ICP. Prompts if not provided.
- `amount_e8s`: Optional. Amount of ICP to mint in e8s. Prompts if not provided.

The command will display the available balance in the minting account.

### `get-icp-balance`

Get the ICP ledger balance for an account.

**Usage:**

```bash
cargo run --bin local_sns -- get-icp-balance [principal] [subaccount_hex]
```

**Arguments:**

- `principal`: Optional. Principal to query balance for. If not provided, shows participant selection menu or prompts.
- `subaccount_hex`: Optional. Subaccount in hex format. If not provided, uses default account.

### `get-sns-balance`

Get the SNS ledger balance for an account.

**Usage:**

```bash
cargo run --bin local_sns -- get-sns-balance [principal] [subaccount_hex]
```

**Arguments:**

- `principal`: Optional. Principal to query balance for. If not provided, shows participant selection menu or prompts.
- `subaccount_hex`: Optional. Subaccount in hex format. If not provided, uses default account.

### `get-icp-neuron`

Get full information about an ICP neuron.

**Usage:**

```bash
cargo run --bin local_sns -- get-icp-neuron [neuron_id]
```

**Arguments:**

- `neuron_id`: Optional. Specific neuron ID to query. If not provided:
  - Uses neuron ID from deployment data if available
  - Otherwise prompts for neuron ID

Returns full neuron information as JSON.

### `increase-sns-dissolve-delay`

Increase the dissolve delay for an SNS neuron by adding additional seconds.

**Usage:**

```bash
cargo run --bin local_sns -- increase-sns-dissolve-delay [participant_principal] [neuron_id_hex] [additional_dissolve_delay_seconds]
```

**Arguments (all optional - interactive prompts if omitted):**

- `participant_principal`: Optional. Principal of the participant who owns the neuron. If not provided, shows participant selection menu.
- `neuron_id_hex`: Optional. Neuron ID in hex format. If not provided, shows neuron selection menu.
- `additional_dissolve_delay_seconds`: Optional. Additional dissolve delay in seconds to add. If not provided, prompts interactively.

The command will show available neurons and allow you to select which one to modify.

### `manage-sns-dissolving`

Start or stop dissolving for an SNS neuron.

**Usage:**

```bash
cargo run --bin local_sns -- manage-sns-dissolving [participant_principal] [start|stop] [neuron_id_hex]
```

**Arguments (all optional - interactive prompts if omitted):**

- `participant_principal`: Optional. Principal of the participant who owns the neuron. If not provided, shows participant selection menu.
- `start|stop`: Optional. Action to perform. If not provided, shows interactive menu:
  - `[1] Start Dissolving`
  - `[2] Stop Dissolving`
- `neuron_id_hex`: Optional. Neuron ID in hex format. If not provided, shows neuron selection menu.

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

The binary is automatically built when you run `start.sh` for the first time (if it doesn't exist). You can also build it manually:

```bash
# Build using the build script (tries release, falls back to dev)
bash scripts/build.sh

# Or build directly with cargo
cargo build --bin local_sns              # Debug build
cargo build --release --bin local_sns    # Release build
```

> **Note**: Individual scripts no longer rebuild the binary. The binary is built automatically on first run via `start.sh`, or you can run `build.sh` manually before running individual scripts. The menu also provides a "Rebuild Binary" option for rebuilding after code changes.

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

All scripts are located in the `scripts/` directory. **All operation scripts are fully interactive** - run them without arguments and they will guide you through the process:

- **`start.sh`** - Interactive menu for all operations (recommended entry point)

  - Automatically checks for SNS deployment status
  - **When no SNS is deployed**: Shows simplified menu with 3 options (Mint ICP, Get ICP Balance, Deploy SNS)
  - **When SNS is deployed**: Shows full hierarchical menu with ICP/SNS/Utils categories
  - Menu automatically refreshes after deploying an SNS to show full options

- **`build.sh`** - Build the local_sns Rust binary (tries release, falls back to dev)

- **`deploy_local_sns.sh`** - Full SNS deployment workflow (fully automated)

- **`add_sns_hotkey.sh`** - Add hotkey to SNS neuron (interactive)

  - Prompts for participant, neuron, hotkey, and permissions if not provided

- **`add_icp_hotkey.sh`** - Add hotkey to ICP neuron (interactive)

  - Prompts for hotkey principal if not provided

- **`get_sns_neurons.sh`** - List all SNS neurons for a principal (interactive)

  - Shows participant selection menu if principal not provided

- **`get_icp_neurons.sh`** - List all ICP neurons for a principal (interactive)

  - Shows participant selection menu if principal not provided
  - Displays table with neuron details and allows selecting a neuron for full details

- **`get_icp_neuron.sh`** - Get detailed ICP neuron information (interactive)

  - Uses deployment data or prompts for neuron ID

- **`set_icp_visibility.sh`** - Set ICP neuron visibility (interactive)

  - Shows menu (public/private) if not provided

- **`create_icp_neuron.sh`** - Create an ICP neuron by staking ICP (interactive)

  - Prompts for principal, amount, memo, and dissolve delay
  - Displays available balance and transfer fee
  - Auto-generates memo based on neuron count
  - **Note**: Requires ICP in ledger balance (mint ICP first using `mint_icp.sh`)

- **`disburse_icp_neuron.sh`** - Disburse tokens from ICP neuron (interactive)

  - Prompts for principal, neuron, receiver, and optional amount if not provided
  - Shows neuron selection menu if neuron ID not specified

- **`increase_icp_dissolve_delay.sh`** - Increase dissolve delay for ICP neuron (interactive)

  - Prompts for principal, neuron selection, and additional dissolve delay if not provided
  - Shows neuron selection menu if neuron ID not specified

- **`manage_icp_dissolving.sh`** - Start or stop dissolving for ICP neuron (interactive)

  - Prompts for principal, action (start/stop), and neuron selection if not provided
  - Shows action menu and neuron selection menu

- **`mint_icp.sh`** - Mint ICP tokens (interactive)

  - Prompts for receiver and amount if not provided
  - Displays available balance in minting account

- **`get_icp_balance.sh`** - Get ICP ledger balance (interactive)

  - Prompts for principal and optional subaccount if not provided

- **`get_sns_balance.sh`** - Get SNS ledger balance (interactive)

  - Prompts for principal and optional subaccount if not provided

- **`mint_sns_tokens.sh`** - Mint tokens via governance proposal (interactive)

  - Prompts for proposer, receiver, and amount if not provided
  - **Important**: Mint tokens before creating neurons - you need tokens in your ledger balance to stake them

- **`create_sns_neuron.sh`** - Create an SNS neuron by staking tokens (interactive)

  - Prompts for principal, amount, memo, and dissolve delay
  - Displays available balance, transfer fee, and minimum stake
  - Auto-generates memo based on neuron count
  - **Note**: Requires tokens in ledger balance (mint tokens first using `mint_sns_tokens.sh`)

- **`disburse_sns_neuron.sh`** - Disburse tokens from SNS neuron (interactive)

  - Prompts for participant, neuron, and receiver if not provided

- **`increase_sns_dissolve_delay.sh`** - Increase dissolve delay for SNS neuron (interactive)

  - Prompts for participant, neuron selection, and additional dissolve delay if not provided
  - Shows neuron selection menu if neuron ID not specified

- **`manage_sns_dissolving.sh`** - Start or stop dissolving for SNS neuron (interactive)
  - Prompts for participant, action (start/stop), and neuron selection if not provided
  - Shows action menu and neuron selection menu

All scripts can be run with arguments to skip prompts, or without arguments for full interactivity.

## Copying to Another Repository

This entire `local_sns/` directory is self-contained. To use it in another project:

1. Copy the entire `local_sns/` directory to your repository
2. Update paths if necessary (all paths are relative to the `local_sns/` root)
3. Run `cargo build` to verify it compiles
4. Ensure `dfx` is configured with system canisters

All generated files will be created in the `local_sns/generated/` directory.

## License

Same as the parent project.
