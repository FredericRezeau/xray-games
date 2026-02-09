/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{contracttype, contracterror, Address, Vec, BytesN};

#[derive(Clone, Copy)]
pub struct GameParams {
    pub max_balls: u32,
    pub diameter_squared: i128,
    pub radius_squared: i128,
    pub game_duration: u64,
    pub rand_range: u16,
    pub rand_offset: u16,
    pub target_scores: [u32; 5],
    pub target_count: u32,
    pub target_expire: u64,
}

impl GameParams {
    pub const fn default() -> Self {
        Self {
            max_balls: 5,
            diameter_squared: 1_000_000,
            radius_squared: 562_500,
            game_duration: 180,
            rand_range: 5001,
            rand_offset: 2500,
            target_scores: [21, 39, 66, 102, 147],
            target_count: 10,
            target_expire: 86400,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[contracttype]
pub struct Ball(pub i128, pub i128, pub i128, pub i128);
// Destructure as Ball(position_x, position_y, velocity_x, velocity_y)

#[derive(Clone, Copy, Debug)]
#[contracttype]
pub struct Pocket(pub i128, pub i128);
// Destructure as Pocket(position_x, position_y)

#[derive(Clone, Debug)]
#[contracttype]
pub struct Session {
    pub balls: Vec<Ball>,
    pub pockets: Vec<Pocket>,
    pub target: u32,
    pub timestamp: u64,
    pub id: Option<u32>,
    pub commitment: BytesN<32>,
}

pub struct Pool {
    pub cue_ball: Ball,
    pub color_ball: Ball,
    pub pocket: Pocket,
}

#[derive(Clone)]
#[contracttype]
pub enum StorageKey {
    Admin,
    Session(Address),
    House,
    Targets,
}

#[derive(Clone, Copy)]
#[contracterror]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvalidSession = 3,
    BadAuth = 4,
}
