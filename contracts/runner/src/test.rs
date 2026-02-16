/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![cfg(test)]
extern crate std;
use crate::circuit;
use common::zk::verify_groth16;
use std::println;
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Logs},
    vec, Bytes, BytesN, Env, U256, Vec,
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine},
};

fn hex_encode(bytes: &soroban_sdk::Bytes) -> std::string::String {
    let mut s = std::string::String::new();
    for i in 0..bytes.len() {
        s.push_str(&std::format!("{:02x}", bytes.get(i).unwrap()));
    }
    s
}

fn hex_decode(env: &Env, hex: &str) -> Bytes {
    let bytes: std::vec::Vec<u8> = (0..hex.len())
        .step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16)
        .unwrap()).collect();
    Bytes::from_slice(env, &bytes)
}

#[test]
fn test_game_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let admin = Address::generate(&env);
    let contract_id = env.register(Runner, ());
    let client = RunnerClient::new(&env, &contract_id);
    client.initialize(&admin);
    let attestor = BytesN::from_array(&env, &[1u8; 32]);
    client.set_attestor(&attestor);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    let session = client.start(&player, &commitment, &0i128);
    println!("Session seed: 0x{}", hex_encode(&session.seed.to_be_bytes()));
    println!("Session nonce: {}", session.nonce);
    println!("Session target: {}", session.target);
    assert!(session.target >= 5 && session.target <= 15);
    assert!(session.id.is_none());

    println!("{}", env.logs().all().join("\n"));
}

#[test]
fn test_to_u32() {
    let env = Env::default();
    let val = U256::from_u32(&env, 42);
    assert_eq!(circuit::to_u32(&val), 42);
    let val_zero = U256::from_u32(&env, 0);
    assert_eq!(circuit::to_u32(&val_zero), 0);
    let val_max = U256::from_u32(&env, u32::MAX);
    assert_eq!(circuit::to_u32(&val_max), u32::MAX);

    println!("U256 to u32: PASS");
}

#[test]
fn test_extract_public_inputs() {
    let env = Env::default();

    // Fake proof (256 bytes) + 2 public inputs (64 bytes) = 320 bytes.
    let mut proof_data = std::vec::Vec::new();
    proof_data.extend_from_slice(&[0u8; 256]);

    // Public inputs as 32-byte big-endian values.
    // pi[0] = level_hash
    // pi[1] = score = 3
    let mut pi0 = [0u8; 32];
    pi0[31] = 0xAB;
    pi0[30] = 0xCD;
    proof_data.extend_from_slice(&pi0);
    let mut pi1 = [0u8; 32];
    pi1[31] = 3;
    proof_data.extend_from_slice(&pi1);
    let proof = Bytes::from_slice(&env, &proof_data);
    let (raw, inputs, _outputs) = circuit::extract(&env, &proof);

    assert_eq!(raw.len(), 256);
    assert_eq!(circuit::to_u32(&inputs[1]), 3);

    println!("Extract public inputs: PASS");
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_start_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let contract_id = env.register(Runner, ());
    let client = RunnerClient::new(&env, &contract_id);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    client.start(&player, &commitment, &0i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(Runner, ());
    let client = RunnerClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_seed_output() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let admin = Address::generate(&env);
    let contract_id = env.register(Runner, ());
    let client = RunnerClient::new(&env, &contract_id);
    client.initialize(&admin);

    let attestor = BytesN::from_array(&env, &[1u8; 32]);
    client.set_attestor(&attestor);

    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    let session = client.start(&player, &commitment, &0i128);
    // Server integration testing.
    println!("\n========== USE THIS FOR SERVER TEST ==========");
    println!("Seed (hex): 0x{}", hex_encode(&session.seed.to_be_bytes()));
    println!("Seed (dec): {:?}", session.seed);
    println!("Target: {}", session.target);
    println!("===============================================\n");
}

#[test]
fn test_pairing_ethereum_vectors() {
    // Groth16 verify with Ethereum test vectors.
    // e(G1, G2) * e(-G1, G2) = e(G1, G2) * e(G1, G2)^(-1) = 1
    // pairing_check([G1, -G1], [G2, G2]) should return true.
    let g2_gen_bytes: [u8; 128] = [
        0x19, 0x8e, 0x93, 0x93, 0x92, 0x0d, 0x48, 0x3a, 0x72, 0x60, 0xbf, 0xb7, 0x31, 0xfb, 0x5d, 0x25,
        0xf1, 0xaa, 0x49, 0x33, 0x35, 0xa9, 0xe7, 0x12, 0x97, 0xe4, 0x85, 0xb7, 0xae, 0xf3, 0x12, 0xc2,
        0x18, 0x00, 0xde, 0xef, 0x12, 0x1f, 0x1e, 0x76, 0x42, 0x6a, 0x00, 0x66, 0x5e, 0x5c, 0x44, 0x79,
        0x67, 0x43, 0x22, 0xd4, 0xf7, 0x5e, 0xda, 0xdd, 0x46, 0xde, 0xbd, 0x5c, 0xd9, 0x92, 0xf6, 0xed,
        0x09, 0x06, 0x89, 0xd0, 0x58, 0x5f, 0xf0, 0x75, 0xec, 0x9e, 0x99, 0xad, 0x69, 0x0c, 0x33, 0x95,
        0xbc, 0x4b, 0x31, 0x33, 0x70, 0xb3, 0x8e, 0xf3, 0x55, 0xac, 0xda, 0xdc, 0xd1, 0x22, 0x97, 0x5b,
        0x12, 0xc8, 0x5e, 0xa5, 0xdb, 0x8c, 0x6d, 0xeb, 0x4a, 0xab, 0x71, 0x80, 0x8d, 0xcb, 0x40, 0x8f,
        0xe3, 0xd1, 0xe7, 0x69, 0x0c, 0x43, 0xd3, 0x7b, 0x4c, 0xe6, 0xcc, 0x01, 0x66, 0xfa, 0x7d, 0xaa,
    ];

    let env = Env::default();
    let bn254 = env.crypto().bn254();
    let g1_gen = Bn254G1Affine::from_array(&env, &{
        let mut b = [0u8; 64];
        b[31] = 1;
        b[63] = 2;
        b
    });

    let g2_gen = Bn254G2Affine::from_array(&env, &g2_gen_bytes);
    let neg_g1 = -g1_gen.clone();
    let g1_vec: Vec<Bn254G1Affine> = vec![&env, g1_gen, neg_g1];
    let g2_vec: Vec<Bn254G2Affine> = vec![&env, g2_gen.clone(), g2_gen];
    let result = bn254.pairing_check(g1_vec, g2_vec);
    assert!(result, "e(G1,G2) * e(-G1,G2) should equal 1");

    println!("Pairing check two pairs: PASS");
}

#[test]
fn test_circom_real_proof() {
    // Groth16 verify with REAL game proof.
    let env = Env::default();

    // Proof (256 bytes): pi_a(64) || pi_b(128) || pi_c(64)
    let proof_hex = "05cf1d9b6736ff93a9f53c2feaeb4741f51f0e3bd3fb0c4b4f84d884a57d60ea1ac3f24d2b4a2cef4f00b4bf1bc2309b4fe6743705f4c1e889af6182cb32e97003ea634effca89bb16cb301af54b853e7130b548af1fb64906ee22c5b081259c21963d5e45acf7c6cc5a4dd8cf043b0a58cfa192d4c7e864d59eab2f834e8c251a4aeb5e8b7ddcc53971669d2cdba15d17fc59ad2b24a49a6cb292c11e0decb20bf551e4bb22ca215000191d0a8fa5d8467f01732dcaec8abe0bdb9f632add75029c4435804e82191f6330b704f1981afcfb805dbc7ca8b7bb4c46b4b8c486302f6b74722204735614066034b036802dc820813550afb52592935f9edafbe0aa";
    let proof = hex_decode(&env, proof_hex);
    assert_eq!(proof.len(), 256);

    // Public inputs: [level_hash, score(15)]
    let public_inputs = [
        U256::from_be_bytes(&env, &hex_decode(&env, "217211a9ee90573a5e7c1ffb0669dd09dee35c158fd2bb54923aab36ec29d0fc").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000f").try_into().unwrap()),
    ];

    let result = verify_groth16(&env, &circuit::KEYS, &proof, &public_inputs);
    assert!(result, "Real Groth16 proof should verify");
    let score = circuit::to_u32(&public_inputs[1]);
    assert_eq!(score, 15);

    println!("Real Groth16 proof verification: PASS (score={})", score);
}

#[test]
fn test_circom_tampered_proof() {
    let env = Env::default();

    // Correct proof but wrong level_hash (tamper public input, not proof bytes).
    let proof_hex = "05cf1d9b6736ff93a9f53c2feaeb4741f51f0e3bd3fb0c4b4f84d884a57d60ea1ac3f24d2b4a2cef4f00b4bf1bc2309b4fe6743705f4c1e889af6182cb32e97003ea634effca89bb16cb301af54b853e7130b548af1fb64906ee22c5b081259c21963d5e45acf7c6cc5a4dd8cf043b0a58cfa192d4c7e864d59eab2f834e8c251a4aeb5e8b7ddcc53971669d2cdba15d17fc59ad2b24a49a6cb292c11e0decb20bf551e4bb22ca215000191d0a8fa5d8467f01732dcaec8abe0bdb9f632add75029c4435804e82191f6330b704f1981afcfb805dbc7ca8b7bb4c46b4b8c486302f6b74722204735614066034b036802dc820813550afb52592935f9edafbe0aa";
    let proof = hex_decode(&env, proof_hex);

    // Tamper level_hash.
    let public_inputs = [
        U256::from_be_bytes(&env, &hex_decode(&env, "ff7211a9ee90573a5e7c1ffb0669dd09dee35c158fd2bb54923aab36ec29d0fc").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000f").try_into().unwrap()),
    ];

    let result = verify_groth16(&env, &circuit::KEYS, &proof, &public_inputs);
    assert!(!result, "Tampered level_hash should NOT verify");

    println!("Tampered proof rejection: PASS");
}

#[test]
fn test_circom_wrong_inputs() {
    let env = Env::default();

    // Correct proof but wrong score.
    let proof_hex = "05cf1d9b6736ff93a9f53c2feaeb4741f51f0e3bd3fb0c4b4f84d884a57d60ea1ac3f24d2b4a2cef4f00b4bf1bc2309b4fe6743705f4c1e889af6182cb32e97003ea634effca89bb16cb301af54b853e7130b548af1fb64906ee22c5b081259c21963d5e45acf7c6cc5a4dd8cf043b0a58cfa192d4c7e864d59eab2f834e8c251a4aeb5e8b7ddcc53971669d2cdba15d17fc59ad2b24a49a6cb292c11e0decb20bf551e4bb22ca215000191d0a8fa5d8467f01732dcaec8abe0bdb9f632add75029c4435804e82191f6330b704f1981afcfb805dbc7ca8b7bb4c46b4b8c486302f6b74722204735614066034b036802dc820813550afb52592935f9edafbe0aa";
    let proof = hex_decode(&env, proof_hex);
    let public_inputs = [
        U256::from_be_bytes(&env, &hex_decode(&env, "217211a9ee90573a5e7c1ffb0669dd09dee35c158fd2bb54923aab36ec29d0fc").try_into().unwrap()),
        // 3 instead of 15.
        U256::from_be_bytes(&env, &hex_decode(&env, "0000000000000000000000000000000000000000000000000000000000000003").try_into().unwrap()),
    ];

    let result = verify_groth16(&env, &circuit::KEYS, &proof, &public_inputs);
    assert!(!result, "Wrong public inputs should NOT verify");
    println!("Wrong public inputs rejection: PASS");
}

#[test]
#[should_panic(expected = "unknown ZK backend")]
fn test_backend_unknown() {
    let env = Env::default();

    let att = types::Attestation {
        seed: U256::from_u32(&env, 0),
        hash_noir: BytesN::from_array(&env, &[0xAA; 32]),
        hash_circom: BytesN::from_array(&env, &[0xBB; 32]),
    };

    circuit::get_backend(&BytesN::from_array(&env, &[0xFF; 32]), &att);
}

#[test]
fn test_backend() {
    let env = Env::default();

    let hash_noir = BytesN::from_array(&env, &[
        0x16, 0xda, 0x3a, 0xc1, 0x01, 0xb4, 0xda, 0x24, 0xe3, 0x21, 0x5d, 0x54, 0xb2, 0xab, 0xab, 0xd4,
        0x1e, 0x05, 0xa9, 0x10, 0xd1, 0xc9, 0x3f, 0x74, 0x84, 0xe3, 0x50, 0x3a, 0x13, 0xe2, 0x29, 0xb5,
    ]);
    let hash_circom = BytesN::from_array(&env, &[
        0x30, 0x2d, 0xc8, 0x62, 0x25, 0xa1, 0xbc, 0xfe, 0xc1, 0x5e, 0x8e, 0x2f, 0x83, 0x76, 0x58, 0xa8,
        0xee, 0x75, 0x73, 0xab, 0x11, 0x74, 0x6a, 0xd6, 0x63, 0x3e, 0x9e, 0xaf, 0xe9, 0x36, 0x47, 0x6f,
    ]);

    let att = types::Attestation {
        seed: U256::from_u32(&env, 0),
        hash_noir: hash_noir.clone(),
        hash_circom: hash_circom.clone(),
    };

    let backend_n = circuit::get_backend(&hash_noir, &att);
    assert!(matches!(backend_n, common::types::Backend::Noir));

    let backend_c = circuit::get_backend(&hash_circom, &att);
    assert!(matches!(backend_c, common::types::Backend::Circom));

    println!("Backend detection: PASS");
}
