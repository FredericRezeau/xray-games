/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::storage;
use crate::pool::Pool;
use crate::types::{Ball, Pocket, Session, Error, GameParams};
use common::ohloss::Ohloss;
use common::utils::roll_target;
use common::types::{House, TargetEntry};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, Vec};

const PARAMS: GameParams = GameParams::default();

fn ohloss(env: &Env) -> Option<Ohloss<'_>> {
    let house = storage::get_house(env)?;
    Some(Ohloss::new(env, &house))
}

pub fn score<F, B, P>(env: &Env, shots: &F, balls: &B, pockets: &P) -> u32 
where
    F: Fn(usize) -> Ball,
    B: Fn(usize) -> Ball,
    P: Fn(usize) -> Pocket,
{
    // Scoring rules based on the winning streak length (not actual snooker).
    // 147 still represents the maximum break.
    let mut score = 0_u32;
    let mut streak = 0_u32;
    for i in 0..(PARAMS.max_balls as usize) {
        let mut pool = Pool(shots(i), balls(i), pockets(i));
        if pool.is_potted(env) {
            streak += 1;
            if score == 0 {
                score += 12;
            }
        } else {
            streak = 0;
        }
        score += streak * 9;
    }
    score
}

pub fn start(env: &Env, player: Address, commitment: BytesN<32>, wager: i128) -> Result<Session, Error> {
    if !storage::has_admin(env) {
        return Err(Error::NotInitialized);
    }

    player.require_auth();

    // Handle reconnection.
    let now = env.ledger().timestamp();
    if let Some(session) = storage::get_session(env, &player) {
       if session.timestamp + PARAMS.game_duration >= now {
            return Ok(session);
        }
        return Err(Error::InvalidSession); // Must be settled (end) first.
    }

    let (targets, streak) = roll_target(env, &storage::get_targets(env),
        1,
        5,
        PARAMS.target_count,
        PARAMS.target_expire,
    );
    storage::set_targets(env, &targets);
    let target = PARAMS.target_scores[streak as usize - 1];

    // Prepare the session.
    let mut seed: u64 = env.prng().gen();
    let mut balls: Vec<Ball> = vec![env];
    let mut pockets: Vec<Pocket> = vec![env];
    for _i in 0..PARAMS.max_balls {
        balls.push_back(Ball(rand(&mut seed) as i128, 6000, 0, 0));
        pockets.push_back(Pocket(rand(&mut seed) as i128, 2000));
    }

    // Ohloss
    let id = if wager > 0 {
        ohloss(env).map(|ol| ol.start(&player, wager))
    } else {
        None
    };

    let new_session = Session { commitment, balls, pockets, target, timestamp: now, id };
    storage::set_session(env, &player, &new_session);

    Ok(new_session)
}

pub fn end(env: &Env, player: Address, preimage: BytesN<32>, shots: Vec<Ball>) -> Result<u32, Error> {
    let session = storage::get_session(env, &player).ok_or(Error::InvalidSession)?;
    let expired = session.timestamp + PARAMS.game_duration < env.ledger().timestamp();
    let commitment: BytesN<32> = env.crypto().sha256(&Bytes::from_slice(env, &preimage.to_array())).into();
    let forfeit = commitment == session.commitment && shots.len() == 0;

    if expired || forfeit {
        if let Some(id) = session.id {
            if let Some(ol) = ohloss(env) {
                ol.end(id, false);
            }
        }
        storage::remove_session(env, &player);
        return Ok(0);
    }

    if commitment != session.commitment {
        return Err(Error::BadAuth);
    }

    if session.balls.len() < PARAMS.max_balls
        || session.pockets.len() < PARAMS.max_balls
        || shots.len() < PARAMS.max_balls {
        return Err(Error::InvalidSession);
    }

    let score = score(
        env,
        &|i| shots.get(i as u32).unwrap(),
        &|i| session.balls.get(i as u32).unwrap(),
        &|i| session.pockets.get(i as u32).unwrap()
    );
    let won = score >= session.target;
    if let Some(id) = session.id {
        if let Some(ol) = ohloss(env) {
            ol.end(id, won);
        }
    }

    if won {
        let mut targets = storage::get_targets(env);
        let streak = PARAMS.target_scores.iter().position(|&s| s <= score).map(|i| i as u32 + 1).unwrap_or(1);
        targets.push_back(TargetEntry { score: streak, timestamp: env.ledger().timestamp() });
        storage::set_targets(env, &targets);
    }

    storage::remove_session(env, &player);

    Ok(score)
}

pub fn initialize(env: &Env, admin: Address) -> Result<(), Error> {
    if storage::has_admin(env) {
        return Err(Error::AlreadyInitialized);
    }
    storage::set_admin(env, &admin);
    storage::extend_ttl(env);

    Ok(())
}

pub fn upgrade(env: &Env, hash: BytesN<32>) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();
    env.deployer().update_current_contract_wasm(hash);
    storage::extend_ttl(env);

    Ok(())
}

pub fn set_house(env: &Env, faction0: Address, faction1: Address,
    faction2: Address, min_wager: i128, ohloss: Address) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();
    let house = House { faction0, faction1, faction2, min_wager, ohloss };
    storage::set_house(env, &house);
    storage::extend_ttl(env);

    Ok(())
}

// Simple RNG for randomizing balls positions.
fn rand(x: &mut u64) -> u16 {
    *x ^= *x << 21;
    *x ^= *x >> 35;
    *x ^= *x << 4;
    // Restrict the value to the range [0, (2^14 - 1 = 16383)]
    let mask: u64 = (1 << 14) - 1;
    let masked_value = *x & mask;
    (masked_value as u16) % PARAMS.rand_range + PARAMS.rand_offset
}