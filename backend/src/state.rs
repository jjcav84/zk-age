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
}

impl AppState {
    pub fn new() -> Self {
        Self {
            proofs_generated: AtomicU64::new(0),
            proofs_verified: AtomicU64::new(0),
            zkverify_submissions: AtomicU64::new(0),
            unique_users: Mutex::new(HashSet::new()),
            last_proof_at: Mutex::new(None),
        }
    }

    pub fn record_proof(&self, user_id: &str) {
        self.proofs_generated.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut users) = self.unique_users.lock() {
            users.insert(user_id.to_string());
        }
        if let Ok(mut ts) = self.last_proof_at.lock() {
            *ts = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    pub fn record_verification(&self) {
        self.proofs_verified.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_zkverify_submission(&self) {
        self.zkverify_submissions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stats(&self) -> (u64, u64, u64, u64, Option<String>) {
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
        (
            self.proofs_generated.load(Ordering::Relaxed),
            self.proofs_verified.load(Ordering::Relaxed),
            self.zkverify_submissions.load(Ordering::Relaxed),
            users,
            last,
        )
    }
}
