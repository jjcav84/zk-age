//! zk-age backend — privacy-preserving age verification with ZK proofs.
//!
//! Architecture:
//! - `POST /api/issue` — issuer signs a birthdate commitment (simulated ID authority)
//! - `POST /api/prove` — user generates a Groth16 proof locally (via snarkjs subprocess)
//! - `POST /api/verify` — proof is submitted to zkVerify (via Kurier REST API) for on-chain verification
//! - `GET /api/health` — health check
//! - `GET /api/stats` — proof count, verification count, metrics for Thrive milestones
//!
//! The frontend never touches blockchain. Users just click "I'm 18+" and get
//! a verified result. The Rust backend orchestrates the ZK workflow.

pub mod attestation_energy;
pub mod issuer;
pub mod prover;
pub mod zkverify;
pub mod routes;
pub mod state;
pub mod types;

use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let state = Arc::new(state::AppState::new());
    let app = routes::router(state.clone()).layer(CorsLayer::permissive());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("zk-age backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
