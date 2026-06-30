//! zkVerify integration — submits proofs to zkVerify via the Kurier REST API
//! for on-chain verification.
//!
//! zkVerify is a high-performance blockchain for ZK proof verification.
//! Kurier is its REST API: https://docs.zkverify.io/overview/getting-started/kurier
//!
//! Flow:
//! 1. Register the verification key (done once, cached)
//! 2. Submit proof via POST /submit-proof
//! 3. Poll job status until verified
//!
//! If zkVerify is unavailable (no API key configured), we fall back to
//! local snarkjs verification so the demo still works.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::VerifyResponse;

/// Kurier API base URL (testnet by default).
const KURIER_TESTNET: &str = "https://testnet.kurier.xyz/api/v1";

#[derive(Debug, Clone)]
pub struct ZkVerifyConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    /// Path to the verification key JSON file.
    pub vk_path: String,
    /// Whether zkVerify is configured (API key present).
    pub enabled: bool,
}

impl Default for ZkVerifyConfig {
    fn default() -> Self {
        let api_key = std::env::var("ZKVERIFY_API_KEY").ok();
        Self {
            enabled: api_key.is_some(),
            api_key,
            base_url: std::env::var("ZKVERIFY_URL")
                .unwrap_or_else(|_| KURIER_TESTNET.to_string()),
            vk_path: "build/verification_key.json".to_string(),
        }
    }
}

/// Kurier register-vk response.
#[derive(Debug, Deserialize)]
struct RegisterVkResponse {
    vk_hash: String,
}

/// Kurier submit-proof response.
#[derive(Debug, Deserialize)]
struct SubmitProofResponse {
    job_id: String,
    #[serde(default)]
    tx_hash: Option<String>,
}

/// Kurier job status response.
#[derive(Debug, Deserialize)]
struct JobStatusResponse {
    status: String,
    #[serde(default)]
    verified: Option<bool>,
    #[serde(default)]
    tx_hash: Option<String>,
    #[serde(default)]
    statement_hash: Option<String>,
}

/// Cached VK hash after registration.
static VK_HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Submit a proof to zkVerify for on-chain verification.
///
/// If zkVerify is not configured (no API key), falls back to local
/// snarkjs verification.
pub async fn verify_proof(
    config: &ZkVerifyConfig,
    proof: &serde_json::Value,
    public_signals: &[String],
) -> Result<VerifyResponse> {
    if !config.enabled {
        // Fallback: local snarkjs verification
        tracing::info!("zkVerify not configured, falling back to local verification");
        return local_verify(proof, public_signals).await;
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // 1. Register VK (cached)
    let vk_hash = get_or_register_vk(config, &client).await?;

    // 2. Submit proof
    let submit_body = serde_json::json!({
        "proofType": "groth16",
        "vkRegistered": true,
        "proofOptions": {
            "curve": "bn128"
        },
        "proofData": {
            "proof": proof,
            "publicSignals": public_signals,
            "vkHash": vk_hash
        }
    });

    let submit_resp = client
        .post(format!("{}/submit-proof", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key.as_ref().unwrap()))
        .header("Content-Type", "application/json")
        .json(&submit_body)
        .send()
        .await?;

    if !submit_resp.status().is_success() {
        let status = submit_resp.status();
        let body = submit_resp.text().await.unwrap_or_default();
        tracing::warn!("zkVerify submit failed: {} {}", status, body);
        return local_verify(proof, public_signals).await;
    }

    let submit_result: SubmitProofResponse = submit_resp.json().await?;
    let job_id = submit_result.job_id;

    // 3. Poll job status
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_secs(2)).await;

        let status_resp = client
            .get(format!("{}/job-status/{}", config.base_url, job_id))
            .header("Authorization", format!("Bearer {}", config.api_key.as_ref().unwrap()))
            .send()
            .await?;

        if !status_resp.status().is_success() {
            continue;
        }

        let status: JobStatusResponse = status_resp.json().await?;

        match status.status.as_str() {
            "Verified" | "Finalized" => {
                return Ok(VerifyResponse {
                    verified: true,
                    zkverify_tx_hash: status.tx_hash.or(submit_result.tx_hash),
                    statement_hash: status.statement_hash,
                    verification_method: "zkverify".to_string(),
                });
            }
            "Failed" | "Rejected" => {
                return Ok(VerifyResponse {
                    verified: false,
                    zkverify_tx_hash: None,
                    statement_hash: None,
                    verification_method: "zkverify".to_string(),
                });
            }
            _ => continue,
        }
    }

    // Timeout — fall back to local
    tracing::warn!("zkVerify polling timed out, falling back to local verification");
    local_verify(proof, public_signals).await
}

/// Get or register the verification key with zkVerify.
async fn get_or_register_vk(
    config: &ZkVerifyConfig,
    client: &reqwest::Client,
) -> Result<String> {
    if let Some(hash) = VK_HASH.get() {
        return Ok(hash.clone());
    }

    let vk_content = std::fs::read_to_string(&config.vk_path)?;
    let vk: serde_json::Value = serde_json::from_str(&vk_content)?;

    let body = serde_json::json!({
        "proofType": "groth16",
        "vk": vk,
        "proofOptions": {
            "curve": "bn128"
        }
    });

    let resp = client
        .post(format!("{}/register-vk", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key.as_ref().unwrap()))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("zkVerify register-vk failed: {} {}", status, body));
    }

    let result: RegisterVkResponse = resp.json().await?;
    let _ = VK_HASH.set(result.vk_hash.clone());
    Ok(result.vk_hash)
}

/// Fallback: local snarkjs verification.
async fn local_verify(proof: &serde_json::Value, public_signals: &[String]) -> Result<VerifyResponse> {
    let tmp = std::env::temp_dir().join(format!("zk-age-verify-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp)?;

    let proof_path = tmp.join("proof.json");
    let public_path = tmp.join("public.json");

    std::fs::write(&proof_path, serde_json::to_string(proof)?)?;
    std::fs::write(
        &public_path,
        serde_json::to_string(&public_signals.iter().collect::<Vec<_>>())?,
    )?;

    // Find VK
    let vk_path = find_vk()?;

    let output = tokio::process::Command::new("snarkjs")
        .arg("groth16")
        .arg("verify")
        .arg(&vk_path)
        .arg(&public_path)
        .arg(&proof_path)
        .output()
        .await?;

    let _ = std::fs::remove_dir_all(&tmp);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let verified = stdout.contains("OK!");

    Ok(VerifyResponse {
        verified,
        zkverify_tx_hash: None,
        statement_hash: None,
        verification_method: "local-snarkjs".to_string(),
    })
}

fn find_vk() -> Result<std::path::PathBuf> {
    let mut cwd = std::env::current_dir()?;
    for _ in 0..5 {
        let candidate = cwd.join("build/verification_key.json");
        if candidate.exists() {
            return Ok(candidate);
        }
        if !cwd.pop() {
            break;
        }
    }
    Err(anyhow::anyhow!("verification_key.json not found"))
}
