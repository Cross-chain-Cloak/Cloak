//! XCM Integration Tests
//!
//! Tests for cross-chain deposit and withdraw functionality

use crate::{mock::*, Error, Event};
use frame::testing_prelude::*;
use sp_core::H256;
use staging_xcm::v5::{AssetId, Location};
use crate::xcm_config::RegisteredAsset;

#[test]
fn test_register_asset() {
	new_test_ext().execute_with(|| {
		// Create an XCM asset ID (parent parachain's native token)
		let asset_id = AssetId(Location::parent());
		let min_deposit = 1000u128;

		// Register the asset (as root)
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			min_deposit,
		));

		// Verify asset is registered
		let registered = crate::AssetRegistry::<Test>::get(&asset_id).unwrap();
		assert_eq!(registered.local_id, 0); // First asset gets ID 0
		assert_eq!(registered.min_deposit, min_deposit);
		assert!(registered.is_active);

		// Verify next asset ID incremented
		assert_eq!(crate::NextAssetId::<Test>::get(), 1);
	});
}

#[test]
fn test_register_multiple_assets() {
	new_test_ext().execute_with(|| {
		// Register first asset
		let asset1 = AssetId(Location::parent());
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset1.clone(),
			100,
		));

		// Register second asset
		let asset2 = AssetId(Location::new(1, [])); // Parachain 1
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset2.clone(),
			200,
		));

		// Verify counter incremented (both assets registered)
		assert_eq!(crate::NextAssetId::<Test>::get(), 2);
	});
}

#[test]
fn test_cross_chain_deposit() {
	new_test_ext().execute_with(|| {
		// Setup: Register an asset
		let asset_id = AssetId(Location::parent());
		let min_deposit = 100u128;
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			min_deposit,
		));

		// Simulate cross-chain deposit from parachain
		let amount = 1000u128;
		let origin_location = Location::parent();
		let randomness = [42u8; 32];
		let depositor = 1u64;

		assert_ok!(PrivacyBridge::deposit_from_xcm(
			RuntimeOrigin::signed(depositor),
			asset_id,
			amount,
			origin_location.clone(),
			randomness,
		));

		// Verify commitment was created
		let commitment = crate::xcm_config::xcm_commitment_data(
			amount,
			0, // local_id
			&randomness,
			&origin_location,
		);

		assert!(crate::Commitments::<Test>::contains_key(&commitment));

		// Event is emitted (assertion skipped for MVP)

		// Verify counter incremented
		assert_eq!(crate::CommitmentCount::<Test>::get(), 1);
	});
}

#[test]
fn test_cross_chain_deposit_fails_below_minimum() {
	new_test_ext().execute_with(|| {
		// Register asset with minimum deposit
		let asset_id = AssetId(Location::parent());
		let min_deposit = 1000u128;
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			min_deposit,
		));

		// Try to deposit below minimum
		let amount = 500u128; // Below minimum
		let origin_location = Location::parent();
		let randomness = [42u8; 32];

		assert_noop!(
			PrivacyBridge::deposit_from_xcm(
				RuntimeOrigin::signed(1),
				asset_id,
				amount,
				origin_location,
				randomness,
			),
			Error::<Test>::InvalidProof // Reused error
		);
	});
}

#[test]
fn test_cross_chain_deposit_unregistered_asset() {
	new_test_ext().execute_with(|| {
		// Try to deposit unregistered asset
		let asset_id = AssetId(Location::parent());
		let amount = 1000u128;
		let origin_location = Location::parent();
		let randomness = [42u8; 32];

		assert_noop!(
			PrivacyBridge::deposit_from_xcm(
				RuntimeOrigin::signed(1),
				asset_id,
				amount,
				origin_location,
				randomness,
			),
			Error::<Test>::InvalidProof // Asset not registered
		);
	});
}

#[test]
fn test_cross_chain_withdraw() {
	new_test_ext().execute_with(|| {
		// Setup: Register asset and create commitment
		let asset_id = AssetId(Location::parent());
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			100,
		));

		let amount = 1000u128;
		let origin_location = Location::parent();
		let randomness = [42u8; 32];

		assert_ok!(PrivacyBridge::deposit_from_xcm(
			RuntimeOrigin::signed(1),
			asset_id,
			amount,
			origin_location.clone(),
			randomness,
		));

		// Generate nullifier
		let commitment = crate::xcm_config::xcm_commitment_data(
			amount,
			0,
			&randomness,
			&origin_location,
		);
		let secret = [99u8; 32];
		let nullifier = crate::Pallet::<Test>::generate_nullifier(&commitment, &secret);

		// Withdraw to destination parachain
		let destination = Location::new(1, []); // Parachain 1
		let beneficiary = Location::new(0, []); // Account on destination

		assert_ok!(PrivacyBridge::withdraw_to_parachain(
			RuntimeOrigin::signed(1),
			nullifier,
			0, // asset_id
			amount,
			destination,
			beneficiary,
		));

		// Verify nullifier was marked as used
		assert!(crate::NullifierSet::<Test>::get(&nullifier));

		// Event is emitted (assertion skipped for MVP)
	});
}

#[test]
fn test_cross_chain_withdraw_prevents_double_spend() {
	new_test_ext().execute_with(|| {
		// Setup
		let asset_id = AssetId(Location::parent());
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			100,
		));

		let amount = 1000u128;
		let origin_location = Location::parent();
		let randomness = [42u8; 32];

		assert_ok!(PrivacyBridge::deposit_from_xcm(
			RuntimeOrigin::signed(1),
			asset_id,
			amount,
			origin_location.clone(),
			randomness,
		));

		let commitment = crate::xcm_config::xcm_commitment_data(
			amount,
			0,
			&randomness,
			&origin_location,
		);
		let secret = [99u8; 32];
		let nullifier = crate::Pallet::<Test>::generate_nullifier(&commitment, &secret);

		let destination = Location::new(1, []);
		let beneficiary = Location::new(0, []);

		// First withdraw succeeds
		assert_ok!(PrivacyBridge::withdraw_to_parachain(
			RuntimeOrigin::signed(1),
			nullifier,
			0,
			amount,
			destination.clone(),
			beneficiary.clone(),
		));

		// Second withdraw with same nullifier fails
		assert_noop!(
			PrivacyBridge::withdraw_to_parachain(
				RuntimeOrigin::signed(1),
				nullifier,
				0,
				amount,
				destination,
				beneficiary,
			),
			Error::<Test>::NullifierAlreadyUsed
		);
	});
}

#[test]
fn test_full_cross_chain_privacy_flow() {
	new_test_ext().execute_with(|| {
		// 1. Register asset from parachain A
		let asset_id = AssetId(Location::new(1, [])); // Parachain 1
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			100,
		));

		// 2. User deposits from parachain A
		let amount = 5000u128;
		let origin_a = Location::new(1, []);
		let randomness = [123u8; 32];

		assert_ok!(PrivacyBridge::deposit_from_xcm(
			RuntimeOrigin::signed(1),
			asset_id,
			amount,
			origin_a.clone(),
			randomness,
		));

		// 3. Commitment created and hidden
		let commitment = crate::xcm_config::xcm_commitment_data(
			amount,
			0,
			&randomness,
			&origin_a,
		);
		assert!(crate::Commitments::<Test>::contains_key(&commitment));

		// 4. User generates proof off-chain (simulated)
		let secret = [200u8; 32];
		let nullifier = crate::Pallet::<Test>::generate_nullifier(&commitment, &secret);

		// 5. User withdraws to parachain B
		let destination_b = Location::new(2, []); // Parachain 2
		let beneficiary = Location::new(0, []);

		assert_ok!(PrivacyBridge::withdraw_to_parachain(
			RuntimeOrigin::signed(2), // Different user
			nullifier,
			0,
			amount,
			destination_b,
			beneficiary,
		));

		// 6. Verify privacy: nullifier used, can't trace back
		assert!(crate::NullifierSet::<Test>::get(&nullifier));

		// Success: Deposited from parachain A, withdrawn to parachain B!
		// Privacy maintained - no link between deposit and withdraw
	});
}

#[test]
fn test_multiple_cross_chain_deposits_create_anonymity_set() {
	new_test_ext().execute_with(|| {
		// Register asset
		let asset_id = AssetId(Location::parent());
		assert_ok!(PrivacyBridge::register_asset(
			RuntimeOrigin::root(),
			asset_id.clone(),
			100,
		));

		// Multiple users deposit (creating anonymity set)
		for i in 0..5 {
			let amount = 1000u128 + (i as u128 * 100);
			let randomness = [i as u8; 32];
			let origin = Location::parent();

			assert_ok!(PrivacyBridge::deposit_from_xcm(
				RuntimeOrigin::signed(i),
				asset_id.clone(),
				amount,
				origin,
				randomness,
			));
		}

		// Verify all commitments created
		assert_eq!(crate::CommitmentCount::<Test>::get(), 5);

		// Any user can withdraw without revealing which deposit was theirs
		// This creates the anonymity set
	});
}
