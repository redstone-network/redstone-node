// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
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

//! Genesis Configuration.

use crate::keyring::*;
pub use node_primitives::{
	currency::{
		TokenInfo, ACA, AUSD, BNC, DOT, KAR, KBTC, KINT, KSM, KUSD, LCDOT, LDOT, LKSM, PHA, RENBTC,
		VSKSM,
	},
	TokenSymbol, TradingPair,
};
use node_runtime::{
	wasm_binary_unwrap, AccountId, BabeConfig, BalancesConfig, DexConfig, GenesisConfig,
	GrandpaConfig, IndicesConfig, SessionConfig, SocietyConfig, StakerStatus, StakingConfig,
	SystemConfig, TokensConfig, BABE_GENESIS_EPOCH_CONFIG,
};
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use sp_runtime::Perbill;

const MILLICENTS: u128 = 1_000_000_000;
const CENTS: u128 = 1_000 * MILLICENTS; // assume this is worth about a cent.
const DOLLARS: u128 = 100 * CENTS;

const INITIAL_BALANCE: u128 = 10_000_000 * DOLLARS;

/// Create genesis runtime configuration for tests.
pub fn config(code: Option<&[u8]>) -> GenesisConfig {
	config_endowed(code, Default::default())
}

/// Create genesis runtime configuration for tests with some extra
/// endowed accounts.
pub fn config_endowed(code: Option<&[u8]>, extra_endowed: Vec<AccountId>) -> GenesisConfig {
	let mut endowed = vec![
		(alice(), 111 * DOLLARS),
		(bob(), 100 * DOLLARS),
		(charlie(), 100_000_000 * DOLLARS),
		(dave(), 111 * DOLLARS),
		(eve(), 101 * DOLLARS),
		(ferdie(), 100 * DOLLARS),
	];

	endowed.extend(extra_endowed.clone().into_iter().map(|endowed| (endowed, 100 * DOLLARS)));

	GenesisConfig {
		system: SystemConfig {
			code: code.map(|x| x.to_vec()).unwrap_or_else(|| wasm_binary_unwrap().to_vec()),
		},
		indices: IndicesConfig { indices: vec![] },
		balances: BalancesConfig { balances: endowed },
		session: SessionConfig {
			keys: vec![
				(alice(), dave(), to_session_keys(&Ed25519Keyring::Alice, &Sr25519Keyring::Alice)),
				(bob(), eve(), to_session_keys(&Ed25519Keyring::Bob, &Sr25519Keyring::Bob)),
				(
					charlie(),
					ferdie(),
					to_session_keys(&Ed25519Keyring::Charlie, &Sr25519Keyring::Charlie),
				),
			],
		},
		staking: StakingConfig {
			stakers: vec![
				(dave(), alice(), 111 * DOLLARS, StakerStatus::Validator),
				(eve(), bob(), 100 * DOLLARS, StakerStatus::Validator),
				(ferdie(), charlie(), 100 * DOLLARS, StakerStatus::Validator),
			],
			validator_count: 3,
			minimum_validator_count: 0,
			slash_reward_fraction: Perbill::from_percent(10),
			invulnerables: vec![alice(), bob(), charlie()],
			..Default::default()
		},
		babe: BabeConfig { authorities: vec![], epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: GrandpaConfig { authorities: vec![] },
		im_online: Default::default(),
		authority_discovery: Default::default(),
		democracy: Default::default(),
		council: Default::default(),
		technical_committee: Default::default(),
		technical_membership: Default::default(),
		elections: Default::default(),
		sudo: Default::default(),
		treasury: Default::default(),
		society: SocietyConfig { members: vec![alice(), bob()], pot: 0, max_members: 999 },
		vesting: Default::default(),
		assets: Default::default(),
		gilt: Default::default(),
		transaction_storage: Default::default(),
		transaction_payment: Default::default(),
		nomination_pools: Default::default(),
		tokens: TokensConfig {
			balances: extra_endowed
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), AUSD, INITIAL_BALANCE),
						(x.clone(), RENBTC, INITIAL_BALANCE),
						(x.clone(), DOT, INITIAL_BALANCE),
					]
				})
				.collect(),
		},
		dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: vec![
				TradingPair::from_currency_ids(AUSD, RENBTC).unwrap(),
				TradingPair::from_currency_ids(AUSD, DOT).unwrap(),
			],
			initial_added_liquidity_pools: vec![(
				alice(),
				vec![
					(
						TradingPair::from_currency_ids(AUSD, RENBTC).unwrap(),
						(1_000_000 * DOLLARS, 2_000_000 * DOLLARS),
					),
					(
						TradingPair::from_currency_ids(AUSD, DOT).unwrap(),
						(1_000_000 * DOLLARS, 2_000_000 * DOLLARS),
					),
				],
			)],
		},
	}
}
