/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![no_std]

mod pool;
mod snooker;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
use types::{Ball, Error, Session};

#[contract]
pub struct Snooker;

#[contractimpl]
impl Snooker {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        snooker::initialize(&env, admin)
    }

    pub fn start(env: Env, player: Address, commitment: BytesN<32>, wager: i128) -> Result<Session, Error> {
        snooker::start(&env, player, commitment, wager)
    }

    pub fn end(env: Env, player: Address, preimage: BytesN<32>, shots: Vec<Ball>) -> Result<u32, Error> {
        snooker::end(&env, player, preimage, shots)
    }

    pub fn set_house(env: Env, faction0: Address, faction1: Address,
        faction2: Address, min_wager: i128, ohloss: Address) -> Result<(), Error> {
        snooker::set_house(&env, faction0, faction1, faction2, min_wager, ohloss)
    }

    pub fn upgrade(env: Env, hash: BytesN<32>) -> Result<(), Error> {
        snooker::upgrade(&env, hash)
    }
}

#[cfg(test)]
mod test;
