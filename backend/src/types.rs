//! Shared types for the zk-age API.

use serde::{Deserialize, Serialize};

/// Request to issue a signed birthdate commitment (simulated ID authority).
#[derive(Debug, Deserialize)]
pub struct IssueRequest {
    /// The user's birth year (e.g., 2000).
    pub birth_year: u64,
}

/// Response from the issuer — contains the signed commitment the user
/// needs to generate a ZK proof.
#[derive(Debug, Serialize)]
pub struct IssueResponse {
    /// The issuer's public key hash (public input to the circuit).
    pub issuer_pubkey_hash: String,
    /// The issuer's signature on (birth_year, pubkey_hash, randomness).
    /// In production this would be a Poseidon/EdDSA signature.
    pub issuer_signature: String,
    /// Randomness used in the signature (private input).
    pub signature_randomness: String,
    /// The birth year (private input — user keeps this, only used locally).
    pub birth_year: u64,
}

/// Request to generate a ZK proof of age.
#[derive(Debug, Deserialize)]
pub struct ProveRequest {
    pub birth_year: u64,
    pub issuer_pubkey_hash: String,
    pub issuer_signature: String,
    pub signature_randomness: String,
    /// Age threshold to prove (e.g., 18, 21).
    pub threshold: u64,
}

/// Response containing the generated proof and public inputs.
#[derive(Debug, Serialize)]
pub struct ProveResponse {
    pub proof: serde_json::Value,
    pub public_signals: Vec<String>,
    /// Statement hash for zkVerify tracking.
    pub proof_id: String,
    /// Proof generation latency in milliseconds.
    pub proof_latency_ms: u64,
    /// Attestation energy score (FMD physics model).
    pub energy: crate::attestation_energy::ProofEnergyResult,
}

/// Request to verify a proof via zkVerify.
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub proof: serde_json::Value,
    pub public_signals: Vec<String>,
    pub proof_id: String,
}

/// Response from zkVerify verification.
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub verified: bool,
    /// zkVerify transaction hash (if submitted to zkVerify).
    pub zkverify_tx_hash: Option<String>,
    /// zkVerify statement hash.
    pub statement_hash: Option<String>,
    /// Whether this was verified locally (fallback) or via zkVerify.
    pub verification_method: String,
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Metrics for Thrive grant milestone tracking.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_proofs_generated: u64,
    pub total_proofs_verified: u64,
    pub total_zkverify_submissions: u64,
    pub unique_users: u64,
    /// Timestamp of last proof generation.
    pub last_proof_at: Option<String>,
    /// Average proof energy score across all proofs (FMD physics model).
    pub avg_energy: f64,
    /// Total negentropy extracted across all proofs (bits).
    pub total_negentropy_bits: f64,
}
