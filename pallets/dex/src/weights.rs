// This file is part of Acala.

// Copyright (C) 2020-2022 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for module_dex
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-12-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/release/acala
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=module_dex
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./modules/dex/src/weights.rs
// --template=./templates/module-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for module_dex.
pub trait WeightInfo {
	fn enable_trading_pair() -> Weight;
	fn disable_trading_pair() -> Weight;
	fn list_provisioning() -> Weight;
	fn update_provisioning_parameters() -> Weight;
	fn end_provisioning() -> Weight;
	fn add_provision() -> Weight;
	fn claim_dex_share() -> Weight;
	fn add_liquidity() -> Weight;
	fn add_liquidity_and_stake() -> Weight;
	fn remove_liquidity() -> Weight;
	fn remove_liquidity_by_unstake() -> Weight;
	fn swap_with_exact_supply(u: u32, ) -> Weight;
	fn swap_with_exact_target(u: u32, ) -> Weight;
	fn refund_provision() -> Weight;
	fn abort_provisioning() -> Weight;
}

/// Weights for module_dex using the Acala node and recommended hardware.
pub struct AcalaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for AcalaWeight<T> {
	fn enable_trading_pair() -> Weight {
		(24_728_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn disable_trading_pair() -> Weight {
		(24_891_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn list_provisioning() -> Weight {
		(37_619_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn update_provisioning_parameters() -> Weight {
		(11_808_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn end_provisioning() -> Weight {
		(78_617_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn add_provision() -> Weight {
		(127_543_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn claim_dex_share() -> Weight {
		(105_716_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn add_liquidity() -> Weight {
		(184_975_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn add_liquidity_and_stake() -> Weight {
		(258_276_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	fn remove_liquidity() -> Weight {
		(158_440_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn remove_liquidity_by_unstake() -> Weight {
		(277_297_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	fn swap_with_exact_supply(u: u32, ) -> Weight {
		(93_799_000 as Weight)
			// Standard Error: 117_000
			.saturating_add((16_008_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(u as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
	}
	fn swap_with_exact_target(u: u32, ) -> Weight {
		(93_966_000 as Weight)
			// Standard Error: 226_000
			.saturating_add((16_058_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(u as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
	}
	fn refund_provision() -> Weight {
		(105_716_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn abort_provisioning() -> Weight {
		(78_617_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn enable_trading_pair() -> Weight {
		(24_728_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn disable_trading_pair() -> Weight {
		(24_891_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn list_provisioning() -> Weight {
		(37_619_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn update_provisioning_parameters() -> Weight {
		(11_808_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn end_provisioning() -> Weight {
		(78_617_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn add_provision() -> Weight {
		(127_543_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn claim_dex_share() -> Weight {
		(105_716_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn add_liquidity() -> Weight {
		(184_975_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(9 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}
	fn add_liquidity_and_stake() -> Weight {
		(258_276_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(10 as Weight))
	}
	fn remove_liquidity() -> Weight {
		(158_440_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn remove_liquidity_by_unstake() -> Weight {
		(277_297_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(10 as Weight))
	}
	fn swap_with_exact_supply(u: u32, ) -> Weight {
		(93_799_000 as Weight)
			// Standard Error: 117_000
			.saturating_add((16_008_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().reads((2 as Weight).saturating_mul(u as Weight)))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
	}
	fn swap_with_exact_target(u: u32, ) -> Weight {
		(93_966_000 as Weight)
			// Standard Error: 226_000
			.saturating_add((16_058_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().reads((2 as Weight).saturating_mul(u as Weight)))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
	}
	fn refund_provision() -> Weight {
		(105_716_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn abort_provisioning() -> Weight {
		(78_617_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
}
