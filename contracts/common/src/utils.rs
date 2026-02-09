/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{Env, Vec};
use crate::types::TargetEntry;

pub fn roll_target(env: &Env, targets: &Vec<TargetEntry>,
    min: u32, max: u32, count: u32, expire: u64,
) -> (Vec<TargetEntry>, u32) {
    let now = env.ledger().timestamp();
    let range = max - min + 1;
    let mut pool: Vec<TargetEntry> = Vec::new(env);

    // Expired.
    for i in 0..targets.len() {
        let entry = targets.get(i).unwrap();
        if now.saturating_sub(entry.timestamp) < expire {
            pool.push_back(entry);
        }
    }

    // Overflow.
    while pool.len() > count {
        pool.pop_front();
    }

    // Synthetics.
    while pool.len() < count {
        let score = min + (env.prng().gen::<u64>() % range as u64) as u32;
        pool.push_back(TargetEntry { score, timestamp: now });
    }
    let index = (env.prng().gen::<u64>() % pool.len() as u64) as u32;
    let target = pool.get(index).unwrap().score;

    (pool, target)
}