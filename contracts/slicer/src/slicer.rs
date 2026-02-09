/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::circuit;
use crate::attestation::{verify_attestation};
use crate::types::{Error, Session, GameParams};
use crate::storage;
use common::ohloss::Ohloss;
use common::utils::roll_target;
use common::zk::{SeedGenerator, verify_groth16};
use common::types::{ Backend, House, TargetEntry };
use soroban_sdk::{ Address, Bytes, BytesN, Env};

const PARAMS: GameParams = GameParams::default();

fn ohloss(env: &Env) -> Option<Ohloss<'_>> {
    let house = storage::get_house(env)?;
    Some(Ohloss::new(env, &house))
}

pub(crate) fn calculate_score(polygons: u32, objects: u32, partitions: u32) -> u32 {
    let base = polygons * 30 + objects * 8;
    let bonus = if partitions > objects {
        (partitions - objects) * 15
    } else {
        0
    };
    base + bonus
}

pub fn start(env: &Env, player: Address, commitment: BytesN<32>, wager: i128) -> Result<Session, Error> {
    if !storage::has_admin(env) {
        return Err(Error::NotInitialized);
    }

    player.require_auth();

    // Reconnection.
    let now = env.ledger().timestamp();
    if let Some(session) = storage::get_session(env, &player) {
        if session.timestamp + PARAMS.duration >= now {
            return Ok(session);
        }
        return Err(Error::InvalidSession); // Must be settled (end) first.
    }

    let nonce: u64 = env.prng().gen();
    let generator = SeedGenerator::new(env);
    let seed = generator.generate(&player, nonce);
    let (targets, target) = roll_target(env, &storage::get_targets(env),
        PARAMS.min_target,
        PARAMS.max_target,
        PARAMS.target_count,
        PARAMS.target_expire,
    );
    storage::set_targets(env, &targets);

    // Ohloss.
    let id = if wager > 0 {
        ohloss(env).map(|ol| ol.start(&player, wager))
    } else {
        None
    };

    let result = Session { commitment, seed, nonce, target, timestamp: now, id };
    storage::set_session(env, &player, &result);

    Ok(result)
}

pub fn end(env: &Env, player: Address, preimage: BytesN<32>, proof: Bytes, attestation: Bytes) -> Result<u32, Error> {
    let session = storage::get_session(env, &player).ok_or(Error::InvalidSession)?;
    let expired = session.timestamp + PARAMS.duration < env.ledger().timestamp();
    let commitment: BytesN<32> = env.crypto().sha256(&Bytes::from_slice(env, &preimage.to_array())).into();
    let forfeit = commitment == session.commitment && (proof.len() == 0 || attestation.len() == 0);

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

    let verified = verify_attestation(env, &attestation);
    if verified.seed != session.seed {
        return Err(Error::InvalidSeed);
    }

    let (raw, public_inputs, outputs) = circuit::extract(env, &proof);
    let hash: BytesN<32> = outputs.level_hash.to_be_bytes().try_into().unwrap();
    let backend = circuit::get_backend(&hash, &verified);
    match backend {
        Backend::Circom => {
            // Full onchain verification (Groth16).
            if !verify_groth16(env, &circuit::KEYS, &raw, &public_inputs) {
                return Err(Error::InvalidProof);
            }
        }
        Backend::Noir => {
            // IMPORTANT. Soroban does not have primitives for Noir verification.
            // Only check attestation signature and require admin auth.
            storage::get_admin(env).require_auth();
        }
    }

    // Ohloss.
    let score = calculate_score(outputs.polygon_count, outputs.object_count, outputs.partition_count);
    let won = score >= session.target;
    if let Some(id) = session.id {
        if let Some(ol) = ohloss(env) {
            ol.end(id, won);
        }
    }

    if won {
        let mut targets = storage::get_targets(env);
        targets.push_back(TargetEntry { score, timestamp: env.ledger().timestamp() });
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

pub fn set_house(
    env: &Env,
    faction0: Address,
    faction1: Address,
    faction2: Address,
    min_wager: i128,
    ohloss: Address,
) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();
    let house = House { faction0, faction1, faction2, min_wager, ohloss };
    storage::set_house(env, &house);
    storage::extend_ttl(env);
    Ok(())
}

pub fn set_attestor(env: &Env, pubkey: BytesN<32>) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();
    storage::set_attestor(env, &pubkey);
    storage::extend_ttl(env);
    Ok(())
}
