/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![cfg(test)]
extern crate std;
use std::println;
use super::*;
use crate::snooker::{score, streak};
use crate::types::{Ball, Pocket};
use soroban_sdk::{
    testutils::{Address as _, Logs},
    vec, Env, Bytes,
};

#[test]
fn test_game_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let admin = Address::generate(&env);
    let contract_id = env.register(Snooker, ());
    let client = SnookerClient::new(&env, &contract_id);
    client.initialize(&admin);

    let preimage = BytesN::from_array(&env, &[42u8; 32]);
    let commitment: BytesN<32> = env.crypto()
        .sha256(&Bytes::from_slice(&env, &preimage.to_array()))
        .into();
    let session = client.start(&player, &commitment, &0i128);
    println!("Session: {:?}", &session);
    assert_eq!(session.balls.len(), 5);
    assert_eq!(session.pockets.len(), 5);
    let shots = vec![
        &env,
        Ball(6017, 6900, 0, 0),
        Ball(6099, 6949, 0, 0),
        Ball(6035, 6971, 0, 0),
        Ball(5875, 6875, 0, 0),
        Ball(6029, 5796, 0, 0),
    ];
    let score = client.end(&player, &preimage, &shots);
    assert_eq!(score, 0);
    // println!("{:?}", env.cost_estimate().budget());
    println!("{}", env.logs().all().join("\n"));
}

#[test]
fn test_score_147() {
    let env = Env::default();
    let balls = [
        Ball(5337, 6000, 0, 0),
        Ball(4714, 6000, 0, 0),
        Ball(4379, 6000, 0, 0),
        Ball(5408, 6000, 0, 0),
        Ball(5112, 6000, 0, 0),
    ];
    let pockets = [
        Pocket(3311, 2000),
        Pocket(2908, 2000),
        Pocket(6354, 2000),
        Pocket(7224, 2000),
        Pocket(2946, 2000),
    ];
    let shots = [
        Ball(5719, 6883, 2696, -11693),
        Ball(5000, 6850, 0, -11118),
        Ball(3981, 6872, -2910, -8936),
        Ball(5155, 6840, 582, -11846),
        Ball(5540, 6846, 2026, -11828),
    ];
    let _expected_wins = [true, true, true, true, true];
    let score = score(
        &env,
        &|i| shots[i],
        &|i| balls[i],
        &|i| pockets[i]
    );
    assert_eq!(score, 147);
}

#[test]
fn test_score_66() {
    let env = Env::default();
    let balls = [
        Ball(4832, 6000, 0, 0),
        Ball(3852, 6000, 0, 0),
        Ball(4955, 6000, 0, 0),
        Ball(5014, 6000, 0, 0),
        Ball(4286, 6000, 0, 0),
    ];
    let pockets = [
        Pocket(5219, 2000),
        Pocket(2657, 2000),
        Pocket(7334, 2000),
        Pocket(3436, 2000),
        Pocket(4562, 2000),
    ];
    let shots = [
        Ball(4783, 6807, -813, -11972),
        Ball(4147, 6801, -2693, -10100),
        Ball(4156, 6599, -2110, -8499),
        Ball(5358, 6821, 1342, -11925),
        Ball(4320, 6891, -2401, -10973),
    ];
    let _expected_wins = [true, true, false, true, true];
    let score = score(
        &env,
        &|i| shots[i],
        &|i| balls[i],
        &|i| pockets[i]
    );
    assert_eq!(score, 66);
}

#[test]
fn test_streaks() {
    // Below minimum — clamp to 1.
    assert_eq!(streak(0), 1);
    assert_eq!(streak(20), 1);

    // Exact thresholds.
    assert_eq!(streak(21), 1);
    assert_eq!(streak(39), 2);
    assert_eq!(streak(66), 3);
    assert_eq!(streak(102), 4);
    assert_eq!(streak(147), 5);

    // Inexact thresholds.
    assert_eq!(streak(30), 1);
    assert_eq!(streak(50), 2);
    assert_eq!(streak(80), 3);
    assert_eq!(streak(120), 4);

    // Max score still 5.
    assert_eq!(streak(200), 5);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_upgrade() {
    let env = Env::default();
    let contract_id = env.register(Snooker, ());
    let client = SnookerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let wasm = include_bytes!("../../../target/wasm32v1-none/release/chain_snooker.wasm");
    let wasm_hash = env.deployer().upload_contract_wasm(Bytes::from_slice(&env, wasm));
    env.mock_all_auths();
    client.initialize(&admin);
    client.upgrade(&wasm_hash);
    client.initialize(&admin)
}
