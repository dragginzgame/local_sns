// SNS configuration for deployment
// Modify this file to customize SNS parameters

use base64::{Engine as _, engine::general_purpose};
use candid::Principal;

use crate::core::declarations::icp_governance::{
    Countries, CreateServiceNervousSystem, DeveloperDistribution, Duration, GovernanceParameters,
    Image, InitialTokenDistribution, LedgerParameters, NeuronBasketConstructionParameters,
    NeuronDistribution, Percentage, SwapDistribution, SwapParameters, Tokens,
    VotingRewardParameters,
};

/// Name of the PNG logo file in the src directory
/// Set this to the filename of your logo (e.g., "logo.png")
pub const LOGO_FILENAME: &str = "logo.png";

/// Default logo as base64 (fallback if logo.png is not found)
#[allow(dead_code)]
pub const DEFAULT_LOGO_BASE64: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAFJElEQVR4nG2WT4slZxXGf8+puvd298z0ZDLp9BCTjAlGBnElCAoDLty6cCTgwm8gfoCAe79CNu7cKAiShW6yUBAkKEgEEYOQsZ1BkkxPt9N/771VdR4Xb1Xd23On4HKr3fe9589zznnOq//+8K4YnuDxw+bkuJMAbHZvVW/cnRjAlXmiyQ9uv31YRQBm6Xj/zqMHN48XDkA1zz7xkz9mVL2+hFoYKDaEimqBKT+QleXVoyfjq2RpJSohW4AE";

/// Load PNG image from init directory and convert to base64 data URI
/// Returns the base64-encoded image with data URI prefix, or falls back to default if file not found
fn load_logo_base64() -> String {
    // Get the project root directory (where Cargo.toml is)
    if let Ok(current_dir) = std::env::current_dir() {
        // Look for logo.png in the src/init/ directory
        let mut logo_path = current_dir;
        logo_path.push("src");
        logo_path.push("init");
        logo_path.push(LOGO_FILENAME);

        if logo_path.exists() {
            match std::fs::read(&logo_path) {
                Ok(image_bytes) => {
                    let base64_encoded = general_purpose::STANDARD.encode(&image_bytes);
                    return format!("data:image/png;base64,{}", base64_encoded);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read src/init/{}: {}. Using default logo.",
                        LOGO_FILENAME, e
                    );
                }
            }
        } else {
            eprintln!(
                "Info: src/init/{} not found. Using default logo.",
                LOGO_FILENAME
            );
        }
    }

    // Fallback to default
    DEFAULT_LOGO_BASE64.to_string()
}

/// Build SNS configuration
///
/// This function constructs the `CreateServiceNervousSystem` struct with all
/// the initial parameters for the SNS deployment. Modify the values below to
/// customize your SNS configuration.
pub fn build_sns_config(owner_principal: Principal) -> CreateServiceNervousSystem {
    // ============================================================================
    // BASIC SNS INFORMATION
    // ============================================================================
    let sns_name = "AcmeDAO";
    let sns_description = "AcmeDAO is a decentralized autonomous organization built on the Internet Computer Protocol. It enables community governance, token distribution, and collaborative decision-making through transparent voting mechanisms and smart contract automation.";
    let sns_url = "https://acmedao.io";

    // Load logo from PNG file in project root, or use default if not found
    let logo_base64 = load_logo_base64();

    // ============================================================================
    // LEDGER PARAMETERS
    // ============================================================================
    let transaction_fee_e8s = 10_000; // 0.0001 tokens
    let token_symbol = "ACME";
    let token_name = "Acme Token";

    // ============================================================================
    // GOVERNANCE PARAMETERS
    // ============================================================================
    let neuron_maximum_dissolve_delay_bonus_bp = 10_000; // 100% (basis points)
    let neuron_maximum_age_bonus_bp = 0; // 0% (basis points)
    let neuron_minimum_stake_e8s = 10_000_000; // 0.1 tokens
    let neuron_maximum_age_for_age_bonus_seconds = 4 * 365 * 24 * 60 * 60; // 4 years
    let neuron_maximum_dissolve_delay_seconds = 8 * 365 * 24 * 60 * 60; // 8 years
    let neuron_minimum_dissolve_delay_to_vote_seconds = 30 * 24 * 60 * 60; // 30 days
    let proposal_initial_voting_period_seconds = 4 * 24 * 60 * 60; // 4 days
    let proposal_wait_for_quiet_deadline_increase_seconds = 24 * 60 * 60; // 1 day
    let proposal_rejection_fee_e8s = 11_000_000; // 0.11 tokens

    // Voting reward parameters
    let initial_reward_rate_bp = 0; // 0% (basis points)
    let final_reward_rate_bp = 0; // 0% (basis points)
    let reward_rate_transition_duration_seconds = 0; // 0 seconds

    // ============================================================================
    // SWAP PARAMETERS
    // ============================================================================
    let minimum_participants = 5;
    let neurons_fund_participation = false;
    let minimum_direct_participation_icp_e8s = 100_000_000 * 5; // 5 ICP
    let maximum_direct_participation_icp_e8s = 1_000_000_000 * 5; // 50 ICP
    let minimum_participant_icp_e8s = 100_000_000; // 1 ICP
    let maximum_participant_icp_e8s = 1_000_000_000; // 10 ICP
    let swap_duration_seconds = 7 * 24 * 60 * 60; // 7 days

    // Neuron basket construction parameters
    let neuron_basket_count = 3;
    let neuron_basket_dissolve_delay_interval_seconds = 30 * 24 * 60 * 60; // 30 days

    // Restricted countries (ISO codes)
    let restricted_countries = vec!["AQ".to_string()]; // Antarctica (placeholder)

    // ============================================================================
    // INITIAL TOKEN DISTRIBUTION
    // ============================================================================
    // Treasury distribution (tokens held by the treasury)
    let treasury_distribution_e8s = 1_000_000_000; // 10 tokens

    // Developer distribution (tokens allocated to developers)
    let developer_neuron_stake_e8s = 100_000_000; // 1 token
    let developer_neuron_dissolve_delay_seconds = 2 * 365 * 24 * 60 * 60; // 2 years
    let developer_neuron_vesting_period_seconds = 4 * 365 * 24 * 60 * 60; // 4 years

    // Swap distribution (tokens available in the swap)
    let swap_distribution_e8s = 2_000_000_000; // 20 tokens

    // ============================================================================
    // BUILD CONFIGURATION
    // ============================================================================
    CreateServiceNervousSystem {
        name: Some(sns_name.to_string()),
        description: Some(sns_description.to_string()),
        url: Some(sns_url.to_string()),
        logo: Some(Image {
            base64_encoding: Some(logo_base64.to_string()),
        }),
        fallback_controller_principal_ids: vec![owner_principal],
        dapp_canisters: vec![],
        ledger_parameters: Some(LedgerParameters {
            transaction_fee: Some(Tokens {
                e8s: Some(transaction_fee_e8s),
            }),
            token_symbol: Some(token_symbol.to_string()),
            token_logo: Some(Image {
                base64_encoding: Some(logo_base64.to_string()),
            }),
            token_name: Some(token_name.to_string()),
        }),
        governance_parameters: Some(GovernanceParameters {
            neuron_maximum_dissolve_delay_bonus: Some(Percentage {
                basis_points: Some(neuron_maximum_dissolve_delay_bonus_bp),
            }),
            neuron_maximum_age_bonus: Some(Percentage {
                basis_points: Some(neuron_maximum_age_bonus_bp),
            }),
            neuron_minimum_stake: Some(Tokens {
                e8s: Some(neuron_minimum_stake_e8s),
            }),
            neuron_maximum_age_for_age_bonus: Some(Duration {
                seconds: Some(neuron_maximum_age_for_age_bonus_seconds),
            }),
            neuron_maximum_dissolve_delay: Some(Duration {
                seconds: Some(neuron_maximum_dissolve_delay_seconds),
            }),
            neuron_minimum_dissolve_delay_to_vote: Some(Duration {
                seconds: Some(neuron_minimum_dissolve_delay_to_vote_seconds),
            }),
            proposal_initial_voting_period: Some(Duration {
                seconds: Some(proposal_initial_voting_period_seconds),
            }),
            proposal_wait_for_quiet_deadline_increase: Some(Duration {
                seconds: Some(proposal_wait_for_quiet_deadline_increase_seconds),
            }),
            proposal_rejection_fee: Some(Tokens {
                e8s: Some(proposal_rejection_fee_e8s),
            }),
            voting_reward_parameters: Some(VotingRewardParameters {
                initial_reward_rate: Some(Percentage {
                    basis_points: Some(initial_reward_rate_bp),
                }),
                final_reward_rate: Some(Percentage {
                    basis_points: Some(final_reward_rate_bp),
                }),
                reward_rate_transition_duration: Some(Duration {
                    seconds: Some(reward_rate_transition_duration_seconds),
                }),
            }),
        }),
        swap_parameters: Some(SwapParameters {
            minimum_participants: Some(minimum_participants),
            neurons_fund_participation: Some(neurons_fund_participation),
            minimum_direct_participation_icp: Some(Tokens {
                e8s: Some(minimum_direct_participation_icp_e8s),
            }),
            maximum_direct_participation_icp: Some(Tokens {
                e8s: Some(maximum_direct_participation_icp_e8s),
            }),
            minimum_participant_icp: Some(Tokens {
                e8s: Some(minimum_participant_icp_e8s),
            }),
            maximum_participant_icp: Some(Tokens {
                e8s: Some(maximum_participant_icp_e8s),
            }),
            confirmation_text: None,
            minimum_icp: None,
            maximum_icp: None,
            neurons_fund_investment_icp: None,
            restricted_countries: Some(Countries {
                iso_codes: restricted_countries,
            }),
            start_time: None,
            duration: Some(Duration {
                seconds: Some(swap_duration_seconds),
            }),
            neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
                count: Some(neuron_basket_count),
                dissolve_delay_interval: Some(Duration {
                    seconds: Some(neuron_basket_dissolve_delay_interval_seconds),
                }),
            }),
        }),
        initial_token_distribution: Some(InitialTokenDistribution {
            treasury_distribution: Some(SwapDistribution {
                total: Some(Tokens {
                    e8s: Some(treasury_distribution_e8s),
                }),
            }),
            developer_distribution: Some(DeveloperDistribution {
                developer_neurons: vec![NeuronDistribution {
                    controller: Some(owner_principal),
                    dissolve_delay: Some(Duration {
                        seconds: Some(developer_neuron_dissolve_delay_seconds),
                    }),
                    memo: Some(0),
                    vesting_period: Some(Duration {
                        seconds: Some(developer_neuron_vesting_period_seconds),
                    }),
                    stake: Some(Tokens {
                        e8s: Some(developer_neuron_stake_e8s),
                    }),
                }],
            }),
            swap_distribution: Some(SwapDistribution {
                total: Some(Tokens {
                    e8s: Some(swap_distribution_e8s),
                }),
            }),
        }),
    }
}

/// Get default proposal title
pub fn default_proposal_title() -> String {
    "Deploy AcmeDAO SNS".to_string()
}

/// Get default proposal summary
pub fn default_proposal_summary() -> String {
    "This proposal creates a new Service Nervous System (SNS) for AcmeDAO with configured governance parameters, token distribution, and swap mechanics.".to_string()
}
