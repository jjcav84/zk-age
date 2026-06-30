// zk-age: Age verification circuit
//
// Proves that a person is at least `threshold` years old without revealing
// their actual birthdate. The issuer (e.g., a government ID authority)
// signs a commitment to the user's birthdate. The user proves:
//
//   1. They possess a valid signed birthdate commitment
//   2. current_year - birth_year >= threshold
//
// Public inputs:  current_year, threshold, issuer_pubkey_hash
// Private inputs: birth_year, issuer_signature, signature_randomness
//
// The signature scheme is a simplified Schnorr-like commitment for demo
// purposes. A production system would use Poseidon-based signatures or
// EdDSA on BabyJubJub.

pragma circom 2.2.3;

include "node_modules/circomlib/circuits/comparators.circom";

// Age verification circuit
// Proves: current_year - birth_year >= threshold
// Without revealing: birth_year
template AgeVerify() {
    // Public inputs
    signal input current_year;
    signal input threshold;
    signal input issuer_pubkey_hash;

    // Private inputs
    signal input birth_year;
    signal input issuer_signature;
    signal input signature_randomness;

    // --- Constraint 1: Age check ---
    // age = current_year - birth_year
    // We need age >= threshold, i.e., current_year - birth_year >= threshold
    // Equivalently: current_year - threshold >= birth_year
    // We prove this with a non-negative difference:
    //   diff = current_year - threshold - birth_year
    //   diff >= 0  (enforced via bit decomposition)

    signal diff;
    diff <== current_year - threshold - birth_year;

    // Enforce diff >= 0 by requiring it fits in 16 bits
    // (max age we support is 65535, more than enough)
    component n2b = Num2Bits(16);
    n2b.in <== diff;

    // --- Constraint 2: Signature verification (simplified) ---
    // In a real system, this would verify a Poseidon hash signature.
    // For demo: we check that issuer_signature == Poseidon(birth_year, issuer_pubkey_hash, randomness)
    // Here we use a simplified constraint: the signature must equal
    // a hash of (birth_year, issuer_pubkey_hash, randomness).
    //
    // Using circomlib's Poseidon:
    // signal expected_sig <== Poseidon(3)([birth_year, issuer_pubkey_hash, signature_randomness]);
    // expected_sig === issuer_signature;
    //
    // For the demo without circomlib Poseidon, we use a simple algebraic check:
    // This is NOT secure — it's a placeholder. The circuit structure is correct;
    // swapping in Poseidon is a one-line change.
    signal expected_sig;
    expected_sig <== birth_year + issuer_pubkey_hash * signature_randomness;
    expected_sig === issuer_signature;
}

// Main component
component main { public [current_year, threshold, issuer_pubkey_hash] } = AgeVerify();
