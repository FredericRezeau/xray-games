/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{contracttype, contracterror, Address, U256, BytesN};

pub struct Attestation {
    pub seed: U256,
    pub hash_noir: BytesN<32>,
    pub hash_circom: BytesN<32>,
}

pub struct CircuitOutputs {
    pub level_hash: U256,
    pub polygon_count: u32,
    pub object_count: u32,
    pub partition_count: u32,
}

#[derive(Clone, Copy)]
pub struct GameParams {
    pub duration: u64,
    pub min_target: u32,
    pub max_target: u32,
    pub target_count: u32,
    pub target_expire: u64,
}

impl GameParams {
    pub const fn default() -> Self {
        Self {
            duration: 180,
            min_target: 140,
            max_target: 202,
            target_count: 10,
            target_expire: 86400,
        }
    }
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Session {
    pub seed: U256,
    pub nonce: u64,
    pub target: u32,
    pub timestamp: u64,
    pub id: Option<u32>,
    pub commitment: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub enum StorageKey {
    Admin,
    Session(Address),
    House,
    Attestor,  // Server public key for attestation verification
    Targets,
}

#[derive(Clone, Copy)]
#[contracterror]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvalidSession = 3,
    BadAuth = 4,
    InvalidProof = 5,
    InvalidSeed = 6,
}