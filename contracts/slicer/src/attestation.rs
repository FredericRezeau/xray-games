/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::storage;
use crate::types::Attestation;
use soroban_sdk::{Bytes, BytesN, Env, U256};

// Layout: [version:1][seed:32][index:4][hash_noir:32][hash_circom:32][timestamp:8][pad:2]
// ed25519 signature covers 111 bytes.

const SEED_LEN: u32 = 32;
const INDEX_LEN: u32 = 4;
const HASH_LEN: u32 = 32;
const BODY_LEN: u32 = 111;
const HASH_NOIR_OFFSET: u32 = 1u32 + SEED_LEN + INDEX_LEN;
const HASH_CIRCOM_OFFSET: u32 = HASH_NOIR_OFFSET + HASH_LEN;

pub(crate) fn verify_attestation(env: &Env, data: &Bytes) -> Attestation {
    let body = data.slice(0..BODY_LEN);
    let attestor: BytesN<32> = storage::get_attestor(env).unwrap();
    let signature: BytesN<64> = data.slice(BODY_LEN..BODY_LEN + 64).try_into().unwrap();
    env.crypto().ed25519_verify(&attestor, &body, &signature);

    let bytes: BytesN<32> = data.slice(1u32..1u32 + SEED_LEN).try_into().unwrap();
    let seed = U256::from_be_bytes(env, &Bytes::from_slice(env, &bytes.to_array()));
    let hash_noir: BytesN<32> = data.slice(HASH_NOIR_OFFSET..HASH_NOIR_OFFSET + HASH_LEN)
        .try_into().unwrap();
    let hash_circom: BytesN<32> = data.slice(HASH_CIRCOM_OFFSET..HASH_CIRCOM_OFFSET + HASH_LEN)
        .try_into().unwrap();
    Attestation { seed, hash_noir, hash_circom }
}