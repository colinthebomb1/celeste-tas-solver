use super::*;

pub fn set_lift_speed(state: &mut PlayerState, lift_speed: Vec2) {
    state.lift_speed = lift_speed;

    if lift_speed != NO_LIFT {
        state.last_lift_speed = lift_speed;
        state.lift_speed_timer = LIFT_SPEED_GRACE_TIME;
    }
}

pub fn lift_boost(state: &PlayerState) -> Vec2 {
    let lift = if state.lift_speed == NO_LIFT {
        state.last_lift_speed
    } else {
        state.lift_speed
    };

    Vec2 {
        x: lift.x.clamp(-LIFT_X_CAP, LIFT_X_CAP),
        y: lift.y.clamp(LIFT_Y_CAP, 0.0),
    }
}

pub fn move_exact_h(state: &mut PlayerState, room: &Room, amount: i32) -> bool {
    let step = amount.signum();
    for _ in 0..amount.abs() {
        if collides(state, room, step, 0) {
            state.remainder.x = 0.0;
            return true;
        }
        state.position.x += step;
    }
    false
}

pub fn move_exact_v(state: &mut PlayerState, room: &Room, amount: i32) -> bool {
    let step = amount.signum();
    for _ in 0..amount.abs() {
        if blocked(state, room, 0, step) {
            state.remainder.y = 0.0;
            return true;
        }
        state.position.y += step;
    }
    false
}

pub fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value > target {
        (value - max_move).max(target)
    } else {
        (value + max_move).min(target)
    }
}

pub fn dash_corner_correction_h(state: &mut PlayerState, room: &Room, ground: bool) -> bool {
    if !dashing(state) {
        return false;
    }

    let dir = sign(state.speed.x) as i32;

    if ground && duck_free_at(state, room, dir) {
        state.ducking = true;
        return true;
    }

    if state.speed.y == 0.0 && state.speed.x != 0.0 {
        for i in 1..=DASH_CORNER_CORRECTION {
            for dy in [i, -i] {
                if !collides(state, room, dir, dy) {
                    for _ in 0..i {
                        if collides(state, room, 0, dy.signum()) {
                            break;
                        }
                        state.position.y += dy.signum();
                    }
                    if !collides(state, room, dir, 0) {
                        state.position.x += dir;
                    }
                    return true;
                }
            }
        }
    }

    false
}

// ground is whether she was on the ground at the start of the frame
pub fn move_h(state: &mut PlayerState, room: &Room, amount: f32, ground: bool) {
    let new_remainder = state.remainder.x + amount;
    // C# Math.Round
    let movement_x = new_remainder.round_ties_even() as i32;
    state.remainder.x = new_remainder - (movement_x as f32);

    let step = movement_x.signum();
    for _ in 0..movement_x.abs() {
        if collides(state, room, step, 0) {
            if dash_corner_correction_h(state, room, ground) {
                state.remainder.x = 0.0;
                return;
            }

            if state.wall_speed_retention_timer <= 0.0 {
                state.wall_speed_retained = state.speed.x;
                state.wall_speed_retention_timer = WALL_SPEED_RETENTION_TIME;
            }

            state.remainder.x = 0.0;
            state.speed.x = 0.0;
            state.dash_attack_timer = 0.0;
            return;
        }
        state.position.x += step;
    }
}

pub fn move_v(state: &mut PlayerState, room: &Room, amount: f32) {
    let new_remainder = state.remainder.y + amount;
    // C# Math.Round
    let movement_y = new_remainder.round_ties_even() as i32;
    state.remainder.y = new_remainder - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if blocked(state, room, 0, step) {
            state.remainder.y = 0.0;

            if state.speed.y < 0.0 {
                if upward_corner_correction(state, room) {
                    return;
                }

                if state.var_jump_timer < VAR_JUMP_TIME - CEILING_VAR_JUMP_GRACE {
                    state.var_jump_timer = 0.0;
                }
            } else if state.speed.y > 0.0 {
                if dash_corner_correction_down(state, room) {
                    return;
                }

                // dash slide
                if state.dash_dir.x != 0.0 && state.dash_dir.y > 0.0 {
                    state.dash_dir = Vec2 {
                        x: sign(state.dash_dir.x),
                        y: 0.0,
                    };
                    state.speed.x *= DODGE_SLIDE_SPEED_MULT;
                    state.ducking = true;
                }
            }

            state.speed.y = 0.0;
            state.dash_attack_timer = 0.0;
            return;
        }
        state.position.y += step;
    }
}

pub fn nudge_v(state: &mut PlayerState, room: &Room, amount: f32) {
    let new_remainder = state.remainder.y + amount;
    let movement_y = new_remainder.round_ties_even() as i32;
    state.remainder.y = new_remainder - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if blocked(state, room, 0, step) {
            state.remainder.y = 0.0;
            return;
        }
        state.position.y += step;
    }
}

pub fn dash_jump_thru_nudge(state: &mut PlayerState, room: &Room) {
    for jump_thru in &room.jump_thrus {
        let depth = state.position.y - jump_thru.hitbox.top;

        if jump_thru.hitbox.overlaps(&hitbox(state, 0, 0)) && depth <= DASH_H_JUMP_THRU_NUDGE {
            for _ in 0..depth {
                if collides(state, room, 0, -1) {
                    break;
                }
                state.position.y -= 1;
            }
        }
    }
}

pub fn dash_corner_correction_down(state: &mut PlayerState, room: &Room) -> bool {
    if !dashing(state) || state.dash_started_on_ground {
        return false;
    }

    let mut dirs = Vec::new();
    if state.speed.x <= 0.0 {
        dirs.push(-1);
    }
    if state.speed.x >= 0.0 {
        dirs.push(1);
    }

    for dir in dirs {
        for i in 1..=DASH_CORNER_CORRECTION {
            if !blocked(state, room, dir * i, 1) {
                for _ in 0..i {
                    if collides(state, room, dir, 0) {
                        break;
                    }
                    state.position.x += dir;
                }
                if !blocked(state, room, 0, 1) {
                    state.position.y += 1;
                }
                return true;
            }
        }
    }

    false
}

pub fn upward_corner_correction(state: &mut PlayerState, room: &Room) -> bool {
    let slack = CORNER_CORRECTION_SPEED_SLACK;
    let reach = if state.dash_attack_timer > 0.0 && state.speed.x.abs() < slack {
        DASH_UPWARD_CORNER_CORRECTION
    } else {
        UPWARD_CORNER_CORRECTION
    };

    if state.speed.x <= slack {
        for i in 1..=reach {
            if !collides(state, room, -i, -1) {
                state.position.x -= i;
                state.position.y -= 1;
                return true;
            }
        }
    }

    if state.speed.x >= -slack {
        for i in 1..=reach {
            if !collides(state, room, i, -1) {
                state.position.x += i;
                state.position.y -= 1;
                return true;
            }
        }
    }

    false
}

pub fn lerp(value1: f32, value2: f32, amount: f32) -> f32 {
    value1 * (1.0 - amount) + value2 * amount
}

pub(crate) fn add_lift_boost(state: &mut PlayerState) {
    let boost = lift_boost(state);
    state.speed.x += boost.x;
    state.speed.y += boost.y;
}

pub fn update_wall_speed_retention(state: &mut PlayerState, room: &Room) {
    if state.wall_speed_retention_timer <= 0.0 {
        return;
    }

    let dir = sign(state.wall_speed_retained);

    if sign(state.speed.x) == -dir {
        state.wall_speed_retention_timer = 0.0;
    } else if !collides(state, room, dir as i32, 0) {
        state.speed.x = state.wall_speed_retained;
        state.wall_speed_retention_timer = 0.0;
    } else {
        state.wall_speed_retention_timer -= DELTA_TIME;
    }
}

// C# Math.Sign
pub(crate) fn sign(v: f32) -> f32 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}
