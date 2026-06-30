//! HTTP routes — the API surface for the zk-age backend.

use std::sync::Arc;

use axum::{routing::{get, post}, Json, Router};

use crate::state::AppState;
use crate::types::*;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/issue", post(issue))
        .route("/api/prove", post(prove))
        .route("/api/verify", post(verify))
        .route("/api/stats", get(stats))
        .fallback_service(tower_http::services::ServeFile::new("frontend/index.html"))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn issue(
    Json(req): Json<IssueRequest>,
) -> Result<Json<IssueResponse>, (axum::http::StatusCode, String)> {
    crate::issuer::issue(&req)
        .map(Json)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn prove(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (axum::http::StatusCode, String)> {
    let result = crate::prover::generate_proof(&req)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Track metrics
    let user_id = format!("user-{}", req.birth_year); // simplified
    state.record_proof(&user_id);

    tracing::info!(
        "proof generated: id={}, threshold={}",
        result.proof_id,
        req.threshold
    );

    Ok(Json(result))
}

async fn verify(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (axum::http::StatusCode, String)> {
    let config = crate::zkverify::ZkVerifyConfig::default();

    let result = crate::zkverify::verify_proof(&config, &req.proof, &req.public_signals)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.verified {
        state.record_verification();
        if result.verification_method == "zkverify" {
            state.record_zkverify_submission();
        }
    }

    tracing::info!(
        "proof verified: id={}, method={}, verified={}",
        req.proof_id,
        result.verification_method,
        result.verified
    );

    Ok(Json(result))
}

async fn stats(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<StatsResponse> {
    let (generated, verified, zkverify, users, last) = state.stats();

    Json(StatsResponse {
        total_proofs_generated: generated,
        total_proofs_verified: verified,
        total_zkverify_submissions: zkverify,
        unique_users: users,
        last_proof_at: last,
    })
}
