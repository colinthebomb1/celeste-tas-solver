use super::collision::{blocked, collides, crouch_free_at, hitbox};
use super::constants::*;
use super::dash::dashing;
use super::state::Player;
use crate::entities::Entities;
use crate::geom::Vec2;
use crate::level::room::Room;

pub(crate) fn set_lift_speed(player: &mut Player, lift_speed: Vec2) {
    player.lift.speed = lift_speed;

    if lift_speed != NO_LIFT {
        player.lift.last_speed = lift_speed;
        player.lift.timer = LIFT_SPEED_GRACE_TIME;
    }
}

pub(crate) fn liftboost(player: &Player) -> Vec2 {
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

pub(crate) fn move_exact_h(
    player: &mut Player,
    room: &Room,
    entities: &Entities,
    amount: i32,
) -> bool {
    let step = amount.signum();
    for _ in 0..amount.abs() {
        if collides(player, room, entities, step, 0) {
            player.subpixel.x = 0.0;
            return true;
        }
        player.position.x += step;
    }
    false
}

pub(crate) fn move_exact_v(
    player: &mut Player,
    room: &Room,
    entities: &Entities,
    amount: i32,
) -> bool {
    let step = amount.signum();
    for _ in 0..amount.abs() {
        if blocked(player, room, entities, 0, step) {
            player.subpixel.y = 0.0;
            return true;
        }
        player.position.y += step;
    }
    false
}

pub(crate) fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value > target {
        (value - max_move).max(target)
    } else {
        (value + max_move).min(target)
    }
}

fn dash_corner_correction_h(
    player: &mut Player,
    room: &Room,
    entities: &Entities,
    ground: bool,
) -> bool {
    if !dashing(player) {
        return false;
    }

    let dir = sign(player.speed.x) as i32;

    if ground && crouch_free_at(player, room, entities, dir) {
        player.crouching = true;
        return true;
    }

    if player.speed.y == 0.0 && player.speed.x != 0.0 {
        for i in 1..=DASH_CORNER_CORRECTION {
            for dy in [i, -i] {
                if !collides(player, room, entities, dir, dy) {
                    for _ in 0..i {
                        if collides(player, room, entities, 0, dy.signum()) {
                            break;
                        }
                        player.position.y += dy.signum();
                    }
                    if !collides(player, room, entities, dir, 0) {
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
pub(crate) fn move_h(
    player: &mut Player,
    room: &Room,
    entities: &Entities,
    amount: f32,
    ground: bool,
) {
    let new_subpixel = player.subpixel.x + amount;
    // C# Math.Round
    let movement_x = new_subpixel.round_ties_even() as i32;
    player.subpixel.x = new_subpixel - (movement_x as f32);

    let step = movement_x.signum();
    for _ in 0..movement_x.abs() {
        if collides(player, room, entities, step, 0) {
            if dash_corner_correction_h(player, room, entities, ground) {
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

pub(crate) fn move_v(player: &mut Player, room: &Room, entities: &Entities, amount: f32) {
    let new_subpixel = player.subpixel.y + amount;
    // C# Math.Round
    let movement_y = new_subpixel.round_ties_even() as i32;
    player.subpixel.y = new_subpixel - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if blocked(player, room, entities, 0, step) {
            player.subpixel.y = 0.0;

            if player.speed.y < 0.0 {
                if upward_corner_correction(player, room, entities) {
                    return;
                }

                if player.jump_timer < JUMP_HOLD_TIME - CEILING_JUMP_GRACE {
                    player.jump_timer = 0.0;
                }
            } else if player.speed.y > 0.0 {
                if dash_corner_correction_down(player, room, entities) {
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

pub(crate) fn nudge_v(player: &mut Player, room: &Room, entities: &Entities, amount: f32) {
    let new_subpixel = player.subpixel.y + amount;
    let movement_y = new_subpixel.round_ties_even() as i32;
    player.subpixel.y = new_subpixel - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if blocked(player, room, entities, 0, step) {
            player.subpixel.y = 0.0;
            return;
        }
        player.position.y += step;
    }
}

pub(crate) fn dash_jump_thru_nudge(player: &mut Player, room: &Room, entities: &Entities) {
    for jump_thru in &room.jump_thrus {
        let depth = player.position.y - jump_thru.hitbox.top;

        if jump_thru.hitbox.overlaps(&hitbox(player, 0, 0)) && depth <= DASH_JUMP_THRU_NUDGE {
            for _ in 0..depth {
                if collides(player, room, entities, 0, -1) {
                    break;
                }
                player.position.y -= 1;
            }
        }
    }
}

fn dash_corner_correction_down(player: &mut Player, room: &Room, entities: &Entities) -> bool {
    if !dashing(player) || player.dash.started_on_ground {
        return false;
    }

    let slack = CORNER_CORRECTION_SPEED_SLACK;
    let mut dirs = Vec::new();
    if player.speed.x <= slack {
        dirs.push(-1);
    }
    if player.speed.x >= -slack {
        dirs.push(1);
    }

    for dir in dirs {
        for i in 1..=DASH_CORNER_CORRECTION {
            if !blocked(player, room, entities, dir * i, 1) {
                for _ in 0..i {
                    if collides(player, room, entities, dir, 0) {
                        break;
                    }
                    player.position.x += dir;
                }
                if !blocked(player, room, entities, 0, 1) {
                    player.position.y += 1;
                }
                return true;
            }
        }
    }

    false
}

fn upward_corner_correction(player: &mut Player, room: &Room, entities: &Entities) -> bool {
    let slack = CORNER_CORRECTION_SPEED_SLACK;
    let reach = if player.dash.attack_timer > 0.0 && player.speed.x.abs() < slack {
        DASH_UPWARD_CORNER_CORRECTION
    } else {
        UPWARD_CORNER_CORRECTION
    };

    if player.speed.x <= slack {
        for i in 1..=reach {
            if !collides(player, room, entities, -i, -1) {
                player.position.x -= i;
                player.position.y -= 1;
                return true;
            }
        }
    }

    if player.speed.x >= -slack {
        for i in 1..=reach {
            if !collides(player, room, entities, i, -1) {
                player.position.x += i;
                player.position.y -= 1;
                return true;
            }
        }
    }

    false
}

pub(crate) fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    (from as f64 + (to as f64 - from as f64) * amount as f64) as f32
}

pub(crate) fn add_liftboost(player: &mut Player) {
    let boost = liftboost(player);
    player.speed.x += boost.x;
    player.speed.y += boost.y;
}

pub(crate) fn update_retention(player: &mut Player, room: &Room, entities: &Entities) {
    if player.wall.retention_timer <= 0.0 {
        return;
    }

    let dir = sign(player.wall.retained_speed);

    if sign(player.speed.x) == -dir {
        player.wall.retention_timer = 0.0;
    } else if !collides(player, room, entities, dir as i32, 0) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::IVec2;
    use crate::player::collision::on_ground;
    use crate::player::fixtures::*;
    use crate::player::jump::jump;
    use crate::player::state::Input;
    use crate::player::step::step;

    #[test]
    fn approach_moves_toward_target() {
        assert_eq!(approach(0.0, 10.0, 3.0), 3.0);
        assert_eq!(approach(10.0, 0.0, 3.0), 7.0);
    }

    #[test]
    fn approach_does_not_overshoot() {
        assert_eq!(approach(9.0, 10.0, 3.0), 10.0);
        assert_eq!(approach(1.0, 0.0, 3.0), 0.0);
        assert_eq!(approach(-1.0, 0.0, 3.0), 0.0);
    }

    #[test]
    fn approach_at_target_stays() {
        assert_eq!(approach(5.0, 5.0, 3.0), 5.0);
    }

    #[test]
    fn move_h_small_amount_only_changes_subpixel() {
        let room = room();
        let mut s = at_rest();
        move_h(&mut s, &room, &room.initial, 0.25, false);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.subpixel.x, 0.25);
    }

    #[test]
    fn move_h_rounds_to_whole_pixels() {
        let room = room();
        let mut s = at_rest();
        move_h(&mut s, &room, &room.initial, 0.75, false);
        assert_eq!(s.position.x, 1);
        assert_eq!(s.subpixel.x, -0.25);

        move_h(&mut s, &room, &room.initial, -2.0, false);
        assert_eq!(s.position.x, -1);
        assert_eq!(s.subpixel.x, -0.25);
    }

    #[test]
    fn move_h_ties_round_to_even_like_csharp() {
        let room = room();
        let mut s = at_rest();
        move_h(&mut s, &room, &room.initial, 0.5, false);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.subpixel.x, 0.5);

        move_h(&mut s, &room, &room.initial, 1.0, false);
        assert_eq!(s.position.x, 2);
        assert_eq!(s.subpixel.x, -0.5);
    }

    #[test]
    fn on_ground_when_standing_on_floor() {
        let room = room();
        assert!(on_ground(&standing(), &room, &room.initial));
    }

    #[test]
    fn move_v_moves_up_freely() {
        let room = room();
        let mut s = standing();
        move_v(&mut s, &room, &room.initial, -1.75);
        assert_eq!(s.position.y, FLOOR_Y - 2);
        assert_eq!(s.subpixel.y, 0.25);
    }

    #[test]
    fn move_v_stops_at_floor() {
        let room = room();
        let mut s = standing();
        s.position.y = FLOOR_Y - 2;
        s.subpixel.y = 0.3;
        s.speed.y = 160.0;
        move_v(&mut s, &room, &room.initial, 2.6);
        assert_eq!(s.position.y, FLOOR_Y);
        assert_eq!(s.subpixel.y, 0.0);
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn move_h_stops_at_wall() {
        let room = room();
        let mut s = at_rest();
        s.position.x = WALL_X - 1;
        s.subpixel.x = 0.3;
        s.speed.x = 90.0;
        move_h(&mut s, &room, &room.initial, 1.5, false);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(s.subpixel.x, 0.0);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn move_h_reaching_wall_exactly_is_not_a_collision() {
        let room = room();
        let mut s = at_rest();
        s.position.x = WALL_X - 1;
        s.speed.x = 90.0;
        move_h(&mut s, &room, &room.initial, 1.25, false);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(s.subpixel.x, 0.25);
        assert_eq!(s.speed.x, 90.0);
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        assert_eq!(lerp(160.0, 20.0, 0.0), 160.0);
        assert_eq!(lerp(160.0, 20.0, 1.0), 20.0);
        assert_eq!(lerp(160.0, 20.0, 0.5), 90.0);
    }

    #[test]
    fn lerp_is_summed_in_doubles() {
        // sixteen frames into a wall slide. summed in f32 this comes out as 51.111107
        assert_eq!(lerp(160.0, 20.0, 0.7777778), 51.11111);
    }

    #[test]
    fn move_v_falls_through_floor_level_past_ledge() {
        let room = room();
        let mut s = standing();
        s.position.x = LEDGE_X + 1;
        s.speed.y = 60.0;
        move_v(&mut s, &room, &room.initial, 1.0);
        assert_eq!(s.position.y, FLOOR_Y + 1);
        assert_eq!(s.speed.y, 60.0);
    }

    #[test]
    fn move_h_blocked_by_side_of_ledge() {
        let room = room();
        let mut s = at_rest();
        s.position = IVec2 {
            x: LEDGE_X + 1,
            y: FLOOR_Y + 5,
        };
        s.speed.x = -90.0;
        move_h(&mut s, &room, &room.initial, -3.0, false);
        assert_eq!(s.position.x, LEDGE_X + 1);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn move_h_into_a_wall_remembers_her_speed() {
        let room = room();
        let mut s = player_at(WALL_X - 1, 100, 120.0, 0.0);
        move_h(&mut s, &room, &room.initial, 3.0, false);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall.retained_speed, 120.0);
        assert_eq!(s.wall.retention_timer, RETENTION_TIME);
    }

    #[test]
    fn move_h_keeps_the_first_speed_while_the_timer_runs() {
        let room = room();
        let mut s = player_at(WALL_X - 1, 100, 50.0, 0.0);
        s.wall.retained_speed = 200.0;
        s.wall.retention_timer = 0.03;
        move_h(&mut s, &room, &room.initial, 3.0, false);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall.retained_speed, 200.0);
        assert_eq!(s.wall.retention_timer, 0.03);
    }

    #[test]
    fn move_h_without_a_collision_retains_nothing() {
        let room = room();
        let mut s = player_at(0, 100, 120.0, 0.0);
        move_h(&mut s, &room, &room.initial, 3.0, false);
        assert_eq!(s.wall.retention_timer, 0.0);
    }

    #[test]
    fn retention_gives_the_speed_back_once_the_wall_is_gone() {
        let room = room();
        let mut s = retaining(WALL_X - 1, 0.0, 120.0);
        update_retention(&mut s, &room, &room.initial);
        assert_eq!(s.speed.x, 120.0);
        assert_eq!(s.wall.retention_timer, 0.0);
    }

    #[test]
    fn retention_waits_while_the_wall_is_still_there() {
        let room = room();
        let mut s = retaining(WALL_X, 0.0, 120.0);
        update_retention(&mut s, &room, &room.initial);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall.retention_timer, 0.03 - DELTA_TIME);
    }

    #[test]
    fn retention_is_cancelled_by_moving_the_other_way() {
        let room = room();
        let mut s = retaining(WALL_X - 1, -10.0, 120.0);
        update_retention(&mut s, &room, &room.initial);
        assert_eq!(s.speed.x, -10.0);
        assert_eq!(s.wall.retention_timer, 0.0);
    }

    #[test]
    fn retention_works_to_the_left() {
        let room = left_wall_room();
        let mut s = retaining(LEFT_WALL_X, 0.0, -120.0);
        update_retention(&mut s, &room, &room.initial);
        assert_eq!(s.speed.x, 0.0);

        let mut s = retaining(LEFT_WALL_X + 1, 0.0, -120.0);
        update_retention(&mut s, &room, &room.initial);
        assert_eq!(s.speed.x, -120.0);
    }

    #[test]
    fn retention_does_nothing_once_the_timer_is_out() {
        let room = room();
        let mut s = retaining(WALL_X - 1, 0.0, 120.0);
        s.wall.retention_timer = 0.0;
        update_retention(&mut s, &room, &room.initial);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn step_running_into_a_wall_starts_retention() {
        let room = room();
        let s = step(
            player_at(WALL_X, 100, 90.0, 0.0),
            Input {
                move_x: 1,
                ..Default::default()
            },
            &room,
            &room.initial,
        );
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall.retained_speed, 90.0);
        assert_eq!(s.wall.retention_timer, RETENTION_TIME);
    }

    #[test]
    fn collides_uses_the_shorter_hitbox_when_crouching() {
        let room = tunnel_room();
        assert!(!collides(
            &crouching_at(40, FLOOR_Y),
            &room,
            &room.initial,
            0,
            0
        ));
        assert!(collides(
            &player_at(40, FLOOR_Y, 0.0, 0.0),
            &room,
            &room.initial,
            0,
            0
        ));
        assert!(collides(
            &crouching_at(40, FLOOR_Y),
            &room,
            &room.initial,
            0,
            1
        ));
        assert!(collides(
            &crouching_at(40, FLOOR_Y),
            &room,
            &room.initial,
            0,
            -3
        ));
    }

    #[test]
    fn step_crouches_when_down_is_held_on_the_ground() {
        let room = room();
        assert!(step(standing(), DOWN, &room, &room.initial).crouching);
    }

    #[test]
    fn step_does_not_crouch_in_the_air() {
        let room = room();
        assert!(!step(at_rest(), DOWN, &room, &room.initial).crouching);
    }

    #[test]
    fn step_stands_up_when_down_is_released() {
        let room = room();
        assert!(
            !step(
                crouching_at(0, FLOOR_Y),
                Input::default(),
                &room,
                &room.initial
            )
            .crouching
        );
    }

    #[test]
    fn step_stays_crouched_while_down_is_held() {
        let room = room();
        assert!(step(crouching_at(0, FLOOR_Y), DOWN, &room, &room.initial).crouching);
    }

    #[test]
    fn step_stays_crouched_under_a_low_ceiling() {
        let room = tunnel_room();
        assert!(
            step(
                crouching_at(40, FLOOR_Y),
                Input::default(),
                &room,
                &room.initial
            )
            .crouching
        );
    }

    #[test]
    fn step_crouching_on_the_ground_uses_crouch_friction() {
        let room = room();
        let mut s = crouching_at(0, FLOOR_Y);
        s.speed.x = 90.0;
        s = step(s, Input { move_x: 1, ..DOWN }, &room, &room.initial);
        assert_eq!(s.speed.x, approach(90.0, 0.0, CROUCH_FRICTION * DELTA_TIME));
    }

    #[test]
    fn step_stands_up_when_falling() {
        let room = room();
        let mut s = crouching_at(0, 64);
        s.speed.y = 50.0;
        assert!(!step(s, Input::default(), &room, &room.initial).crouching);
    }

    #[test]
    fn step_stands_up_when_falling_even_with_down_held() {
        let room = room();
        let mut s = crouching_at(0, 64);
        s.speed.y = 50.0;
        assert!(!step(s, DOWN, &room, &room.initial).crouching);
    }

    #[test]
    fn step_stays_crouched_while_rising() {
        let room = room();
        let mut s = crouching_at(0, 64);
        s.speed.y = -100.0;
        assert!(step(s, Input::default(), &room, &room.initial).crouching);
    }

    #[test]
    fn step_wallboost_happens_the_frame_after_the_press() {
        let room = room();
        let mut s = step(wallboosting(), LEFT, &room, &room.initial);
        assert_eq!(s.stamina, 50.0);
        assert_eq!(
            s.speed.x,
            approach(0.0, -MAX_RUN, RUN_ACCEL * DELTA_TIME * AIR_ACCEL_MULT)
        );

        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.stamina, 50.0 + CLIMB_JUMP_COST);
        assert_eq!(s.wall.boost_timer, 0.0);
        assert_eq!(
            s.speed.x,
            approach(
                -WALL_KICK_SPEED,
                0.0,
                RUN_ACCEL * DELTA_TIME * AIR_ACCEL_MULT
            )
        );
    }

    #[test]
    fn move_v_up_into_a_corner_slides_around_it() {
        let room = ceiling_room();
        let mut s = rising_at(50, 0.0);
        s.subpixel.y = 0.25;
        move_v(&mut s, &room, &room.initial, -4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 52,
                y: UNDER_CEILING_Y - 1
            }
        );
        assert_eq!(s.speed.y, -240.0);
        assert_eq!(s.subpixel.y, 0.0);
    }

    #[test]
    fn corner_correction_reaches_four_pixels() {
        let room = ceiling_room();
        let mut s = rising_at(48, 0.0);
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 52,
                y: UNDER_CEILING_Y - 1
            }
        );

        let mut s = rising_at(47, 0.0);
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 47,
                y: UNDER_CEILING_Y
            }
        );
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn corner_correction_reaches_five_pixels_in_a_dash_straight_up() {
        let room = ceiling_room();
        let mut s = rising_at(47, 0.0);
        s.dash.attack_timer = DASH_ATTACK_TIME;
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 52,
                y: UNDER_CEILING_Y - 1
            }
        );

        let mut s = rising_at(46, 0.0);
        s.dash.attack_timer = DASH_ATTACK_TIME;
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 46,
                y: UNDER_CEILING_Y
            }
        );
    }

    #[test]
    fn corner_correction_reaches_four_pixels_in_a_dash_with_sideways_speed() {
        let room = ceiling_room();
        let mut s = rising_at(47, 10.0);
        s.dash.attack_timer = DASH_ATTACK_TIME;
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 47,
                y: UNDER_CEILING_Y
            }
        );
    }

    #[test]
    fn corner_correction_slides_left_too() {
        let room = ceiling_room();
        let mut s = rising_at(14, 0.0);
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 12,
                y: UNDER_CEILING_Y - 1
            }
        );
    }

    #[test]
    fn corner_correction_only_slides_the_way_she_is_moving() {
        let room = ceiling_room();
        let mut s = rising_at(50, -10.0);
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 50,
                y: UNDER_CEILING_Y
            }
        );
        assert_eq!(s.speed.y, 0.0);

        let mut s = rising_at(14, 10.0);
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 14,
                y: UNDER_CEILING_Y
            }
        );

        let mut s = rising_at(50, 10.0);
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: 52,
                y: UNDER_CEILING_Y - 1
            }
        );
    }

    #[test]
    fn ceiling_bonk_ends_the_variable_jump() {
        let room = ceiling_room();
        let mut s = rising_at(32, 0.0);
        s.jump_timer = 0.1;
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(s.speed.y, 0.0);
        assert_eq!(s.jump_timer, 0.0);
    }

    #[test]
    fn ceiling_bonk_at_the_start_of_a_jump_keeps_the_variable_jump() {
        let room = ceiling_room();
        let mut s = rising_at(32, 0.0);
        s.jump_timer = 0.18;
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(s.speed.y, 0.0);
        assert_eq!(s.jump_timer, 0.18);
    }

    #[test]
    fn corner_correction_keeps_the_variable_jump() {
        let room = ceiling_room();
        let mut s = rising_at(50, 0.0);
        s.jump_timer = 0.1;
        move_v(&mut s, &room, &room.initial, -1.0);
        assert_eq!(s.jump_timer, 0.1);
    }

    #[test]
    fn landing_does_not_corner_correct() {
        let room = room();
        let mut s = player_at(LEDGE_X, FLOOR_Y, 0.0, 100.0);
        move_v(&mut s, &room, &room.initial, 2.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X,
                y: FLOOR_Y
            }
        );
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn dash_into_a_ledge_just_below_its_top_pops_up_onto_it() {
        let room = room();
        let mut s = dashing_at(PIT_X, FLOOR_Y + 2, -240.0, 0.0);
        s.subpixel.x = 0.25;
        move_h(&mut s, &room, &room.initial, -3.0, false);
        assert_eq!(
            s.position,
            IVec2 {
                x: PIT_X - 1,
                y: FLOOR_Y
            }
        );
        assert_eq!(s.speed.x, -240.0);
        assert_eq!(s.subpixel.x, 0.0);
        assert_eq!(s.wall.retention_timer, 0.0);
    }

    #[test]
    fn dash_corner_correction_reaches_four_pixels() {
        let room = room();
        let mut s = dashing_at(PIT_X, FLOOR_Y + 4, -240.0, 0.0);
        move_h(&mut s, &room, &room.initial, -3.0, false);
        assert_eq!(
            s.position,
            IVec2 {
                x: PIT_X - 1,
                y: FLOOR_Y
            }
        );

        let mut s = dashing_at(PIT_X, FLOOR_Y + 5, -240.0, 0.0);
        move_h(&mut s, &room, &room.initial, -3.0, false);
        assert_eq!(
            s.position,
            IVec2 {
                x: PIT_X,
                y: FLOOR_Y + 5
            }
        );
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_corner_correction_needs_a_dash() {
        let room = room();
        let mut s = player_at(PIT_X, FLOOR_Y + 2, -240.0, 0.0);
        move_h(&mut s, &room, &room.initial, -3.0, false);
        assert_eq!(
            s.position,
            IVec2 {
                x: PIT_X,
                y: FLOOR_Y + 2
            }
        );
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_corner_correction_needs_a_level_dash() {
        let room = room();
        let mut s = dashing_at(PIT_X, FLOOR_Y + 2, -170.0, -170.0);
        move_h(&mut s, &room, &room.initial, -3.0, false);
        assert_eq!(
            s.position,
            IVec2 {
                x: PIT_X,
                y: FLOOR_Y + 2
            }
        );
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn ground_dash_into_a_low_gap_crouches() {
        let room = tunnel_room();
        let mut s = dashing_at(20, FLOOR_Y, 240.0, 0.0);
        move_h(&mut s, &room, &room.initial, 4.0, true);
        assert!(s.crouching);
        assert_eq!(s.position.x, 20);
        assert_eq!(s.speed.x, 240.0);

        move_h(&mut s, &room, &room.initial, 4.0, true);
        assert_eq!(s.position.x, 24);
    }

    #[test]
    fn air_dash_into_a_low_gap_does_not_crouch() {
        let room = tunnel_room();
        let mut s = dashing_at(20, FLOOR_Y, 240.0, 0.0);
        move_h(&mut s, &room, &room.initial, 4.0, false);
        assert!(!s.crouching);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn air_dash_down_onto_a_ledge_edge_slips_off() {
        let room = room();
        let mut s = air_dashing_down_at(LEDGE_X - 1, 0.0);
        s.subpixel.y = 0.25;
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X + 1,
                y: FLOOR_Y + 1
            }
        );
        assert_eq!(s.speed.y, 240.0);
        assert_eq!(s.subpixel.y, 0.0);
    }

    #[test]
    fn downward_corner_correction_reaches_four_pixels() {
        let room = room();
        let mut s = air_dashing_down_at(LEDGE_X - 3, 0.0);
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X + 1,
                y: FLOOR_Y + 1
            }
        );

        let mut s = air_dashing_down_at(LEDGE_X - 4, 0.0);
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X - 4,
                y: FLOOR_Y
            }
        );
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn downward_corner_correction_only_slips_the_way_she_is_moving() {
        let room = room();
        let mut s = air_dashing_down_at(LEDGE_X - 1, -10.0);
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X - 1,
                y: FLOOR_Y
            }
        );

        let mut s = air_dashing_down_at(LEDGE_X - 1, 10.0);
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X + 1,
                y: FLOOR_Y + 1
            }
        );
    }

    #[test]
    fn downward_corner_correction_treats_a_tiny_speed_as_none() {
        // drifting left by less than the slack, she can still slip right off the ledge
        let room = room();
        let mut s = air_dashing_down_at(LEDGE_X - 1, -0.005);
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X + 1,
                y: FLOOR_Y + 1
            }
        );
    }

    #[test]
    fn downward_corner_correction_not_for_ground_dashes_or_falls() {
        let room = room();
        let mut s = air_dashing_down_at(LEDGE_X - 1, 0.0);
        s.dash.started_on_ground = true;
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X - 1,
                y: FLOOR_Y
            }
        );

        let mut s = player_at(LEDGE_X - 1, FLOOR_Y - 1, 0.0, 240.0);
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(
            s.position,
            IVec2 {
                x: LEDGE_X - 1,
                y: FLOOR_Y
            }
        );
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn landing_after_a_down_diagonal_dash_slides() {
        let room = room();
        let d = 1.0 / 2f32.sqrt();
        let mut s = player_at(0, FLOOR_Y - 1, 100.0, 200.0);
        s.dash.dir = Vec2 { x: d, y: d };
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(s.position.y, FLOOR_Y);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 100.0 * ULTRA_SPEED_MULT,
                y: 0.0
            }
        );
        assert_eq!(s.dash.dir, Vec2 { x: 1.0, y: 0.0 });
        assert!(s.crouching);
    }

    #[test]
    fn landing_after_a_straight_down_dash_does_not_slide() {
        let room = room();
        let mut s = player_at(0, FLOOR_Y - 1, 100.0, 200.0);
        s.dash.dir = Vec2 { x: 0.0, y: 1.0 };
        move_v(&mut s, &room, &room.initial, 4.0);
        assert_eq!(s.speed, Vec2 { x: 100.0, y: 0.0 });
        assert!(!s.crouching);
    }

    #[test]
    fn jumps_and_collisions_end_the_dash_attack() {
        let room = room();
        let mut s = standing();
        s.dash.attack_timer = 0.1;
        jump(&mut s, Input::default());
        assert_eq!(s.dash.attack_timer, 0.0);

        let mut s = player_at(WALL_X, 100, 90.0, 0.0);
        s.dash.attack_timer = 0.1;
        move_h(&mut s, &room, &room.initial, 3.0, false);
        assert_eq!(s.dash.attack_timer, 0.0);

        let mut s = player_at(0, FLOOR_Y, 0.0, 100.0);
        s.dash.attack_timer = 0.1;
        move_v(&mut s, &room, &room.initial, 3.0);
        assert_eq!(s.dash.attack_timer, 0.0);
    }

    #[test]
    fn step_forced_direction_overrides_her_input() {
        let room = room();
        let mut s = player_at(40, 100, -WALL_KICK_SPEED, -100.0);
        s.facing = -1;
        s.forced_move = -1;
        s.forced_move_timer = 0.1;
        s = step(
            s,
            Input {
                move_x: 1,
                ..Input::default()
            },
            &room,
            &room.initial,
        );
        assert_eq!(s.facing, -1);
        assert_eq!(
            s.speed.x,
            approach(
                -WALL_KICK_SPEED,
                -MAX_RUN,
                RUN_DECEL * DELTA_TIME * AIR_ACCEL_MULT
            )
        );
        assert_eq!(s.forced_move_timer, 0.1 - DELTA_TIME);
    }

    #[test]
    fn falling_onto_a_jump_thru_lands_on_it() {
        let room = platform_room();
        let mut s = player_at(60, PLATFORM_Y - 2, 0.0, 100.0);
        move_v(&mut s, &room, &room.initial, 5.0);
        assert_eq!(s.position.y, PLATFORM_Y);
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn rising_goes_through_a_jump_thru() {
        let room = platform_room();
        let mut s = player_at(60, PLATFORM_Y + 15, 0.0, -100.0);
        move_v(&mut s, &room, &room.initial, -20.0);
        assert_eq!(s.position.y, PLATFORM_Y - 5);
        assert_eq!(s.speed.y, -100.0);
    }

    #[test]
    fn falling_while_inside_a_jump_thru_keeps_falling() {
        let room = platform_room();
        let mut s = player_at(60, PLATFORM_Y + 2, 0.0, 100.0);
        move_v(&mut s, &room, &room.initial, 5.0);
        assert_eq!(s.position.y, PLATFORM_Y + 7);
        assert_eq!(s.speed.y, 100.0);
    }

    #[test]
    fn jump_thrus_do_not_block_sideways_movement() {
        let room = platform_room();
        let mut s = player_at(30, PLATFORM_Y + 2, 100.0, 0.0);
        move_h(&mut s, &room, &room.initial, 20.0, false);
        assert_eq!(s.position.x, 50);
        assert_eq!(s.speed.x, 100.0);
    }

    #[test]
    fn dash_jump_thru_nudge_reaches_six_pixels() {
        let room = platform_room();
        let mut s = player_at(60, PLATFORM_Y + 6, 0.0, 0.0);
        dash_jump_thru_nudge(&mut s, &room, &room.initial);
        assert_eq!(s.position.y, PLATFORM_Y);

        let mut s = player_at(60, PLATFORM_Y + 7, 0.0, 0.0);
        dash_jump_thru_nudge(&mut s, &room, &room.initial);
        assert_eq!(s.position.y, PLATFORM_Y + 7);
    }

    #[test]
    fn liftboost_is_capped_and_never_downward() {
        assert_eq!(
            liftboost(&lifted(standing(), 100.0, -60.0)),
            Vec2 { x: 100.0, y: -60.0 }
        );
        assert_eq!(
            liftboost(&lifted(standing(), 400.0, -300.0)),
            Vec2 {
                x: LIFT_X_CAP,
                y: LIFT_Y_CAP
            }
        );
        assert_eq!(
            liftboost(&lifted(standing(), -400.0, 80.0)),
            Vec2 {
                x: -LIFT_X_CAP,
                y: 0.0
            }
        );
        assert_eq!(liftboost(&standing()), Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn step_lift_speed_lasts_ten_frames() {
        let room = room();
        let mut s = lifted(standing(), 0.0, -70.0);
        for _ in 0..9 {
            s = step(s, Input::default(), &room, &room.initial);
        }
        assert_eq!(liftboost(&s).y, -70.0);
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(liftboost(&s).y, 0.0);
    }
}
