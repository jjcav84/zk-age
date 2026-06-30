//! Attestation Energy — adapts the FMD (Financial Molecular Dynamics) route
//! energy scoring framework from the orkid workspace for ranking ZK age proofs.
//!
//! In the orkid FMD physics engine (`fmd-physics/src/route_energy.rs`), route
//! energy scores arbitrage paths by:
//!
//!   energy = net_bps * sqrt(depth_ratio * timing_factor) * latency_decay * (1 - gas_penalty)
//!
//! Here, we apply the same thermodynamic framework to age verification proofs.
//! Each ZK proof is a **negentropy extraction** — converting private, chaotic
//! data (a birthdate) into structured, verifiable order (a proof that age >=
//! threshold) without revealing the underlying value.
//!
//! ## The Thermodynamic Framing
//!
//! From the orkid blog posts on blockchain thermodynamics and negentropy:
//!
//! - **Shannon entropy**: H = -sum(p_i * log(p_i)) — uncertainty about the
//!   user's age before the proof
//! - **Negentropy = Information** (Brillouin, 1953): N = H_max - H_actual =
//!   D_KL(p_informed || p_uninformed) — the information gained by the proof
//! - **Landauer's principle**: Erasing information costs energy:
//!   E >= k_B * T * ln(2) per bit — each bit of negentropy has a thermodynamic
//!   cost, which the proof generation pays in compute
//! - **MEV closure equation** (orkid formal negentropy model):
//!   dM/dt = a*delta + b*H_M - c*chi(I)*M — information closes arbitrage
//!   opportunities; analogously, the ZK proof "closes" the uncertainty about
//!   the user's age
//!
//! ## The Energy Formula
//!
//! FMD route energy (orkid):
//!   energy = net_bps * sqrt(depth_ratio * timing_factor) * latency_decay * (1 - gas_penalty)
//!
//! Age proof energy (adapted):
//!   energy = confidence * sqrt(depth_ratio * timing_factor) * latency_decay * (1 - cost_penalty)
//!
//! Where:
//! - confidence: issuer trust score (analogous to pool TVL / liquidity depth)
//! - depth_ratio: confidence / log10(threshold) — higher threshold = harder to
//!   prove = more negentropy extracted
//! - timing_factor: exp(-age / half_life) — recency decay
//! - latency_decay: 1 / (1 + total_latency_ms * decay_rate) — proof gen speed
//! - cost_penalty: zkVerify submission cost, normalized
//!
//! ## Negentropy Extraction
//!
//! Each ZK proof extracts negentropy (information) from private data:
//!
//!   N = constraint_count * log2(threshold)
//!
//! For the zk-age circuit (17 constraints, threshold=18):
//!   N = 17 * log2(18) ≈ 70.9 bits
//!
//! This is the Shannon entropy reduction — the amount of uncertainty about
//! the user's age that is eliminated by the proof. The verifier learns the
//! user is above the threshold without learning the exact age.
//!
//! ## Committor Function
//!
//! Adapted from the TPS (Transition Path Sampling) committor in the FMD
//! engine, which predicts the probability of reaching a profitable state:
//!
//!   committor = (depth_ratio / (1 + depth_ratio)) * timing_factor * (1 - cost_penalty * 0.5)
//!
//! This estimates the probability that the proof is valid and uncontested —
//! a "rare event" prediction for proof quality.

use serde::{Deserialize, Serialize};

/// Age proof energy evaluation result — mirrors RouteEnergyResult from FMD.
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

impl ProofPotential {
    /// Evaluate proof energy — adapts the FMD route energy formula.
    ///
    /// FMD route energy (orkid `fmd-physics/src/route_energy.rs`):
    ///   energy = net_bps * sqrt(depth_ratio * timing_factor) * latency_decay * (1 - gas_penalty)
    ///
    /// Age proof energy:
    ///   energy = confidence * sqrt(depth_ratio * timing_factor) * latency_decay * (1 - cost_penalty)
    pub fn energy(&self, threshold: u64, issuer_trust: f64) -> ProofEnergyResult {
        // Confidence: issuer trust score (0..1) scaled to a base depth
        // Analogous to pool TVL in the FMD engine — higher trust = more confidence
        let confidence = 100.0 * issuer_trust.clamp(0.0, 1.0);

        // Depth ratio: confidence relative to threshold strictness
        // Higher threshold = harder to prove = more negentropy extracted
        // Using log10 because the marginal difficulty of proving age >= 21
        // vs age >= 18 is sublinear
        let threshold_f = threshold.max(1) as f64;
        let depth_ratio = confidence / threshold_f.log10().max(1.0);

        // Timing factor: exponential decay based on proof age
        // Half-life of 1 hour (3600s) — stale proofs lose energy
        // Analogous to FMD timing_factor = 1/sqrt(hops) but here we use
        // recency because ZK proofs are point-in-time assertions
        let half_life = 3600.0;
        let timing_factor = (-self.proof_age_secs / half_life).exp();

        // Latency decay: total proof generation + verification latency
        // Analogous to FMD: (1 - 0.001 * hops * stage_latency_ms).max(0)
        // Here we use a softer decay: 1 / (1 + latency * rate)
        let total_latency = self.proof_latency_ms + self.verify_latency_ms;
        let latency_decay = 1.0 / (1.0 + total_latency as f64 * 0.0001);

        // Cost penalty: zkVerify submission cost, normalized
        // Analogous to FMD gas_penalty = gas_units * gas_cost * 0.005
        let cost_penalty = (self.zkverify_cost_usd * 0.01).min(0.5);

        // Energy: the core formula, adapted from FMD route_energy.rs
        let energy = confidence
            * (depth_ratio * timing_factor).sqrt()
            * latency_decay
            * (1.0 - cost_penalty).max(0.0);

        // Committor: probability proof is valid & uncontested
        // Adapted from TPS committor function — uses depth, timing, and cost
        // as features for a simplified probability estimate
        let committor = (depth_ratio / (1.0 + depth_ratio))
            * timing_factor
            * (1.0 - cost_penalty * 0.5)
            .clamp(0.0, 1.0);

        // Negentropy: information extracted by the proof (in bits)
        // Each constraint contributes ~1 bit of negentropy (order from chaos)
        // This is the Shannon entropy reduction: H = -sum(p_i * log2(p_i))
        // For a ZK proof with N constraints proving threshold T:
        //   N_bits = constraint_count * log2(threshold)
        // This captures the idea that more constraints = more information,
        // and higher thresholds = more information per constraint.
        let negentropy_bits = self.constraint_count as f64
            * threshold_f.log2().max(1.0);

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
