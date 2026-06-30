//! Application state — tracks metrics for Thrive grant milestones.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::collections::HashSet;

pub struct AppState {
    pub proofs_generated: AtomicU64,
    pub proofs_verified: AtomicU64,
    pub zkverify_submissions: AtomicU64,
    pub unique_users: Mutex<HashSet<String>>,
    pub last_proof_at: Mutex<Option<String>>,
    pub energy_sum: Mutex<f64>,
    pub negentropy_sum: Mutex<f64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            proofs_generated: AtomicU64::new(0),
            proofs_verified: AtomicU64::new(0),
            zkverify_submissions: AtomicU64::new(0),
            unique_users: Mutex::new(HashSet::new()),
            last_proof_at: Mutex::new(None),
            energy_sum: Mutex::new(0.0),
            negentropy_sum: Mutex::new(0.0),
        }
    }

    pub fn record_proof(&self, user_id: &str, energy: f64, negentropy_bits: f64) {
        self.proofs_generated.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut users) = self.unique_users.lock() {
            users.insert(user_id.to_string());
        }
        if let Ok(mut ts) = self.last_proof_at.lock() {
            *ts = Some(chrono::Utc::now().to_rfc3339());
        }
        if let Ok(mut sum) = self.energy_sum.lock() {
            *sum += energy;
        }
        if let Ok(mut sum) = self.negentropy_sum.lock() {
            *sum += negentropy_bits;
        }
    }

    pub fn record_verification(&self) {
        self.proofs_verified.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_zkverify_submission(&self) {
        self.zkverify_submissions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stats(&self) -> (u64, u64, u64, u64, Option<String>, f64, f64) {
        let users = self
            .unique_users
            .lock()
            .map(|u| u.len() as u64)
            .unwrap_or(0);
        let last = self
            .last_proof_at
            .lock()
            .ok()
            .and_then(|t| t.clone());
        let count = self.proofs_generated.load(Ordering::Relaxed);
        let avg_energy = if count == 0 {
            0.0
        } else {
            let sum = self.energy_sum.lock().map(|s| *s).unwrap_or(0.0);
            sum / count as f64
        };
        let total_negentropy = self.negentropy_sum.lock().map(|s| *s).unwrap_or(0.0);
        (
            self.proofs_generated.load(Ordering::Relaxed),
            self.proofs_verified.load(Ordering::Relaxed),
            self.zkverify_submissions.load(Ordering::Relaxed),
            users,
            last,
            avg_energy,
            total_negentropy,
        )
    }
}
