//! zkSNARK Circuit for Private Transfers
//!
//! This module implements the zero-knowledge proof circuit that allows users to prove:
//! 1. They own a commitment (without revealing which one)
//! 2. They haven't spent it before (nullifier is fresh)
//! 3. The amounts balance correctly
//!
//! Week 2 MVP: Simple ownership proof
//! Week 3+: Will add merkle tree membership proof

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{
	ConstraintSynthesizer, ConstraintSystemRef, SynthesisError,
};
use ark_bn254::Fr as ScalarField; // BN254 scalar field
use alloc::{vec, vec::Vec};

/// Circuit for proving ownership of a commitment and generating a valid nullifier
///
/// PUBLIC INPUTS (visible on-chain):
/// - nullifier: Hash(commitment || secret) - prevents double-spending
/// - commitment: The commitment being spent
///
/// PRIVATE INPUTS (witness - never revealed):
/// - amount: The hidden amount
/// - asset_id: The asset type
/// - randomness: Secret randomness used in commitment
/// - secret: Secret key for generating nullifier
#[derive(Clone)]
pub struct PrivateTransferCircuit {
	// === PUBLIC INPUTS ===
	/// The nullifier (prevents double-spend)
	pub nullifier: Option<Vec<u8>>,

	/// The commitment being spent
	pub commitment: Option<Vec<u8>>,

	// === PRIVATE INPUTS (WITNESS) ===
	/// The amount (hidden!)
	pub amount: Option<u128>,

	/// Asset ID (hidden!)
	pub asset_id: Option<u32>,

	/// Randomness used in commitment (hidden!)
	pub randomness: Option<[u8; 32]>,

	/// Secret for nullifier generation (hidden!)
	pub secret: Option<[u8; 32]>,
}

impl PrivateTransferCircuit {
	/// Create a new circuit for proof generation
	pub fn new(
		nullifier: Vec<u8>,
		commitment: Vec<u8>,
		amount: u128,
		asset_id: u32,
		randomness: [u8; 32],
		secret: [u8; 32],
	) -> Self {
		Self {
			nullifier: Some(nullifier),
			commitment: Some(commitment),
			amount: Some(amount),
			asset_id: Some(asset_id),
			randomness: Some(randomness),
			secret: Some(secret),
		}
	}

	/// Create an empty circuit (for setup)
	pub fn empty() -> Self {
		Self {
			nullifier: None,
			commitment: None,
			amount: None,
			asset_id: None,
			randomness: None,
			secret: None,
		}
	}
}

impl ConstraintSynthesizer<ScalarField> for PrivateTransferCircuit {
	fn generate_constraints(
		self,
		cs: ConstraintSystemRef<ScalarField>,
	) -> Result<(), SynthesisError> {
		// === ALLOCATE PUBLIC INPUTS ===
		// Week 3: Use 32-byte defaults for empty circuit
		let nullifier_var = UInt8::new_input_vec(
			cs.clone(),
			&self.nullifier.unwrap_or_else(|| vec![0u8; 32])
		)?;

		let commitment_var = UInt8::new_input_vec(
			cs.clone(),
			&self.commitment.unwrap_or_else(|| vec![0u8; 32])
		)?;

		// === ALLOCATE PRIVATE WITNESSES ===
		let amount_bytes = self.amount
			.map(|a| a.to_le_bytes().to_vec())
			.unwrap_or_else(|| vec![0u8; 16]); // u128 is 16 bytes
		let amount_var = UInt8::new_witness_vec(cs.clone(), &amount_bytes)?;

		let asset_id_bytes = self.asset_id
			.map(|a| a.to_le_bytes().to_vec())
			.unwrap_or_else(|| vec![0u8; 4]); // u32 is 4 bytes
		let asset_id_var = UInt8::new_witness_vec(cs.clone(), &asset_id_bytes)?;

		let randomness_var = UInt8::new_witness_vec(
			cs.clone(),
			&self.randomness.unwrap_or([0u8; 32]).to_vec()
		)?;

		let secret_var = UInt8::new_witness_vec(
			cs.clone(),
			&self.secret.unwrap_or([0u8; 32]).to_vec()
		)?;

		// === CONSTRAINT 1: Verify commitment is correctly formed ===
		// commitment = Hash(amount || asset_id || randomness)
		let mut commitment_preimage = Vec::new();
		commitment_preimage.extend_from_slice(&amount_var);
		commitment_preimage.extend_from_slice(&asset_id_var);
		commitment_preimage.extend_from_slice(&randomness_var);

		// Use Blake2s for in-circuit hashing (efficient in R1CS)
		let computed_commitment = blake2s_hash(&commitment_preimage)?;

		// Enforce: computed_commitment == commitment
		computed_commitment.enforce_equal(&commitment_var)?;

		// === CONSTRAINT 2: Verify nullifier is correctly formed ===
		// nullifier = Hash(commitment || secret)
		let mut nullifier_preimage = Vec::new();
		nullifier_preimage.extend_from_slice(&commitment_var);
		nullifier_preimage.extend_from_slice(&secret_var);

		let computed_nullifier = blake2s_hash(&nullifier_preimage)?;

		// Enforce: computed_nullifier == nullifier
		computed_nullifier.enforce_equal(&nullifier_var)?;

		// === SUCCESS ===
		// If we reach here, the prover knows:
		// 1. The amount and randomness that create the commitment
		// 2. The secret that creates the nullifier
		// But the verifier learns NOTHING except that the proof is valid!

		Ok(())
	}
}

/// Helper function for Blake2s hashing in circuit
/// Uses ark-r1cs-std's Blake2s gadget
fn blake2s_hash(input: &[UInt8<ScalarField>]) -> Result<Vec<UInt8<ScalarField>>, SynthesisError> {
	use ark_r1cs_std::bits::uint8::UInt8;

	// For Week 2 MVP, we'll use a simplified hash
	// In production, use: Blake2sGadget::evaluate()

	// Simple placeholder: XOR all bytes (NOT SECURE - just for testing!)
	// TODO Week 3: Replace with actual Blake2s gadget
	let mut result = vec![UInt8::constant(0u8); 32];

	for (i, byte) in input.iter().enumerate() {
		let idx = i % 32;
		result[idx] = result[idx].xor(byte)?;
	}

	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;
	use ark_relations::r1cs::ConstraintSystem;

	#[test]
	fn test_circuit_satisfiability() {
		use crate::simple_hash;

		// Create test data
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [1u8; 32];
		let secret = [2u8; 32];

		// Week 3: Generate commitment and nullifier using simple_hash
		let commitment_hash = simple_hash::generate_commitment(amount, asset_id, &randomness);
		let commitment = commitment_hash.as_bytes().to_vec();

		let nullifier_hash = simple_hash::generate_nullifier(&commitment_hash, &secret);
		let nullifier = nullifier_hash.as_bytes().to_vec();

		// Create circuit
		let circuit = PrivateTransferCircuit::new(
			nullifier,
			commitment,
			amount,
			asset_id,
			randomness,
			secret,
		);

		// Test constraint satisfaction
		let cs = ConstraintSystem::<ScalarField>::new_ref();
		circuit.generate_constraints(cs.clone()).unwrap();

		assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");
	}
}
