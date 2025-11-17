//! Simple Hash Function for zkSNARK Compatibility
//!
//! This module provides a simple, deterministic hash function that works
//! consistently in both:
//! - On-chain runtime (Substrate pallet)
//! - Off-chain zkSNARK circuit (Arkworks R1CS)
//!
//! **IMPORTANT**: This is a simplified hash for hackathon/MVP purposes.
//! In production, this should be replaced with:
//! - Poseidon hash (designed for zkSNARKs)
//! - Blake2s with proper R1CS gadget
//! - Or another zkSNARK-friendly hash function
//!
//! The current implementation uses XOR-based hashing which is:
//! ✅ Simple and deterministic
//! ✅ Works in both environments
//! ✅ Fast to compute
//! ❌ NOT cryptographically secure
//! ❌ Vulnerable to collision attacks
//! ❌ Should NOT be used in production

use sp_core::H256;
use alloc::vec::Vec;

/// Simple hash function using XOR
///
/// This creates a deterministic 32-byte hash by XORing input bytes.
/// Each input byte is XORed into the corresponding position (mod 32).
///
/// Example:
/// ```
/// use pallet_privacy_bridge::simple_hash::simple_hash_bytes;
///
/// let data = b"hello world";
/// let hash = simple_hash_bytes(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn simple_hash_bytes(data: &[u8]) -> [u8; 32] {
	let mut result = [0u8; 32];

	for (i, byte) in data.iter().enumerate() {
		result[i % 32] ^= byte;
	}

	result
}

/// Simple hash returning H256 (for Substrate compatibility)
pub fn simple_hash(data: &[u8]) -> H256 {
	H256::from(simple_hash_bytes(data))
}

/// Generate commitment using simple hash
///
/// Commitment = Hash(amount || asset_id || randomness)
pub fn generate_commitment(amount: u128, asset_id: u32, randomness: &[u8; 32]) -> H256 {
	let mut data = Vec::new();
	data.extend_from_slice(&amount.to_le_bytes());
	data.extend_from_slice(&asset_id.to_le_bytes());
	data.extend_from_slice(randomness);

	simple_hash(&data)
}

/// Generate nullifier using simple hash
///
/// Nullifier = Hash(commitment || secret)
pub fn generate_nullifier(commitment: &H256, secret: &[u8; 32]) -> H256 {
	let mut data = Vec::new();
	data.extend_from_slice(commitment.as_bytes());
	data.extend_from_slice(secret);

	simple_hash(&data)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple_hash_deterministic() {
		let data = b"test data";
		let hash1 = simple_hash(data);
		let hash2 = simple_hash(data);
		assert_eq!(hash1, hash2, "Hash should be deterministic");
	}

	#[test]
	fn test_simple_hash_different_inputs() {
		let hash1 = simple_hash(b"data1");
		let hash2 = simple_hash(b"data2");
		assert_ne!(hash1, hash2, "Different inputs should produce different hashes");
	}

	#[test]
	fn test_commitment_generation() {
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [42u8; 32];

		let commitment1 = generate_commitment(amount, asset_id, &randomness);
		let commitment2 = generate_commitment(amount, asset_id, &randomness);

		assert_eq!(commitment1, commitment2, "Commitment generation should be deterministic");
	}

	#[test]
	fn test_different_amounts_different_commitments() {
		let randomness = [42u8; 32];

		let commitment1 = generate_commitment(100, 0, &randomness);
		let commitment2 = generate_commitment(200, 0, &randomness);

		assert_ne!(commitment1, commitment2, "Different amounts should produce different commitments");
	}

	#[test]
	fn test_nullifier_generation() {
		let commitment = H256::from([1u8; 32]);
		let secret = [2u8; 32];

		let nullifier1 = generate_nullifier(&commitment, &secret);
		let nullifier2 = generate_nullifier(&commitment, &secret);

		assert_eq!(nullifier1, nullifier2, "Nullifier generation should be deterministic");
	}

	#[test]
	fn test_different_secrets_different_nullifiers() {
		let commitment = H256::from([1u8; 32]);

		let nullifier1 = generate_nullifier(&commitment, &[2u8; 32]);
		let nullifier2 = generate_nullifier(&commitment, &[3u8; 32]);

		assert_ne!(nullifier1, nullifier2, "Different secrets should produce different nullifiers");
	}
}
