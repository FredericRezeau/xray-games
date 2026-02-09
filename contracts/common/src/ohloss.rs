/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::types::{House, OhlossPlayer};
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env};

#[allow(dead_code)]
#[contractclient(name = "OhlossClient")]
pub trait OhlossContract {
    fn start_game(
        env: Env,
        game_id: Address,
        session_id: u32,
        player1: Address,
        player2: Address,
        player1_wager: i128,
        player2_wager: i128,
    );

    fn end_game(env: Env, session_id: u32, player1_won: bool);

    fn get_player(env: Env, player: Address) -> OhlossPlayer;
}

pub struct Ohloss<'a> {
    env: &'a Env,
    contract: Address,
    min_wager: i128,
    factions: [Address; 3],
}

impl<'a> Ohloss<'a> {
    pub fn new(env: &'a Env, house: &House) -> Self {
        Self {
            env,
            contract: house.ohloss.clone(),
            min_wager: house.min_wager,
            factions: [
                house.faction0.clone(),
                house.faction1.clone(),
                house.faction2.clone(),
            ],
        }
    }

    #[inline]
    fn client(&self) -> OhlossClient<'a> {
        OhlossClient::new(self.env, &self.contract)
    }

    pub fn get_faction(&self, player: &Address) -> u32 {
        self.client()
            .get_player(player)
            .selected_faction
    }

    pub fn start(&self, player: &Address, wager: i128, commitment: &BytesN<32>) -> u32 {
        assert!(wager >= self.min_wager, "wager too small");

        let hash = self.env.crypto()
            .sha256(&Bytes::from_slice(self.env, &commitment.to_array())).to_array();
        let player_faction = self.get_faction(player);
        let opponent_faction = self.pick_faction(player_faction, (hash[4] as u32) % 2);
        let opponent = self.get_opponent(opponent_faction);
        let session = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);

        opponent.require_auth();

        self.client().start_game(
            &self.env.current_contract_address(),
            &session,
            player,
            &opponent,
            &wager,
            &self.min_wager,
        );

        session
    }

    pub fn end(&self, session: u32, won: bool) {
        self.client().end_game(&session, &won);
    }

    fn pick_faction(&self, faction: u32, bit: u32) -> u32 {
        match faction {
            0 => 1 + bit,
            1 => if bit == 0 { 0 } else { 2 },
            2 => bit,
            _ => panic!("invalid faction"),
        }
    }

    fn get_opponent(&self, faction: u32) -> Address {
        self.factions
            .get(faction as usize)
            .expect("invalid faction")
            .clone()
    }
}
