use super::collision::can_stand;
use super::constants::*;
use super::movement::sign;
use super::state::{Frame, Input, Player, State, set_normal};
use crate::geom::Vec2;
use crate::level::room::{Room, RoomState};

pub(crate) fn dashing(player: &Player) -> bool {
    matches!(player.state, State::DashStart | State::Dash { .. })
}

pub(crate) fn aim_vector(input: Input, facing: i8) -> Vec2 {
    if input.move_x == 0 && input.move_y == 0 {
        Vec2 {
            x: facing as f32,
            y: 0.0,
        }
    } else if input.move_x != 0 && input.move_y != 0 {
        // diagonal
        let d = 1.0 / 2f32.sqrt();
        Vec2 {
            x: input.move_x as f32 * d,
            y: input.move_y as f32 * d,
        }
    } else {
        Vec2 {
            x: input.move_x as f32,
            y: input.move_y as f32,
        }
    }
}

pub(crate) fn can_dash(player: &Player, dash_pressed: bool) -> bool {
    dash_pressed && player.dash.cooldown <= 0.0 && player.dashes > 0
}

pub(crate) fn dash_begin(player: &mut Player, room: &Room, state: &RoomState, frame: Frame) {
    let Frame {
        input,
        ground,
        demodash_pressed,
        ..
    } = frame;

    player.dashes -= 1;
    player.freeze_timer = DASH_FREEZE_TIME;
    player.dash.cooldown = DASH_COOLDOWN;
    player.dash.refill_cooldown = DASH_REFILL_COOLDOWN;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.dash.speed_before = player.speed;
    player.dash.attack_timer = DASH_ATTACK_TIME;
    player.dash.started_on_ground = ground;
    player.speed = Vec2 { x: 0.0, y: 0.0 };
    player.dash.dir = Vec2 { x: 0.0, y: 0.0 };

    // holding down at the start of a dash crouchs
    if !ground && player.crouching && can_stand(player, room, state) {
        player.crouching = false;
    } else if !player.crouching && (demodash_pressed || input.move_y == 1) {
        player.crouching = true;
    }

    player.state = State::DashStart
}

pub(crate) fn dash_launch(player: &mut Player, aim: Vec2, ground: bool) {
    let mut new_speed_x = aim.x * DASH_SPEED;
    let new_speed_y = aim.y * DASH_SPEED;

    if sign(player.dash.speed_before.x) == sign(new_speed_x)
        && player.dash.speed_before.x.abs() > new_speed_x.abs()
    {
        new_speed_x = player.dash.speed_before.x;
    }

    player.speed = Vec2 {
        x: new_speed_x,
        y: new_speed_y,
    };
    player.dash.dir = aim;

    if aim.x != 0.0 {
        player.facing = sign(aim.x) as i8;
    }

    // ultra: a down diagonal dash on the ground becomes a crouched horizontal one, a bit faster
    if ground && player.dash.dir.x != 0.0 && player.dash.dir.y > 0.0 && player.speed.y > 0.0 {
        player.dash.dir = Vec2 {
            x: sign(player.dash.dir.x),
            y: 0.0,
        };
        player.speed.y = 0.0;
        player.speed.x *= ULTRA_SPEED_MULT;
        player.crouching = true;
    }

    player.state = State::Dash { timer: DASH_TIME };
}

pub(crate) fn dash_end(player: &mut Player) {
    player.auto_jump = true;

    if player.dash.dir.y <= 0.0 {
        player.speed.x = player.dash.dir.x * END_DASH_SPEED;
        player.speed.y = player.dash.dir.y * END_DASH_SPEED;
    }

    if player.speed.y < 0.0 {
        player.speed.y *= END_DASH_UP_MULT;
    }

    set_normal(player);
}

// never lowers her count, so a second dash from a two-dash crystal is kept
pub(crate) fn refill_dash(player: &mut Player) {
    player.dashes = player.dashes.max(MAX_DASHES);
}

pub(crate) fn use_crystal(player: &mut Player, two_dashes: bool) -> bool {
    let dashes = if two_dashes { TWO_DASHES } else { MAX_DASHES };

    if player.dashes < dashes || player.stamina < CLIMB_TIRED_THRESHOLD {
        player.dashes = dashes;
        player.stamina = CLIMB_MAX_STAMINA;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refill_dash_gives_one_back_and_keeps_two() {
        let mut s = standing();
        s.dashes = 0;
        refill_dash(&mut s);
        assert_eq!(s.dashes, MAX_DASHES);

        s.dashes = TWO_DASHES;
        refill_dash(&mut s);
        assert_eq!(s.dashes, TWO_DASHES);
    }
    use crate::player::fixtures::*;
    use crate::player::step::step;

    #[test]
    fn aim_defaults_to_facing() {
        assert_eq!(aim(0, 0, 1), Vec2 { x: 1.0, y: 0.0 });
        assert_eq!(aim(0, 0, -1), Vec2 { x: -1.0, y: 0.0 });
    }

    #[test]
    fn aim_cardinal_ignores_facing() {
        assert_eq!(aim(0, -1, -1), Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(aim(0, 1, 1), Vec2 { x: 0.0, y: 1.0 });
        assert_eq!(aim(-1, 0, 1), Vec2 { x: -1.0, y: 0.0 });
    }

    #[test]
    fn aim_diagonal_is_normalized() {
        let d = 1.0 / 2f32.sqrt();
        assert_eq!(aim(1, -1, 1), Vec2 { x: d, y: -d });
        assert_eq!(aim(-1, 1, 1), Vec2 { x: -d, y: d });
    }

    #[test]
    fn can_dash_needs_press_dash_and_no_cooldown() {
        let mut s = standing();
        assert!(can_dash(&s, true));
        assert!(!can_dash(&s, false));
        s.dash.cooldown = 0.1;
        assert!(!can_dash(&s, true));
        s.dash.cooldown = 0.0;
        s.dashes = 0;
        assert!(!can_dash(&s, true));
    }

    #[test]
    fn dash_begin_freezes_and_zeroes_speed() {
        let room = room();
        let mut s = standing();
        s.speed = Vec2 { x: 90.0, y: 0.0 };
        s.wall.slide_timer = 0.3;
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        assert_eq!(s.state, State::DashStart);
        assert_eq!(s.dashes, MAX_DASHES - 1);
        assert_eq!(s.freeze_timer, DASH_FREEZE_TIME);
        assert_eq!(s.dash.cooldown, DASH_COOLDOWN);
        assert_eq!(s.dash.refill_cooldown, DASH_REFILL_COOLDOWN);
        assert_eq!(s.wall.slide_timer, WALL_SLIDE_TIME);
        assert_eq!(s.dash.speed_before, Vec2 { x: 90.0, y: 0.0 });
        assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
        assert_eq!(s.dash.dir, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn dash_launch_sets_speed_and_starts_timer() {
        let room = room();
        let mut s = standing();
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: -DASH_SPEED
            }
        );
        assert_eq!(s.dash.dir, Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(s.state, State::Dash { timer: DASH_TIME });
    }

    #[test]
    fn dash_launch_keeps_faster_speed_in_same_direction() {
        let room = room();
        let mut s = standing();
        s.speed.x = 300.0;
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
        assert_eq!(s.speed.x, 300.0);

        let mut s = standing();
        s.speed.x = -300.0;
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
        assert_eq!(s.speed.x, DASH_SPEED);
    }

    #[test]
    fn dash_launch_up_drops_horizontal_speed() {
        let room = room();
        let mut s = standing();
        s.speed.x = 300.0;
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_launch_turns_to_face_horizontal_dashes() {
        let room = room();
        let mut s = standing();
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        dash_launch(&mut s, Vec2 { x: -1.0, y: 0.0 }, true);
        assert_eq!(s.facing, -1);

        let mut s = standing();
        s.facing = -1;
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
        assert_eq!(s.facing, -1);
    }

    #[test]
    fn dash_end_horizontal() {
        let mut s = mid_dash(Vec2 { x: 1.0, y: 0.0 });
        dash_end(&mut s);
        assert_eq!(s.state, State::Normal);
        assert!(s.auto_jump);
        assert_eq!(
            s.speed,
            Vec2 {
                x: END_DASH_SPEED,
                y: 0.0
            }
        );
    }

    #[test]
    fn dash_end_up_is_slower() {
        let mut s = mid_dash(Vec2 { x: 0.0, y: -1.0 });
        dash_end(&mut s);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: -END_DASH_SPEED * END_DASH_UP_MULT
            }
        );
    }

    #[test]
    fn dash_end_down_keeps_speed() {
        let mut s = mid_dash(Vec2 { x: 0.0, y: 1.0 });
        dash_end(&mut s);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: DASH_SPEED
            }
        );
    }

    #[test]
    fn dash_refills_on_ground_after_refill_cooldown() {
        let room = room();
        let mut s = standing();
        s.dashes = 0;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.dashes, MAX_DASHES);
    }

    #[test]
    fn dash_does_not_refill_during_refill_cooldown() {
        let room = room();
        let mut s = standing();
        s.dashes = 0;
        s.dash.refill_cooldown = DASH_REFILL_COOLDOWN;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.dashes, 0);
    }

    #[test]
    fn dash_does_not_refill_in_air() {
        let room = room();
        let mut s = at_rest();
        s.dashes = 0;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.dashes, 0);
    }

    #[test]
    fn step_super_dashes_out_of_a_dash() {
        let room = room();
        let s = step(
            dash_for(standing(), 5, DASH_RIGHT),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert_eq!(s.state, State::Normal);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn step_super_dashes_on_the_launch_frame() {
        let room = room();
        let s = step(
            dash_for(standing(), 4, DASH_RIGHT),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert_eq!(s.state, State::Normal);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn step_super_dashes_on_the_last_dash_frame() {
        let room = room();
        let s = step(
            dash_for(standing(), 14, DASH_RIGHT),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert_eq!(s.state, State::Normal);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn step_no_super_dash_in_the_air() {
        let room = room();
        let s = step(
            dash_for(at_rest(), 5, DASH_RIGHT),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert!(matches!(s.state, State::Dash { .. }));
        assert_eq!(
            s.speed,
            Vec2 {
                x: DASH_SPEED,
                y: 0.0
            }
        );
    }

    #[test]
    fn step_no_super_dash_out_of_an_up_dash() {
        let room = room();
        let up_dash = Input {
            move_x: 0,
            move_y: -1,
            jump: false,
            jump2: false,
            dash: true,
            grab: false,
            demodash: false,
        };
        let s = step(
            dash_for(standing(), 5, up_dash),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert!(matches!(s.state, State::Dash { .. }));
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: -DASH_SPEED
            }
        );
    }

    #[test]
    fn dash_begin_crouches_when_down_is_held() {
        let room = room();
        let mut s = standing();
        dash_begin(&mut s, &room, &room.initial, frame(DOWN, true));
        assert!(s.crouching);
    }

    #[test]
    fn dash_begin_stays_standing_without_down() {
        let room = room();
        let mut s = standing();
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        assert!(!s.crouching);
    }

    #[test]
    fn dash_begin_stays_crouched_on_the_ground() {
        let room = room();
        let mut s = crouching_at(0, FLOOR_Y);
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        assert!(s.crouching);
    }

    #[test]
    fn dash_begin_stands_up_in_the_air() {
        let room = room();
        let mut s = crouching_at(0, 64);
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), false));
        assert!(!s.crouching);
    }

    #[test]
    fn dash_launch_down_diagonal_on_the_ground_slides() {
        let d = 1.0 / 2f32.sqrt();
        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: -d, y: d }, true);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -d * DASH_SPEED * ULTRA_SPEED_MULT,
                y: 0.0
            }
        );
        assert_eq!(s.dash.dir, Vec2 { x: -1.0, y: 0.0 });
        assert!(s.crouching);
    }

    #[test]
    fn dash_launch_down_diagonal_in_the_air_does_not_slide() {
        let d = 1.0 / 2f32.sqrt();
        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: d, y: d }, false);
        assert_eq!(
            s.speed,
            Vec2 {
                x: d * DASH_SPEED,
                y: d * DASH_SPEED
            }
        );
        assert_eq!(s.dash.dir, Vec2 { x: d, y: d });
        assert!(!s.crouching);
    }

    #[test]
    fn dash_launch_straight_down_or_sideways_does_not_slide() {
        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: 0.0, y: 1.0 }, true);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: DASH_SPEED
            }
        );
        assert!(!s.crouching);

        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
        assert_eq!(
            s.speed,
            Vec2 {
                x: DASH_SPEED,
                y: 0.0
            }
        );
        assert!(!s.crouching);
    }

    #[test]
    fn step_hyper_from_a_down_dash() {
        let room = room();
        let down_dash = Input { dash: true, ..DOWN };
        let s = step(
            dash_for(standing(), 4, down_dash),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert_eq!(s.state, State::Normal);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED * HYPER_X_MULT,
                y: JUMP_SPEED * HYPER_Y_MULT
            }
        );
        assert!(!s.crouching);
    }

    #[test]
    fn dash_begin_remembers_whether_it_started_on_the_ground() {
        let room = room();
        let mut s = standing();
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        assert!(s.dash.started_on_ground);

        let mut s = at_rest();
        s.dash.started_on_ground = true;
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), false));
        assert!(!s.dash.started_on_ground);
    }

    #[test]
    fn dash_attack_runs_from_the_start_of_a_dash() {
        let room = room();
        let mut s = standing();
        dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
        assert_eq!(s.dash.attack_timer, DASH_ATTACK_TIME);

        let s = dash_for(standing(), 6, DASH_RIGHT);
        assert!(s.dash.attack_timer > 0.0 && s.dash.attack_timer < DASH_ATTACK_TIME);
    }

    #[test]
    fn step_wallbounce_after_an_upward_dash() {
        let room = room();
        let left_wall = left_wall_room();
        let s = step(bouncing_at(WALL_X - 4), JUMP, &room, &room.initial);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALLBOUNCE_SPEED_X,
                y: WALLBOUNCE_SPEED_Y
            }
        );

        let s = step(
            bouncing_at(LEFT_WALL_X + 4),
            JUMP,
            &left_wall,
            &left_wall.initial,
        );
        assert_eq!(
            s.speed,
            Vec2 {
                x: WALLBOUNCE_SPEED_X,
                y: WALLBOUNCE_SPEED_Y
            }
        );
    }

    #[test]
    fn step_no_wallbounce_once_the_dash_attack_is_over() {
        let room = room();
        let mut s = bouncing_at(WALL_X - 4);
        s.dash.attack_timer = 0.0;
        s = step(s, JUMP, &room, &room.initial);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn step_wall_kicks_out_of_a_sideways_dash() {
        let room = room();
        let mut s = dashing_at(WALL_X - 2, 100, -DASH_SPEED, 0.0);
        s.dash.dir = Vec2 { x: -1.0, y: 0.0 };
        s.facing = -1;
        s = step(s, JUMP, &room, &room.initial);
        assert_eq!(s.state, State::Normal);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALL_KICK_SPEED,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn step_forced_direction_does_not_change_the_dash_aim() {
        let mut s = player_at(40, 100, -WALL_KICK_SPEED, -100.0);
        s.forced_move = -1;
        s.forced_move_timer = 0.16;
        let right_dash = Input {
            move_x: 1,
            dash: true,
            ..Input::default()
        };
        let s = dash_for(s, 5, right_dash);
        assert_eq!(s.speed.x, DASH_SPEED);
    }

    #[test]
    fn level_dash_clipping_a_jump_thru_is_lifted_onto_it() {
        let room = platform_room();
        let mut s = dashing_at(60, PLATFORM_Y + 4, DASH_SPEED, 0.0);
        s.dash.dir = Vec2 { x: 1.0, y: 0.0 };
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.position.y, PLATFORM_Y);
    }

    #[test]
    fn dash_up_through_a_jump_thru_is_not_lifted() {
        let room = platform_room();
        let mut s = dashing_at(60, PLATFORM_Y + 4, 0.0, -DASH_SPEED);
        s.dash.dir = Vec2 { x: 0.0, y: -1.0 };
        let with = step(s, Input::default(), &room, &room.initial);
        assert_eq!(with.position.y, PLATFORM_Y + 4 - 5);
    }

    #[test]
    fn dash_begin_from_the_demodash_button_crouches() {
        let room = room();
        let mut s = standing();
        dash_begin(
            &mut s,
            &room,
            &room.initial,
            Frame {
                demodash_pressed: true,
                ..frame(Input::default(), true)
            },
        );
        assert!(s.crouching);

        let mut s = at_rest();
        dash_begin(
            &mut s,
            &room,
            &room.initial,
            Frame {
                demodash_pressed: true,
                ..frame(Input::default(), false)
            },
        );
        assert!(s.crouching);
    }

    #[test]
    fn step_demodash_starts_a_crouched_dash() {
        let room = room();
        let s = step(standing(), DEMODASH, &room, &room.initial);
        assert_eq!(s.state, State::DashStart);
        assert!(s.crouching);
        assert_eq!(s.dashes, 0);
        assert!(s.held.demodash);
    }

    #[test]
    fn step_demodash_goes_the_way_she_aims() {
        let s = dash_for(
            standing(),
            5,
            Input {
                move_x: 1,
                ..DEMODASH
            },
        );
        assert_eq!(
            s.speed,
            Vec2 {
                x: DASH_SPEED,
                y: 0.0
            }
        );
        assert!(s.crouching);
    }

    #[test]
    fn step_demodash_then_jump_is_a_hyper() {
        let room = room();
        let s = step(
            dash_for(
                standing(),
                5,
                Input {
                    move_x: 1,
                    ..DEMODASH
                },
            ),
            JUMP_RIGHT,
            &room,
            &room.initial,
        );
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED * HYPER_X_MULT,
                y: JUMP_SPEED * HYPER_Y_MULT
            }
        );
        assert!(!s.crouching);
    }

    #[test]
    fn step_holding_the_demodash_button_does_not_dash_again() {
        let room = room();
        let mut s = standing();
        s.held.demodash = true;
        s = step(s, DEMODASH, &room, &room.initial);
        assert_eq!(s.state, State::Normal);
        assert!(!s.crouching);
    }

    #[test]
    fn step_normal_dash_while_demodash_is_held_does_not_crouch() {
        let room = room();
        let mut s = standing();
        s.held.demodash = true;
        s = step(
            s,
            Input {
                dash: true,
                ..DEMODASH
            },
            &room,
            &room.initial,
        );
        assert_eq!(s.state, State::DashStart);
        assert!(!s.crouching);
    }

    #[test]
    fn step_dash_start_adds_liftboost_to_the_speed_it_remembers() {
        let room = room();
        let s = step(
            lifted(standing(), 60.0, 0.0),
            DASH_RIGHT,
            &room,
            &room.initial,
        );
        assert_eq!(s.dash.speed_before.x, 60.0);
    }

    #[test]
    fn level_dash_just_above_a_floor_snaps_down_onto_it() {
        let room = room();
        let mut s = dashing_at(0, FLOOR_Y - 2, DASH_SPEED, 0.0);
        s.dash.dir = Vec2 { x: 1.0, y: 0.0 };
        s.dash.attack_timer = 0.2;
        s.subpixel.y = 0.25;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.position.y, FLOOR_Y);
        assert_eq!(s.subpixel.y, 0.0);
    }

    #[test]
    fn no_floor_snap_from_three_pixels_up_or_in_a_diagonal_dash() {
        let room = room();
        let mut s = dashing_at(0, FLOOR_Y - 3, DASH_SPEED, 0.0);
        s.dash.dir = Vec2 { x: 1.0, y: 0.0 };
        s.dash.attack_timer = 0.2;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.position.y, FLOOR_Y - 3);

        let d = 1.0 / 2f32.sqrt();
        let mut s = dashing_at(0, FLOOR_Y - 2, 0.0, 0.0);
        s.dash.dir = Vec2 { x: d, y: -d };
        s.dash.attack_timer = 0.2;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.position.y, FLOOR_Y - 2);
    }

    #[test]
    fn dash_end_resets_the_fall_limit() {
        let mut s = falling(0.0);
        s.max_fall = FASTFALL_SPEED;
        s.state = State::Dash { timer: 0.0 };
        s.dash.dir = Vec2 { x: 0.0, y: 1.0 };
        dash_end(&mut s);
        assert_eq!(s.max_fall, MAX_FALL);
    }

    #[test]
    fn dashing_cancels_the_wait_for_the_sideways_push() {
        let room = room();
        let mut s = player_at(WALL_X, 100, 0.0, -50.0);
        s.climb.hop_wait = 1;
        s.climb.hop_speed = CLIMBHOP_X;
        s = step(
            s,
            Input {
                dash: true,
                ..Input::default()
            },
            &room,
            &room.initial,
        );

        assert_eq!(s.state, State::DashStart);
        assert_eq!(s.climb.hop_wait, 0);
    }
}
