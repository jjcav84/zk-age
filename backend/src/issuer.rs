//! Simulated issuer — represents a government ID authority that signs
//! birthdate commitments.
//!
//! In production, this would be a real ID verification service (e.g., a
//! government digital ID API, or a decentralized identity protocol like
//! Polygon ID). For the demo, we simulate the signing with a simple
//! algebraic signature.

use anyhow::Result;

use crate::types::{IssueRequest, IssueResponse};

/// The issuer's public key hash. In production this would be a Poseidon hash
/// of the issuer's BabyJubJub public key.
const ISSUER_PUBKEY_HASH: u64 = 12345;

/// Simulated issuer: signs (birth_year, pubkey_hash, randomness).
///
/// Signature = birth_year + pubkey_hash * randomness (mod p)
///
/// This is NOT cryptographically secure — it's a demo. Production would use
/// EdDSA on BabyJubJub or Poseidon-based signatures.
pub fn issue(req: &IssueRequest) -> Result<IssueResponse> {
    let randomness: u64 = rand::random::<u32>() as u64;
    let signature = req.birth_year + ISSUER_PUBKEY_HASH * randomness;

    Ok(IssueResponse {
        issuer_pubkey_hash: ISSUER_PUBKEY_HASH.to_string(),
        issuer_signature: signature.to_string(),
        signature_randomness: randomness.to_string(),
        birth_year: req.birth_year,
    })
}
