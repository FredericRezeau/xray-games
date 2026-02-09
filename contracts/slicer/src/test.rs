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
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    Bytes::from_slice(env, &bytes)
}

#[test]
fn test_game_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let admin = Address::generate(&env);
    let contract_id = env.register(Slicer, ());
    let client = SlicerClient::new(&env, &contract_id);
    client.initialize(&admin);
    let attestor = BytesN::from_array(&env, &[1u8; 32]);
    client.set_attestor(&attestor);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    let session = client.start(&player, &commitment, &0i128);
    println!("Session seed: 0x{}", hex_encode(&session.seed.to_be_bytes()));
    println!("Session nonce: {}", session.nonce);
    println!("Session target: {}", session.target);
    assert!(session.target >= 140 && session.target <= 232);
    assert!(session.id.is_none());

    println!("{}", env.logs().all().join("\n"));
}

#[test]
fn test_score_calculation() {
    // polygon_count=2, object_count=5, partition_count=8
    assert_eq!(crate::slicer::calculate_score(2, 5, 8), 145);
    // polygon_count=1, object_count=3, partition_count=2
    assert_eq!(crate::slicer::calculate_score(1, 3, 2), 54);
    // Edge case: all zeros
    assert_eq!(crate::slicer::calculate_score(0, 0, 0), 0);

    println!("Score calculation: PASS");
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

    // Fake proof (256 bytes) + 4 public inputs (128 bytes) = 384 bytes.
    let mut proof_data = std::vec::Vec::new();
    proof_data.extend_from_slice(&[0u8; 256]);

    // Public inputs as 32-byte big-endian values.
    // pi[0] = level_hash
    // pi[1] = polygon_count = 3
    // pi[2] = object_count = 7
    // pi[3] = partition_count = 10
    let mut pi0 = [0u8; 32];
    pi0[31] = 0xAB;
    pi0[30] = 0xCD;
    proof_data.extend_from_slice(&pi0);
    let mut pi1 = [0u8; 32];
    pi1[31] = 3;
    proof_data.extend_from_slice(&pi1);
    let mut pi2 = [0u8; 32];
    pi2[31] = 7;
    proof_data.extend_from_slice(&pi2);
    let mut pi3 = [0u8; 32];
    pi3[31] = 10;
    proof_data.extend_from_slice(&pi3);
    let proof = Bytes::from_slice(&env, &proof_data);
    let (raw, inputs, _outputs) = circuit::extract(&env, &proof);

    assert_eq!(raw.len(), 256);
    assert_eq!(circuit::to_u32(&inputs[1]), 3);
    assert_eq!(circuit::to_u32(&inputs[2]), 7);
    assert_eq!(circuit::to_u32(&inputs[3]), 10);

    let score = crate::slicer::calculate_score(3, 7, 10);
    assert_eq!(score, 191);

    println!("Extract public inputs: PASS");
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_start_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let contract_id = env.register(Slicer, ());
    let client = SlicerClient::new(&env, &contract_id);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    client.start(&player, &commitment, &0i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(Slicer, ());
    let client = SlicerClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_seed_output() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let admin = Address::generate(&env);
    let contract_id = env.register(Slicer, ());
    let client = SlicerClient::new(&env, &contract_id);
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
    let proof_hex = "1b584a1d090e3537ccce8733a15d750d5027997e350ac78c87df1b85ae58ad862714e0dcf162aa456adb4944292ecbef49acc44434c8a5912e319984817695f217e5c0d83ea29a9b99a6ee1ed3006ce23c7dc825d63c7e40a95c963e079f1f6403e9af7b90c7cf5c76d3f3beac864589abfef430d54984c5fee35b2826c096541e60662256e716e98e9019236b378c8945e79a1462448f4fb141d0b775bf33ea1fadad52d3c549ed24eda72a2afdf9efb986b4e15162c7b03d5d8312f396386727f3ea4e9ab67d050f235bd97a35be66c7aacd76e78acee74c9b161558c4d6eb2302be0439d11473714356af23fc27b141a09acf05f9552f5ea6dfda9beab25b";
    let proof = hex_decode(&env, proof_hex);
    assert_eq!(proof.len(), 256);

    // Public inputs: [level_hash, polygon_count(2), object_count(12), partition_count(12)]
    let public_inputs = [
        U256::from_be_bytes(&env, &hex_decode(&env, "2a84df0b7b3159ddafce1f916226d7df8523fe5e57210535f6b1607accaff8ea").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "0000000000000000000000000000000000000000000000000000000000000002").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000a").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000a").try_into().unwrap()),
    ];

    let result = verify_groth16(&env, &circuit::KEYS, &proof, &public_inputs);
    assert!(result, "Real Groth16 proof should verify");
    let polygon_count = circuit::to_u32(&public_inputs[1]);
    let object_count = circuit::to_u32(&public_inputs[2]);
    let partition_count = circuit::to_u32(&public_inputs[3]);
    let score = crate::slicer::calculate_score(polygon_count, object_count, partition_count);
    assert_eq!(polygon_count, 2);
    assert_eq!(object_count, 10);
    assert_eq!(partition_count, 10);
    assert_eq!(score, 140);

    println!("Real Groth16 proof verification: PASS (score={})", score);
}

#[test]
fn test_circom_tampered_proof() {
    let env = Env::default();

    // Correct proof but wrong level_hash (tamper public input, not proof bytes).
    let proof_hex = "2e348ddd020a1df34bda2e17dc5e617b4024ed6a9285dcb225f4ea51345c5e8a2711b060d20ef83d6ffd3bfff4f25dde2a168f649d8b11f578deb3450952c85808650ede25d495d41bb3417b25df484292ec16cae09268123e55eb9c6f5a4d75146ad4983d5700fc6bee5be8db1594bb42e313a3717f414d9233835aa2968a041c7c64ad7563409664b6f8d44dafbaad83013adb64d13aa6c0e06cbaaab6cdb31bd3cc21721f0eb9c0f1b6623996b0e3401af5f5fd6c53081a24ac79788672162757910f1056e92f73bb1574b4a9587d5966030bf76cd8abaa2cabd2355c2b350ef9130ce22c8a7b453ed3c52baa964b4c2485abbdce30d348d8488f67d96dfe";
    let proof = hex_decode(&env, proof_hex);

    // Tamper level_hash (first byte changed).
    let public_inputs = [
        U256::from_be_bytes(&env, &hex_decode(&env, "ff33816e7f359937d83a45049b2dd69ec2cc7eb43537af912ee43c4e6e5e3edd").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "0000000000000000000000000000000000000000000000000000000000000002").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000c").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000c").try_into().unwrap()),
    ];

    let result = verify_groth16(&env, &circuit::KEYS, &proof, &public_inputs);
    assert!(!result, "Tampered level_hash should NOT verify");

    println!("Tampered proof rejection: PASS");
}

#[test]
fn test_circom_wrong_inputs() {
    let env = Env::default();

    // Correct proof but wrong polygon_count.
    let proof_hex = "2e348ddd020a1df34bda2e17dc5e617b4024ed6a9285dcb225f4ea51345c5e8a2711b060d20ef83d6ffd3bfff4f25dde2a168f649d8b11f578deb3450952c85808650ede25d495d41bb3417b25df484292ec16cae09268123e55eb9c6f5a4d75146ad4983d5700fc6bee5be8db1594bb42e313a3717f414d9233835aa2968a041c7c64ad7563409664b6f8d44dafbaad83013adb64d13aa6c0e06cbaaab6cdb31bd3cc21721f0eb9c0f1b6623996b0e3401af5f5fd6c53081a24ac79788672162757910f1056e92f73bb1574b4a9587d5966030bf76cd8abaa2cabd2355c2b350ef9130ce22c8a7b453ed3c52baa964b4c2485abbdce30d348d8488f67d96dfe";
    let proof = hex_decode(&env, proof_hex);
    let public_inputs = [
        U256::from_be_bytes(&env, &hex_decode(&env, "2d33816e7f359937d83a45049b2dd69ec2cc7eb43537af912ee43c4e6e5e3edd").try_into().unwrap()),
        // 3 instead of 2.
        U256::from_be_bytes(&env, &hex_decode(&env, "0000000000000000000000000000000000000000000000000000000000000003").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000c").try_into().unwrap()),
        U256::from_be_bytes(&env, &hex_decode(&env, "000000000000000000000000000000000000000000000000000000000000000c").try_into().unwrap()),
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