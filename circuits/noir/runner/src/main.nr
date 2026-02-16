/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau <hello@kyungj.in>
*/

use poseidon::poseidon2::Poseidon2;

global MAX_PARABOLAS: u32 = 50;
global MAX_BYTES: u32 = 500;
global MAX_STEPS: u32 = 20;
global SCALE: i64 = 1000;
global GM: i64 = 29340;
global FRICTION: i64 = 9000;
global WALL_L: i64 = 2000;
global WALL_R: i64 = 9000;
global MID_X: i64 = 5500;
global DELTA: i64 = 100;
global MAX_SCENE_Y: i64 = 13000;
global BLOCK_HEIGHT: i64 = 1400;
global PLAYER_RADIUS: i64 = 450;
global PROOF_EPSILON: i64 = 1000;
global STATE_BOUNCING: u8 = 1;
global STATE_DASHING: u8 = 2;
global STATE_HOLDING: u8 = 3;
global STATE_READY: u8 = 4;
global BLOCK_NONE: u8 = 0;
global BLOCK_GLASS: u8 = 1;
global BLOCK_BUMPER: u8 = 2;

struct Point {
    x: i64,
    y: i64,
}

struct Parabola {
    x: u32,
    x_sign: u8,
    y: u32,
    y_sign: u8,
    vx: u32,
    vx_sign: u8,
    vy: u32,
    vy_sign: u8,
    vy_factor: u32,
    vy_factor_sign: u8,
    to_state: u8,
    from_state: u8,
    hint: u32,
}

fn hash2(a: Field, b: Field) -> Field {
    Poseidon2::hash([a, b], 2)
}

fn to_signed(abs_val: u32, sign: u8) -> i64 {
    let value = abs_val as i64;
    if sign == 0 { value } else { -value }
}

fn mul(a: i64, b: i64) -> i64 {
    (a * b) / SCALE
}

fn abs(value: i64) -> i64 {
    if value < 0 {
        -value
    } else {
        value
    }
}

fn apply_friction(value: i64, friction: i64) -> i64 {
    if value > 0 {
        let result = value - friction;
        if result < 0 { 0 } else { result }
    } else if value < 0 {
        let result = value + friction;
        if result > 0 { 0 } else { result }
    } else {
        0
    }
}

fn hash_level(level: [u8; MAX_BYTES], level_length: u32) -> Field {
    let mut hash: Field = 0;
    let mut acc: Field = 0;
    let mut count: u32 = 0;
    for i in 0..MAX_BYTES {
        if i < level_length {
            acc = acc * 256 + level[i] as Field;
            count += 1;
            if count == 31 {
                hash = hash2(hash, acc);
                acc = 0;
                count = 0;
            }
        }
    }
    if count > 0 {
        hash = hash2(hash, acc);
    }
    hash
}

fn calculate_parabola(start_x: i64, start_y: i64, start_vx: i64, start_vy: i64,
    start_vy_factor: i64, state: u8, target_step: u32) -> (Point, Point) {
    let mut x = start_x;
    let mut y = start_y;
    let mut vx = start_vx;
    let mut vy = start_vy;
    let mut vy_factor = start_vy_factor;
    let mut target_point = Point { x, y };
    let is_free_flight = (state != STATE_DASHING) & (state != STATE_HOLDING);
    let vy_fric = (FRICTION * 3 * DELTA) / SCALE;
    let vx_fric = (FRICTION * 3 / 2 * DELTA) / SCALE;

    for i in 0..MAX_STEPS {
        vy = apply_friction(vy, vy_fric);
        if vy_factor > 0 {
            vy_factor -= DELTA;
            if vy_factor < 0 { vy_factor = 0; }
        }

        vx = apply_friction(vx, vx_fric);
        x += mul(vx, DELTA);
        if is_free_flight {
            y += mul(vy - vy_factor, DELTA);
            y += mul(GM, DELTA);
        }

        if x <= WALL_L {
            x = WALL_L;
        }

        if x >= WALL_R {
            x = WALL_R;
        }

        if (i + 1) == target_step {
            target_point = Point { x, y };
        }
    }

    let endpoint = Point { x, y };
    (target_point, endpoint)
}

fn get_block(level: [u8; MAX_BYTES], row: u32, is_left: bool) -> u8 {
    let byte_idx = row / 2;
    let b = level[byte_idx];
    let nibble = if row % 2 == 0 {
        (b >> 4) & 0xF
    } else {
        b & 0xF
    };
    if is_left {
        (nibble >> 2) & 0x3
    } else {
        nibble & 0x3
    }
}

fn get_row(y: i64) -> u32 {
    let offset = MAX_SCENE_Y - BLOCK_HEIGHT - (y - PLAYER_RADIUS);
    let adjusted = offset - 200;
    let row_signed = if adjusted < 0 { 0 } else {
        (adjusted + BLOCK_HEIGHT / 2) / BLOCK_HEIGHT
    };
    row_signed as u32
}

fn main(
    // Public inputs.
    level_hash: pub Field,
    score: pub u32,

    // Private inputs.
    level: [u8; MAX_BYTES],
    level_length: u32,
    parabola_count: u32,
    parabolas: [Parabola; MAX_PARABOLAS],
) {
    assert(hash_level(level, level_length) == level_hash);
    assert(parabola_count >= 2);
    assert(parabola_count <= MAX_PARABOLAS);

    // let computed = hash_level(level, level_length);
    // std::println(computed);

    let mut computed_score: u32 = 0;
    let mut glass_slots: [u32; MAX_PARABOLAS] = [0xFFFF; MAX_PARABOLAS];
    let mut bumper_slots: [u32; MAX_PARABOLAS] = [0xFFFF; MAX_PARABOLAS];

    for i in 0..MAX_PARABOLAS {
        if (i > 0) & (i < parabola_count) {
            let prev = parabolas[i - 1];
            let curr = parabolas[i];
            let curr_x = to_signed(curr.x, curr.x_sign);
            let curr_y = to_signed(curr.y, curr.y_sign);
            let prev_x = to_signed(prev.x, prev.x_sign);
            let prev_y = to_signed(prev.y, prev.y_sign);
            let prev_vx = to_signed(prev.vx, prev.vx_sign);
            let prev_vy = to_signed(prev.vy, prev.vy_sign);
            let prev_vyf = to_signed(prev.vy_factor, prev.vy_factor_sign);
            let prev_is_holding = prev.to_state == STATE_HOLDING;
            if !prev_is_holding {
                let (hint_point, endpoint) = calculate_parabola(
                    prev_x, prev_y, prev_vx, prev_vy, prev_vyf,
                    prev.to_state, curr.hint
                );
                let is_bounce = (curr.to_state == STATE_READY) | (curr.to_state == STATE_BOUNCING);
                if is_bounce {
                    let dx = abs(curr_x - endpoint.x);
                    let dy = abs(curr_y - endpoint.y);
                    assert(dx <= PROOF_EPSILON);
                    assert(dy <= PROOF_EPSILON);
                } else {
                    let dist = abs(hint_point.x - curr_x) + abs(hint_point.y - curr_y);
                    assert(dist <= PROOF_EPSILON * 2);
                }
            }

            let is_block_check = (curr.to_state == STATE_HOLDING) | (curr.to_state == STATE_BOUNCING) | (curr.to_state == STATE_READY);
            if is_block_check {
                let row = get_row(curr_y);
                let is_left = curr_x <= MID_X;
                let block = get_block(level, row, is_left);
                let side: u32 = if is_left { 0 } else { 1 };
                let claim_idx = row * 2 + side;
                assert(block != BLOCK_NONE);
                if block == BLOCK_GLASS {
                    glass_slots[i] = claim_idx;
                }
                if block == BLOCK_BUMPER {
                    let mut scored = false;
                    for k in 0..MAX_PARABOLAS {
                        if (k < i) & (bumper_slots[k] == claim_idx) {
                            scored = true;
                        }
                    }
                    if !scored {
                        computed_score += 1;
                        bumper_slots[i] = claim_idx;
                    }
                }
            }
        }
    }

    for i in 0..MAX_PARABOLAS {
        if glass_slots[i] != 0xFFFF {
            for j in 0..MAX_PARABOLAS {
                if (j > i) & (glass_slots[j] != 0xFFFF) {
                    assert(glass_slots[i] != glass_slots[j]);
                }
            }
        }
    }

    assert(computed_score == score);
}

#[test]
fn test_apply_friction() {
    assert(apply_friction(1000, 300) == 700);
    assert(apply_friction(-1000, 300) == -700);
    assert(apply_friction(100, 300) == 0);
    assert(apply_friction(-100, 300) == 0);
    assert(apply_friction(0, 300) == 0);
}

#[test]
fn test_get_block() {
    let mut level: [u8; MAX_BYTES] = [0; MAX_BYTES];
    level[0] = 0xA0;
    assert(get_block(level, 0, true) == BLOCK_BUMPER);
    assert(get_block(level, 0, false) == BLOCK_BUMPER);
    assert(get_block(level, 1, true) == BLOCK_NONE);
}

#[test]
fn test_calculate_parabola() {
    let (_, endpoint) = calculate_parabola(5500, 13000, 35000, 0, 0, STATE_DASHING, 7);
    assert(endpoint.y == 13000);
    assert(endpoint.x == WALL_R);
}

#[test]
fn test_hash_deterministic() {
    assert(hash2(1, 2) == hash2(1, 2));
}
