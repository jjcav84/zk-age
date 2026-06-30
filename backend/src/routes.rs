//! HTTP routes — the API surface for the zk-age backend.

use std::sync::Arc;

use axum::{routing::{get, post}, Json, Router, extract::Path};

use crate::state::AppState;
use crate::types::*;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/issue", post(issue))
        .route("/api/prove", post(prove))
        .route("/api/verify", post(verify))
        .route("/api/stats", get(stats))
        .route("/api/energy/:proof_id", get(energy))
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

    // Track metrics with energy score
    let user_id = format!("user-{}", req.birth_year); // simplified
    state.record_proof(&user_id, result.energy.energy, result.energy.negentropy_bits);

    tracing::info!(
        "proof generated: id={}, threshold={}, energy={:.2}, negentropy={:.1} bits, latency={}ms",
        result.proof_id,
        req.threshold,
        result.energy.energy,
        result.energy.negentropy_bits,
        result.proof_latency_ms,
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
    let (generated, verified, zkverify, users, last, avg_energy, total_negentropy) = state.stats();

    Json(StatsResponse {
        total_proofs_generated: generated,
        total_proofs_verified: verified,
        total_zkverify_submissions: zkverify,
        unique_users: users,
        last_proof_at: last,
        avg_energy,
        total_negentropy_bits: total_negentropy,
    })
}

async fn energy(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(_proof_id): Path<String>,
) -> Json<serde_json::Value> {
    let (_, _, _, _, _, avg_energy, total_negentropy) = state.stats();
    Json(serde_json::json!({
        "avg_energy": avg_energy,
        "total_negentropy_bits": total_negentropy,
        "model": "FMD Route Energy (adapted from orkid fmd-physics)",
        "formula": "energy = confidence * sqrt(depth_ratio * timing_factor) * latency_decay * (1 - cost_penalty)",
        "negentropy_formula": "N = constraint_count * log2(threshold)",
        "constraint_count": 17,
        "origin": "orkid fmd-physics/src/route_energy.rs",
        "references": {
            "route_energy": "https://github.com/jjcav84/orkid/blob/main/fmd-physics/src/route_energy.rs",
            "blog_thermodynamics": "Blockchain Thermodynamics: How Negentropy Explains MEV",
            "blog_negentropy": "Negentropy = Information: A Generalized Mathematical Framework",
            "blog_route_scoring": "Complex Microstructure and Route Scoring in DeFi"
        }
    }))
}
