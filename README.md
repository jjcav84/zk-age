# zk-age

Privacy-preserving age verification for web2 applications, powered by zero-knowledge proofs and [zkVerify](https://zkverify.io).

> Prove you're 18+ without revealing your birthdate.

## What it does

zk-age lets any web2 application verify a user's age without collecting or storing their birthdate. The user generates a Groth16 zero-knowledge proof that demonstrates `current_year - birth_year >= threshold`, and the proof is verified on zkVerify — a high-performance blockchain dedicated to ZK proof verification.

**The user never reveals their actual age. The verifier never sees the birthdate. Only the boolean result (eligible / not eligible) is confirmed.**

### Why this matters

Every age-gated website today collects birthdates — creating massive PII liability, GDPR exposure, and data breach risk. zk-age eliminates the need to store this data entirely. The proof is the verification.

## How it works

```
┌──────────┐     ┌──────────────────┐     ┌─────────────┐     ┌───────────┐
│  User    │────▶│  Rust Backend    │────▶│  snarkjs    │────▶│  zkVerify │
│  (web)   │     │  (axum server)   │     │  (Groth16)  │     │  (Kurier) │
│          │◀────│                  │◀────│  proof gen  │◀────│  verify   │
└──────────┘     └──────────────────┘     └─────────────┘     └───────────┘
                         │
                         ▼
                 ┌───────────────┐
                 │  ID Issuer    │
                 │  (simulated)  │
                 │  signs DOB    │
                 └───────────────┘
```

1. **Issue**: An ID authority (government, digital ID provider) signs a commitment to the user's birth year. In production, this would be Polygon ID, a government digital ID API, or an OIDC provider with ZK support.

2. **Prove**: The user's browser requests a proof from the Rust backend. The backend generates a Groth16 proof using snarkjs + the compiled circom circuit. The proof demonstrates:
   - The user possesses a valid signed birthdate commitment
   - `current_year - birth_year >= threshold`
   - Without revealing `birth_year`

3. **Verify**: The proof is submitted to zkVerify via the Kurier REST API for on-chain verification. zkVerify returns a transaction hash and statement hash, providing a permanent, auditable verification record.

4. **Result**: The web frontend displays "Age verified — you're 18+" with zero knowledge of the user's actual age.

## The circuit

The circom circuit (`circuit/age.circom`) has:

- **3 public inputs**: `current_year`, `threshold`, `issuer_pubkey_hash`
- **3 private inputs**: `birth_year`, `issuer_signature`, `signature_randomness`
- **17 constraints**: age range check (Num2Bits) + signature verification

```
Public outputs:  current_year, threshold, issuer_pubkey_hash
Private (hidden): birth_year, issuer_signature, signature_randomness
```

The birth year is never in the public signals. The verifier learns only that the age check passed.

### Production upgrade path

The demo uses a simplified algebraic signature (`sig = birth_year + pubkey * randomness`). Production would swap in:
- **Poseidon hash** for the signature (one-line change in circom)
- **EdDSA on BabyJubJub** for real cryptographic signatures
- The circuit structure and zkVerify integration remain identical

## Tech stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Circuit | circom 2.2.3 | Industry standard ZK circuit compiler |
| Proof system | Groth16 (snarkjs) | Smallest proofs (~200 bytes), fastest verification |
| Curve | BN128 / BN254 | Supported by zkVerify, EVM-compatible |
| Backend | Rust (axum, tokio) | Performance, safety, production-grade |
| zkVerify integration | Kurier REST API | No blockchain knowledge required for web2 teams |
| Frontend | Vanilla HTML/JS | Zero dependencies, instant load, works everywhere |

## zkVerify integration

zk-age integrates with zkVerify via the [Kurier REST API](https://docs.zkverify.io/overview/getting-started/kurier):

1. **Register VK**: The circuit's verification key is registered once via `POST /register-vk`, returning a `vkHash` cached for all future submissions.

2. **Submit proof**: Each generated proof is submitted via `POST /submit-proof` with the `vkHash`, proof, and public signals.

3. **Poll status**: The backend polls `GET /job-status/{job_id}` until the proof is `Verified` or `Finalized` on zkVerify.

4. **Return result**: The verification result includes the zkVerify transaction hash and statement hash, providing an auditable on-chain verification record.

### Configuration

Set the `ZKVERIFY_API_KEY` environment variable to enable zkVerify integration:

```bash
export ZKVERIFY_API_KEY="your-kurier-api-key"
# Optional: override zkVerify endpoint (defaults to testnet)
export ZKVERIFY_URL="https://testnet.kurier.xyz/api/v1"
```

Without an API key, the backend falls back to local snarkjs verification — the demo still works, but proofs aren't submitted to zkVerify.

## Quick start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- [circom](https://docs.circom.io/getting-started/installation/) 2.2.3+
- [snarkjs](https://github.com/iden3/snarkjs): `npm install -g snarkjs`

### Build the circuit

```bash
cd circuit
npm install                    # install circomlib
circom age.circom --r1cs --wasm --sym -o ../build

# Trusted setup (Groth16)
cd ../build
snarkjs powersoftau new bn128 8 pot0_0000.ptau
snarkjs powersoftau contribute pot0_0000.ptau pot0_0001.ptau --name="zk-age" -e="entropy"
snarkjs powersoftau prepare phase2 pot0_0001.ptau pot0_final.ptau
snarkjs groth16 setup age.r1cs pot0_final.ptau age_0000.zkey
snarkjs zkey contribute age_0000.zkey age_final.zkey --name="zk-age" -e="entropy"
snarkjs zkey export verificationkey age_final.zkey verification_key.json
```

### Run the backend

```bash
cargo run
# zk-age backend listening on http://0.0.0.0:3000
```

Open http://localhost:3000 in your browser. Enter a birth year, select a threshold, and click "Prove my age."

### API endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/issue` | POST | Issue a signed birthdate credential (simulated ID authority) |
| `/api/prove` | POST | Generate a Groth16 ZK proof of age |
| `/api/verify` | POST | Verify a proof via zkVerify (or local fallback) |
| `/api/stats` | GET | Metrics for Thrive grant milestone tracking |

## Thrive zkVerify Web2 Program — Grant Plan

### Ecosystem value proposition

zk-age drives **proof verification volume** to zkVerify. Every age-gated page view generates a proof and a zkVerify verification. A single e-commerce site with 100K daily age-gated page views generates 3M proofs/month — far exceeding the Milestone 3 target of 250K proofs.

### Revenue model

| Tier | Price | Target | Monthly proofs |
|------|-------|--------|----------------|
| Free | $0 | 1K verifications/month | Developer testing |
| Starter | $49/mo | 10K verifications/month | Small sites |
| Growth | $199/mo | 100K verifications/month | Mid-market |
| Enterprise | Custom | Unlimited | High-volume (alcohol, cannabis, gambling) |

Revenue is sustainable beyond the grant period through SaaS subscriptions. The marginal cost per verification approaches zero as zkVerify reduces on-chain verification costs by ~90% vs. verifying proofs directly on Ethereum.

### Milestone roadmap

**Application (10%)**: This repo — working circuit, Rust backend, zkVerify integration plan, frontend.

**Milestone 1 — Live Deployment (10%, 45 days)**:
- Deploy backend to production (Railway/Fly.io)
- Integrate real zkVerify Kurier API key
- Publish integration documentation
- Beta test with 3 age-gated websites

**Milestone 2 — Initial Traction (30%, 90 days)**:
- Target: 25,000+ ZK proofs sent to zkVerify
- Onboard 10+ websites via SDK/iframe embed
- Launch developer documentation and integration guide
- Implement proof batching for efficiency

**Milestone 3 — Scale (50%, 150 days)**:
- Target: 250,000+ ZK proofs sent to zkVerify
- Onboard 50+ websites
- Launch hosted verification widget (one-line script tag)
- Implement Poseidon-based signatures (production-grade crypto)
- Add age range proofs (e.g., "18-25", "25-35") for analytics

### Performance tracking

The `/api/stats` endpoint tracks all metrics required by the Thrive program:
- `total_proofs_generated` — proofs created
- `total_proofs_verified` — proofs verified
- `total_zkverify_submissions` — proofs submitted to zkVerify
- `unique_users` — distinct users
- `last_proof_at` — timestamp of last proof

## Use cases

- **Age-gated e-commerce**: Alcohol, cannabis, vape, gambling sites
- **Social media**: COPPA compliance (13+), platform age requirements
- **Content platforms**: R-rated content, mature content filters
- **Financial services**: KYC age verification without PII storage
- **Healthcare**: Age-based service eligibility
- **Gaming**: ESRB rating enforcement

## License

MIT
