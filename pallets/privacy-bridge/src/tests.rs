use crate::{mock::*, Error, Pallet, CommitmentCount, Commitments, NullifierSet};
use frame::testing_prelude::*;
use sp_core::H256;

#[test]
fn deposit_creates_commitment() {
	new_test_ext().execute_with(|| {
		// Test user account
		let user = 1u64;
		let amount = 100u128;
		let asset_id = 0u32; // Native token
		let randomness = [1u8; 32];

		// Deposit should succeed
		assert_ok!(PrivacyBridge::deposit(
			RuntimeOrigin::signed(user),
			amount,
			asset_id,
			randomness
		));

		// Verify commitment was created
		let commitment = Pallet::<Test>::generate_commitment(amount, asset_id, &randomness);
		assert!(Commitments::<Test>::contains_key(&commitment));

		// Verify commitment count increased
		assert_eq!(CommitmentCount::<Test>::get(), 1);

		// Verify commitment metadata
		let commitment_data = Commitments::<Test>::get(&commitment).unwrap();
		assert_eq!(commitment_data.depositor, user);
		assert_eq!(commitment_data.asset_id, asset_id);
	});
}

#[test]
fn deposit_fails_for_duplicate_commitment() {
	new_test_ext().execute_with(|| {
		let user = 1u64;
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [1u8; 32];

		// First deposit succeeds
		assert_ok!(PrivacyBridge::deposit(
			RuntimeOrigin::signed(user),
			amount,
			asset_id,
			randomness
		));

		// Second deposit with same parameters should fail
		assert_noop!(
			PrivacyBridge::deposit(
				RuntimeOrigin::signed(user),
				amount,
				asset_id,
				randomness
			),
			Error::<Test>::CommitmentAlreadyExists
		);
	});
}

#[test]
fn commitment_hides_amount() {
	new_test_ext().execute_with(|| {
		let user = 1u64;
		let amount1 = 100u128;
		let amount2 = 200u128;
		let asset_id = 0u32;
		let randomness1 = [1u8; 32];
		let randomness2 = [2u8; 32];

		// Deposit two different amounts
		assert_ok!(PrivacyBridge::deposit(
			RuntimeOrigin::signed(user),
			amount1,
			asset_id,
			randomness1
		));

		assert_ok!(PrivacyBridge::deposit(
			RuntimeOrigin::signed(user),
			amount2,
			asset_id,
			randomness2
		));

		// Generate commitments
		let commitment1 = Pallet::<Test>::generate_commitment(amount1, asset_id, &randomness1);
		let commitment2 = Pallet::<Test>::generate_commitment(amount2, asset_id, &randomness2);

		// Commitments should be different
		assert_ne!(commitment1, commitment2);

		// But on-chain metadata doesn't reveal amounts
		let data1 = Commitments::<Test>::get(&commitment1).unwrap();
		let data2 = Commitments::<Test>::get(&commitment2).unwrap();

		// Both have same asset_id but amounts are hidden
		assert_eq!(data1.asset_id, data2.asset_id);
	});
}

#[test]
fn withdraw_marks_nullifier_as_used() {
	new_test_ext().execute_with(|| {
		let user = 1u64;
		let amount = 100u128;
		let asset_id = 0u32;
		let nullifier = H256::from([3u8; 32]);

		// Withdraw should succeed
		assert_ok!(PrivacyBridge::withdraw(
			RuntimeOrigin::signed(user),
			nullifier,
			amount,
			asset_id
		));

		// Verify nullifier was marked as used
		assert!(NullifierSet::<Test>::get(&nullifier));
	});
}

#[test]
fn withdraw_fails_for_used_nullifier() {
	new_test_ext().execute_with(|| {
		let user = 1u64;
		let amount = 100u128;
		let asset_id = 0u32;
		let nullifier = H256::from([3u8; 32]);

		// First withdraw succeeds
		assert_ok!(PrivacyBridge::withdraw(
			RuntimeOrigin::signed(user),
			nullifier,
			amount,
			asset_id
		));

		// Second withdraw with same nullifier should fail (double-spend prevention)
		assert_noop!(
			PrivacyBridge::withdraw(
				RuntimeOrigin::signed(user),
				nullifier,
				amount,
				asset_id
			),
			Error::<Test>::NullifierAlreadyUsed
		);
	});
}

#[test]
fn generate_commitment_is_deterministic() {
	new_test_ext().execute_with(|| {
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [5u8; 32];

		let commitment1 = Pallet::<Test>::generate_commitment(amount, asset_id, &randomness);
		let commitment2 = Pallet::<Test>::generate_commitment(amount, asset_id, &randomness);

		// Same inputs should produce same commitment
		assert_eq!(commitment1, commitment2);
	});
}

#[test]
fn generate_nullifier_is_deterministic() {
	new_test_ext().execute_with(|| {
		let commitment = H256::from([1u8; 32]);
		let secret = [2u8; 32];

		let nullifier1 = Pallet::<Test>::generate_nullifier(&commitment, &secret);
		let nullifier2 = Pallet::<Test>::generate_nullifier(&commitment, &secret);

		// Same inputs should produce same nullifier
		assert_eq!(nullifier1, nullifier2);
	});
}

#[test]
fn different_randomness_produces_different_commitments() {
	new_test_ext().execute_with(|| {
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness1 = [1u8; 32];
		let randomness2 = [2u8; 32];

		let commitment1 = Pallet::<Test>::generate_commitment(amount, asset_id, &randomness1);
		let commitment2 = Pallet::<Test>::generate_commitment(amount, asset_id, &randomness2);

		// Different randomness should produce different commitments even with same amount
		assert_ne!(commitment1, commitment2);
	});
}

#[test]
fn multiple_deposits_increment_counter() {
	new_test_ext().execute_with(|| {
		let user = 1u64;
		let amount = 100u128;
		let asset_id = 0u32;

		// Deposit 3 times with different randomness
		for i in 0..3 {
			let mut randomness = [0u8; 32];
			randomness[0] = i as u8;

			assert_ok!(PrivacyBridge::deposit(
				RuntimeOrigin::signed(user),
				amount,
				asset_id,
				randomness
			));
		}

		// Counter should be 3
		assert_eq!(CommitmentCount::<Test>::get(), 3);
	});
}

#[test]
fn full_deposit_withdraw_cycle() {
	new_test_ext().execute_with(|| {
		let user = 1u64;
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [7u8; 32];

		// Step 1: Deposit
		assert_ok!(PrivacyBridge::deposit(
			RuntimeOrigin::signed(user),
			amount,
			asset_id,
			randomness
		));

		let commitment = Pallet::<Test>::generate_commitment(amount, asset_id, &randomness);
		assert!(Commitments::<Test>::contains_key(&commitment));

		// Step 2: Generate nullifier (user would do this off-chain)
		let secret = [8u8; 32];
		let nullifier = Pallet::<Test>::generate_nullifier(&commitment, &secret);

		// Step 3: Withdraw
		assert_ok!(PrivacyBridge::withdraw(
			RuntimeOrigin::signed(user),
			nullifier,
			amount,
			asset_id
		));

		// Verify nullifier is used
		assert!(NullifierSet::<Test>::get(&nullifier));

		// Commitment should still exist (it's never deleted)
		assert!(Commitments::<Test>::contains_key(&commitment));
	});
}
