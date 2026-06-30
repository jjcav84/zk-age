//! Proof generation — orchestrates snarkjs via subprocess to generate
//! Groth16 proofs from the compiled circom circuit.
//!
//! In production, this would use native Rust crates (ark-groth16, ark-snark)
//! instead of subprocess calls. The subprocess approach keeps the demo
//! simple while using the standard circom/snarkjs toolchain.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Result};
use serde_json::json;
use uuid::Uuid;

use crate::types::{ProveRequest, ProveResponse};

/// Paths to compiled circuit artifacts (relative to repo root).
const BUILD_DIR: &str = "build";
const WASM_FILE: &str = "age_js/age.wasm";
const ZKEY_FILE: &str = "age_final.zkey";

/// Generate a Groth16 proof of age >= threshold.
///
/// Steps:
/// 1. Write circuit inputs to a temp JSON file
/// 2. Generate witness via the compiled WASM
/// 3. Generate proof via snarkjs
/// 4. Read proof + public signals
pub fn generate_proof(req: &ProveRequest) -> Result<ProveResponse> {
    let proof_id = Uuid::new_v4().to_string();
    let tmp_dir = std::env::temp_dir().join(format!("zk-age-{}", proof_id));
    std::fs::create_dir_all(&tmp_dir)?;

    let current_year = chrono::Utc::now().format("%Y").to_string().parse::<u64>()?;

    // 1. Write inputs
    let inputs = json!({
        "current_year": current_year.to_string(),
        "threshold": req.threshold.to_string(),
        "issuer_pubkey_hash": req.issuer_pubkey_hash,
        "birth_year": req.birth_year.to_string(),
        "issuer_signature": req.issuer_signature,
        "signature_randomness": req.signature_randomness,
    });
    let input_path = tmp_dir.join("input.json");
    std::fs::write(&input_path, inputs.to_string())?;

    // 2. Generate witness
    let build_dir = find_build_dir()?;
    let wasm_path = build_dir.join(WASM_FILE);
    let witness_path = tmp_dir.join("witness.wtns");

    // Use the generate_witness.js helper from circom
    let gen_witness_js = build_dir.join("age_js/generate_witness.js");
    let witness_output = Command::new("node")
        .arg(&gen_witness_js)
        .arg(&wasm_path)
        .arg(&input_path)
        .arg(&witness_path)
        .output()?;

    if !witness_output.status.success() {
        let err = String::from_utf8_lossy(&witness_output.stderr);
        return Err(anyhow!("witness generation failed: {}", err));
    }

    // 3. Generate proof
    let zkey_path = build_dir.join(ZKEY_FILE);
    let proof_path = tmp_dir.join("proof.json");
    let public_path = tmp_dir.join("public.json");

    let prove_output = Command::new("snarkjs")
        .arg("groth16")
        .arg("prove")
        .arg(&zkey_path)
        .arg(&witness_path)
        .arg(&proof_path)
        .arg(&public_path)
        .output()?;

    if !prove_output.status.success() {
        let err = String::from_utf8_lossy(&prove_output.stderr);
        return Err(anyhow!("proof generation failed: {}", err));
    }

    // 4. Read results
    let proof: serde_json::Value = serde_json::from_slice(&std::fs::read(&proof_path)?)?;
    let public_raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&public_path)?)?;

    // snarkjs outputs public signals as an array
    let public_signals: Vec<String> = public_raw
        .as_array()
        .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("0").to_string()).collect())
        .unwrap_or_default();

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    Ok(ProveResponse {
        proof,
        public_signals,
        proof_id,
    })
}

/// Find the build directory by walking up from CWD.
fn find_build_dir() -> Result<PathBuf> {
    let mut cwd = std::env::current_dir()?;
    for _ in 0..5 {
        let candidate = cwd.join(BUILD_DIR);
        if candidate.join(ZKEY_FILE).exists() {
            return Ok(candidate);
        }
        if !cwd.pop() {
            break;
        }
    }
    Err(anyhow!(
        "could not find build/ directory with compiled circuit artifacts. \
         Run the circuit setup script first."
    ))
}
