use super::*;

pub fn set_lift_speed(player: &mut Player, lift_speed: Vec2) {
    player.lift.speed = lift_speed;

    if lift_speed != NO_LIFT {
        player.lift.last_speed = lift_speed;
        player.lift.timer = LIFT_SPEED_GRACE_TIME;
    }
}

pub fn liftboost(player: &Player) -> Vec2 {
    let lift = if player.lift.speed == NO_LIFT {
        player.lift.last_speed
    } else {
        player.lift.speed
    };

    Vec2 {
        x: lift.x.clamp(-LIFT_X_CAP, LIFT_X_CAP),
        y: lift.y.clamp(LIFT_Y_CAP, 0.0),
    }
}

pub fn move_exact_h(player: &mut Player, room: &Room, amount: i32) -> bool {
    let step = amount.signum();
    for _ in 0..amount.abs() {
        if collides(player, room, step, 0) {
            player.subpixel.x = 0.0;
            return true;
        }
        player.position.x += step;
    }
    false
}

pub fn move_exact_v(player: &mut Player, room: &Room, amount: i32) -> bool {
    let step = amount.signum();
    for _ in 0..amount.abs() {
        if blocked(player, room, 0, step) {
            player.subpixel.y = 0.0;
            return true;
        }
        player.position.y += step;
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

pub fn dash_corner_correction_h(player: &mut Player, room: &Room, ground: bool) -> bool {
    if !dashing(player) {
        return false;
    }

    let dir = sign(player.speed.x) as i32;

    if ground && crouch_free_at(player, room, dir) {
        player.crouching = true;
        return true;
    }

    if player.speed.y == 0.0 && player.speed.x != 0.0 {
        for i in 1..=DASH_CORNER_CORRECTION {
            for dy in [i, -i] {
                if !collides(player, room, dir, dy) {
                    for _ in 0..i {
                        if collides(player, room, 0, dy.signum()) {
                            break;
                        }
                        player.position.y += dy.signum();
                    }
                    if !collides(player, room, dir, 0) {
                        player.position.x += dir;
                    }
                    return true;
                }
            }
        }
    }

    false
}

// ground is whether she was on the ground at the start of the frame
pub fn move_h(player: &mut Player, room: &Room, amount: f32, ground: bool) {
    let new_subpixel = player.subpixel.x + amount;
    // C# Math.Round
    let movement_x = new_subpixel.round_ties_even() as i32;
    player.subpixel.x = new_subpixel - (movement_x as f32);

    let step = movement_x.signum();
    for _ in 0..movement_x.abs() {
        if collides(player, room, step, 0) {
            if dash_corner_correction_h(player, room, ground) {
                player.subpixel.x = 0.0;
                return;
            }

            if player.wall.retention_timer <= 0.0 {
                player.wall.retained_speed = player.speed.x;
                player.wall.retention_timer = RETENTION_TIME;
            }

            player.subpixel.x = 0.0;
            player.speed.x = 0.0;
            player.dash.attack_timer = 0.0;
            return;
        }
        player.position.x += step;
    }
}

pub fn move_v(player: &mut Player, room: &Room, amount: f32) {
    let new_subpixel = player.subpixel.y + amount;
    // C# Math.Round
    let movement_y = new_subpixel.round_ties_even() as i32;
    player.subpixel.y = new_subpixel - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if blocked(player, room, 0, step) {
            player.subpixel.y = 0.0;

            if player.speed.y < 0.0 {
                if upward_corner_correction(player, room) {
                    return;
                }

                if player.jump_timer < JUMP_HOLD_TIME - CEILING_JUMP_GRACE {
                    player.jump_timer = 0.0;
                }
            } else if player.speed.y > 0.0 {
                if dash_corner_correction_down(player, room) {
                    return;
                }

                // ultra
                if player.dash.dir.x != 0.0 && player.dash.dir.y > 0.0 {
                    player.dash.dir = Vec2 {
                        x: sign(player.dash.dir.x),
                        y: 0.0,
                    };
                    player.speed.x *= ULTRA_SPEED_MULT;
                    player.crouching = true;
                }
            }

            player.speed.y = 0.0;
            player.dash.attack_timer = 0.0;
            return;
        }
        player.position.y += step;
    }
}

pub fn nudge_v(player: &mut Player, room: &Room, amount: f32) {
    let new_subpixel = player.subpixel.y + amount;
    let movement_y = new_subpixel.round_ties_even() as i32;
    player.subpixel.y = new_subpixel - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if blocked(player, room, 0, step) {
            player.subpixel.y = 0.0;
            return;
        }
        player.position.y += step;
    }
}

pub fn dash_jump_thru_nudge(player: &mut Player, room: &Room) {
    for jump_thru in &room.jump_thrus {
        let depth = player.position.y - jump_thru.hitbox.top;

        if jump_thru.hitbox.overlaps(&hitbox(player, 0, 0)) && depth <= DASH_H_JUMP_THRU_NUDGE {
            for _ in 0..depth {
                if collides(player, room, 0, -1) {
                    break;
                }
                player.position.y -= 1;
            }
        }
    }
}

pub fn dash_corner_correction_down(player: &mut Player, room: &Room) -> bool {
    if !dashing(player) || player.dash.started_on_ground {
        return false;
    }

    let mut dirs = Vec::new();
    if player.speed.x <= 0.0 {
        dirs.push(-1);
    }
    if player.speed.x >= 0.0 {
        dirs.push(1);
    }

    for dir in dirs {
        for i in 1..=DASH_CORNER_CORRECTION {
            if !blocked(player, room, dir * i, 1) {
                for _ in 0..i {
                    if collides(player, room, dir, 0) {
                        break;
                    }
                    player.position.x += dir;
                }
                if !blocked(player, room, 0, 1) {
                    player.position.y += 1;
                }
                return true;
            }
        }
    }

    false
}

pub fn upward_corner_correction(player: &mut Player, room: &Room) -> bool {
    let slack = CORNER_CORRECTION_SPEED_SLACK;
    let reach = if player.dash.attack_timer > 0.0 && player.speed.x.abs() < slack {
        DASH_UPWARD_CORNER_CORRECTION
    } else {
        UPWARD_CORNER_CORRECTION
    };

    if player.speed.x <= slack {
        for i in 1..=reach {
            if !collides(player, room, -i, -1) {
                player.position.x -= i;
                player.position.y -= 1;
                return true;
            }
        }
    }

    if player.speed.x >= -slack {
        for i in 1..=reach {
            if !collides(player, room, i, -1) {
                player.position.x += i;
                player.position.y -= 1;
                return true;
            }
        }
    }

    false
}

pub fn lerp(value1: f32, value2: f32, amount: f32) -> f32 {
    value1 * (1.0 - amount) + value2 * amount
}

pub(crate) fn add_liftboost(player: &mut Player) {
    let boost = liftboost(player);
    player.speed.x += boost.x;
    player.speed.y += boost.y;
}

pub fn update_retention(player: &mut Player, room: &Room) {
    if player.wall.retention_timer <= 0.0 {
        return;
    }

    let dir = sign(player.wall.retained_speed);

    if sign(player.speed.x) == -dir {
        player.wall.retention_timer = 0.0;
    } else if !collides(player, room, dir as i32, 0) {
        player.speed.x = player.wall.retained_speed;
        player.wall.retention_timer = 0.0;
    } else {
        player.wall.retention_timer -= DELTA_TIME;
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
