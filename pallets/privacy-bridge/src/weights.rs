
//! Placeholder weights for pallet_privacy_bridge
//!
//! These are temporary weights for the Week 1 MVP
//! Will be benchmarked properly in later weeks

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame::{deps::frame_support::weights::constants::RocksDbWeight, prelude::*};
use core::marker::PhantomData;

/// Weight functions needed for pallet_privacy_bridge.
pub trait WeightInfo {
	fn deposit() -> Weight;
	fn withdraw() -> Weight;
}

/// Temporary weights for privacy bridge pallet
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn deposit() -> Weight {
		Weight::from_parts(50_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn withdraw() -> Weight {
		Weight::from_parts(40_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn deposit() -> Weight {
		Weight::from_parts(50_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn withdraw() -> Weight {
		Weight::from_parts(40_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
