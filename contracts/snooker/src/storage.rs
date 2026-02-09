/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::types::{StorageKey, Session};
use soroban_sdk::{Address, Env, Vec};
pub use common::types::{House, TargetEntry};

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Admin)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&StorageKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&StorageKey::Admin, admin);
}

pub fn get_house(env: &Env) -> Option<House> {
    env.storage().instance().get(&StorageKey::House)
}

pub fn set_house(env: &Env, house: &House) {
    env.storage().instance().set(&StorageKey::House, house);
}

pub fn get_session(env: &Env, player: &Address) -> Option<Session> {
    env.storage().temporary().get(&StorageKey::Session(player.clone()))
}

pub fn set_session(env: &Env, player: &Address, session: &Session) {
    env.storage().temporary().set(&StorageKey::Session(player.clone()), session);
    env.storage().temporary().extend_ttl(&StorageKey::Session(player.clone()), 17280u32, 7 * 17280u32);
}

pub fn remove_session(env: &Env, player: &Address) {
    env.storage().temporary().remove(&StorageKey::Session(player.clone()));
}

pub fn get_targets(env: &Env) -> Vec<TargetEntry> {
    env.storage().instance().get(&StorageKey::Targets).unwrap_or_else(|| Vec::new(env))
}

pub fn set_targets(env: &Env, targets: &Vec<TargetEntry>) {
    env.storage().instance().set(&StorageKey::Targets, targets);
}

pub fn extend_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    let threshold = max_ttl.saturating_sub(120_960);
    env.storage().instance().extend_ttl(threshold, max_ttl);
}