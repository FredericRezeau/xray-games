/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![no_std]

mod slicer;
mod storage;
mod attestation;
mod circuit;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};
use types::{Error, Session};

#[contract]
pub struct Slicer;

#[contractimpl]
impl Slicer {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        slicer::initialize(&env, admin)
    }

    pub fn start(env: Env, player: Address,
        commitment: BytesN<32>, wager: i128) -> Result<Session, Error> {
        slicer::start(&env, player, commitment, wager)
    }

    pub fn end(env: Env, player: Address, preimage: BytesN<32>,
        proof: Bytes, attestation: Bytes) -> Result<u32, Error> {
        slicer::end(&env, player, preimage, proof, attestation)
    }

    pub fn set_house(env: Env, faction0: Address, faction1: Address,
        faction2: Address, min_wager: i128, ohloss: Address,
    ) -> Result<(), Error> {
        slicer::set_house(&env, faction0, faction1, faction2, min_wager, ohloss)
    }

    pub fn set_attestor(env: Env, pubkey: BytesN<32>) -> Result<(), Error> {
        slicer::set_attestor(&env, pubkey)
    }

    pub fn upgrade(env: Env, hash: BytesN<32>) -> Result<(), Error> {
        slicer::upgrade(&env, hash)
    }
}

#[cfg(test)]
mod test;
