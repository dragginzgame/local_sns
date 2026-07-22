// Identity loading and agent creation

use anyhow::{Context, Result};
use ic_agent::{Agent, Identity};
use k256::pkcs8::DecodePrivateKey;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration as StdDuration;

// Minting account PEM (from prepare_sns_deploy.sh)
const MINTING_PEM: &str = r#"-----BEGIN EC PRIVATE KEY-----
MHQCAQEEICJxApEbuZznKFpV+VKACRK30i6+7u5Z13/DOl18cIC+oAcGBSuBBAAK
oUQDQgAEPas6Iag4TUx+Uop+3NhE6s3FlayFtbwdhRVjvOar0kPTfE/N8N6btRnd
74ly5xXEBNSXiENyxhEuzOZrIWMCNQ==
-----END EC PRIVATE KEY-----"#;

fn run_icp_command(args: &[&str]) -> Result<String> {
    let mut command = Command::new("icp");

    if let Ok(password_file) = std::env::var("ICP_IDENTITY_PASSWORD_FILE") {
        command.arg("--identity-password-file").arg(password_file);
    }

    let output = command
        .args(args)
        .output()
        .context("Failed to run icp-cli. Make sure `icp` is installed and on PATH")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("`icp {}` failed: {}", args.join(" "), stderr.trim());
    }

    String::from_utf8(output.stdout).context("icp-cli output was not valid UTF-8")
}

fn parse_pem_identity(pem_content: &str, source: &str) -> Result<Box<dyn Identity>> {
    // SEC1 format: `-----BEGIN EC PRIVATE KEY-----`
    if let Ok(identity) = ic_agent::identity::Secp256k1Identity::from_pem(pem_content) {
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    // PKCS#8 format: `-----BEGIN PRIVATE KEY-----`. This is what `icp identity export`
    // emits for secp256k1 keys, which `Secp256k1Identity::from_pem` does not accept.
    if let Ok(secret_key) = k256::SecretKey::from_pkcs8_pem(pem_content) {
        let identity = ic_agent::identity::Secp256k1Identity::from_private_key(secret_key);
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    // Ed25519 (BasicIdentity handles both SEC1-style and PKCS#8 ed25519 keys).
    if let Ok(identity) = ic_agent::identity::BasicIdentity::from_pem(pem_content) {
        return Ok(Box::new(identity) as Box<dyn Identity>);
    }

    anyhow::bail!("Failed to load {source}: could not parse as Secp256k1 or Ed25519")
}

fn default_icp_identity_name() -> Result<String> {
    if let Ok(identity_name) = std::env::var("LOCAL_SNS_ICP_IDENTITY")
        && !identity_name.trim().is_empty()
    {
        return Ok(identity_name);
    }

    if let Ok(identity_name) = std::env::var("ICP_IDENTITY")
        && !identity_name.trim().is_empty()
    {
        return Ok(identity_name);
    }

    let name = run_icp_command(&["identity", "default"])
        .context("Failed to get the default icp-cli identity")?
        .trim()
        .to_string();

    if name.is_empty() {
        anyhow::bail!("icp-cli did not return a default identity");
    }

    Ok(name)
}

/// Load an icp-cli identity.
///
/// If `identity_name` is not provided, `LOCAL_SNS_ICP_IDENTITY`, `ICP_IDENTITY`,
/// and then `icp identity default` are checked in that order.
pub fn load_icp_identity(identity_name: Option<&str>) -> Result<Box<dyn Identity>> {
    let name = match identity_name {
        Some(name) => name.to_string(),
        None => default_icp_identity_name()?,
    };

    if name == "anonymous" {
        return Ok(Box::new(ic_agent::identity::AnonymousIdentity) as Box<dyn Identity>);
    }

    let pem_content = run_icp_command(&["identity", "export", &name])
        .with_context(|| format!("Failed to export icp-cli identity `{name}`"))?;

    parse_pem_identity(&pem_content, &format!("icp-cli identity `{name}`"))
}

/// Load minting identity from PEM string
pub fn load_minting_identity() -> Result<Box<dyn Identity>> {
    parse_pem_identity(MINTING_PEM, "minting identity")
}

/// Get icp-cli replica URL from configuration or environment
/// Checks in order:
/// 1. LOCAL_SNS_REPLICA_URL environment variable
/// 2. ICP_REPLICA_URL environment variable
/// 3. `icp network status --json`
/// 4. Default: http://127.0.0.1:8000
fn get_icp_replica_url() -> String {
    if let Ok(url) = std::env::var("LOCAL_SNS_REPLICA_URL") {
        return url;
    }

    if let Ok(url) = std::env::var("ICP_REPLICA_URL") {
        return url;
    }

    if let Ok(status) = run_icp_command(&["network", "status", "--json"])
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&status)
    {
        for key in ["api_url", "api-url", "gateway_url", "gateway-url", "url"] {
            if let Some(url) = json.get(key).and_then(|value| value.as_str()) {
                return url.to_string();
            }
        }
    }

    "http://127.0.0.1:8000".to_string()
}

/// Create agent with identity
pub async fn create_agent(identity: Box<dyn Identity>) -> Result<Agent> {
    let url = get_icp_replica_url();
    let agent = Agent::builder()
        .with_url(&url)
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
