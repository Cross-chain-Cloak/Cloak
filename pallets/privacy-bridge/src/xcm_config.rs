//! XCM Configuration for Privacy Bridge
//!
//! Week 4: Cross-chain integration using XCM (Cross-Consensus Messaging)
//!
//! This module provides XCM support for:
//! - Receiving deposits from other parachains
//! - Sending withdrawals to destination parachains
//! - Multi-asset support across the Polkadot ecosystem
//!
//! ## Architecture
//!
//! ```text
//! Parachain A ──[XCM]──> Privacy Bridge ──[XCM]──> Parachain B
//!                           │
//!                           ├─ Asset Registry (MultiAsset -> local ID)
//!                           ├─ Commitments (shielded)
//!                           └─ Nullifiers (prevent double-spend)
//! ```
//!
//! ## MVP Simplifications (Hackathon)
//!
//! - Support Reserve Transfers only (most common pattern)
//! - Simple asset ID mapping (no complex conversions)
//! - Basic XCM message construction
//! - Mock testing (no actual parachain deployment)

use frame::prelude::*;
use sp_core::H256;
use staging_xcm::v5::{Asset as XcmAsset, AssetId, Location, Fungibility};

/// Asset registry entry
/// Maps XCM MultiAsset to local asset ID for privacy operations
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, RuntimeDebug)]
pub struct RegisteredAsset {
	/// The XCM asset identifier
	pub asset_id: AssetId,
	/// Local asset ID used in commitments
	pub local_id: u32,
	/// Minimum deposit amount
	pub min_deposit: u128,
	/// Whether asset is active
	pub is_active: bool,
}

impl RegisteredAsset {
	pub fn new(asset_id: AssetId, local_id: u32) -> Self {
		Self {
			asset_id,
			local_id,
			min_deposit: 0,
			is_active: true,
		}
	}
}

/// Helper to extract amount from XCM Asset
pub fn extract_asset_amount(asset: &XcmAsset) -> Option<u128> {
	match &asset.fun {
		Fungibility::Fungible(amount) => Some(*amount),
		Fungibility::NonFungible(_) => None, // We don't support NFTs for MVP
	}
}

/// Helper to construct XCM MultiAsset from components
pub fn construct_asset(asset_id: AssetId, amount: u128) -> XcmAsset {
	XcmAsset {
		id: asset_id,
		fun: Fungibility::Fungible(amount),
	}
}

/// Generate commitment from XCM asset
///
/// For Week 4, we extend the commitment to include parachain origin
pub fn xcm_commitment_data(
	amount: u128,
	local_asset_id: u32,
	randomness: &[u8; 32],
	_origin: &Location, // Future: include in commitment
) -> H256 {
	// Week 4 MVP: Use simple_hash just like local deposits
	// Future: Include origin parachain ID in commitment
	crate::simple_hash::generate_commitment(amount, local_asset_id, randomness)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_registered_asset_creation() {
		let asset_id = AssetId(Location::parent());
		let registered = RegisteredAsset::new(asset_id.clone(), 1);

		assert_eq!(registered.local_id, 1);
		assert_eq!(registered.asset_id, asset_id);
		assert!(registered.is_active);
	}

	#[test]
	fn test_extract_fungible_amount() {
		let asset = construct_asset(
			AssetId(Location::parent()),
			1000,
		);

		assert_eq!(extract_asset_amount(&asset), Some(1000));
	}

	#[test]
	fn test_xcm_commitment_matches_local() {
		let amount = 1000u128;
		let asset_id = 1u32;
		let randomness = [42u8; 32];
		let origin = Location::parent();

		// XCM commitment should match local commitment for MVP
		let xcm_commit = xcm_commitment_data(amount, asset_id, &randomness, &origin);
		let local_commit = crate::simple_hash::generate_commitment(amount, asset_id, &randomness);

		assert_eq!(xcm_commit, local_commit);
	}
}
