// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig};
use alloc::{vec, vec::Vec};
use frame_support::build_struct_json_patch;
use serde_json::Value;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::{self, PresetId};
use sp_keyring::Sr25519Keyring;

// Returns the genesis config presets populated with given parameters.
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
) -> Value {
	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1u128 << 60))
				.collect::<Vec<_>>(),
		},
		aura: pallet_aura::GenesisConfig {
			authorities: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},
		grandpa: pallet_grandpa::GenesisConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
                sudo: pallet_sudo::GenesisConfig {
                        key: Some(root),
                },
})}

/// Return the development genesis config.
pub fn development_config_genesis() -> Value {
    use sp_core::crypto::Ss58Codec;

    let sudo: AccountId = AccountId::from_ss58check(
        "5H3K2d3us8qTGjVkRXQXgmqqZiBifJWiggCLPooPgdY2ynTR"
    ).expect("sudo ss58 invalide");

    testnet_genesis(
        vec![
            (
                AuraId::from_ss58check("5CwAD6VNxigTdMwwykjpj2fjpHrYB72f9hEHboyojug9BdQF").unwrap(),
                GrandpaId::from_ss58check("5EJbWjhZbgg3ftodyqornJE3NFdcDZifQBFjSc7bbD1rVaET").unwrap(),
            ),
            (
                AuraId::from_ss58check("5GUETb1fTksQLzgZ1Tc2zQUNdS2uwB115GcSFjVLT5SGJq4z").unwrap(),
                GrandpaId::from_ss58check("5DyBSLyZGT1DmLf7WGEe59qb9g13YodjPHtwLacNBLW6ddFG").unwrap(),
            ),
            (
                AuraId::from_ss58check("5FvR78h8UHLBpB8kNxUkYNKToFbKwNf9896vLqrEfK9dhAk1").unwrap(),
                GrandpaId::from_ss58check("5F15hnv3vzJesgRfeQDStpUpi5RK97zU3ERutrHpayNCqQmK").unwrap(),
            ),
            (
                AuraId::from_ss58check("5F2Kjzq2iACX3y1ucu668nbWJ5VRyDciK2dFxpRqg1MhrMZK").unwrap(),
                GrandpaId::from_ss58check("5Gaeg9EZ247UNaBx4YHZXH6WrcFGGHGAFYmhrXUfBj96ohHm").unwrap(),
            ),
        ],
        vec![sudo.clone()],
        sudo,
    )
}
/// Return the local genesis config preset.
pub fn local_config_genesis() -> Value {
	testnet_genesis(
		vec![
			(
				sp_keyring::Sr25519Keyring::Alice.public().into(),
				sp_keyring::Ed25519Keyring::Alice.public().into(),
			),
			(
				sp_keyring::Sr25519Keyring::Bob.public().into(),
				sp_keyring::Ed25519Keyring::Bob.public().into(),
			),
		],
		Sr25519Keyring::iter()
			.filter(|v| v != &Sr25519Keyring::One && v != &Sr25519Keyring::Two)
			.map(|v| v.to_account_id())
			.collect::<Vec<_>>(),
		Sr25519Keyring::Alice.to_account_id(),
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
	]
}
