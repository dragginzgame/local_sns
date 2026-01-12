// Identity loading and agent creation

use anyhow::{Context, Result};
use ic_agent::{Agent, Identity};
use std::path::PathBuf;
use std::time::Duration as StdDuration;

// Minting account PEM (from prepare_sns_deploy.sh)
const MINTING_PEM: &str = r#"-----BEGIN EC PRIVATE KEY-----
MHQCAQEEICJxApEbuZznKFpV+VKACRK30i6+7u5Z13/DOl18cIC+oAcGBSuBBAAK
oUQDQgAEPas6Iag4TUx+Uop+3NhE6s3FlayFtbwdhRVjvOar0kPTfE/N8N6btRnd
74ly5xXEBNSXiENyxhEuzOZrIWMCNQ==
-----END EC PRIVATE KEY-----"#;

/// Load dfx identity from default location
/// Tries both Secp256k1 and Ed25519 formats
pub fn load_dfx_identity(identity_name: Option<&str>) -> Result<Box<dyn Identity>> {
    let name = identity_name.unwrap_or("default");
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let identity_path = PathBuf::from(home)
        .join(".config/dfx/identity")
        .join(name)
        .join("identity.pem");

    if !identity_path.exists() {
        anyhow::bail!("Identity not found at: {}", identity_path.display());
    }

    let pem_content = std::fs::read_to_string(&identity_path)
        .with_context(|| format!("Failed to read identity file: {}", identity_path.display()))?;

    // Try Secp256k1 first (older dfx format)
    if let Ok(identity) = ic_agent::identity::Secp256k1Identity::from_pem(&pem_content) {
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    // Try Ed25519 (newer dfx format)
    if let Ok(identity) = ic_agent::identity::BasicIdentity::from_pem(&pem_content) {
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    anyhow::bail!("Failed to load identity: could not parse as Secp256k1 or Ed25519")
}

/// Load minting identity from PEM string
pub fn load_minting_identity() -> Result<Box<dyn Identity>> {
    // Try Secp256k1 first
    if let Ok(identity) = ic_agent::identity::Secp256k1Identity::from_pem(MINTING_PEM) {
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    // Try Ed25519
    if let Ok(identity) = ic_agent::identity::BasicIdentity::from_pem(MINTING_PEM) {
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    anyhow::bail!("Failed to load minting identity: could not parse as Secp256k1 or Ed25519")
}

/// Create agent with identity
pub async fn create_agent(identity: Box<dyn Identity>) -> Result<Agent> {
    let url = "http://127.0.0.1:8080";
    let agent = Agent::builder()
        .with_url(url)
        .with_ingress_expiry(StdDuration::from_secs(300))
        .with_identity(identity)
        .build()?;

    agent.fetch_root_key().await?;
    Ok(agent)
}

/// Save seed to file (for deterministic identity regeneration)
pub fn save_seed_to_file(seed: &[u8; 32], path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Save as hex string for readability
    let hex_seed = hex::encode(seed);
    std::fs::write(path, hex_seed)
        .with_context(|| format!("Failed to write seed file: {}", path.display()))?;
    Ok(())
}

/// Load identity from seed file
pub fn load_identity_from_seed_file(path: &PathBuf) -> Result<Box<dyn Identity>> {
    let hex_content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read seed file: {}", path.display()))?;

    let seed_bytes = hex::decode(hex_content.trim()).context("Failed to decode hex seed")?;

    if seed_bytes.len() != 32 {
        anyhow::bail!("Seed file must contain exactly 32 bytes (64 hex characters)");
    }

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_bytes);

    let identity = ic_agent::identity::BasicIdentity::from_raw_key(&seed);
    Ok(Box::new(identity) as Box<dyn Identity>)
}
