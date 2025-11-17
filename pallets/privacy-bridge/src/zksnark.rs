//! zkSNARK Proof Generation and Verification
//!
//! This module handles:
//! - Proof generation (off-chain, by users)
//! - Proof verification (on-chain, by the pallet)
//! - Trusted setup parameter management

use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, PreparedVerifyingKey};
use ark_bn254::{Bn254, Fr as ScalarField}; // BN254 pairing-friendly curve
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use ark_std::rand::SeedableRng;
use ark_ff::PrimeField; // For from_le_bytes_mod_order
use rand_chacha::ChaCha20Rng;
use alloc::{vec::Vec, string::String, format};

use crate::circuit::PrivateTransferCircuit;

/// Serialized proof bytes (for storage/transmission)
pub type SerializedProof = Vec<u8>;

/// Serialized verifying key bytes
pub type SerializedVK = Vec<u8>;

/// Generate a proof for a private transfer
///
/// This runs off-chain (client-side) because proof generation is computationally expensive
///
/// Returns: Serialized proof bytes that can be sent in a transaction
pub fn generate_proof(
	proving_key: &ProvingKey<Bn254>,
	nullifier: Vec<u8>,
	commitment: Vec<u8>,
	amount: u128,
	asset_id: u32,
	randomness: [u8; 32],
	secret: [u8; 32],
) -> Result<SerializedProof, String> {
	// Create circuit with all inputs
	let circuit = PrivateTransferCircuit::new(
		nullifier,
		commitment,
		amount,
		asset_id,
		randomness,
		secret,
	);

	// Generate random coins for proof (deterministic in production)
	let mut rng = ChaCha20Rng::seed_from_u64(0u64);

	// Generate the proof!
	let proof = Groth16::<Bn254>::create_random_proof_with_reduction(circuit, proving_key, &mut rng)
		.map_err(|e| format!("Proof generation failed: {:?}", e))?;

	// Serialize proof to bytes
	let mut proof_bytes = Vec::new();
	proof.serialize_compressed(&mut proof_bytes)
		.map_err(|e| format!("Proof serialization failed: {:?}", e))?;

	Ok(proof_bytes)
}

/// Verify a proof on-chain
///
/// This is fast and can run in the blockchain runtime
///
/// Returns: true if proof is valid, false otherwise
pub fn verify_proof(
	verifying_key: &VerifyingKey<Bn254>,
	proof_bytes: &[u8],
	nullifier: &[u8],
	commitment: &[u8],
) -> Result<bool, String> {
	// Deserialize proof
	let proof = Proof::<Bn254>::deserialize_compressed(proof_bytes)
		.map_err(|e| format!("Proof deserialization failed: {:?}", e))?;

	// Prepare public inputs
	let mut public_inputs = Vec::new();

	// Convert nullifier bytes to field elements
	for chunk in nullifier.chunks(31) { // Field elements are ~31 bytes
		let mut bytes = [0u8; 32];
		bytes[..chunk.len()].copy_from_slice(chunk);
		public_inputs.push(ScalarField::from_le_bytes_mod_order(&bytes));
	}

	// Convert commitment bytes to field elements
	for chunk in commitment.chunks(31) {
		let mut bytes = [0u8; 32];
		bytes[..chunk.len()].copy_from_slice(chunk);
		public_inputs.push(ScalarField::from_le_bytes_mod_order(&bytes));
	}

	// Verify the proof!
	let pvk = PreparedVerifyingKey::from(verifying_key.clone());
	let is_valid = Groth16::<Bn254>::verify_proof(&pvk, &proof, &public_inputs)
		.map_err(|e| format!("Proof verification failed: {:?}", e))?;

	Ok(is_valid)
}

/// Generate trusted setup parameters (proving key + verifying key)
///
/// **WARNING:** This is a TRUSTED SETUP!
/// In production, use a multi-party computation (MPC) ceremony
/// For hackathon/demo, this simple version is fine
pub fn generate_setup_parameters() -> Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>), String> {

	// Create an empty circuit for setup
	let circuit = PrivateTransferCircuit::empty();

	// Generate random parameters
	let mut rng = ChaCha20Rng::seed_from_u64(12345u64); // Deterministic for testing

	// Run Groth16 setup
	let pk = Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, &mut rng)
		.map_err(|e| format!("Setup failed: {:?}", e))?;

	// Extract verifying key from proving key
	let vk = pk.vk.clone();

	Ok((pk, vk))
}

/// Serialize verifying key to bytes (for storage)
pub fn serialize_vk(vk: &VerifyingKey<Bn254>) -> Result<SerializedVK, String> {
	let mut bytes = Vec::new();
	vk.serialize_compressed(&mut bytes)
		.map_err(|e| format!("VK serialization failed: {:?}", e))?;
	Ok(bytes)
}

/// Deserialize verifying key from bytes
pub fn deserialize_vk(bytes: &[u8]) -> Result<VerifyingKey<Bn254>, String> {
	VerifyingKey::<Bn254>::deserialize_compressed(bytes)
		.map_err(|e| format!("VK deserialization failed: {:?}", e))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_proof_generation_and_verification() {
		use crate::simple_hash;

		// Generate setup parameters
		let (pk, vk) = generate_setup_parameters().unwrap();

		// Test data
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [1u8; 32];
		let secret = [2u8; 32];

		// Week 3: Generate commitment and nullifier using simple_hash
		let commitment_hash = simple_hash::generate_commitment(amount, asset_id, &randomness);
		let commitment = commitment_hash.as_bytes().to_vec();

		let nullifier_hash = simple_hash::generate_nullifier(&commitment_hash, &secret);
		let nullifier = nullifier_hash.as_bytes().to_vec();

		// Generate proof
		let proof_bytes = generate_proof(
			&pk,
			nullifier.clone(),
			commitment.clone(),
			amount,
			asset_id,
			randomness,
			secret,
		).unwrap();

		// Verify proof
		let is_valid = verify_proof(&vk, &proof_bytes, &nullifier, &commitment).unwrap();

		assert!(is_valid, "Proof should be valid!");
	}

	#[test]
	fn test_invalid_proof_rejected() {
		use crate::simple_hash;

		// Generate setup
		let (pk, vk) = generate_setup_parameters().unwrap();

		// Test data
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [1u8; 32];
		let secret = [2u8; 32];

		// Week 3: Generate commitment and nullifier using simple_hash
		let commitment_hash = simple_hash::generate_commitment(amount, asset_id, &randomness);
		let commitment = commitment_hash.as_bytes().to_vec();

		let nullifier_hash = simple_hash::generate_nullifier(&commitment_hash, &secret);
		let nullifier = nullifier_hash.as_bytes().to_vec();

		// Generate proof with correct inputs
		let proof_bytes = generate_proof(
			&pk,
			nullifier.clone(),
			commitment.clone(),
			amount,
			asset_id,
			randomness,
			secret,
		).unwrap();

		// Try to verify with WRONG commitment
		let wrong_commitment = vec![0u8; 32];
		let is_valid = verify_proof(&vk, &proof_bytes, &nullifier, &wrong_commitment).unwrap();

		assert!(!is_valid, "Invalid proof should be rejected!");
	}

	#[test]
	fn test_vk_serialization() {
		let (_, vk) = generate_setup_parameters().unwrap();

		// Serialize
		let bytes = serialize_vk(&vk).unwrap();

		// Deserialize
		let vk2 = deserialize_vk(&bytes).unwrap();

		// Should be equal
		assert_eq!(vk, vk2);
	}
}
