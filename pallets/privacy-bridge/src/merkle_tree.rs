//! Simple Incremental Merkle Tree for Commitment Anonymity
//!
//! This module provides a simple incremental merkle tree implementation
//! for creating an anonymity set of commitments. When a user withdraws,
//! they prove their commitment exists in the tree without revealing which one.
//!
//! ## Design (Week 3 - Hackathon MVP)
//!
//! - **Tree Depth**: 20 (supports 2^20 = ~1 million commitments)
//! - **Hash Function**: simple_hash (matches circuit implementation)
//! - **Construction**: Incremental (append-only, no deletions)
//! - **Storage**: Only store leaf commitments + computed root
//!
//! ## Production Improvements Needed
//!
//! - Use Poseidon hash for better zkSNARK efficiency
//! - Implement full sparse merkle tree for better privacy
//! - Add merkle proof caching/optimization
//! - Consider using existing libraries like `rs-merkle`

use sp_core::H256;
use alloc::vec::Vec;
use crate::simple_hash::simple_hash_bytes;

/// Tree depth (20 levels = 2^20 = ~1 million leaves)
pub const TREE_DEPTH: usize = 20;

/// Calculate parent hash from two children
pub fn hash_pair(left: &H256, right: &H256) -> H256 {
	let mut data = Vec::new();
	data.extend_from_slice(left.as_bytes());
	data.extend_from_slice(right.as_bytes());

	let hash = simple_hash_bytes(&data);
	H256::from(hash)
}

/// Calculate the merkle root from a list of leaf commitments
///
/// Uses incremental construction: fills remaining slots with zero hashes
pub fn calculate_root(leaves: &[H256]) -> H256 {
	if leaves.is_empty() {
		return H256::zero();
	}

	// Start with the leaves
	let mut current_level = leaves.to_vec();

	// Build tree level by level
	for _level in 0..TREE_DEPTH {
		if current_level.len() == 1 {
			return current_level[0];
		}

		let mut next_level = Vec::new();

		// Process pairs
		for i in (0..current_level.len()).step_by(2) {
			let left = current_level[i];
			let right = if i + 1 < current_level.len() {
				current_level[i + 1]
			} else {
				H256::zero() // Pad with zero if odd number
			};

			next_level.push(hash_pair(&left, &right));
		}

		current_level = next_level;
	}

	// Should have single root
	current_level[0]
}

/// Generate a merkle proof for a specific leaf
///
/// Returns the sibling hashes needed to recompute the root
pub fn generate_proof(leaves: &[H256], leaf_index: usize) -> Result<Vec<H256>, &'static str> {
	if leaf_index >= leaves.len() {
		return Err("Leaf index out of bounds");
	}

	let mut proof = Vec::new();
	let mut current_level = leaves.to_vec();
	let mut current_index = leaf_index;

	// Build proof by collecting siblings at each level
	for _level in 0..TREE_DEPTH {
		if current_level.len() == 1 {
			break;
		}

		// Get sibling index
		let sibling_index = if current_index % 2 == 0 {
			current_index + 1
		} else {
			current_index - 1
		};

		// Get sibling value (or zero if doesn't exist)
		let sibling = if sibling_index < current_level.len() {
			current_level[sibling_index]
		} else {
			H256::zero()
		};

		proof.push(sibling);

		// Move to next level
		let mut next_level = Vec::new();
		for i in (0..current_level.len()).step_by(2) {
			let left = current_level[i];
			let right = if i + 1 < current_level.len() {
				current_level[i + 1]
			} else {
				H256::zero()
			};
			next_level.push(hash_pair(&left, &right));
		}

		current_level = next_level;
		current_index /= 2;
	}

	Ok(proof)
}

/// Verify a merkle proof
///
/// Recomputes the root using the leaf and proof, returns true if it matches expected_root
pub fn verify_proof(
	leaf: &H256,
	proof: &[H256],
	leaf_index: usize,
	expected_root: &H256,
) -> bool {
	let mut current_hash = *leaf;
	let mut current_index = leaf_index;

	// Recompute root using proof
	for sibling in proof {
		current_hash = if current_index % 2 == 0 {
			// We're on the left, sibling on the right
			hash_pair(&current_hash, sibling)
		} else {
			// We're on the right, sibling on the left
			hash_pair(sibling, &current_hash)
		};

		current_index /= 2;
	}

	&current_hash == expected_root
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hash_pair() {
		let left = H256::from([1u8; 32]);
		let right = H256::from([2u8; 32]);
		let hash = hash_pair(&left, &right);

		// Should be deterministic
		assert_eq!(hash, hash_pair(&left, &right));

		// Note: XOR hash is commutative, so hash(a,b) == hash(b,a)
		// This is a known limitation of simple_hash for MVP
		// Production should use Poseidon or other non-commutative hash
		// For now, we ensure ordering in the merkle tree construction
	}

	#[test]
	fn test_calculate_root_single_leaf() {
		let leaves = vec![H256::from([1u8; 32])];
		let root = calculate_root(&leaves);
		assert_eq!(root, leaves[0]);
	}

	#[test]
	fn test_calculate_root_two_leaves() {
		let leaves = vec![
			H256::from([1u8; 32]),
			H256::from([2u8; 32]),
		];
		let root = calculate_root(&leaves);
		let expected = hash_pair(&leaves[0], &leaves[1]);
		assert_eq!(root, expected);
	}

	#[test]
	fn test_calculate_root_multiple_leaves() {
		let leaves = vec![
			H256::from([1u8; 32]),
			H256::from([2u8; 32]),
			H256::from([3u8; 32]),
			H256::from([4u8; 32]),
		];
		let root = calculate_root(&leaves);

		// Manually compute expected root
		let h01 = hash_pair(&leaves[0], &leaves[1]);
		let h23 = hash_pair(&leaves[2], &leaves[3]);
		let expected = hash_pair(&h01, &h23);

		assert_eq!(root, expected);
	}

	#[test]
	fn test_generate_and_verify_proof() {
		let leaves = vec![
			H256::from([1u8; 32]),
			H256::from([2u8; 32]),
			H256::from([3u8; 32]),
			H256::from([4u8; 32]),
		];

		let root = calculate_root(&leaves);

		// Generate proof for each leaf and verify
		for (i, leaf) in leaves.iter().enumerate() {
			let proof = generate_proof(&leaves, i).unwrap();
			assert!(verify_proof(leaf, &proof, i, &root), "Proof should verify for leaf {}", i);
		}
	}

	#[test]
	fn test_verify_proof_fails_for_wrong_leaf() {
		let leaves = vec![
			H256::from([1u8; 32]),
			H256::from([2u8; 32]),
		];

		let root = calculate_root(&leaves);
		let proof = generate_proof(&leaves, 0).unwrap();

		// Try to verify with wrong leaf
		let wrong_leaf = H256::from([99u8; 32]);
		assert!(!verify_proof(&wrong_leaf, &proof, 0, &root), "Proof should fail for wrong leaf");
	}

	#[test]
	fn test_verify_proof_fails_for_wrong_root() {
		let leaves = vec![
			H256::from([1u8; 32]),
			H256::from([2u8; 32]),
		];

		let root = calculate_root(&leaves);
		let proof = generate_proof(&leaves, 0).unwrap();

		// Try to verify with wrong root
		let wrong_root = H256::from([99u8; 32]);
		assert!(!verify_proof(&leaves[0], &proof, 0, &wrong_root), "Proof should fail for wrong root");
	}

	#[test]
	fn test_incremental_root_updates() {
		// Test that adding leaves incrementally works correctly
		let leaf1 = H256::from([1u8; 32]);
		let leaf2 = H256::from([2u8; 32]);
		let leaf3 = H256::from([3u8; 32]);

		let root1 = calculate_root(&[leaf1]);
		let root2 = calculate_root(&[leaf1, leaf2]);
		let root3 = calculate_root(&[leaf1, leaf2, leaf3]);

		// Roots should be different
		assert_ne!(root1, root2);
		assert_ne!(root2, root3);
		assert_ne!(root1, root3);

		// Proofs for earlier leaves should still verify
		let proof_leaf1_in_tree3 = generate_proof(&[leaf1, leaf2, leaf3], 0).unwrap();
		assert!(verify_proof(&leaf1, &proof_leaf1_in_tree3, 0, &root3));
	}
}
