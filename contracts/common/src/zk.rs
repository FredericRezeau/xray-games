/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::types::VerificationKeys;
use soroban_poseidon::poseidon2_hash;
use soroban_sdk::{
    crypto::BnScalar,
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    xdr::ToXdr, Address, Bytes, Env, U256, Vec, vec
};


pub struct SeedGenerator<'a> {
    env: &'a Env,
}

impl<'a> SeedGenerator<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    pub fn generate(&self, player: &Address, nonce: u64) -> U256 {
        let addr = address_to_field(self.env, player);
        let nonce = u64_to_field(self.env, nonce);
        hash(self.env, &addr, &nonce)
    }
}

pub fn verify_groth16(env: &Env, vk: &VerificationKeys, proof: &Bytes, inputs: &[U256]) -> bool {
    // snarkjs binary is c0|c1 vs Soroban SDK expecting c1|c0.
    let raw = proof.slice(64..192);
    let mut bytes = Bytes::new(env);
    bytes.append(&raw.slice(32..64));
    bytes.append(&raw.slice(0..32));
    bytes.append(&raw.slice(96..128));
    bytes.append(&raw.slice(64..96));

    let a = Bn254G1Affine::from_bytes(proof.slice(0..64).try_into().unwrap());
    let b = Bn254G2Affine::from_bytes(bytes.try_into().unwrap());
    let c = Bn254G1Affine::from_bytes(proof.slice(192..256).try_into().unwrap());
    let alpha = Bn254G1Affine::from_array(env, &vk.alpha);
    let beta = Bn254G2Affine::from_array(env, &vk.beta);
    let gamma = Bn254G2Affine::from_array(env, &vk.gamma);
    let delta = Bn254G2Affine::from_array(env, &vk.delta);

    // l = ic[0] + sum(inputs[i] * ic[i+1])
    let bn254 = env.crypto().bn254();
    let mut l = Bn254G1Affine::from_array(env, &vk.ic[0]);
    for i in 0..inputs.len() {
        let ic = Bn254G1Affine::from_array(env, &vk.ic[i + 1]);
        let scalar = Fr::from_u256(inputs[i].clone());
        let term = bn254.g1_mul(&ic, &scalar);
        l = bn254.g1_add(&l, &term);
    }

    // e(-a, b) * e(alpha, beta) * e(l, gamma) * e(c, delta) == 1
    let g1: Vec<Bn254G1Affine> = vec![env, -a, alpha, l, c];
    let g2: Vec<Bn254G2Affine> = vec![env, b, beta, gamma, delta];
    bn254.pairing_check(g1, g2)
}

pub fn hash(env: &Env, a: &U256, b: &U256) -> U256 {
    // Poseidon2 (BN254, t=4)
    let inputs = vec![env, a.clone(), b.clone()];
    poseidon2_hash::<4, BnScalar>(env, &inputs)
}

pub fn u64_to_field(env: &Env, val: u64) -> U256 {
    U256::from_u128(env, val as u128)
}

pub fn u32_to_field(env: &Env, val: u32) -> U256 {
    U256::from_u32(env, val)
}

pub fn address_to_field(env: &Env, addr: &Address) -> U256 {
    // 31 bytes for BN254.
    let bytes: Bytes = addr.to_xdr(env);
    let hash = env.crypto().sha256(&bytes);
    let arr = hash.to_array();
    let mut field = [0u8; 32];
    field[1..32].copy_from_slice(&arr[0..31]);
    U256::from_be_bytes(env, &Bytes::from_slice(env, &field))
}
