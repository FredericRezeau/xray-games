#![cfg(test)]
extern crate std;
use crate::zk::{hash, u32_to_field, u64_to_field, SeedGenerator};
use crate::utils::roll_target;
use crate::types::TargetEntry;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
use std::{format, println, string::String};

fn hex_encode(bytes: &soroban_sdk::Bytes) -> String {
    let mut s = String::new();
    for i in 0..bytes.len() {
        s.push_str(&format!("{:02x}", bytes.get(i).unwrap()));
    }
    s
}

#[test]
fn test_seed() {
    let env = Env::default();
    let player = Address::generate(&env);
    let nonce = 12345u64;
    let generator = SeedGenerator::new(&env);
    let seed1 = generator.generate(&player, nonce);
    let seed2 = generator.generate(&player, nonce);
    assert_eq!(seed1, seed2);
}

#[test]
fn test_seed_nonce() {
    let env = Env::default();
    let player = Address::generate(&env);
    let generator = SeedGenerator::new(&env);
    let seed1 = generator.generate(&player, 111);
    let seed2 = generator.generate(&player, 222);
    assert_ne!(seed1, seed2);
}

#[test]
fn test_player_seed() {
    let env = Env::default();
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let nonce = 12345u64;
    let generator = SeedGenerator::new(&env);
    let seed1 = generator.generate(&player1, nonce);
    let seed2 = generator.generate(&player2, nonce);
    assert_ne!(seed1, seed2);
}

#[test]
fn test_hash() {
    let env = Env::default();
    let a = u64_to_field(&env, 123);
    let b = u64_to_field(&env, 456);
    let h1 = hash(&env, &a, &b);
    let h2 = hash(&env, &a, &b);
    assert_eq!(h1, h2);
}

#[test]
fn test_to_field() {
    let env = Env::default();
    let f1 = u32_to_field(&env, 0xDEADBEEF);
    let f2 = u64_to_field(&env, 0xDEADBEEF);
    assert_eq!(f1, f2);
}

#[test]
fn test_output_seed() {
    // Integration testing.
    let env = Env::default();
    let player = Address::generate(&env);
    let nonce = 42u64;
    let generator = SeedGenerator::new(&env);
    let seed = generator.generate(&player, nonce);
    let seed_bytes = seed.to_be_bytes();

    println!("Player address: {:?}", player);
    println!("Nonce: {}", nonce);
    println!("Seed (hex): 0x{}", hex_encode(&seed_bytes));

    let a = u64_to_field(&env, 1);
    let b = u64_to_field(&env, 2);
    let simple_hash = hash(&env, &a, &b);
    let hash_bytes = simple_hash.to_be_bytes();

    println!("hash(1, 2) = 0x{}", hex_encode(&hash_bytes));
}

#[test]
fn test_roll_target() {
    let env = Env::default();
    #[soroban_sdk::contract]
    pub struct Dummy;
    #[soroban_sdk::contractimpl]
    impl Dummy {}
    let contract_id = env.register(Dummy, ());
    env.as_contract(&contract_id, || {
        // Synthetics.
        let empty: Vec<TargetEntry> = Vec::new(&env);
        let (pool, target) = roll_target(&env, &empty, 140, 232, 5, 86400);
        assert_eq!(pool.len(), 5);
        assert!(target >= 140 && target <= 232);

        // Overflow.
        let mut targets = pool;
        targets.push_back(TargetEntry { score: 237, timestamp: 1000 });
        targets.push_back(TargetEntry { score: 140, timestamp: 1000 });
        targets.push_back(TargetEntry { score: 140, timestamp: 1000 });
        targets.push_back(TargetEntry { score: 140, timestamp: 1000 });
        targets.push_back(TargetEntry { score: 140, timestamp: 1000 });
        let (_, _) = roll_target(&env, &targets, 140, 235, 5, 86400);
        targets.push_back(TargetEntry { score: 140, timestamp: 1000 });
        let (pool, _) = roll_target(&env, &targets, 140, 235, 5, 86400);
        assert_eq!(pool.len(), 5);
        assert!(!(0..pool.len()).any(|i| pool.get(i).unwrap().score == 237), "237 gone");

        println!("roll_target: PASS");
    });
}