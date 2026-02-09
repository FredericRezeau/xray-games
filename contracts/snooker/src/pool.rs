/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::types::{Ball, Pocket, Pool, GameParams};
use soroban_sdk::Env;

const PARAMS: GameParams = GameParams::default();

impl Pool {
    pub fn is_potted(&mut self, _env: &Env) -> bool {
        let xd = self.color_ball.0 - self.cue_ball.0;
        let yd = self.color_ball.1 - self.cue_ball.1;

        // Avoid square root calculation.
        let distance_squared = xd * xd + yd * yd;
        if distance_squared < PARAMS.diameter_squared && distance_squared != 0 {
            // Momentum exchange.
            let mag_inv = (1_i128 << 36) / distance_squared;
            let nx = xd * mag_inv;
            let ny = yd * mag_inv;
            let rel = -self.cue_ball.2 * nx - self.cue_ball.3 * ny;
            self.cue_ball.2 += (rel * nx) >> 36;
            self.cue_ball.3 += (rel * ny) >> 36;
            self.color_ball.2 -= (rel * nx) >> 36;
            self.color_ball.3 -= (rel * ny) >> 36;
            let vx = self.color_ball.2;
            let vy = self.color_ball.3;
            let px = self.pocket.0 - self.color_ball.0;
            let py = self.pocket.1 - self.color_ball.1;
            if vx * px + vy * py <= 0 {
                return false;
            }
            // |cross(v, toPocket)|^2 <= r^2 * |v|^2
            let cross = vx * py - vy * px;
            let v_len_sq = vx * vx + vy * vy;
            return cross * cross <= PARAMS.radius_squared * v_len_sq;
        }
        false
    }
}

#[allow(non_snake_case)]
pub fn Pool(cue_ball: Ball, color_ball: Ball, pocket: Pocket) -> Pool {
    Pool {
        cue_ball,
        color_ball,
        pocket,
    }
}
