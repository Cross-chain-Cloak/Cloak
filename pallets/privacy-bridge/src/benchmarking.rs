//! Benchmarking setup for pallet-privacy-bridge

use super::*;
use frame::{deps::frame_benchmarking::v2::*, prelude::*};

#[benchmarks]
mod benchmarks {
	use super::*;
	#[cfg(test)]
	use crate::pallet::Pallet as PrivacyBridge;
	use frame_system::RawOrigin;

	#[benchmark]
	fn deposit() {
		let caller: T::AccountId = whitelisted_caller();
		let amount = 100u128;
		let asset_id = 0u32;
		let randomness = [1u8; 32];

		#[extrinsic_call]
		deposit(RawOrigin::Signed(caller), amount, asset_id, randomness);

		assert_eq!(CommitmentCount::<T>::get(), 1);
	}

	#[benchmark]
	fn withdraw() {
		let caller: T::AccountId = whitelisted_caller();
		let amount = 100u128;
		let asset_id = 0u32;
		let nullifier = sp_core::H256::from([1u8; 32]);

		#[extrinsic_call]
		withdraw(RawOrigin::Signed(caller), nullifier, amount, asset_id);

		assert!(NullifierSet::<T>::get(&nullifier));
	}

	impl_benchmark_test_suite!(PrivacyBridge, crate::mock::new_test_ext(), crate::mock::Test);
}
