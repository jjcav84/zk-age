<p align="center">
  <a href="https://www.orkidlabs.com"><img src="assets/logo.png" alt="Orkid Labs" width="220" /></a>
</p>

# zk-age

**By [Orkid Labs](https://www.orkidlabs.com)** — privacy-first crypto engineering

Privacy-preserving age verification for web2 applications, powered by zero-knowledge proofs, [zkVerify](https://zkverify.io), and an **FMD physics energy model** adapted from the [orkid](https://github.com/jjcav84/orkid) MEV detection engine.

> **Note:** The orkid repository is private. Access can be provided to
> Thrive Protocol reviewers and other appropriate cases on request —
> contact [Orkid Labs](https://www.orkidlabs.com). The theoretical
> foundation is published as a preprint:
> ["Negative EV per Unit Time as Blockchain Inefficiency"](https://www.researchgate.net/publication/399474539_Negative_EV_per_Unit_Time_as_Blockchain_Inefficiency)
> — [Jacob Cavazos, ResearchGate](https://www.researchgate.net/profile/Jacob-Cavazos).

> Prove you're 18+ without revealing your birthdate. Every proof is scored by its thermodynamic energy — the negentropy extracted from private data.

## What it does

zk-age lets any web2 application verify a user's age without collecting or storing their birthdate. The user generates a Groth16 zero-knowledge proof that demonstrates `current_year - birth_year >= threshold`, and the proof is verified on zkVerify — a high-performance blockchain dedicated to ZK proof verification.

**The user never reveals their actual age. The verifier never sees the birthdate. Only the boolean result (eligible / not eligible) is confirmed.**

### Why this matters

Every age-gated website today collects birthdates — creating massive PII liability, GDPR exposure, and data breach risk. zk-age eliminates the need to store this data entirely. The proof is the verification.

## The thermodynamic framing

zk-age applies the **Financial Molecular Dynamics (FMD)** physics framework from the orkid MEV detection engine to score ZK proofs by quality. This is not metaphor — the mathematics of statistical mechanics, information theory, and zero-knowledge proofs are fundamentally connected.

### Negentropy = Information = Order

From Brillouin's negentropy principle (1953) and the orkid blog post ["Negentropy = Information: A Generalized Mathematical Framework"](https://www.orkidlabs.com/blog/negentropy-information-generalized-framework/):

> **Negentropy = H_max − H_actual = D_KL(p_informed || p_uninformed)**

A birthdate is a **high-entropy state** — without verification, anyone could claim any age. A ZK proof is a **negentropy extraction**: it converts private, chaotic data into structured, verifiable order (the boolean "age >= threshold") without revealing the underlying value.

For the zk-age circuit (17 constraints, threshold=18):

```
N = constraint_count × log₂(threshold) = 17 × log₂(18) ≈ 70.9 bits
```

This is the Shannon entropy reduction — the amount of uncertainty about the user's age eliminated by the proof. Each constraint in the circuit contributes ~1 bit of negentropy. The verifier learns the user is above the threshold without learning the exact age.

### Landauer's principle

From Landauer (1961) and the orkid blog post ["Blockchain Thermodynamics: How Negentropy Explains MEV, Consensus, and Arbitrage"](https://www.orkidlabs.com/blog/blockchain-thermodynamics-negentropy-mev-physics/):

> **E ≥ k_B × T × ln(2) per bit erased**

Proof generation pays the thermodynamic cost of extracting negentropy. The compute energy spent generating the Groth16 proof is the Landauer cost of creating 70.9 bits of order from private chaos. The verifier receives this order without paying the cost.

### The MEV closure analogy

From the orkid blog post ["A Formal Mathematical Model of Blockchain Negentropy and MEV Dynamics"](https://www.orkidlabs.com/blog/formal-negentropy-model-mev-dynamics-graph-diffusion/):

> **dM/dt = a·δ + b·H_M − c·χ(I)·M**

In MEV: information closes arbitrage opportunities. In zk-age: the ZK proof "closes" the uncertainty about the user's age. The proof is the information injection that collapses the entropy of the unverifiable claim into a deterministic boolean.

## The energy model

The proof energy model is adapted from the **route energy formula** in the orkid FMD physics engine (`fmd-physics/src/route_energy.rs`):

### FMD route energy (orkid)

```
energy = net_bps × √(depth_ratio × timing_factor) × latency_decay × (1 − gas_penalty)
```

This scores arbitrage paths by net output, liquidity depth, timing, and gas cost. Higher energy = more profitable route.

### Age proof energy (zk-age)

```
energy = confidence × √(depth_ratio × timing_factor) × latency_decay × (1 − cost_penalty)
```

This scores ZK proofs by issuer confidence, threshold strictness, recency, proof speed, and verification cost. Higher energy = higher quality proof.

| Factor | FMD (MEV) | zk-age (ZK proofs) |
|--------|-----------|---------------------|
| **Confidence** | Pool TVL (liquidity depth) | Issuer trust score (credential strength) |
| **Depth ratio** | Reserve ratio / trade size | Confidence / log₁₀(threshold) |
| **Timing factor** | 1/√(hops) | exp(−age / half_life) |
| **Latency decay** | (1 − 0.001 × hops × latency) | 1 / (1 + total_latency × 0.0001) |
| **Cost penalty** | Gas units × gas cost | zkVerify submission cost |

### Committor function

Adapted from the TPS (Transition Path Sampling) committor in the FMD engine, which predicts the probability of reaching a profitable state:

```
committor = (depth_ratio / (1 + depth_ratio)) × timing_factor × (1 − cost_penalty × 0.5)
```

This estimates the probability that the proof is valid and uncontested — a "rare event" prediction for proof quality. A fresh proof from a trusted issuer with a high threshold yields a committor near 1.0.

### Example

For a proof of age >= 18, issued by a government ID (trust=0.95), generated in 572ms:

| Metric | Value |
|--------|-------|
| Energy | 779.51 |
| Negentropy | 70.9 bits |
| Committor | 98.7% |
| Depth ratio | 75.68 |
| Latency decay | 0.943 |
| Cost penalty | 0.00001 |

## How it works

```
┌──────────┐     ┌──────────────────┐     ┌─────────────┐     ┌───────────┐
│  User    │────▶│  Rust Backend    │────▶│  snarkjs    │────▶│  zkVerify │
│  (web)   │     │  (axum server)   │     │  (Groth16)  │     │  (Kurier) │
│          │◀────│  + FMD energy    │◀────│  proof gen  │◀────│  verify   │
└──────────┘     └──────────────────┘     └─────────────┘     └───────────┘
                         │
                         ▼
                 ┌───────────────┐
                 │  ID Issuer    │
                 │  (simulated)  │
                 │  signs DOB    │
                 └───────────────┘
```

1. **Issue**: An ID authority (government, digital ID provider) signs a commitment to the user's birth year.

2. **Prove**: The backend generates a Groth16 proof using snarkjs + the compiled circom circuit. The proof demonstrates:
   - The user possesses a valid signed birthdate commitment
   - `current_year - birth_year >= threshold`
   - Without revealing `birth_year`
   - The FMD energy model scores the proof by quality (negentropy, committor, latency)

3. **Verify**: The proof is submitted to zkVerify via the Kurier REST API for on-chain verification. zkVerify returns a transaction hash and statement hash, providing a permanent, auditable verification record.

4. **Result**: The frontend displays "Age verified — you're 18+" with the energy score and negentropy extracted.

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
| Energy model | FMD physics (adapted from orkid) | Thermodynamic proof quality scoring |
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
| `/api/prove` | POST | Generate a Groth16 ZK proof of age + compute FMD energy score |
| `/api/verify` | POST | Verify a proof via zkVerify (or local fallback) |
| `/api/stats` | GET | Metrics for Thrive grant milestone tracking + energy stats |
| `/api/energy/:id` | GET | FMD physics energy model details and references |

## Build & test

```bash
cargo build
cargo test --bin zk-age-backend
```

## Project structure

```
zk-age/
├── circuit/
│   └── age.circom              # Age verification ZK circuit (17 constraints)
├── build/                      # Compiled circuit artifacts (gitignored)
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # axum server entry
│       ├── routes.rs           # HTTP API + energy endpoint
│       ├── types.rs            # API types (with energy fields)
│       ├── state.rs            # Metrics tracking (with energy sum)
│       ├── issuer.rs           # Credential issuance (simulated authority)
│       ├── prover.rs           # snarkjs proof generation + energy computation
│       ├── zkverify.rs         # zkVerify Kurier REST integration
│       └── attestation_energy.rs  # FMD physics energy model
├── frontend/
│   └── index.html              # Zero-dependency web UI with energy display
├── Cargo.toml                  # Workspace
└── README.md
```

## Thrive zkVerify Web2 Program (#44) — Grant Plan

### Ecosystem value proposition

zk-age drives **proof verification volume** to zkVerify. Every age-gated page view generates a proof and a zkVerify verification. A single e-commerce site with 100K daily age-gated page views generates 3M proofs/month — far exceeding the Milestone 3: Scale target of 250,000+ ZK Proofs sent to zkVerify.

### Revenue model

| Tier | Price | Target | Monthly proofs |
|------|-------|--------|----------------|
| Free | $0 | 1K verifications/month | Developer testing |
| Starter | $49/mo | 10K verifications/month | Small sites |
| Growth | $199/mo | 100K verifications/month | Mid-market |
| Enterprise | Custom | Unlimited | High-volume (alcohol, cannabis, gambling) |

Revenue is sustainable beyond the grant period through SaaS subscriptions. The marginal cost per verification approaches zero as zkVerify reduces on-chain verification costs by ~90% vs. verifying proofs directly on Ethereum.

### Milestone roadmap

Progressive achievement over 150 days, following Thrive zkVerify Web2 Program (#44) milestone structure.

**KYC Verification**: Mandatory identity verification for compliance.

**Sign Funding Agreement**: Review and execute the funding program contract.

**Acceptance Payment (10%, 10 days to complete — up to $4,500 USDC)**:
- Application approved — this repo: working circuit, Rust backend, zkVerify integration plan, FMD energy model, frontend

**Milestone 1: Live Deployment (10%, 55 days to complete — up to $4,500 USDC)**:
- Production deployment with fully functional zkVerify integration and proof verification
- Demo video showing user interaction and proof verification (1-5 minutes)
- Beta testing with proof verification validation
- Published documentation covering zkVerify integration and proof verification processes

**Deliverables:**
- Production Deployment Evidence: Application URL, zkVerify explorer links showing proof submissions, or API endpoints demonstrating verification
- Demo Video: 1-5 minute video demonstrating the application with zkVerify proof verification operational, with narration or subtitles
- Beta Testing Results: Testing reports, user feedback, or verification logs demonstrating successful proof generation and verification
- Technical Documentation: Published documentation covering zkVerify integration and proof verification processes

**Milestone 2: Initial Traction (30%, 100 days to complete — up to $13,500 USDC)**:
- Early traction metrics, choose one of the following:
  - Transaction Volume: 25,000+ ZK Proofs sent to zkVerify
  - Unique Users: 250+ unique addresses interacting with zkVerify integration

**Milestone 3: Scale (50%, 160 days to complete — up to $22,500 USDC)**:
- Choose one of the following:
  - Transaction Volume: 250,000+ ZK Proofs sent to zkVerify
  - Unique Users: 2,500+ unique addresses interacting with zkVerify integration

## References

The FMD physics energy model is adapted from the orkid workspace (private repo — access available for reviewers on request). The theoretical foundation is published as a preprint: ["Negative EV per Unit Time as Blockchain Inefficiency"](https://www.researchgate.net/publication/399474539_Negative_EV_per_Unit_Time_as_Blockchain_Inefficiency) by [Jacob Cavazos](https://www.researchgate.net/profile/Jacob-Cavazos).

- **Route energy formula**: `orkid/fmd-physics/src/route_energy.rs`
- **TPS committor function**: `orkid/fmd-physics/src/tps.rs`
- **Profit potential energy**: `orkid/fmd-physics/src/profit_potential.rs`

Blog posts establishing the thermodynamic framework (publicly available at [orkidlabs.com/blog](https://www.orkidlabs.com/blog/)):

- ["Blockchain Thermodynamics: How Negentropy Explains MEV, Consensus, and Arbitrage"](https://www.orkidlabs.com/blog/blockchain-thermodynamics-negentropy-mev-physics/) — Landauer's principle, Shannon entropy, negentropy extraction
- ["Negentropy = Information: A Generalized Mathematical Framework"](https://www.orkidlabs.com/blog/negentropy-information-generalized-framework/) — D_KL, Brillouin's negentropy principle
- ["A Formal Mathematical Model of Blockchain Negentropy and MEV Dynamics"](https://www.orkidlabs.com/blog/formal-negentropy-model-mev-dynamics-graph-diffusion/) — MEV closure equation, graph diffusion
- ["Complex Microstructure and Route Scoring in DeFi: Beyond Simple EV"](https://www.orkidlabs.com/blog/complex-microstructure-route-scoring-defi/) — Complex microstructure factor, phase conjugation, time-normalized scoring

## About

Built by [Orkid Labs](https://www.orkidlabs.com) — a privacy-first crypto
engineering lab building thermodynamic infrastructure for decentralized
systems. See our other work at [orkidlabs.com](https://www.orkidlabs.com).

## License

MIT
