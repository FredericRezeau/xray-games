/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Copy, PartialEq)]
pub enum Backend {
    Circom,
    Noir,
}

// Groth16 verification keys.
// Each circuit should define own instance.
pub struct VerificationKeys {
    pub alpha: [u8; 64],
    pub beta: [u8; 128],
    pub gamma: [u8; 128],
    pub delta: [u8; 128],
    pub ic: &'static [[u8; 64]],
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct House {
    pub faction0: Address,
    pub faction1: Address,
    pub faction2: Address,
    pub min_wager: i128,
    pub ohloss: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct OhlossPlayer {
    pub selected_faction: u32,
    pub time_multiplier_start: u64,
    pub last_epoch_balance: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TargetEntry {
    pub score: u32,
    pub timestamp: u64,
}