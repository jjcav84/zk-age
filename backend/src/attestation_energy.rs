//! Attestation Energy — thin domain adapter over the `negentropy` physics
//! engine for ranking ZK age proofs.
//!
//! The core thermodynamic formula (route energy, committor, negentropy
//! extraction) lives in the [`negentropy`] crate. This module maps age-proof
//! domain quantities onto that engine:
//!
//! - **confidence** ← issuer trust score (analogous to pool TVL)
//! - **depth_ratio** ← confidence / log₁₀(threshold)
//! - **timing_factor** ← exp(-age / half_life)
//! - **latency_decay** ← 1 / (1 + total_latency × decay_rate)
//! - **cost_penalty** ← zkVerify submission cost, normalized
//!
//! See <https://github.com/jjcav84/negentropy> for the physics.

use serde::{Deserialize, Serialize};

use negentropy::{Committor, Negentropy, RouteEnergy};

/// Age proof energy evaluation result.
///
/// Produced by [`ProofPotential::energy`]. The fields mirror the core
/// `negentropy::RouteEnergyResult` plus domain-specific extras (committor
/// and negentropy bits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEnergyResult {
    /// Total energy score (higher = better quality proof)
    pub energy: f64,
    /// Confidence depth ratio (issuer trust / threshold strictness)
    pub depth_ratio: f64,
    /// Timing factor (recency decay, 0..1)
    pub timing_factor: f64,
    /// Latency decay (proof gen + verify speed, 0..1)
    pub latency_decay: f64,
    /// Cost penalty (zkVerify submission cost, 0..1)
    pub cost_penalty: f64,
    /// Committor probability (likelihood proof is valid & uncontested)
    pub committor: f64,
    /// Negentropy extracted (information created by the proof, in bits)
    pub negentropy_bits: f64,
}

/// Configuration for proof energy evaluation.
#[derive(Debug, Clone)]
pub struct ProofPotential {
    /// zkVerify Kurier API cost per proof submission (USD)
    pub zkverify_cost_usd: f64,
    /// Proof generation latency in milliseconds
    pub proof_latency_ms: u64,
    /// Verification latency in milliseconds
    pub verify_latency_ms: u64,
    /// Proof age in seconds (time since proof was generated)
    pub proof_age_secs: f64,
    /// Circuit constraint count (more constraints = more negentropy)
    pub constraint_count: u64,
}

impl Default for ProofPotential {
    fn default() -> Self {
        Self {
            zkverify_cost_usd: 0.001, // ~$0.001 per zkVerify submission
            proof_latency_ms: 800,
            verify_latency_ms: 30,
            proof_age_secs: 0.0,
            constraint_count: 17, // zk-age circuit: 17 non-linear constraints
        }
    }
}

/// Half-life for proof recency decay (1 hour, in seconds).
const HALF_LIFE_SECS: f64 = 3600.0;

impl ProofPotential {
    /// Evaluate proof energy via the `negentropy` physics engine.
    ///
    /// Delegates:
    /// - energy → `negentropy::RouteEnergy::new`
    /// - committor → `negentropy::Committor::score`
    /// - negentropy_bits → `negentropy::Negentropy::from_constraints`
    pub fn energy(&self, threshold: u64, issuer_trust: f64) -> ProofEnergyResult {
        // Domain mapping: issuer trust (0..1) → confidence (0..100)
        let confidence = 100.0 * issuer_trust.clamp(0.0, 1.0);

        // Depth ratio: confidence relative to threshold strictness
        let threshold_f = threshold.max(1) as f64;
        let depth_ratio = confidence / threshold_f.log10().max(1.0);

        // Timing factor: exponential decay based on proof age
        let timing_factor = (-self.proof_age_secs / HALF_LIFE_SECS).exp();

        // Latency decay: total proof generation + verification latency
        let total_latency = self.proof_latency_ms + self.verify_latency_ms;
        let latency_decay = 1.0 / (1.0 + total_latency as f64 * 0.0001);

        // Cost penalty: zkVerify submission cost, normalized
        let cost_penalty = (self.zkverify_cost_usd * 0.01).min(0.5);

        // Core energy from negentropy
        let energy = RouteEnergy::new(
            confidence,
            depth_ratio,
            timing_factor,
            latency_decay,
            cost_penalty,
        )
        .energy;

        // Committor from negentropy (TPS rare-event prediction)
        let committor = Committor::score(depth_ratio, timing_factor, cost_penalty);

        // Negentropy extracted: N = constraint_count × log₂(threshold)
        let negentropy_bits =
            Negentropy::from_constraints(self.constraint_count, threshold).bits();

        ProofEnergyResult {
            energy,
            depth_ratio,
            timing_factor,
            latency_decay,
            cost_penalty,
            committor,
            negentropy_bits,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_proof_energy() {
        let pot = ProofPotential::default();
        let result = pot.energy(18, 0.95);

        assert!(result.energy > 0.0, "energy should be positive");
        assert!(result.depth_ratio > 0.0);
        assert!(result.timing_factor > 0.99, "fresh proof should have high timing");
        assert!(result.latency_decay > 0.0);
        assert!(result.committor > 0.0 && result.committor <= 1.0);
        assert!(result.negentropy_bits > 0.0);
    }

    #[test]
    fn test_stale_proof_decays() {
        let mut pot = ProofPotential::default();
        pot.proof_age_secs = 7200.0; // 2 hours = 2 half-lives

        let fresh = ProofPotential::default().energy(18, 0.9);
        let stale = pot.energy(18, 0.9);

        assert!(
            stale.energy < fresh.energy,
            "stale proof should have lower energy"
        );
        assert!(
            stale.timing_factor < fresh.timing_factor * 0.5,
            "2 half-lives should reduce timing by >50%"
        );
    }

    #[test]
    fn test_higher_threshold_more_negentropy() {
        let pot = ProofPotential::default();

        let low_threshold = pot.energy(13, 0.9);
        let high_threshold = pot.energy(25, 0.9);

        assert!(
            high_threshold.negentropy_bits > low_threshold.negentropy_bits,
            "higher threshold extracts more negentropy"
        );
    }

    #[test]
    fn test_low_trust_reduces_energy() {
        let pot = ProofPotential::default();

        let high_trust = pot.energy(18, 0.95);
        let low_trust = pot.energy(18, 0.3);

        assert!(
            low_trust.energy < high_trust.energy,
            "lower issuer trust should reduce energy"
        );
    }

    #[test]
    fn test_negentropy_formula() {
        // 17 constraints, threshold 18: N = 17 * log2(18) ≈ 70.9 bits
        let pot = ProofPotential::default();
        let result = pot.energy(18, 0.9);
        let expected = 17.0 * (18.0f64).log2();
        assert!(
            (result.negentropy_bits - expected).abs() < 0.01,
            "negentropy should be 17 * log2(18) ≈ {:.1}, got {:.1}",
            expected,
            result.negentropy_bits
        );
    }
}
