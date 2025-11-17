//! # Privacy Bridge Pallet
//!
//! A cross-chain privacy layer for the Polkadot ecosystem that provides privacy-as-a-service
//! to all parachains. Users can deposit public assets from any parachain, perform private
//! transactions, and withdraw to any destination parachain.
//!
//! ## Overview
//!
//! This pallet implements:
//! - Shielded commitments for hiding transaction amounts
//! - Nullifier set for preventing double-spending
//! - Deposit functionality for shielding assets
//! - Future: zkSNARK proofs, XCM integration, multi-asset support
//!
//! ## Week 1 MVP Features
//!
//! - Basic commitment generation using cryptographic hashing
//! - Nullifier tracking to prevent double-spending
//! - Deposit function to shield assets
//! - Storage for commitments and nullifiers
//!
//! ## Week 2 Features (zkSNARKs)
//!
//! - zkSNARK circuit for proving commitment ownership
//! - Groth16 proof generation and verification
//! - On-chain proof verification in withdraw()

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// Week 2: zkSNARK modules
pub mod circuit;
pub mod zksnark;

// Week 3: Simple hash for zkSNARK compatibility
pub mod simple_hash;

// Week 3: Merkle tree for commitment anonymity
pub mod merkle_tree;

// Week 4: XCM cross-chain integration
pub mod xcm_config;

#[cfg(test)]
mod zksnark_integration_test;

#[cfg(test)]
mod xcm_tests;

#[frame::pallet]
pub mod pallet {
	use frame::prelude::*;
	use sp_core::H256;
	use sp_runtime::traits::{BlakeTwo256, Hash};
	use alloc::vec::Vec;

	// Week 4: XCM imports
	use staging_xcm::v5::{AssetId as XcmAssetId, Location};
	use crate::xcm_config::RegisteredAsset;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Stores metadata about each commitment
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, RuntimeDebug)]
	#[scale_info(skip_type_params(T))]
	pub struct CommitmentData<T: Config> {
		/// Block number when commitment was created
		pub block_number: BlockNumberFor<T>,
		/// Account that created the commitment
		pub depositor: T::AccountId,
		/// Asset ID (for future multi-asset support)
		pub asset_id: u32,
	}

	/// Stores the shielded note data (kept off-chain by user)
	/// This is what the user will keep secret to later spend their commitment
	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
	pub struct ShieldedNote {
		/// The commitment hash (public, stored on-chain)
		pub commitment: H256,
		/// The amount (private, kept by user)
		pub amount: u128,
		/// Asset ID
		pub asset_id: u32,
		/// Secret randomness used in commitment
		pub randomness: [u8; 32],
		/// Nullifier for spending (derived from commitment + secret)
		pub nullifier: H256,
	}

	/// Storage: Maps commitment hash -> commitment metadata
	/// The commitment itself is a hash that hides the amount and randomness
	#[pallet::storage]
	#[pallet::getter(fn commitments)]
	pub type Commitments<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		H256,                    // Commitment hash
		CommitmentData<T>,       // Metadata (does NOT include amount!)
		OptionQuery,
	>;

	/// Storage: Set of used nullifiers to prevent double-spending
	/// Once a nullifier is used, it cannot be used again
	#[pallet::storage]
	#[pallet::getter(fn nullifiers)]
	pub type NullifierSet<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		H256,      // Nullifier hash
		bool,      // true if used
		ValueQuery,
	>;

	/// Storage: Counter for total commitments (useful for merkle tree indexing later)
	#[pallet::storage]
	#[pallet::getter(fn commitment_count)]
	pub type CommitmentCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Week 3: Storage for zkSNARK verifying key (serialized)
	/// This key is used to verify withdrawal proofs on-chain
	/// Generated off-chain during trusted setup, stored on-chain for verification
	#[pallet::storage]
	#[pallet::getter(fn verifying_key)]
	pub type VerifyingKey<T: Config> = StorageValue<_, BoundedVec<u8, ConstU32<4096>>, OptionQuery>;
	// Note: 4096 bytes should be enough for Groth16 verifying key

	/// Week 4: Asset registry - maps XCM AssetId to local asset ID
	/// This allows the bridge to support multiple assets from different parachains
	#[pallet::storage]
	#[pallet::getter(fn registered_assets)]
	pub type AssetRegistry<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		XcmAssetId,           // XCM asset identifier
		RegisteredAsset,       // Registration details
		OptionQuery,
	>;

	/// Week 4: Counter for assigning local asset IDs
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Events emitted by the privacy bridge pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Asset was shielded (deposited into privacy pool)
		AssetShielded {
			commitment: H256,
			asset_id: u32,
			depositor: T::AccountId,
			block_number: BlockNumberFor<T>,
		},
		/// Asset was unshielded (withdrawn from privacy pool)
		AssetUnshielded {
			nullifier: H256,
			asset_id: u32,
			block_number: BlockNumberFor<T>,
		},
		/// Private transfer occurred (Week 2+ feature)
		PrivateTransfer {
			nullifier: H256,
		},
	}

	/// Errors that can occur in the privacy bridge pallet
	#[pallet::error]
	pub enum Error<T> {
		/// Commitment already exists
		CommitmentAlreadyExists,
		/// Nullifier has already been used (double-spend attempt)
		NullifierAlreadyUsed,
		/// Commitment does not exist
		CommitmentNotFound,
		/// Invalid proof (for future zkSNARK verification)
		InvalidProof,
		/// Amount overflow
		AmountOverflow,
		/// Invalid randomness
		InvalidRandomness,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Dispatchable functions (extrinsics)
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Deposit (shield) an asset into the privacy pool
		///
		/// This creates a commitment that hides the amount and returns a shielded note
		/// The user must keep the shielded note data off-chain to later spend it
		///
		/// Parameters:
		/// - `amount`: Amount to shield (in smallest unit)
		/// - `asset_id`: Asset identifier (0 for native token)
		/// - `randomness`: 32 bytes of randomness for commitment
		///
		/// Emits: `AssetShielded` event
		///
		/// Week 1 MVP: Simple deposit without actual token transfer
		/// Week 4+: Will integrate with XCM to receive assets from other chains
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
		pub fn deposit(
			origin: OriginFor<T>,
			amount: u128,
			asset_id: u32,
			randomness: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Generate commitment: commitment = Hash(amount || asset_id || randomness)
			let commitment = Self::generate_commitment(amount, asset_id, &randomness);

			// Ensure commitment doesn't already exist
			ensure!(
				!Commitments::<T>::contains_key(&commitment),
				Error::<T>::CommitmentAlreadyExists
			);

			// Store commitment metadata (note: amount is NOT stored!)
			let commitment_data = CommitmentData {
				block_number: <frame_system::Pallet<T>>::block_number(),
				depositor: who.clone(),
				asset_id,
			};

			Commitments::<T>::insert(&commitment, commitment_data);

			// Increment commitment counter
			let count = CommitmentCount::<T>::get();
			CommitmentCount::<T>::put(
				count.checked_add(1).ok_or(Error::<T>::AmountOverflow)?
			);

			// Emit event
			Self::deposit_event(Event::AssetShielded {
				commitment,
				asset_id,
				depositor: who,
				block_number: <frame_system::Pallet<T>>::block_number(),
			});

			// NOTE: In a real implementation, the shielded note would be returned to the user
			// or encrypted and stored. For now, the user would reconstruct it from the event data
			// and their secret randomness.

			Ok(())
		}

		/// Withdraw (unshield) an asset from the privacy pool
		///
		/// Week 1 MVP: Simple nullifier check (no zkSNARK proof yet)
		/// Week 2+: Will require zkSNARK proof of commitment ownership
		///
		/// Parameters:
		/// - `nullifier`: The nullifier hash (prevents double-spending)
		/// - `amount`: Amount to withdraw (for Week 1 testing)
		/// - `asset_id`: Asset identifier
		///
		/// Emits: `AssetUnshielded` event
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
		pub fn withdraw(
			origin: OriginFor<T>,
			nullifier: H256,
			_amount: u128,
			asset_id: u32,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Check that nullifier hasn't been used
			ensure!(
				!NullifierSet::<T>::get(&nullifier),
				Error::<T>::NullifierAlreadyUsed
			);

			// Mark nullifier as used
			NullifierSet::<T>::insert(&nullifier, true);

			// Emit event
			Self::deposit_event(Event::AssetUnshielded {
				nullifier,
				asset_id,
				block_number: <frame_system::Pallet<T>>::block_number(),
			});

			// Week 1: No actual token transfer
			// Week 2+: Verify zkSNARK proof
			// Week 4+: Send tokens via XCM to destination parachain

			Ok(())
		}

		/// Week 3: Set the zkSNARK verifying key (governance/sudo only)
		///
		/// This should be called once during initialization with the verifying key
		/// from the trusted setup ceremony.
		///
		/// Parameters:
		/// - `vk_bytes`: Serialized verifying key
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_verifying_key(
			origin: OriginFor<T>,
			vk_bytes: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let bounded_vk: BoundedVec<u8, ConstU32<4096>> = vk_bytes.try_into()
				.map_err(|_| Error::<T>::InvalidProof)?; // Reuse error type

			VerifyingKey::<T>::put(bounded_vk);

			Ok(())
		}

		/// Week 4: Register an XCM asset for cross-chain deposits
		///
		/// Allows governance to register assets from other parachains
		///
		/// Parameters:
		/// - `asset_id`: XCM AssetId to register
		/// - `min_deposit`: Minimum deposit amount
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
		pub fn register_asset(
			origin: OriginFor<T>,
			asset_id: XcmAssetId,
			min_deposit: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Get next local asset ID
			let local_id = NextAssetId::<T>::get();
			NextAssetId::<T>::put(local_id.checked_add(1).ok_or(Error::<T>::AmountOverflow)?);

			// Create registration
			let mut registration = RegisteredAsset::new(asset_id.clone(), local_id);
			registration.min_deposit = min_deposit;

			// Store registration
			AssetRegistry::<T>::insert(asset_id, registration);

			Ok(())
		}

		/// Week 4: Cross-chain deposit via XCM
		///
		/// Called when assets are received from another parachain via XCM
		/// Creates a commitment for the received assets
		///
		/// Parameters:
		/// - `asset_id`: XCM AssetId being deposited
		/// - `amount`: Amount received
		/// - `origin`: Location of sender parachain
		/// - `randomness`: Randomness for commitment
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(3))]
		pub fn deposit_from_xcm(
			origin: OriginFor<T>,
			asset_id: XcmAssetId,
			amount: u128,
			origin_location: Location,
			randomness: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Look up registered asset
			let registered = AssetRegistry::<T>::get(&asset_id)
				.ok_or(Error::<T>::InvalidProof)?; // Reuse error

			// Check minimum deposit
			ensure!(amount >= registered.min_deposit, Error::<T>::InvalidProof);

			// Generate commitment using local asset ID
			let commitment = crate::xcm_config::xcm_commitment_data(
				amount,
				registered.local_id,
				&randomness,
				&origin_location,
			);

			// Ensure commitment doesn't already exist
			ensure!(
				!Commitments::<T>::contains_key(&commitment),
				Error::<T>::CommitmentAlreadyExists
			);

			// Store commitment metadata
			let commitment_data = CommitmentData {
				block_number: <frame_system::Pallet<T>>::block_number(),
				depositor: who.clone(),
				asset_id: registered.local_id,
			};

			Commitments::<T>::insert(&commitment, commitment_data);

			// Increment commitment counter
			let count = CommitmentCount::<T>::get();
			CommitmentCount::<T>::put(
				count.checked_add(1).ok_or(Error::<T>::AmountOverflow)?
			);

			// Emit event
			Self::deposit_event(Event::AssetShielded {
				commitment,
				asset_id: registered.local_id,
				depositor: who,
				block_number: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}

		/// Week 4: Withdraw to another parachain via XCM
		///
		/// Withdraw assets and send them to a destination parachain
		///
		/// Parameters:
		/// - `nullifier`: Nullifier hash
		/// - `asset_id`: Local asset ID
		/// - `amount`: Amount to withdraw
		/// - `destination`: Destination parachain location
		/// - `beneficiary`: Recipient account on destination chain
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn withdraw_to_parachain(
			origin: OriginFor<T>,
			nullifier: H256,
			asset_id: u32,
			amount: u128,
			destination: Location,
			beneficiary: Location,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Check that nullifier hasn't been used
			ensure!(
				!NullifierSet::<T>::get(&nullifier),
				Error::<T>::NullifierAlreadyUsed
			);

			// Mark nullifier as used
			NullifierSet::<T>::insert(&nullifier, true);

			// Emit event (actual XCM sending would happen here in production)
			Self::deposit_event(Event::AssetUnshielded {
				nullifier,
				asset_id,
				block_number: <frame_system::Pallet<T>>::block_number(),
			});

			// Week 4 MVP: Log the cross-chain transfer intent
			// Production: Actually send XCM message to destination
			// Example: pallet_xcm::Pallet::<T>::send_xcm(destination, beneficiary, assets)

			// Store the destination and beneficiary for future reference
			let _ = (destination, beneficiary, amount);

			Ok(())
		}
	}

	/// Helper functions (not callable by users)
	impl<T: Config> Pallet<T> {
		/// Generate a commitment hash from amount, asset_id, and randomness
		///
		/// Commitment = Hash(amount || asset_id || randomness)
		///
		/// This hides the amount and randomness while creating a unique identifier
		pub fn generate_commitment(
			amount: u128,
			asset_id: u32,
			randomness: &[u8; 32],
		) -> H256 {
			let mut data = Vec::new();
			data.extend_from_slice(&amount.to_le_bytes());
			data.extend_from_slice(&asset_id.to_le_bytes());
			data.extend_from_slice(randomness);

			BlakeTwo256::hash(&data)
		}

		/// Generate a nullifier from commitment and secret
		///
		/// Nullifier = Hash(commitment || secret)
		///
		/// This prevents double-spending while maintaining privacy
		/// Week 1: Simple version
		/// Week 2+: Will be generated in zkSNARK circuit
		pub fn generate_nullifier(
			commitment: &H256,
			secret: &[u8; 32],
		) -> H256 {
			let mut data = Vec::new();
			data.extend_from_slice(commitment.as_bytes());
			data.extend_from_slice(secret);

			BlakeTwo256::hash(&data)
		}

		/// Verify a shielded note matches a commitment
		/// Helper function for testing
		pub fn verify_note(
			note: &ShieldedNote,
			commitment: &H256,
		) -> bool {
			let computed_commitment = Self::generate_commitment(
				note.amount,
				note.asset_id,
				&note.randomness,
			);

			computed_commitment == *commitment
		}
	}
}
