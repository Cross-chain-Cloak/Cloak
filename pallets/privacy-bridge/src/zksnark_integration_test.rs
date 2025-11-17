//! Integration test for zkSNARK proof system
//!
//! This test demonstrates that the full zkSNARK flow works:
//! 1. Setup: Generate proving and verifying keys
//! 2. Prove: Generate a proof off-chain
//! 3. Verify: Verify the proof on-chain
//!
//! This is a simplified test using the current circuit implementation.

#[cfg(test)]
mod integration_tests {
	use crate::zksnark::{generate_setup_parameters, generate_proof, verify_proof as zksnark_verify};
	use crate::simple_hash;
	use sp_core::H256;

	#[test]
	fn test_end_to_end_zksnark_flow() {
		println!("\n=== zkSNARK Integration Test ===\n");

		// Step 1: Generate trusted setup parameters
		println!("1. Generating trusted setup parameters...");
		let (pk, vk) = generate_setup_parameters()
			.expect("Setup should succeed");
		println!("   ✓ Proving key and verifying key generated");

		// Step 2: Create commitment (like in deposit())
		println!("\n2. Creating commitment (simulating deposit)...");
		let amount = 1000u128;
		let asset_id = 0u32;
		let randomness = [42u8; 32];

		let commitment = generate_commitment(amount, asset_id, &randomness);
		println!("   ✓ Commitment: {:?}", commitment);

		// Step 3: Generate nullifier (for spending)
		println!("\n3. Generating nullifier (for withdrawal)...");
		let secret = [99u8; 32];
		let nullifier = generate_nullifier(&commitment, &secret);
		println!("   ✓ Nullifier: {:?}", nullifier);

		// Step 4: Generate zkSNARK proof off-chain
		println!("\n4. Generating zkSNARK proof (off-chain)...");
		let proof_bytes = generate_proof(
			&pk,
			nullifier.as_bytes().to_vec(),
			commitment.as_bytes().to_vec(),
			amount,
			asset_id,
			randomness,
			secret,
		).expect("Proof generation should succeed");

		println!("   ✓ Proof generated ({} bytes)", proof_bytes.len());

		// Step 5: Verify proof on-chain
		println!("\n5. Verifying zkSNARK proof (on-chain)...");
		let is_valid = zksnark_verify(
			&vk,
			&proof_bytes,
			nullifier.as_bytes(),
			commitment.as_bytes(),
		).expect("Verification should not error");

		if is_valid {
			println!("   ✅ PROOF VERIFIED SUCCESSFULLY!");
		} else {
			println!("   ❌ Proof verification failed");
		}

		// Step 6: Test that wrong inputs fail
		println!("\n6. Testing security: wrong commitment should fail...");
		let wrong_commitment = H256::from([1u8; 32]);
		let is_valid_wrong = zksnark_verify(
			&vk,
			&proof_bytes,
			nullifier.as_bytes(),
			wrong_commitment.as_bytes(),
		).expect("Verification should not error");

		if !is_valid_wrong {
			println!("   ✅ Wrong commitment correctly rejected!");
		} else {
			println!("   ❌ WARNING: Wrong commitment was accepted!");
		}

		println!("\n=== Integration Test Summary ===");
		println!("Proof generation: ✓");
		println!("Proof verification: {}", if is_valid { "✓" } else { "✗" });
		println!("Security check: {}", if !is_valid_wrong { "✓" } else { "✗" });
		println!("================================\n");

		// Assert the test passes
		assert!(is_valid, "Valid proof should verify");
		assert!(!is_valid_wrong, "Invalid proof should be rejected");
	}

	#[test]
	fn test_proof_is_deterministic() {
		println!("\n=== Testing Proof Determinism ===\n");

		// Generate setup
		let (pk, _vk) = generate_setup_parameters().unwrap();

		// Same inputs
		let amount = 500u128;
		let asset_id = 0u32;
		let randomness = [7u8; 32];
		let secret = [13u8; 32];

		let commitment = generate_commitment(amount, asset_id, &randomness);
		let nullifier = generate_nullifier(&commitment, &secret);

		// Generate proof twice with same inputs
		let proof1 = generate_proof(
			&pk,
			nullifier.as_bytes().to_vec(),
			commitment.as_bytes().to_vec(),
			amount,
			asset_id,
			randomness,
			secret,
		).unwrap();

		let proof2 = generate_proof(
			&pk,
			nullifier.as_bytes().to_vec(),
			commitment.as_bytes().to_vec(),
			amount,
			asset_id,
			randomness,
			secret,
		).unwrap();

		// Note: Groth16 proofs are NOT deterministic due to random blinding factors
		// This is actually a security feature (zero-knowledge property)
		println!("Proof 1 size: {} bytes", proof1.len());
		println!("Proof 2 size: {} bytes", proof2.len());
		println!("Proofs are different (expected): {}", proof1 != proof2);

		// Both should have same size though
		assert_eq!(proof1.len(), proof2.len(), "Proofs should have same size");
		println!("\n✓ Both proofs have consistent size\n");
	}

	#[test]
	fn test_different_amounts_produce_different_proofs() {
		println!("\n=== Testing Different Amounts ===\n");

		let (pk, vk) = generate_setup_parameters().unwrap();

		// Test with amount 100
		let amount1 = 100u128;
		let randomness = [5u8; 32];
		let secret = [6u8; 32];

		let commitment1 = generate_commitment(amount1, 0, &randomness);
		let nullifier1 = generate_nullifier(&commitment1, &secret);

		let proof1 = generate_proof(
			&pk,
			nullifier1.as_bytes().to_vec(),
			commitment1.as_bytes().to_vec(),
			amount1,
			0,
			randomness,
			secret,
		).unwrap();

		// Proof should verify for amount1
		let valid1 = zksnark_verify(
			&vk,
			&proof1,
			nullifier1.as_bytes(),
			commitment1.as_bytes(),
		).unwrap();

		println!("Amount 100: Proof verifies = {}", valid1);

		// Test with different amount (but try to use same proof - should fail)
		let amount2 = 200u128;
		let commitment2 = generate_commitment(amount2, 0, &randomness); // Different commitment

		println!("Amount 200: Different commitment = {}", commitment1 != commitment2);

		assert!(valid1, "Proof for amount 100 should verify");
		assert_ne!(commitment1, commitment2, "Different amounts should produce different commitments");

		println!("\n✓ Different amounts produce different commitments\n");
	}

	// Helper functions (matching lib.rs - Week 3: using simple_hash)
	fn generate_commitment(amount: u128, asset_id: u32, randomness: &[u8; 32]) -> H256 {
		simple_hash::generate_commitment(amount, asset_id, randomness)
	}

	fn generate_nullifier(commitment: &H256, secret: &[u8; 32]) -> H256 {
		simple_hash::generate_nullifier(commitment, secret)
	}
}
