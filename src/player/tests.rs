use super::*;
use crate::level::room::Direction;

const FLOOR_Y: i32 = 144;
const LEDGE_X: i32 = 43;
const WALL_X: i32 = 76;

// a frame with these buttons held and nothing newly pressed
fn frame(input: Input, ground: bool) -> Frame {
    Frame {
        input,
        ground,
        ..Frame::default()
    }
}

fn room() -> Room {
    let text = "00000000001\n".repeat(18) + &"11111000001\n".repeat(5);
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

fn at_rest() -> PlayerState {
    PlayerState::new(
        IVec2 { x: 0, y: 64 },
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 0.0, y: 0.0 },
    )
}

fn standing() -> PlayerState {
    PlayerState::new(
        IVec2 { x: 0, y: FLOOR_Y },
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 0.0, y: 0.0 },
    )
}

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
fn move_h_small_amount_only_changes_remainder() {
    let mut s = at_rest();
    move_h(&mut s, &room(), 0.25, false);
    assert_eq!(s.position.x, 0);
    assert_eq!(s.remainder.x, 0.25);
}

#[test]
fn move_h_rounds_to_whole_pixels() {
    let mut s = at_rest();
    move_h(&mut s, &room(), 0.75, false);
    assert_eq!(s.position.x, 1);
    assert_eq!(s.remainder.x, -0.25);

    move_h(&mut s, &room(), -2.0, false);
    assert_eq!(s.position.x, -1);
    assert_eq!(s.remainder.x, -0.25);
}

#[test]
fn move_h_ties_round_to_even_like_csharp() {
    let mut s = at_rest();
    move_h(&mut s, &room(), 0.5, false);
    assert_eq!(s.position.x, 0);
    assert_eq!(s.remainder.x, 0.5);

    move_h(&mut s, &room(), 1.0, false);
    assert_eq!(s.position.x, 2);
    assert_eq!(s.remainder.x, -0.5);
}

#[test]
fn collide_at_uses_her_hitbox() {
    let room = room();
    let inside = |x: i32, y: i32| collides(&player_at(x, y, 0.0, 0.0), &room, 0, 0);

    assert!(!inside(0, FLOOR_Y));
    assert!(inside(0, FLOOR_Y + 1));
    assert!(inside(LEDGE_X, FLOOR_Y + 1));
    assert!(!inside(LEDGE_X + 1, FLOOR_Y + 1));
    assert!(!inside(WALL_X, 64));
    assert!(inside(WALL_X + 1, 64));
    assert!(!inside(60, FLOOR_Y + 12));
}

#[test]
fn on_ground_when_standing_on_floor() {
    assert!(on_ground(&standing(), &room()));
}

#[test]
fn not_on_ground_above_floor() {
    let mut s = standing();
    s.position.y -= 1;
    assert!(!on_ground(&s, &room()));
}

#[test]
fn not_on_ground_while_moving_up() {
    let mut s = standing();
    s.speed.y = -10.0;
    assert!(!on_ground(&s, &room()));
}

#[test]
fn move_v_moves_up_freely() {
    let mut s = standing();
    move_v(&mut s, &room(), -1.75);
    assert_eq!(s.position.y, FLOOR_Y - 2);
    assert_eq!(s.remainder.y, 0.25);
}

#[test]
fn move_v_stops_at_floor() {
    let mut s = standing();
    s.position.y = FLOOR_Y - 2;
    s.remainder.y = 0.3;
    s.speed.y = 160.0;
    move_v(&mut s, &room(), 2.6);
    assert_eq!(s.position.y, FLOOR_Y);
    assert_eq!(s.remainder.y, 0.0);
    assert_eq!(s.speed.y, 0.0);
}

#[test]
fn jump_sets_speed_and_timers() {
    let mut s = standing();
    s.jump_grace_timer = JUMP_GRACE_TIME;
    jump(
        &mut s,
        Input {
            move_x: 0,
            jump: true,
            ..Default::default()
        },
    );
    assert_eq!(s.speed.y, JUMP_SPEED);
    assert_eq!(s.var_jump_speed, JUMP_SPEED);
    assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
    assert_eq!(s.jump_grace_timer, 0.0);
}

#[test]
fn jump_resets_wall_slide_timer() {
    let mut s = standing();
    s.wall_slide_timer = 0.3;
    jump(
        &mut s,
        Input {
            move_x: 0,
            jump: true,
            ..Default::default()
        },
    );
    assert_eq!(s.wall_slide_timer, WALL_SLIDE_TIME);
}

#[test]
fn move_h_stops_at_wall() {
    let mut s = at_rest();
    s.position.x = WALL_X - 1;
    s.remainder.x = 0.3;
    s.speed.x = 90.0;
    move_h(&mut s, &room(), 1.5, false);
    assert_eq!(s.position.x, WALL_X);
    assert_eq!(s.remainder.x, 0.0);
    assert_eq!(s.speed.x, 0.0);
}

#[test]
fn move_h_reaching_wall_exactly_is_not_a_collision() {
    let mut s = at_rest();
    s.position.x = WALL_X - 1;
    s.speed.x = 90.0;
    move_h(&mut s, &room(), 1.25, false);
    assert_eq!(s.position.x, WALL_X);
    assert_eq!(s.remainder.x, 0.25);
    assert_eq!(s.speed.x, 90.0);
}

#[test]
fn wall_at_detects_wall_to_the_right_only() {
    let mut s = at_rest();
    s.position.x = WALL_X;
    assert!(collides(&s, &room(), 1, 0));
    assert!(!collides(&s, &room(), -1, 0));
    s.position.x = WALL_X - 1;
    assert!(!collides(&s, &room(), 1, 0));
}

#[test]
fn lerp_endpoints_and_midpoint() {
    assert_eq!(lerp(160.0, 20.0, 0.0), 160.0);
    assert_eq!(lerp(160.0, 20.0, 1.0), 20.0);
    assert_eq!(lerp(160.0, 20.0, 0.5), 90.0);
}

fn falling_against_wall() -> PlayerState {
    let mut s = at_rest();
    s.position.x = WALL_X;
    s.speed.y = 100.0;
    s
}

#[test]
fn wall_slide_starts_at_full_timer() {
    let mut s = falling_against_wall();
    let max = wall_slide_max_fall(
        &mut s,
        &room(),
        Input {
            move_x: 1,
            jump: false,
            ..Default::default()
        },
    );
    assert_eq!(max, WALL_SLIDE_START_MAX);
    assert_eq!(s.wall_slide_dir, 1);
}

#[test]
fn wall_slide_max_rises_as_timer_runs_out() {
    let mut s = falling_against_wall();
    s.wall_slide_timer = WALL_SLIDE_TIME * 0.5;
    let max = wall_slide_max_fall(
        &mut s,
        &room(),
        Input {
            move_x: 1,
            jump: false,
            ..Default::default()
        },
    );
    assert_eq!(max, lerp(MAX_FALL, WALL_SLIDE_START_MAX, 0.5));
}

#[test]
fn no_wall_slide_without_holding_toward_wall() {
    let mut s = falling_against_wall();
    assert_eq!(
        wall_slide_max_fall(
            &mut s,
            &room(),
            Input {
                move_x: 0,
                jump: false,
                ..Default::default()
            }
        ),
        MAX_FALL
    );
    assert_eq!(s.wall_slide_dir, 0);
}

#[test]
fn no_wall_slide_while_moving_up() {
    let mut s = falling_against_wall();
    s.speed.y = -50.0;
    assert_eq!(
        wall_slide_max_fall(
            &mut s,
            &room(),
            Input {
                move_x: 1,
                jump: false,
                ..Default::default()
            }
        ),
        MAX_FALL
    );
    assert_eq!(s.wall_slide_dir, 0);
}

#[test]
fn no_wall_slide_away_from_wall() {
    let mut s = falling_against_wall();
    s.position.x = WALL_X - 5;
    assert_eq!(
        wall_slide_max_fall(
            &mut s,
            &room(),
            Input {
                move_x: 1,
                jump: false,
                ..Default::default()
            }
        ),
        MAX_FALL
    );
    assert_eq!(s.wall_slide_dir, 0);
}

#[test]
fn jump_adds_horizontal_boost() {
    let mut s = standing();
    s.speed.x = 90.0;
    jump(
        &mut s,
        Input {
            move_x: 1,
            jump: true,
            ..Default::default()
        },
    );
    assert_eq!(s.speed.x, 90.0 + JUMP_H_BOOST);
}

#[test]
fn not_on_ground_past_ledge() {
    let mut s = standing();
    s.position.x = LEDGE_X;
    assert!(on_ground(&s, &room()));
    s.position.x = LEDGE_X + 1;
    assert!(!on_ground(&s, &room()));
}

#[test]
fn move_v_falls_through_floor_level_past_ledge() {
    let mut s = standing();
    s.position.x = LEDGE_X + 1;
    s.speed.y = 60.0;
    move_v(&mut s, &room(), 1.0);
    assert_eq!(s.position.y, FLOOR_Y + 1);
    assert_eq!(s.speed.y, 60.0);
}

#[test]
fn move_h_blocked_by_side_of_ledge() {
    let mut s = at_rest();
    s.position = IVec2 {
        x: LEDGE_X + 1,
        y: FLOOR_Y + 5,
    };
    s.speed.x = -90.0;
    move_h(&mut s, &room(), -3.0, false);
    assert_eq!(s.position.x, LEDGE_X + 1);
    assert_eq!(s.speed.x, 0.0);
}

fn aim(move_x: i8, move_y: i8, facing: i8) -> Vec2 {
    aim_vector(
        Input {
            move_x,
            move_y,
            ..Default::default()
        },
        facing,
    )
}

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
    s.dash_cooldown_timer = 0.1;
    assert!(!can_dash(&s, true));
    s.dash_cooldown_timer = 0.0;
    s.dashes = 0;
    assert!(!can_dash(&s, true));
}

#[test]
fn dash_begin_freezes_and_zeroes_speed() {
    let mut s = standing();
    s.speed = Vec2 { x: 90.0, y: 0.0 };
    s.wall_slide_timer = 0.3;
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    assert_eq!(s.mode, Mode::DashStart);
    assert_eq!(s.dashes, MAX_DASHES - 1);
    assert_eq!(s.freeze_timer, DASH_FREEZE_TIME);
    assert_eq!(s.dash_cooldown_timer, DASH_COOLDOWN);
    assert_eq!(s.dash_refill_cooldown_timer, DASH_REFILL_COOLDOWN);
    assert_eq!(s.wall_slide_timer, WALL_SLIDE_TIME);
    assert_eq!(s.before_dash_speed, Vec2 { x: 90.0, y: 0.0 });
    assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
    assert_eq!(s.dash_dir, Vec2 { x: 0.0, y: 0.0 });
}

#[test]
fn dash_launch_sets_speed_and_starts_timer() {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
    assert_eq!(
        s.speed,
        Vec2 {
            x: 0.0,
            y: -DASH_SPEED
        }
    );
    assert_eq!(s.dash_dir, Vec2 { x: 0.0, y: -1.0 });
    assert_eq!(s.mode, Mode::Dashing { timer: DASH_TIME });
}

#[test]
fn dash_launch_keeps_faster_speed_in_same_direction() {
    let mut s = standing();
    s.speed.x = 300.0;
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
    assert_eq!(s.speed.x, 300.0);

    let mut s = standing();
    s.speed.x = -300.0;
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
    assert_eq!(s.speed.x, DASH_SPEED);
}

#[test]
fn dash_launch_up_drops_horizontal_speed() {
    let mut s = standing();
    s.speed.x = 300.0;
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
    assert_eq!(s.speed.x, 0.0);
}

#[test]
fn dash_launch_turns_to_face_horizontal_dashes() {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, Vec2 { x: -1.0, y: 0.0 }, true);
    assert_eq!(s.facing, -1);

    let mut s = standing();
    s.facing = -1;
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
    assert_eq!(s.facing, -1);
}

fn dashing(dir: Vec2) -> PlayerState {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    dash_launch(&mut s, dir, true);
    s
}

#[test]
fn dash_end_horizontal() {
    let mut s = dashing(Vec2 { x: 1.0, y: 0.0 });
    dash_end(&mut s);
    assert_eq!(s.mode, Mode::Normal);
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
    let mut s = dashing(Vec2 { x: 0.0, y: -1.0 });
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
    let mut s = dashing(Vec2 { x: 0.0, y: 1.0 });
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
    let mut s = standing();
    s.dashes = 0;
    s = step(s, Input::default(), &room());
    assert_eq!(s.dashes, MAX_DASHES);
}

#[test]
fn dash_does_not_refill_during_refill_cooldown() {
    let mut s = standing();
    s.dashes = 0;
    s.dash_refill_cooldown_timer = DASH_REFILL_COOLDOWN;
    s = step(s, Input::default(), &room());
    assert_eq!(s.dashes, 0);
}

#[test]
fn dash_does_not_refill_in_air() {
    let mut s = at_rest();
    s.dashes = 0;
    s = step(s, Input::default(), &room());
    assert_eq!(s.dashes, 0);
}

#[test]
fn frozen_frames_do_nothing() {
    let mut s = standing();
    s.speed.x = 90.0;
    s.freeze_timer = DASH_FREEZE_TIME;
    let before = s;
    s = step(
        s,
        Input {
            move_x: -1,
            ..Default::default()
        },
        &room(),
    );
    assert_eq!(s.position, before.position);
    assert_eq!(s.speed, before.speed);
    assert!(s.freeze_timer < DASH_FREEZE_TIME);
}

#[test]
fn auto_jump_cleared_on_ground() {
    let mut s = standing();
    s.auto_jump = true;
    s = step(s, Input::default(), &room());
    assert!(!s.auto_jump);
}
#[test]
fn hurtbox_is_two_pixels_shorter_than_the_hitbox() {
    let mut s = at_rest();
    s.position = IVec2 { x: 100, y: 50 };
    assert_eq!(
        hurtbox(&s),
        Rect {
            left: 96,
            top: 39,
            width: 8,
            height: 9
        }
    );
}

fn player_at(x: i32, y: i32, speed_x: f32, speed_y: f32) -> PlayerState {
    let mut s = at_rest();
    s.position = IVec2 { x, y };
    s.speed = Vec2 {
        x: speed_x,
        y: speed_y,
    };
    s
}

#[test]
fn up_spikes_kill_when_standing_or_falling_on_them() {
    let spikes = Spikes::new(Direction::Up, 0, 100, 16);
    assert!(spikes_kill(&player_at(8, 100, 0.0, 0.0), &spikes));
    assert!(spikes_kill(&player_at(8, 100, 0.0, 160.0), &spikes));
    assert!(!spikes_kill(&player_at(8, 100, 0.0, -10.0), &spikes));
}

#[test]
fn up_spikes_need_the_hurtbox_to_overlap() {
    let spikes = Spikes::new(Direction::Up, 0, 100, 16);
    assert!(!spikes_kill(&player_at(8, 99, 0.0, 0.0), &spikes));
    assert!(spikes_kill(&player_at(19, 100, 0.0, 0.0), &spikes));
    assert!(!spikes_kill(&player_at(20, 100, 0.0, 0.0), &spikes));
}

#[test]
fn up_spikes_spare_her_when_her_hurtbox_reaches_below_them() {
    let spikes = Spikes::new(Direction::Up, 0, 100, 16);
    assert!(spikes_kill(&player_at(8, 102, 0.0, 0.0), &spikes));
    assert!(!spikes_kill(&player_at(8, 103, 0.0, 0.0), &spikes));
}

#[test]
fn down_spikes_kill_unless_moving_down() {
    let spikes = Spikes::new(Direction::Down, 0, 100, 16);
    assert!(spikes_kill(&player_at(8, 111, 0.0, 0.0), &spikes));
    assert!(spikes_kill(&player_at(8, 111, 0.0, -50.0), &spikes));
    assert!(!spikes_kill(&player_at(8, 111, 0.0, 50.0), &spikes));
    assert!(!spikes_kill(&player_at(8, 114, 0.0, -50.0), &spikes));
}

#[test]
fn left_spikes_kill_unless_moving_left() {
    let spikes = Spikes::new(Direction::Left, 100, 0, 16);
    assert!(spikes_kill(&player_at(94, 12, 0.0, 0.0), &spikes));
    assert!(spikes_kill(&player_at(94, 12, 50.0, 0.0), &spikes));
    assert!(!spikes_kill(&player_at(94, 12, -50.0, 0.0), &spikes));
    assert!(!spikes_kill(&player_at(93, 12, 50.0, 0.0), &spikes));
}

#[test]
fn right_spikes_kill_unless_moving_right() {
    let spikes = Spikes::new(Direction::Right, 100, 0, 16);
    assert!(spikes_kill(&player_at(106, 12, 0.0, 0.0), &spikes));
    assert!(spikes_kill(&player_at(106, 12, -50.0, 0.0), &spikes));
    assert!(!spikes_kill(&player_at(106, 12, 50.0, 0.0), &spikes));
    assert!(!spikes_kill(&player_at(107, 12, -50.0, 0.0), &spikes));
}

fn spiked_floor() -> Room {
    let mut room = room();
    room.spikes.push(Spikes::new(Direction::Up, 0, FLOOR_Y, 40));
    room
}

#[test]
fn step_kills_on_spikes() {
    let s = step(standing(), Input::default(), &spiked_floor());
    assert!(s.dead);
    assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
}

#[test]
fn step_kills_on_the_frame_she_lands_on_spikes() {
    let room = spiked_floor();
    let mut s = player_at(0, FLOOR_Y - 5, 0.0, 160.0);
    s = step(s, Input::default(), &room);
    assert!(!s.dead);
    s = step(s, Input::default(), &room);
    assert!(s.dead);
    assert_eq!(s.position.y, FLOOR_Y);
}

#[test]
fn no_spikes_no_death() {
    let s = step(standing(), Input::default(), &room());
    assert!(!s.dead);
}

#[test]
fn fell_out_once_her_head_is_5_px_below_the_room() {
    let room = room();
    assert_eq!(room.bottom(), 180);
    assert!(!fell_out(&player_at(60, 180, 0.0, 0.0), &room));
    assert!(!fell_out(&player_at(60, 195, 0.0, 0.0), &room));
    assert!(fell_out(&player_at(60, 196, 0.0, 0.0), &room));
}

#[test]
fn step_kills_when_she_falls_out() {
    let room = room();
    let mut s = player_at(60, 190, 0.0, 160.0);
    s = step(s, Input::default(), &room);
    assert!(!s.dead);
    s = step(s, Input::default(), &room);
    s = step(s, Input::default(), &room);
    assert!(s.dead);
    assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
}

#[test]
fn falling_inside_the_room_is_not_death() {
    let s = step(player_at(60, 150, 0.0, 160.0), Input::default(), &room());
    assert!(!s.dead);
}

fn dashing_right() -> PlayerState {
    let mut s = standing();
    s.mode = Mode::Dashing { timer: DASH_TIME };
    s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
    s.speed = Vec2 {
        x: DASH_SPEED,
        y: 0.0,
    };
    s.jump_grace_timer = JUMP_GRACE_TIME;
    s
}

#[test]
fn can_super_jump_during_a_horizontal_dash() {
    assert!(can_super_jump(&dashing_right(), &room(), true));
}

#[test]
fn can_super_jump_needs_a_jump_press() {
    assert!(!can_super_jump(&dashing_right(), &room(), false));
}

#[test]
fn can_super_jump_needs_coyote_time() {
    let mut s = dashing_right();
    s.jump_grace_timer = 0.0;
    assert!(!can_super_jump(&s, &room(), true));
}

#[test]
fn can_super_jump_not_in_a_vertical_or_diagonal_dash() {
    let d = 1.0 / 2f32.sqrt();
    let mut s = dashing_right();
    s.dash_dir = Vec2 { x: 0.0, y: -1.0 };
    assert!(!can_super_jump(&s, &room(), true));
    s.dash_dir = Vec2 { x: d, y: -d };
    assert!(!can_super_jump(&s, &room(), true));
    s.dash_dir = Vec2 { x: 0.0, y: 1.0 };
    assert!(!can_super_jump(&s, &room(), true));
}

#[test]
fn super_jump_launches_the_way_she_faces() {
    let mut s = dashing_right();
    super_jump(&mut s);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H,
            y: JUMP_SPEED
        }
    );

    let mut s = dashing_right();
    s.facing = -1;
    super_jump(&mut s);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -SUPER_JUMP_H,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn super_jump_ends_the_dash_and_starts_a_variable_jump() {
    let mut s = dashing_right();
    s.auto_jump = true;
    s.wall_slide_timer = 0.5;
    super_jump(&mut s);
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.jump_grace_timer, 0.0);
    assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
    assert_eq!(s.var_jump_speed, JUMP_SPEED);
    assert_eq!(s.wall_slide_timer, WALL_SLIDE_TIME);
    assert!(!s.auto_jump);
}

fn dash_for(start: PlayerState, frames: usize, input: Input) -> PlayerState {
    let room = room();
    let mut s = start;
    for _ in 0..frames {
        s = step(s, input, &room);
    }
    s
}

const DASH_RIGHT: Input = Input {
    move_x: 1,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: true,
    grab: false,
    crouch_dash: false,
};
const JUMP_RIGHT: Input = Input {
    move_x: 1,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: false,
    crouch_dash: false,
};

#[test]
fn step_super_jumps_out_of_a_dash() {
    let s = step(dash_for(standing(), 5, DASH_RIGHT), JUMP_RIGHT, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn step_super_jumps_on_the_launch_frame() {
    let s = step(dash_for(standing(), 4, DASH_RIGHT), JUMP_RIGHT, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn step_super_jumps_on_the_last_dash_frame() {
    let s = step(dash_for(standing(), 14, DASH_RIGHT), JUMP_RIGHT, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn step_no_super_jump_in_the_air() {
    let s = step(dash_for(at_rest(), 5, DASH_RIGHT), JUMP_RIGHT, &room());
    assert!(matches!(s.mode, Mode::Dashing { .. }));
    assert_eq!(
        s.speed,
        Vec2 {
            x: DASH_SPEED,
            y: 0.0
        }
    );
}

#[test]
fn step_no_super_jump_out_of_an_up_dash() {
    let up_dash = Input {
        move_x: 0,
        move_y: -1,
        jump: false,
        jump2: false,
        dash: true,
        grab: false,
        crouch_dash: false,
    };
    let s = step(dash_for(standing(), 5, up_dash), JUMP_RIGHT, &room());
    assert!(matches!(s.mode, Mode::Dashing { .. }));
    assert_eq!(
        s.speed,
        Vec2 {
            x: 0.0,
            y: -DASH_SPEED
        }
    );
}

const JUMP_1: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: false,
    crouch_dash: false,
};
const JUMP_2: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: true,
    dash: false,
    grab: false,
    crouch_dash: false,
};
const JUMP_BOTH: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: true,
    dash: false,
    grab: false,
    crouch_dash: false,
};

#[test]
fn jump_down_with_either_button() {
    assert!(JUMP_1.jump_down());
    assert!(JUMP_2.jump_down());
    assert!(JUMP_BOTH.jump_down());
    assert!(!Input::default().jump_down());
}

#[test]
fn jump_pressed_when_a_button_goes_down() {
    assert!(jump_pressed(&standing(), JUMP_1));
    assert!(jump_pressed(&standing(), JUMP_2));
    assert!(!jump_pressed(&standing(), Input::default()));
}

#[test]
fn jump_pressed_not_while_the_same_button_stays_down() {
    let mut s = standing();
    s.jump_held = true;
    assert!(!jump_pressed(&s, JUMP_1));
    s.jump_held = false;
    s.jump2_held = true;
    assert!(!jump_pressed(&s, JUMP_2));
}

#[test]
fn jump_pressed_by_one_button_while_the_other_is_held() {
    let mut s = standing();
    s.jump_held = true;
    assert!(jump_pressed(&s, JUMP_BOTH));
    assert!(jump_pressed(&s, JUMP_2));
}

#[test]
fn step_tracks_each_button() {
    let s = step(standing(), JUMP_2, &room());
    assert!(s.jump2_held);
    assert!(!s.jump_held);
    let s = step(s, JUMP_1, &room());
    assert!(s.jump_held);
    assert!(!s.jump2_held);
}

#[test]
fn step_second_button_jumps_while_the_first_is_held() {
    let room = room();
    let mut s = standing();
    s.jump_held = true;
    s = step(s, JUMP_1, &room);
    assert_eq!(s.speed.y, 0.0);
    s = step(s, JUMP_2, &room);
    assert_eq!(s.speed.y, JUMP_SPEED);
}

#[test]
fn step_second_button_holds_a_variable_jump() {
    let room = room();
    let mut s = step(standing(), JUMP_1, &room);
    for _ in 0..5 {
        s = step(s, JUMP_2, &room);
    }
    assert_eq!(s.speed.y, JUMP_SPEED);
}

fn left_wall_room() -> Room {
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &"10000000000\n".repeat(23))
}

// flush against the left wall
const LEFT_WALL_X: i32 = 12;

const JUMP_GRAB: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: true,
    crouch_dash: false,
};

#[test]
fn wall_jump_check_reaches_two_pixels_of_gap() {
    let room = room();
    assert!(wall_jump_check(&player_at(WALL_X, 100, 0.0, 0.0), &room, 1));
    assert!(wall_jump_check(
        &player_at(WALL_X - 2, 100, 0.0, 0.0),
        &room,
        1
    ));
    assert!(!wall_jump_check(
        &player_at(WALL_X - 3, 100, 0.0, 0.0),
        &room,
        1
    ));
}

#[test]
fn wall_jump_check_only_on_the_side_asked() {
    assert!(!wall_jump_check(
        &player_at(WALL_X, 100, 0.0, 0.0),
        &room(),
        -1
    ));

    let room = left_wall_room();
    assert!(wall_jump_check(
        &player_at(LEFT_WALL_X, 100, 0.0, 0.0),
        &room,
        -1
    ));
    assert!(wall_jump_check(
        &player_at(LEFT_WALL_X + 2, 100, 0.0, 0.0),
        &room,
        -1
    ));
    assert!(!wall_jump_check(
        &player_at(LEFT_WALL_X + 3, 100, 0.0, 0.0),
        &room,
        -1
    ));
    assert!(!wall_jump_check(
        &player_at(LEFT_WALL_X, 100, 0.0, 0.0),
        &room,
        1
    ));
}

#[test]
fn can_climb_jump_facing_a_wall_with_grab_held() {
    assert!(can_climb_jump(
        &player_at(WALL_X, 100, 0.0, 0.0),
        &room(),
        JUMP_GRAB
    ));

    let mut s = player_at(LEFT_WALL_X, 100, 0.0, 0.0);
    s.facing = -1;
    assert!(can_climb_jump(&s, &left_wall_room(), JUMP_GRAB));
}

#[test]
fn can_climb_jump_needs_grab() {
    let no_grab = Input {
        grab: false,
        ..JUMP_GRAB
    };
    assert!(!can_climb_jump(
        &player_at(WALL_X, 100, 0.0, 0.0),
        &room(),
        no_grab
    ));
}

#[test]
fn can_climb_jump_needs_stamina() {
    let mut s = player_at(WALL_X, 100, 0.0, 0.0);
    s.stamina = 0.0;
    assert!(!can_climb_jump(&s, &room(), JUMP_GRAB));
}

#[test]
fn can_climb_jump_needs_a_wall() {
    assert!(!can_climb_jump(
        &player_at(WALL_X - 3, 100, 0.0, 0.0),
        &room(),
        JUMP_GRAB
    ));
}

#[test]
fn can_climb_jump_not_facing_away_from_the_wall() {
    let mut s = player_at(WALL_X, 100, 0.0, 0.0);
    s.facing = -1;
    assert!(!can_climb_jump(&s, &room(), JUMP_GRAB));

    let s = player_at(LEFT_WALL_X, 100, 0.0, 0.0);
    assert!(!can_climb_jump(&s, &left_wall_room(), JUMP_GRAB));
}

#[test]
fn climb_jump_in_the_air_costs_stamina() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    s.mode = Mode::Dashing { timer: 0.1 };
    climb_jump(
        &mut s,
        frame(
            Input {
                move_x: 1,
                ..JUMP_GRAB
            },
            false,
        ),
    );
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
    assert_eq!(
        s.speed,
        Vec2 {
            x: JUMP_H_BOOST,
            y: JUMP_SPEED
        }
    );
    assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
    assert_eq!(s.mode, Mode::Normal);
}

#[test]
fn climb_jump_on_the_ground_is_free() {
    let mut s = standing();
    climb_jump(&mut s, frame(JUMP_GRAB, true));
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    assert_eq!(s.speed.y, JUMP_SPEED);
}

#[test]
fn step_refills_stamina_on_the_ground() {
    let mut s = standing();
    s.stamina = 10.0;
    s = step(s, Input::default(), &room());
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn step_does_not_refill_stamina_in_the_air() {
    let mut s = at_rest();
    s.stamina = 10.0;
    s = step(s, Input::default(), &room());
    assert_eq!(s.stamina, 10.0);
}

#[test]
fn step_climb_jumps_next_to_a_wall() {
    let s = step(player_at(WALL_X, 100, 0.0, -30.0), JUMP_GRAB, &room());
    assert_eq!(s.speed.y, JUMP_SPEED);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
}

#[test]
fn step_no_climb_jump_without_grab() {
    let no_grab = Input {
        grab: false,
        ..JUMP_GRAB
    };
    let s = step(player_at(WALL_X, 100, 0.0, 30.0), no_grab, &room());
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn step_ground_jump_wins_over_climb_jump() {
    let mut s = player_at(36, FLOOR_Y, 0.0, 0.0);
    s.stamina = 50.0;
    s = step(
        s,
        Input {
            move_x: 1,
            ..JUMP_GRAB
        },
        &room(),
    );
    assert_eq!(s.speed.y, JUMP_SPEED);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn step_climb_jumps_out_of_a_dash() {
    let mut s = player_at(WALL_X - 2, 100, DASH_SPEED, 0.0);
    s.mode = Mode::Dashing { timer: 0.1 };
    s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
    s = step(
        s,
        Input {
            move_x: 1,
            ..JUMP_GRAB
        },
        &room(),
    );
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
    assert_eq!(s.position.x, WALL_X);
    assert_eq!(
        s.speed,
        Vec2 {
            x: 0.0,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn move_h_into_a_wall_remembers_her_speed() {
    let mut s = player_at(WALL_X - 1, 100, 120.0, 0.0);
    move_h(&mut s, &room(), 3.0, false);
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.wall_speed_retained, 120.0);
    assert_eq!(s.wall_speed_retention_timer, WALL_SPEED_RETENTION_TIME);
}

#[test]
fn move_h_keeps_the_first_speed_while_the_timer_runs() {
    let mut s = player_at(WALL_X - 1, 100, 50.0, 0.0);
    s.wall_speed_retained = 200.0;
    s.wall_speed_retention_timer = 0.03;
    move_h(&mut s, &room(), 3.0, false);
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.wall_speed_retained, 200.0);
    assert_eq!(s.wall_speed_retention_timer, 0.03);
}

#[test]
fn move_h_without_a_collision_retains_nothing() {
    let mut s = player_at(0, 100, 120.0, 0.0);
    move_h(&mut s, &room(), 3.0, false);
    assert_eq!(s.wall_speed_retention_timer, 0.0);
}

fn retaining(x: i32, speed_x: f32, retained: f32) -> PlayerState {
    let mut s = player_at(x, 100, speed_x, 0.0);
    s.wall_speed_retained = retained;
    s.wall_speed_retention_timer = 0.03;
    s
}

#[test]
fn retention_gives_the_speed_back_once_the_wall_is_gone() {
    let mut s = retaining(WALL_X - 1, 0.0, 120.0);
    update_wall_speed_retention(&mut s, &room());
    assert_eq!(s.speed.x, 120.0);
    assert_eq!(s.wall_speed_retention_timer, 0.0);
}

#[test]
fn retention_waits_while_the_wall_is_still_there() {
    let mut s = retaining(WALL_X, 0.0, 120.0);
    update_wall_speed_retention(&mut s, &room());
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.wall_speed_retention_timer, 0.03 - DELTA_TIME);
}

#[test]
fn retention_is_cancelled_by_moving_the_other_way() {
    let mut s = retaining(WALL_X - 1, -10.0, 120.0);
    update_wall_speed_retention(&mut s, &room());
    assert_eq!(s.speed.x, -10.0);
    assert_eq!(s.wall_speed_retention_timer, 0.0);
}

#[test]
fn retention_works_to_the_left() {
    let room = left_wall_room();
    let mut s = retaining(LEFT_WALL_X, 0.0, -120.0);
    update_wall_speed_retention(&mut s, &room);
    assert_eq!(s.speed.x, 0.0);

    let mut s = retaining(LEFT_WALL_X + 1, 0.0, -120.0);
    update_wall_speed_retention(&mut s, &room);
    assert_eq!(s.speed.x, -120.0);
}

#[test]
fn retention_does_nothing_once_the_timer_is_out() {
    let mut s = retaining(WALL_X - 1, 0.0, 120.0);
    s.wall_speed_retention_timer = 0.0;
    update_wall_speed_retention(&mut s, &room());
    assert_eq!(s.speed.x, 0.0);
}

#[test]
fn step_gives_retained_speed_back_before_moving() {
    let s = step(
        retaining(WALL_X - 20, 0.0, 120.0),
        Input {
            move_x: 1,
            ..Default::default()
        },
        &room(),
    );
    assert!(s.speed.x > 100.0);
    assert!(s.position.x > WALL_X - 20);
    assert_eq!(s.wall_speed_retention_timer, 0.0);
}

#[test]
fn step_running_into_a_wall_starts_retention() {
    let s = step(
        player_at(WALL_X, 100, 90.0, 0.0),
        Input {
            move_x: 1,
            ..Default::default()
        },
        &room(),
    );
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.wall_speed_retained, 90.0);
    assert_eq!(s.wall_speed_retention_timer, WALL_SPEED_RETENTION_TIME);
}

// an 8 px high tunnel from x = 24 rightwards: she fits ducking, not standing
fn tunnel_room() -> Room {
    let text =
        "00000000000\n".repeat(16) + "00011111111\n" + "00000000000\n" + &"11111111111\n".repeat(5);
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

fn ducking_at(x: i32, y: i32) -> PlayerState {
    let mut s = player_at(x, y, 0.0, 0.0);
    s.ducking = true;
    s
}

const DOWN: Input = Input {
    move_x: 0,
    move_y: 1,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    crouch_dash: false,
};

#[test]
fn collides_uses_the_shorter_hitbox_when_ducking() {
    let room = tunnel_room();
    assert!(!collides(&ducking_at(40, FLOOR_Y), &room, 0, 0));
    assert!(collides(&player_at(40, FLOOR_Y, 0.0, 0.0), &room, 0, 0));
    assert!(collides(&ducking_at(40, FLOOR_Y), &room, 0, 1));
    assert!(collides(&ducking_at(40, FLOOR_Y), &room, 0, -3));
}

#[test]
fn can_unduck_when_not_ducking() {
    assert!(can_unduck(
        &player_at(40, FLOOR_Y, 0.0, 0.0),
        &tunnel_room()
    ));
}

#[test]
fn can_unduck_with_room_overhead() {
    assert!(can_unduck(&ducking_at(8, FLOOR_Y), &tunnel_room()));
}

#[test]
fn can_unduck_not_under_a_low_ceiling() {
    assert!(!can_unduck(&ducking_at(40, FLOOR_Y), &tunnel_room()));
}

#[test]
fn hurtbox_is_shorter_when_ducking() {
    assert_eq!(
        hurtbox(&ducking_at(100, 50)),
        Rect {
            left: 96,
            top: 44,
            width: 8,
            height: 4
        }
    );
}

#[test]
fn dash_begin_ducks_when_down_is_held() {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(DOWN, true));
    assert!(s.ducking);
}

#[test]
fn dash_begin_stays_standing_without_down() {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    assert!(!s.ducking);
}

#[test]
fn dash_begin_stays_ducked_on_the_ground() {
    let mut s = ducking_at(0, FLOOR_Y);
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    assert!(s.ducking);
}

#[test]
fn dash_begin_stands_up_in_the_air() {
    let mut s = ducking_at(0, 64);
    dash_begin(&mut s, &room(), frame(Input::default(), false));
    assert!(!s.ducking);
}

#[test]
fn super_jump_while_ducking_is_a_hyper() {
    let mut s = dashing_right();
    s.ducking = true;
    super_jump(&mut s);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H * DUCK_SUPER_JUMP_X_MULT,
            y: JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT
        }
    );
    assert_eq!(s.var_jump_speed, JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT);
    assert!(!s.ducking);
}

#[test]
fn can_super_jump_needs_room_to_stand_up() {
    let mut s = ducking_at(40, FLOOR_Y);
    s.mode = Mode::Dashing { timer: DASH_TIME };
    s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
    s.jump_grace_timer = JUMP_GRACE_TIME;
    assert!(!can_super_jump(&s, &tunnel_room(), true));
    s.position.x = 8;
    assert!(can_super_jump(&s, &tunnel_room(), true));
}

fn launching() -> PlayerState {
    let mut s = standing();
    s.mode = Mode::DashStart;
    s
}

#[test]
fn dash_launch_down_diagonal_on_the_ground_slides() {
    let d = 1.0 / 2f32.sqrt();
    let mut s = launching();
    dash_launch(&mut s, Vec2 { x: -d, y: d }, true);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -d * DASH_SPEED * DODGE_SLIDE_SPEED_MULT,
            y: 0.0
        }
    );
    assert_eq!(s.dash_dir, Vec2 { x: -1.0, y: 0.0 });
    assert!(s.ducking);
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
    assert_eq!(s.dash_dir, Vec2 { x: d, y: d });
    assert!(!s.ducking);
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
    assert!(!s.ducking);

    let mut s = launching();
    dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
    assert_eq!(
        s.speed,
        Vec2 {
            x: DASH_SPEED,
            y: 0.0
        }
    );
    assert!(!s.ducking);
}

#[test]
fn step_ducks_when_down_is_held_on_the_ground() {
    assert!(step(standing(), DOWN, &room()).ducking);
}

#[test]
fn step_does_not_duck_in_the_air() {
    assert!(!step(at_rest(), DOWN, &room()).ducking);
}

#[test]
fn step_stands_up_when_down_is_released() {
    assert!(!step(ducking_at(0, FLOOR_Y), Input::default(), &room()).ducking);
}

#[test]
fn step_stays_ducked_while_down_is_held() {
    assert!(step(ducking_at(0, FLOOR_Y), DOWN, &room()).ducking);
}

#[test]
fn step_stays_ducked_under_a_low_ceiling() {
    assert!(step(ducking_at(40, FLOOR_Y), Input::default(), &tunnel_room()).ducking);
}

#[test]
fn step_ducking_on_the_ground_uses_duck_friction() {
    let mut s = ducking_at(0, FLOOR_Y);
    s.speed.x = 90.0;
    s = step(s, Input { move_x: 1, ..DOWN }, &room());
    assert_eq!(s.speed.x, approach(90.0, 0.0, DUCK_FRICTION * DELTA_TIME));
}

#[test]
fn step_stands_up_when_falling() {
    let mut s = ducking_at(0, 64);
    s.speed.y = 50.0;
    assert!(!step(s, Input::default(), &room()).ducking);
}

#[test]
fn step_stays_ducked_when_falling_in_coyote_time() {
    let mut s = ducking_at(0, 64);
    s.speed.y = 50.0;
    s.jump_grace_timer = JUMP_GRACE_TIME;
    assert!(step(s, Input::default(), &room()).ducking);
}

#[test]
fn step_stands_up_when_falling_even_with_down_held() {
    let mut s = ducking_at(0, 64);
    s.speed.y = 50.0;
    assert!(!step(s, DOWN, &room()).ducking);
}

#[test]
fn step_stays_ducked_while_rising() {
    let mut s = ducking_at(0, 64);
    s.speed.y = -100.0;
    assert!(step(s, Input::default(), &room()).ducking);
}

#[test]
fn step_hyper_from_a_down_dash() {
    let down_dash = Input { dash: true, ..DOWN };
    let s = step(dash_for(standing(), 4, down_dash), JUMP_RIGHT, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H * DUCK_SUPER_JUMP_X_MULT,
            y: JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT
        }
    );
    assert!(!s.ducking);
}

#[test]
fn climb_jump_without_a_direction_arms_the_wall_boost() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    climb_jump(&mut s, frame(JUMP_GRAB, false));
    assert_eq!(s.wall_boost_dir, -1);
    assert_eq!(s.wall_boost_timer, CLIMB_JUMP_BOOST_TIME);

    let mut s = player_at(LEFT_WALL_X, 100, 0.0, 30.0);
    s.facing = -1;
    climb_jump(&mut s, frame(JUMP_GRAB, false));
    assert_eq!(s.wall_boost_dir, 1);
}

#[test]
fn climb_jump_with_a_direction_does_not_arm_the_wall_boost() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    climb_jump(
        &mut s,
        frame(
            Input {
                move_x: 1,
                ..JUMP_GRAB
            },
            false,
        ),
    );
    assert_eq!(s.wall_boost_timer, 0.0);
}

#[test]
fn jumps_cancel_the_wall_boost() {
    let mut s = standing();
    s.wall_boost_timer = 0.1;
    jump(&mut s, Input::default());
    assert_eq!(s.wall_boost_timer, 0.0);

    let mut s = dashing_right();
    s.wall_boost_timer = 0.1;
    super_jump(&mut s);
    assert_eq!(s.wall_boost_timer, 0.0);
}

fn wall_boosting() -> PlayerState {
    let mut s = player_at(40, 100, 0.0, -100.0);
    s.wall_boost_dir = -1;
    s.wall_boost_timer = 0.1;
    s.stamina = 50.0;
    s
}

const LEFT: Input = Input {
    move_x: -1,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    crouch_dash: false,
};

#[test]
fn step_remembers_the_direction_held() {
    assert_eq!(step(at_rest(), LEFT, &room()).move_x, -1);
    assert_eq!(step(at_rest(), Input::default(), &room()).move_x, 0);
}

#[test]
fn step_wall_boost_happens_the_frame_after_the_press() {
    let room = room();
    let mut s = step(wall_boosting(), LEFT, &room);
    assert_eq!(s.stamina, 50.0);
    assert_eq!(
        s.speed.x,
        approach(0.0, -MAX_RUN, RUN_ACCEL * DELTA_TIME * AIR_MULT)
    );

    s = step(s, Input::default(), &room);
    assert_eq!(s.stamina, 50.0 + CLIMB_JUMP_COST);
    assert_eq!(s.wall_boost_timer, 0.0);
    assert_eq!(
        s.speed.x,
        approach(-WALL_JUMP_H_SPEED, 0.0, RUN_ACCEL * DELTA_TIME * AIR_MULT)
    );
}

#[test]
fn step_wall_boost_ignores_the_other_direction() {
    let room = room();
    let mut s = step(wall_boosting(), Input { move_x: 1, ..LEFT }, &room);
    s = step(s, Input::default(), &room);
    assert_eq!(s.stamina, 50.0);
    assert!(s.wall_boost_timer > 0.0);
}

#[test]
fn step_wall_boost_runs_out() {
    let room = room();
    let mut s = wall_boosting();
    s.wall_boost_timer = 0.01;
    s = step(s, LEFT, &room);
    s = step(s, LEFT, &room);
    assert_eq!(s.stamina, 50.0);
}

// a ceiling block spanning x 16..48, y 40..48
fn ceiling_room() -> Room {
    let text = "00000000000\n".repeat(5) + "00111100000\n" + &"00000000000\n".repeat(17);
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

// her head touches the ceiling block's underside at this y
const UNDER_CEILING_Y: i32 = 59;

fn rising_at(x: i32, speed_x: f32) -> PlayerState {
    player_at(x, UNDER_CEILING_Y, speed_x, -240.0)
}

#[test]
fn move_v_up_into_a_corner_slides_around_it() {
    let mut s = rising_at(50, 0.0);
    s.remainder.y = 0.25;
    move_v(&mut s, &ceiling_room(), -4.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: 52,
            y: UNDER_CEILING_Y - 1
        }
    );
    assert_eq!(s.speed.y, -240.0);
    assert_eq!(s.remainder.y, 0.0);
}

#[test]
fn corner_correction_reaches_four_pixels() {
    let room = ceiling_room();
    let mut s = rising_at(48, 0.0);
    move_v(&mut s, &room, -1.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: 52,
            y: UNDER_CEILING_Y - 1
        }
    );

    let mut s = rising_at(47, 0.0);
    move_v(&mut s, &room, -1.0);
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
    s.dash_attack_timer = DASH_ATTACK_TIME;
    move_v(&mut s, &room, -1.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: 52,
            y: UNDER_CEILING_Y - 1
        }
    );

    let mut s = rising_at(46, 0.0);
    s.dash_attack_timer = DASH_ATTACK_TIME;
    move_v(&mut s, &room, -1.0);
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
    let mut s = rising_at(47, 10.0);
    s.dash_attack_timer = DASH_ATTACK_TIME;
    move_v(&mut s, &ceiling_room(), -1.0);
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
    let mut s = rising_at(14, 0.0);
    move_v(&mut s, &ceiling_room(), -1.0);
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
    move_v(&mut s, &room, -1.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: 50,
            y: UNDER_CEILING_Y
        }
    );
    assert_eq!(s.speed.y, 0.0);

    let mut s = rising_at(14, 10.0);
    move_v(&mut s, &room, -1.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: 14,
            y: UNDER_CEILING_Y
        }
    );

    let mut s = rising_at(50, 10.0);
    move_v(&mut s, &room, -1.0);
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
    let mut s = rising_at(32, 0.0);
    s.var_jump_timer = 0.1;
    move_v(&mut s, &ceiling_room(), -1.0);
    assert_eq!(s.speed.y, 0.0);
    assert_eq!(s.var_jump_timer, 0.0);
}

#[test]
fn ceiling_bonk_at_the_start_of_a_jump_keeps_the_variable_jump() {
    let mut s = rising_at(32, 0.0);
    s.var_jump_timer = 0.18;
    move_v(&mut s, &ceiling_room(), -1.0);
    assert_eq!(s.speed.y, 0.0);
    assert_eq!(s.var_jump_timer, 0.18);
}

#[test]
fn corner_correction_keeps_the_variable_jump() {
    let mut s = rising_at(50, 0.0);
    s.var_jump_timer = 0.1;
    move_v(&mut s, &ceiling_room(), -1.0);
    assert_eq!(s.var_jump_timer, 0.1);
}

#[test]
fn landing_does_not_corner_correct() {
    let mut s = player_at(LEDGE_X, FLOOR_Y, 0.0, 100.0);
    move_v(&mut s, &room(), 2.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: LEDGE_X,
            y: FLOOR_Y
        }
    );
    assert_eq!(s.speed.y, 0.0);
}

fn dashing_at(x: i32, y: i32, speed_x: f32, speed_y: f32) -> PlayerState {
    let mut s = player_at(x, y, speed_x, speed_y);
    s.mode = Mode::Dashing { timer: 0.1 };
    s
}

// flush against the right face of the floor block, in the pit
const PIT_X: i32 = 44;

#[test]
fn dash_begin_remembers_whether_it_started_on_the_ground() {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    assert!(s.dash_started_on_ground);

    let mut s = at_rest();
    s.dash_started_on_ground = true;
    dash_begin(&mut s, &room(), frame(Input::default(), false));
    assert!(!s.dash_started_on_ground);
}

#[test]
fn dash_into_a_ledge_just_below_its_top_pops_up_onto_it() {
    let mut s = dashing_at(PIT_X, FLOOR_Y + 2, -240.0, 0.0);
    s.remainder.x = 0.25;
    move_h(&mut s, &room(), -3.0, false);
    assert_eq!(
        s.position,
        IVec2 {
            x: PIT_X - 1,
            y: FLOOR_Y
        }
    );
    assert_eq!(s.speed.x, -240.0);
    assert_eq!(s.remainder.x, 0.0);
    assert_eq!(s.wall_speed_retention_timer, 0.0);
}

#[test]
fn dash_corner_correction_reaches_four_pixels() {
    let mut s = dashing_at(PIT_X, FLOOR_Y + 4, -240.0, 0.0);
    move_h(&mut s, &room(), -3.0, false);
    assert_eq!(
        s.position,
        IVec2 {
            x: PIT_X - 1,
            y: FLOOR_Y
        }
    );

    let mut s = dashing_at(PIT_X, FLOOR_Y + 5, -240.0, 0.0);
    move_h(&mut s, &room(), -3.0, false);
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
    let mut s = player_at(PIT_X, FLOOR_Y + 2, -240.0, 0.0);
    move_h(&mut s, &room(), -3.0, false);
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
    let mut s = dashing_at(PIT_X, FLOOR_Y + 2, -170.0, -170.0);
    move_h(&mut s, &room(), -3.0, false);
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
fn duck_free_at_checks_the_ducking_hitbox() {
    let room = tunnel_room();
    let s = player_at(20, FLOOR_Y, 0.0, 0.0);
    assert!(duck_free_at(&s, &room, 1));
    assert!(collides(&s, &room, 1, 0));
    assert!(!duck_free_at(
        &player_at(20, FLOOR_Y - 4, 0.0, 0.0),
        &room,
        1
    ));
}

#[test]
fn ground_dash_into_a_low_gap_ducks() {
    let room = tunnel_room();
    let mut s = dashing_at(20, FLOOR_Y, 240.0, 0.0);
    move_h(&mut s, &room, 4.0, true);
    assert!(s.ducking);
    assert_eq!(s.position.x, 20);
    assert_eq!(s.speed.x, 240.0);

    move_h(&mut s, &room, 4.0, true);
    assert_eq!(s.position.x, 24);
}

#[test]
fn air_dash_into_a_low_gap_does_not_duck() {
    let mut s = dashing_at(20, FLOOR_Y, 240.0, 0.0);
    move_h(&mut s, &tunnel_room(), 4.0, false);
    assert!(!s.ducking);
    assert_eq!(s.speed.x, 0.0);
}

fn air_dashing_down_at(x: i32, speed_x: f32) -> PlayerState {
    dashing_at(x, FLOOR_Y - 1, speed_x, 240.0)
}

#[test]
fn air_dash_down_onto_a_ledge_edge_slips_off() {
    let mut s = air_dashing_down_at(LEDGE_X - 1, 0.0);
    s.remainder.y = 0.25;
    move_v(&mut s, &room(), 4.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: LEDGE_X + 1,
            y: FLOOR_Y + 1
        }
    );
    assert_eq!(s.speed.y, 240.0);
    assert_eq!(s.remainder.y, 0.0);
}

#[test]
fn downward_corner_correction_reaches_four_pixels() {
    let mut s = air_dashing_down_at(LEDGE_X - 3, 0.0);
    move_v(&mut s, &room(), 4.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: LEDGE_X + 1,
            y: FLOOR_Y + 1
        }
    );

    let mut s = air_dashing_down_at(LEDGE_X - 4, 0.0);
    move_v(&mut s, &room(), 4.0);
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
    let mut s = air_dashing_down_at(LEDGE_X - 1, -10.0);
    move_v(&mut s, &room(), 4.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: LEDGE_X - 1,
            y: FLOOR_Y
        }
    );

    let mut s = air_dashing_down_at(LEDGE_X - 1, 10.0);
    move_v(&mut s, &room(), 4.0);
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
    let mut s = air_dashing_down_at(LEDGE_X - 1, 0.0);
    s.dash_started_on_ground = true;
    move_v(&mut s, &room(), 4.0);
    assert_eq!(
        s.position,
        IVec2 {
            x: LEDGE_X - 1,
            y: FLOOR_Y
        }
    );

    let mut s = player_at(LEDGE_X - 1, FLOOR_Y - 1, 0.0, 240.0);
    move_v(&mut s, &room(), 4.0);
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
    let d = 1.0 / 2f32.sqrt();
    let mut s = player_at(0, FLOOR_Y - 1, 100.0, 200.0);
    s.dash_dir = Vec2 { x: d, y: d };
    move_v(&mut s, &room(), 4.0);
    assert_eq!(s.position.y, FLOOR_Y);
    assert_eq!(
        s.speed,
        Vec2 {
            x: 100.0 * DODGE_SLIDE_SPEED_MULT,
            y: 0.0
        }
    );
    assert_eq!(s.dash_dir, Vec2 { x: 1.0, y: 0.0 });
    assert!(s.ducking);
}

#[test]
fn landing_after_a_straight_down_dash_does_not_slide() {
    let mut s = player_at(0, FLOOR_Y - 1, 100.0, 200.0);
    s.dash_dir = Vec2 { x: 0.0, y: 1.0 };
    move_v(&mut s, &room(), 4.0);
    assert_eq!(s.speed, Vec2 { x: 100.0, y: 0.0 });
    assert!(!s.ducking);
}

const JUMP: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: false,
    crouch_dash: false,
};

// just after a dash straight up
fn bouncing_at(x: i32) -> PlayerState {
    let mut s = player_at(x, 100, 0.0, -120.0);
    s.dash_dir = Vec2 { x: 0.0, y: -1.0 };
    s.dash_attack_timer = 0.1;
    s
}

#[test]
fn super_wall_jump_is_faster_and_higher_than_a_jump() {
    let mut s = bouncing_at(LEFT_WALL_X);
    s.ducking = true;
    s.auto_jump = true;
    super_wall_jump(&mut s, 1);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_WALL_JUMP_H,
            y: SUPER_WALL_JUMP_SPEED
        }
    );
    assert_eq!(s.var_jump_speed, SUPER_WALL_JUMP_SPEED);
    assert_eq!(s.var_jump_timer, SUPER_WALL_JUMP_VAR_TIME);
    assert_eq!(s.dash_attack_timer, 0.0);
    assert_eq!(s.mode, Mode::Normal);
    assert!(!s.ducking);
    assert!(!s.auto_jump);
}

#[test]
fn wall_jump_check_reaches_further_for_a_wall_bounce() {
    let room = room();
    assert!(wall_jump_check(&bouncing_at(WALL_X - 4), &room, 1));
    assert!(!wall_jump_check(&bouncing_at(WALL_X - 5), &room, 1));
    assert!(!wall_jump_check(
        &player_at(WALL_X - 4, 100, 0.0, 0.0),
        &room,
        1
    ));
}

#[test]
fn wall_jump_check_is_short_again_when_the_wall_has_spikes() {
    let mut room = room();
    room.spikes
        .push(Spikes::new(Direction::Left, WALL_X + 4, 88, 16));
    assert!(!wall_jump_check(&bouncing_at(WALL_X - 4), &room, 1));
    assert!(wall_jump_check(&bouncing_at(WALL_X - 2), &room, 1));
}

#[test]
fn dash_attack_runs_from_the_start_of_a_dash() {
    let mut s = standing();
    dash_begin(&mut s, &room(), frame(Input::default(), true));
    assert_eq!(s.dash_attack_timer, DASH_ATTACK_TIME);

    let s = dash_for(standing(), 6, DASH_RIGHT);
    assert!(s.dash_attack_timer > 0.0 && s.dash_attack_timer < DASH_ATTACK_TIME);
}

#[test]
fn jumps_and_collisions_end_the_dash_attack() {
    let mut s = standing();
    s.dash_attack_timer = 0.1;
    jump(&mut s, Input::default());
    assert_eq!(s.dash_attack_timer, 0.0);

    let mut s = player_at(WALL_X, 100, 90.0, 0.0);
    s.dash_attack_timer = 0.1;
    move_h(&mut s, &room(), 3.0, false);
    assert_eq!(s.dash_attack_timer, 0.0);

    let mut s = player_at(0, FLOOR_Y, 0.0, 100.0);
    s.dash_attack_timer = 0.1;
    move_v(&mut s, &room(), 3.0);
    assert_eq!(s.dash_attack_timer, 0.0);
}

#[test]
fn step_wall_bounce_after_an_upward_dash() {
    let s = step(bouncing_at(WALL_X - 4), JUMP, &room());
    assert_eq!(
        s.speed,
        Vec2 {
            x: -SUPER_WALL_JUMP_H,
            y: SUPER_WALL_JUMP_SPEED
        }
    );

    let s = step(bouncing_at(LEFT_WALL_X + 4), JUMP, &left_wall_room());
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_WALL_JUMP_H,
            y: SUPER_WALL_JUMP_SPEED
        }
    );
}

#[test]
fn step_wall_bounce_during_an_upward_dash_ignores_grab() {
    let mut s = bouncing_at(WALL_X);
    s.speed.y = -DASH_SPEED;
    s.mode = Mode::Dashing { timer: 0.1 };
    s = step(s, JUMP_GRAB, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -SUPER_WALL_JUMP_H,
            y: SUPER_WALL_JUMP_SPEED
        }
    );
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn step_no_wall_bounce_once_the_dash_attack_is_over() {
    let mut s = bouncing_at(WALL_X - 4);
    s.dash_attack_timer = 0.0;
    s = step(s, JUMP, &room());
    assert_eq!(s.speed.x, 0.0);
}

#[test]
fn step_holding_grab_without_pressing_jump_does_not_climb_jump() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    s.jump_held = true;
    s = step(s, JUMP_GRAB, &room());
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    assert!(s.speed.y > 0.0);
}

#[test]
fn wall_jump_pushes_off_the_wall() {
    let mut s = ducking_at(WALL_X, 100);
    s.mode = Mode::Dashing { timer: 0.1 };
    s.dash_attack_timer = 0.2;
    s.auto_jump = true;
    wall_jump(&mut s, &room(), -1, 0);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -WALL_JUMP_H_SPEED,
            y: JUMP_SPEED
        }
    );
    assert_eq!(s.var_jump_speed, JUMP_SPEED);
    assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.dash_attack_timer, 0.0);
    assert!(!s.ducking);
    assert!(!s.auto_jump);
}

#[test]
fn wall_jump_forces_her_direction_only_if_one_is_held() {
    let mut s = player_at(WALL_X, 100, 0.0, 0.0);
    wall_jump(&mut s, &room(), -1, 0);
    assert_eq!(s.force_move_x_timer, 0.0);

    let mut s = player_at(WALL_X, 100, 0.0, 0.0);
    wall_jump(&mut s, &room(), -1, 1);
    assert_eq!(s.force_move_x, -1);
    assert_eq!(s.force_move_x_timer, WALL_JUMP_FORCE_TIME);
}

#[test]
fn step_wall_jumps_without_grab() {
    let s = step(player_at(WALL_X - 2, 100, 0.0, 30.0), JUMP, &room());
    assert_eq!(
        s.speed,
        Vec2 {
            x: -WALL_JUMP_H_SPEED,
            y: JUMP_SPEED
        }
    );
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);

    let s = step(
        player_at(LEFT_WALL_X + 2, 100, 0.0, 30.0),
        JUMP,
        &left_wall_room(),
    );
    assert_eq!(
        s.speed,
        Vec2 {
            x: WALL_JUMP_H_SPEED,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn step_wall_jumps_when_facing_away_even_with_grab() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    s.facing = -1;
    s = step(s, JUMP_GRAB, &room());
    assert_eq!(s.speed.x, -WALL_JUMP_H_SPEED);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn step_no_wall_jump_away_from_walls() {
    let s = step(player_at(40, 100, 0.0, 30.0), JUMP, &room());
    assert_eq!(s.speed.x, 0.0);
    assert!(s.speed.y > 0.0);
}

#[test]
fn step_wall_jumps_out_of_a_sideways_dash() {
    let mut s = dashing_at(WALL_X - 2, 100, -DASH_SPEED, 0.0);
    s.dash_dir = Vec2 { x: -1.0, y: 0.0 };
    s.facing = -1;
    s = step(s, JUMP, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -WALL_JUMP_H_SPEED,
            y: JUMP_SPEED
        }
    );
}

#[test]
fn step_forced_direction_overrides_her_input() {
    let mut s = player_at(40, 100, -WALL_JUMP_H_SPEED, -100.0);
    s.facing = -1;
    s.force_move_x = -1;
    s.force_move_x_timer = 0.1;
    s = step(
        s,
        Input {
            move_x: 1,
            ..Input::default()
        },
        &room(),
    );
    assert_eq!(s.facing, -1);
    assert_eq!(
        s.speed.x,
        approach(
            -WALL_JUMP_H_SPEED,
            -MAX_RUN,
            RUN_REDUCE * DELTA_TIME * AIR_MULT
        )
    );
    assert_eq!(s.force_move_x_timer, 0.1 - DELTA_TIME);
}

#[test]
fn step_forced_direction_runs_out() {
    let mut s = player_at(40, 100, 0.0, -100.0);
    s.force_move_x = -1;
    s.force_move_x_timer = 0.01;
    let right = Input {
        move_x: 1,
        ..Input::default()
    };
    s = step(s, right, &room());
    assert_eq!(s.facing, -1);
    s = step(s, right, &room());
    assert_eq!(s.facing, 1);
}

#[test]
fn step_forced_direction_does_not_change_the_dash_aim() {
    let mut s = player_at(40, 100, -WALL_JUMP_H_SPEED, -100.0);
    s.force_move_x = -1;
    s.force_move_x_timer = 0.16;
    let right_dash = Input {
        move_x: 1,
        dash: true,
        ..Input::default()
    };
    let s = dash_for(s, 5, right_dash);
    assert_eq!(s.speed.x, DASH_SPEED);
}

#[test]
fn track_buttons_remembers_what_is_down() {
    let mut s = standing();
    track_buttons(
        &mut s,
        Input {
            jump2: true,
            dash: true,
            ..Input::default()
        },
    );
    assert!(!s.jump_held);
    assert!(s.jump2_held);
    assert!(s.dash_held);

    track_buttons(&mut s, Input::default());
    assert!(!s.jump2_held);
    assert!(!s.dash_held);
}

// the usual room with a platform over the pit, spanning x 40..80 with its top at y 120
const PLATFORM_Y: i32 = 120;

fn platform_room() -> Room {
    let mut room = room();
    room.jump_thrus
        .push(crate::entities::JumpThru::new(40, PLATFORM_Y, 40));
    room
}

#[test]
fn standing_on_a_jump_thru_is_on_the_ground() {
    let room = platform_room();
    assert!(on_ground(&player_at(60, PLATFORM_Y, 0.0, 0.0), &room));
    assert!(!on_ground(&player_at(60, PLATFORM_Y - 1, 0.0, 0.0), &room));
    assert!(!on_ground(&player_at(30, PLATFORM_Y, 0.0, 0.0), &room));
}

#[test]
fn inside_a_jump_thru_is_not_on_the_ground() {
    assert!(!on_ground(
        &player_at(60, PLATFORM_Y + 2, 0.0, 0.0),
        &platform_room()
    ));
}

#[test]
fn falling_onto_a_jump_thru_lands_on_it() {
    let mut s = player_at(60, PLATFORM_Y - 2, 0.0, 100.0);
    move_v(&mut s, &platform_room(), 5.0);
    assert_eq!(s.position.y, PLATFORM_Y);
    assert_eq!(s.speed.y, 0.0);
}

#[test]
fn rising_goes_through_a_jump_thru() {
    let mut s = player_at(60, PLATFORM_Y + 15, 0.0, -100.0);
    move_v(&mut s, &platform_room(), -20.0);
    assert_eq!(s.position.y, PLATFORM_Y - 5);
    assert_eq!(s.speed.y, -100.0);
}

#[test]
fn falling_while_inside_a_jump_thru_keeps_falling() {
    let mut s = player_at(60, PLATFORM_Y + 2, 0.0, 100.0);
    move_v(&mut s, &platform_room(), 5.0);
    assert_eq!(s.position.y, PLATFORM_Y + 7);
    assert_eq!(s.speed.y, 100.0);
}

#[test]
fn jump_thrus_do_not_block_sideways_movement() {
    let mut s = player_at(30, PLATFORM_Y + 2, 100.0, 0.0);
    move_h(&mut s, &platform_room(), 20.0, false);
    assert_eq!(s.position.x, 50);
    assert_eq!(s.speed.x, 100.0);
}

fn exact_y(s: &PlayerState) -> f32 {
    s.position.y as f32 + s.remainder.y
}

#[test]
fn step_rising_through_a_jump_thru_is_pushed_up() {
    let start = player_at(60, PLATFORM_Y + 3, 0.0, -50.0);
    let with = step(start, Input::default(), &platform_room());
    let without = step(start, Input::default(), &room());
    let push = exact_y(&with) - exact_y(&without);
    assert!(
        (push - JUMP_THRU_ASSIST_SPEED * DELTA_TIME).abs() < 1e-4,
        "pushed {push}"
    );
}

#[test]
fn step_no_push_when_falling_through_a_jump_thru() {
    let start = player_at(60, PLATFORM_Y + 3, 0.0, 50.0);
    let with = step(start, Input::default(), &platform_room());
    let without = step(start, Input::default(), &room());
    assert_eq!(with, without);
}

#[test]
fn step_stands_on_a_jump_thru() {
    let room = platform_room();
    let mut s = player_at(60, PLATFORM_Y, 0.0, 0.0);
    for _ in 0..10 {
        s = step(s, Input::default(), &room);
    }
    assert_eq!(
        s.position,
        IVec2 {
            x: 60,
            y: PLATFORM_Y
        }
    );
    assert_eq!(s.jump_grace_timer, JUMP_GRACE_TIME);
}

#[test]
fn step_jumps_off_a_jump_thru() {
    let s = step(player_at(60, PLATFORM_Y, 0.0, 0.0), JUMP, &platform_room());
    assert_eq!(s.speed.y, JUMP_SPEED);
}

#[test]
fn level_dash_clipping_a_jump_thru_is_lifted_onto_it() {
    let mut s = dashing_at(60, PLATFORM_Y + 4, DASH_SPEED, 0.0);
    s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
    s = step(s, Input::default(), &platform_room());
    assert_eq!(s.position.y, PLATFORM_Y);
}

#[test]
fn dash_jump_thru_nudge_reaches_six_pixels() {
    let room = platform_room();
    let mut s = player_at(60, PLATFORM_Y + 6, 0.0, 0.0);
    dash_jump_thru_nudge(&mut s, &room);
    assert_eq!(s.position.y, PLATFORM_Y);

    let mut s = player_at(60, PLATFORM_Y + 7, 0.0, 0.0);
    dash_jump_thru_nudge(&mut s, &room);
    assert_eq!(s.position.y, PLATFORM_Y + 7);
}

#[test]
fn dash_up_through_a_jump_thru_is_not_lifted() {
    let mut s = dashing_at(60, PLATFORM_Y + 4, 0.0, -DASH_SPEED);
    s.dash_dir = Vec2 { x: 0.0, y: -1.0 };
    let with = step(s, Input::default(), &platform_room());
    assert_eq!(with.position.y, PLATFORM_Y + 4 - 5);
}

const CROUCH_DASH: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    crouch_dash: true,
};

#[test]
fn dash_begin_from_the_crouch_dash_button_ducks() {
    let mut s = standing();
    dash_begin(
        &mut s,
        &room(),
        Frame {
            crouch_dash_pressed: true,
            ..frame(Input::default(), true)
        },
    );
    assert!(s.ducking);

    let mut s = at_rest();
    dash_begin(
        &mut s,
        &room(),
        Frame {
            crouch_dash_pressed: true,
            ..frame(Input::default(), false)
        },
    );
    assert!(s.ducking);
}

#[test]
fn step_crouch_dash_starts_a_ducked_dash() {
    let s = step(standing(), CROUCH_DASH, &room());
    assert_eq!(s.mode, Mode::DashStart);
    assert!(s.ducking);
    assert_eq!(s.dashes, 0);
    assert!(s.crouch_dash_held);
}

#[test]
fn step_crouch_dash_goes_the_way_she_aims() {
    let s = dash_for(
        standing(),
        5,
        Input {
            move_x: 1,
            ..CROUCH_DASH
        },
    );
    assert_eq!(
        s.speed,
        Vec2 {
            x: DASH_SPEED,
            y: 0.0
        }
    );
    assert!(s.ducking);
}

#[test]
fn step_crouch_dash_then_jump_is_a_hyper() {
    let s = step(
        dash_for(
            standing(),
            5,
            Input {
                move_x: 1,
                ..CROUCH_DASH
            },
        ),
        JUMP_RIGHT,
        &room(),
    );
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H * DUCK_SUPER_JUMP_X_MULT,
            y: JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT
        }
    );
    assert!(!s.ducking);
}

#[test]
fn step_holding_the_crouch_dash_button_does_not_dash_again() {
    let mut s = standing();
    s.crouch_dash_held = true;
    s = step(s, CROUCH_DASH, &room());
    assert_eq!(s.mode, Mode::Normal);
    assert!(!s.ducking);
}

#[test]
fn step_normal_dash_while_crouch_dash_is_held_does_not_duck() {
    let mut s = standing();
    s.crouch_dash_held = true;
    s = step(
        s,
        Input {
            dash: true,
            ..CROUCH_DASH
        },
        &room(),
    );
    assert_eq!(s.mode, Mode::DashStart);
    assert!(!s.ducking);
}

fn lifted(mut s: PlayerState, x: f32, y: f32) -> PlayerState {
    set_lift_speed(&mut s, Vec2 { x, y });
    s
}

#[test]
fn lift_boost_is_capped_and_never_downward() {
    assert_eq!(
        lift_boost(&lifted(standing(), 100.0, -60.0)),
        Vec2 { x: 100.0, y: -60.0 }
    );
    assert_eq!(
        lift_boost(&lifted(standing(), 400.0, -300.0)),
        Vec2 {
            x: LIFT_X_CAP,
            y: LIFT_Y_CAP
        }
    );
    assert_eq!(
        lift_boost(&lifted(standing(), -400.0, 80.0)),
        Vec2 {
            x: -LIFT_X_CAP,
            y: 0.0
        }
    );
    assert_eq!(lift_boost(&standing()), Vec2 { x: 0.0, y: 0.0 });
}

#[test]
fn jumps_add_lift_boost() {
    let mut s = lifted(standing(), 50.0, -70.0);
    jump(&mut s, Input::default());
    assert_eq!(
        s.speed,
        Vec2 {
            x: 50.0,
            y: JUMP_SPEED - 70.0
        }
    );
    assert_eq!(s.var_jump_speed, JUMP_SPEED - 70.0);

    let mut s = lifted(dashing_right(), 0.0, -70.0);
    super_jump(&mut s);
    assert_eq!(
        s.speed,
        Vec2 {
            x: SUPER_JUMP_H,
            y: JUMP_SPEED - 70.0
        }
    );

    let mut s = lifted(player_at(WALL_X, 100, 0.0, 0.0), 0.0, -70.0);
    wall_jump(&mut s, &room(), -1, 0);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -WALL_JUMP_H_SPEED,
            y: JUMP_SPEED - 70.0
        }
    );

    let mut s = lifted(bouncing_at(WALL_X), 0.0, -70.0);
    super_wall_jump(&mut s, -1);
    assert_eq!(
        s.speed,
        Vec2 {
            x: -SUPER_WALL_JUMP_H,
            y: SUPER_WALL_JUMP_SPEED - 70.0
        }
    );
}

#[test]
fn hyper_halves_the_lift_too() {
    let mut s = lifted(dashing_right(), 0.0, -70.0);
    s.ducking = true;
    super_jump(&mut s);
    assert_eq!(s.speed.y, (JUMP_SPEED - 70.0) * DUCK_SUPER_JUMP_Y_MULT);
}

#[test]
fn step_lift_speed_lasts_ten_frames() {
    let room = room();
    let mut s = lifted(standing(), 0.0, -70.0);
    for _ in 0..9 {
        s = step(s, Input::default(), &room);
    }
    assert_eq!(lift_boost(&s).y, -70.0);
    s = step(s, Input::default(), &room);
    assert_eq!(lift_boost(&s).y, 0.0);
}

#[test]
fn step_dash_start_adds_lift_boost_to_the_speed_it_remembers() {
    let s = step(lifted(standing(), 60.0, 0.0), DASH_RIGHT, &room());
    assert_eq!(s.before_dash_speed.x, 60.0);
}

#[test]
fn step_walking_off_a_rising_block_keeps_its_speed() {
    let mut s = lifted(player_at(60, 100, 0.0, 0.0), 0.0, -70.0);
    s.was_on_ground = true;
    s = step(s, Input::default(), &room());
    assert!(s.speed.y < -50.0);
}

#[test]
fn step_remembers_whether_she_was_on_the_ground() {
    assert!(step(standing(), Input::default(), &room()).was_on_ground);
    assert!(!step(at_rest(), Input::default(), &room()).was_on_ground);
}

#[test]
fn wall_jump_takes_lift_from_a_moving_wall_on_her_right() {
    let mut room = room();
    room.add_zip_mover(60, 80, 16, 40, 60, 0);
    room.solids[room.zip_movers[0].solid].lift_speed = Vec2 { x: 0.0, y: -90.0 };

    let mut s = player_at(54, 100, 0.0, 0.0);
    wall_jump(&mut s, &room, -1, 0);
    assert_eq!(s.speed.y, JUMP_SPEED - 90.0);
}

#[test]
fn level_dash_just_above_a_floor_snaps_down_onto_it() {
    let mut s = dashing_at(0, FLOOR_Y - 2, DASH_SPEED, 0.0);
    s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
    s.dash_attack_timer = 0.2;
    s.remainder.y = 0.25;
    s = step(s, Input::default(), &room());
    assert_eq!(s.position.y, FLOOR_Y);
    assert_eq!(s.remainder.y, 0.0);
}

#[test]
fn no_floor_snap_from_three_pixels_up_or_in_a_diagonal_dash() {
    let mut s = dashing_at(0, FLOOR_Y - 3, DASH_SPEED, 0.0);
    s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
    s.dash_attack_timer = 0.2;
    s = step(s, Input::default(), &room());
    assert_eq!(s.position.y, FLOOR_Y - 3);

    let d = 1.0 / 2f32.sqrt();
    let mut s = dashing_at(0, FLOOR_Y - 2, 0.0, 0.0);
    s.dash_dir = Vec2 { x: d, y: -d };
    s.dash_attack_timer = 0.2;
    s = step(s, Input::default(), &room());
    assert_eq!(s.position.y, FLOOR_Y - 2);
}

// the usual room with a spring on the floor, its base centred at x = 20
fn spring_room() -> Room {
    let mut room = room();
    room.springs.push(crate::entities::Spring::new(20, FLOOR_Y));
    room
}

#[test]
fn super_bounce_launches_her_straight_up() {
    let mut s = dashing_at(20, FLOOR_Y, 200.0, 50.0);
    s.dashes = 0;
    s.stamina = 20.0;
    s.dash_attack_timer = 0.2;
    super_bounce(&mut s, &spring_room(), FLOOR_Y - 6);
    assert_eq!(
        s.speed,
        Vec2 {
            x: 0.0,
            y: SUPER_BOUNCE_SPEED
        }
    );
    assert_eq!(s.var_jump_speed, SUPER_BOUNCE_SPEED);
    assert_eq!(s.var_jump_timer, SUPER_BOUNCE_VAR_JUMP_TIME);
    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.dashes, MAX_DASHES);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    assert_eq!(s.dash_attack_timer, 0.0);
    assert_eq!(s.jump_grace_timer, 0.0);
    assert!(s.auto_jump);
}

#[test]
fn super_bounce_lifts_her_to_the_springs_top_and_keeps_her_remainder() {
    let mut s = player_at(20, FLOOR_Y - 2, 0.0, 50.0);
    s.remainder.y = -0.25;
    super_bounce(&mut s, &spring_room(), FLOOR_Y - 6);
    assert_eq!(s.position.y, FLOOR_Y - 6);
    assert_eq!(s.remainder.y, -0.25);
}

#[test]
fn step_landing_on_a_spring_bounces() {
    let s = step(
        player_at(20, FLOOR_Y - 4, 0.0, 120.0),
        Input::default(),
        &spring_room(),
    );
    assert_eq!(
        s.speed,
        Vec2 {
            x: 0.0,
            y: SUPER_BOUNCE_SPEED
        }
    );
    assert_eq!(s.position.y, FLOOR_Y - 6);
    assert!(!s.dead);
}

#[test]
fn step_running_into_a_spring_bounces() {
    let room = spring_room();
    let mut s = standing();
    let right = Input {
        move_x: 1,
        ..Input::default()
    };
    let mut frames = 0;
    while s.speed.y == 0.0 && frames < 30 {
        s = step(s, right, &room);
        frames += 1;
    }
    assert_eq!(
        s.speed,
        Vec2 {
            x: 0.0,
            y: SUPER_BOUNCE_SPEED
        }
    );
}

#[test]
fn step_rising_past_a_spring_does_not_bounce() {
    let s = step(
        player_at(20, FLOOR_Y - 2, 0.0, -100.0),
        Input::default(),
        &spring_room(),
    );
    assert!(s.speed.y > SUPER_BOUNCE_SPEED);
    assert_eq!(s.var_jump_timer, 0.0);
}

#[test]
fn step_no_bounce_beside_a_spring() {
    let s = step(
        player_at(34, FLOOR_Y - 4, 0.0, 120.0),
        Input::default(),
        &spring_room(),
    );
    assert!(s.speed.y >= 0.0);
}

#[test]
fn step_spring_bounce_holds_its_height_without_jump_held() {
    let room = spring_room();
    let mut s = step(
        player_at(20, FLOOR_Y - 4, 0.0, 120.0),
        Input::default(),
        &room,
    );
    for _ in 0..5 {
        s = step(s, Input::default(), &room);
    }
    assert_eq!(s.speed.y, SUPER_BOUNCE_SPEED);
}

fn falling(speed_y: f32) -> PlayerState {
    player_at(60, 20, 0.0, speed_y)
}

#[test]
fn step_holding_down_at_full_fall_speed_falls_faster() {
    let room = room();
    let mut s = step(falling(MAX_FALL), DOWN, &room);
    assert_eq!(s.max_fall, MAX_FALL + FAST_MAX_ACCEL * DELTA_TIME);
    assert_eq!(s.speed.y, s.max_fall);

    // 2 px a frame for long enough would leave the room, so check the limit directly
    s.max_fall = FAST_MAX_FALL - 1.0;
    s.speed.y = FAST_MAX_FALL - 1.0;
    s.position.y = 20;
    s = step(s, DOWN, &room);
    assert_eq!(s.max_fall, FAST_MAX_FALL);
    assert_eq!(s.speed.y, FAST_MAX_FALL);
}

#[test]
fn step_no_fast_fall_below_full_fall_speed() {
    let s = step(falling(100.0), DOWN, &room());
    assert_eq!(s.max_fall, MAX_FALL);
}

#[test]
fn step_no_fast_fall_without_down() {
    let s = step(falling(MAX_FALL), Input::default(), &room());
    assert_eq!(s.max_fall, MAX_FALL);
    assert_eq!(s.speed.y, MAX_FALL);
}

#[test]
fn step_letting_go_of_down_slows_her_back() {
    let mut s = falling(FAST_MAX_FALL);
    s.max_fall = FAST_MAX_FALL;
    s = step(s, Input::default(), &room());
    assert_eq!(s.max_fall, FAST_MAX_FALL - FAST_MAX_ACCEL * DELTA_TIME);
    assert_eq!(s.speed.y, s.max_fall);
}

#[test]
fn set_normal_resets_the_fall_limit_only_when_leaving_another_state() {
    let mut s = falling(FAST_MAX_FALL);
    s.max_fall = FAST_MAX_FALL;
    set_normal(&mut s);
    assert_eq!(s.max_fall, FAST_MAX_FALL);

    s.mode = Mode::Dashing { timer: 0.0 };
    set_normal(&mut s);
    assert_eq!(s.max_fall, MAX_FALL);
    assert_eq!(s.mode, Mode::Normal);
}

#[test]
fn dash_end_resets_the_fall_limit() {
    let mut s = falling(0.0);
    s.max_fall = FAST_MAX_FALL;
    s.mode = Mode::Dashing { timer: 0.0 };
    s.dash_dir = Vec2 { x: 0.0, y: 1.0 };
    dash_end(&mut s);
    assert_eq!(s.max_fall, MAX_FALL);
}

#[test]
fn no_wall_slide_while_holding_down() {
    let mut s = player_at(WALL_X, 100, 0.0, 50.0);
    let max = wall_slide_max_fall(&mut s, &room(), Input { move_x: 1, ..DOWN });
    assert_eq!(max, MAX_FALL);
    assert_eq!(s.wall_slide_dir, 0);
}

#[test]
fn wall_slide_with_grab_held_and_no_direction() {
    let mut s = player_at(WALL_X, 100, 0.0, 50.0);
    let max = wall_slide_max_fall(
        &mut s,
        &room(),
        Input {
            grab: true,
            ..Input::default()
        },
    );
    assert_eq!(s.wall_slide_dir, 1);
    assert_eq!(max, WALL_SLIDE_START_MAX);
}

#[test]
fn wall_slide_stands_her_up() {
    let mut s = ducking_at(WALL_X, 100);
    s.speed.y = 50.0;
    wall_slide_max_fall(
        &mut s,
        &room(),
        Input {
            move_x: 1,
            ..Input::default()
        },
    );
    assert!(!s.ducking);
}

#[test]
fn dead_player_does_not_change() {
    let mut before = standing();
    before.dead = true;
    let after = step(
        before,
        Input {
            move_x: 1,
            jump: true,
            ..Default::default()
        },
        &room(),
    );
    assert_eq!(after, before);
}

const GRAB: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: true,
    crouch_dash: false,
};
const GRAB_UP: Input = Input { move_y: -1, ..GRAB };
const GRAB_DOWN: Input = Input { move_y: 1, ..GRAB };

// holding the wall at WALL_X, free to move along it
fn climbing(y: i32) -> PlayerState {
    let mut s = player_at(WALL_X, y, 0.0, 0.0);
    s.facing = 1;
    s.mode = Mode::Climb;
    s
}

// a wall on the right whose rows are solid only where the flag says. tiles are 8 px
fn partial_wall(solid_rows: std::ops::Range<usize>) -> Room {
    let text: String = (0..23)
        .map(|row| {
            if solid_rows.contains(&row) {
                "00000000001\n"
            } else {
                "00000000000\n"
            }
        })
        .collect();
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

#[test]
fn grabbing_a_wall_while_falling_starts_a_climb() {
    let mut s = player_at(WALL_X, 100, 50.0, 30.0);
    s.facing = 1;
    s = step(s, GRAB, &room());

    assert_eq!(s.mode, Mode::Climb);
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.speed.y, 30.0 * CLIMB_GRAB_Y_MULT);
    assert_eq!(s.climb_no_move_timer, CLIMB_NO_MOVE_TIME);
}

#[test]
fn no_grab_while_rising() {
    let mut s = player_at(WALL_X, 100, 0.0, -30.0);
    s.facing = 1;
    assert_eq!(step(s, GRAB, &room()).mode, Mode::Normal);
}

#[test]
fn no_grab_without_the_button() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    s.facing = 1;
    assert_eq!(step(s, Input::default(), &room()).mode, Mode::Normal);
}

#[test]
fn no_grab_when_tired() {
    let mut s = player_at(WALL_X, 100, 0.0, 30.0);
    s.facing = 1;
    s.stamina = CLIMB_TIRED_THRESHOLD - 1.0;
    assert_eq!(step(s, GRAB, &room()).mode, Mode::Normal);
}

#[test]
fn a_pending_wall_boost_counts_towards_not_being_tired() {
    let mut s = at_rest();
    s.stamina = CLIMB_TIRED_THRESHOLD - 1.0;
    assert!(is_tired(&s));

    s.wall_boost_timer = 0.1;
    assert!(!is_tired(&s));
}

#[test]
fn no_grab_while_moving_away_from_the_wall() {
    let mut s = player_at(WALL_X, 100, -50.0, 30.0);
    s.facing = 1;
    assert!(!try_grab(&mut s, &room(), GRAB));
}

#[test]
fn no_grab_while_ducking() {
    let mut s = ducking_at(WALL_X, 100);
    s.facing = 1;
    assert!(!try_grab(&mut s, &room(), GRAB));
}

#[test]
fn grabbing_pulls_her_in_from_one_pixel_away() {
    let mut s = player_at(WALL_X - 1, 100, 0.0, 0.0);
    s.facing = 1;
    assert!(try_grab(&mut s, &room(), GRAB));
    assert_eq!(s.position.x, WALL_X);

    let mut s = player_at(WALL_X - 2, 100, 0.0, 0.0);
    s.facing = 1;
    assert!(!try_grab(&mut s, &room(), GRAB));
}

#[test]
fn grabbing_reaches_two_pixels_up() {
    // the wall's bottom edge is y = 48. her head is 11 px above her feet
    let room = partial_wall(0..6);

    let mut s = player_at(WALL_X, 60, 0.0, 0.0);
    s.facing = 1;
    assert!(try_grab(&mut s, &room, GRAB));
    assert_eq!(s.position.y, 58);

    let mut s = player_at(WALL_X, 61, 0.0, 0.0);
    s.facing = 1;
    assert!(!try_grab(&mut s, &room, GRAB));
}

#[test]
fn no_reaching_up_while_holding_down() {
    let room = partial_wall(0..6);
    let mut s = player_at(WALL_X, 60, 0.0, 0.0);
    s.facing = 1;
    assert!(!try_grab(&mut s, &room, GRAB_DOWN));
}

#[test]
fn she_cannot_move_along_the_wall_right_after_grabbing() {
    let mut s = climbing(100);
    s.climb_no_move_timer = CLIMB_NO_MOVE_TIME;
    s = step(s, GRAB_UP, &room());

    assert_eq!(s.speed.y, 0.0);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn climbing_up_speeds_up_and_costs_stamina() {
    let mut s = step(climbing(100), GRAB_UP, &room());
    assert_eq!(s.speed.y, -CLIMB_ACCEL * DELTA_TIME);
    assert_eq!(s.last_climb_move, -1);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_UP_COST * DELTA_TIME);

    for _ in 0..10 {
        s = step(s, GRAB_UP, &room());
    }
    assert_eq!(s.speed.y, CLIMB_UP_SPEED);
}

#[test]
fn climbing_down_is_free() {
    let mut s = climbing(100);
    for _ in 0..10 {
        s = step(s, GRAB_DOWN, &room());
    }

    assert_eq!(s.speed.y, CLIMB_DOWN_SPEED);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn holding_still_costs_a_little_stamina() {
    let s = step(climbing(100), GRAB, &room());

    assert_eq!(s.position.y, 100);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_STILL_COST * DELTA_TIME);
}

#[test]
fn a_ceiling_stops_her_climbing_up() {
    let mut room = room();
    room.add_dash_block(64, 80, 16, 9);
    let s = step(climbing(100), GRAB_UP, &room);

    assert_eq!(s.speed.y, 0.0);
    assert_eq!(s.last_climb_move, 0);
}

#[test]
fn letting_go_ends_the_climb() {
    let s = step(climbing(100), Input::default(), &room());
    assert_eq!(s.mode, Mode::Normal);
}

#[test]
fn running_out_of_stamina_ends_the_climb() {
    let mut s = climbing(100);
    s.stamina = 0.1;
    assert_eq!(step(s, GRAB, &room()).mode, Mode::Normal);
}

#[test]
fn jumping_while_climbing_is_a_climb_jump() {
    let s = step(climbing(100), JUMP_GRAB, &room());

    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.speed.y, JUMP_SPEED);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
}

#[test]
fn jumping_away_while_climbing_is_a_wall_jump() {
    let away = Input {
        move_x: -1,
        ..JUMP_GRAB
    };
    let s = step(climbing(100), away, &room());

    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.speed.x, -WALL_JUMP_H_SPEED);
    assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
}

#[test]
fn dashing_out_of_a_climb() {
    let dash = Input { dash: true, ..GRAB };
    assert_eq!(step(climbing(100), dash, &room()).mode, Mode::DashStart);
}

#[test]
fn she_keeps_facing_the_wall_while_climbing() {
    let away = Input { move_x: -1, ..GRAB };
    let s = step(climbing(100), away, &room());

    assert_eq!(s.facing, 1);
    assert_eq!(s.mode, Mode::Climb);
}

#[test]
fn leaving_a_climb_drops_any_retained_wall_speed() {
    let mut s = climbing(100);
    s.wall_speed_retained = 100.0;
    s.wall_speed_retention_timer = WALL_SPEED_RETENTION_TIME;

    assert_eq!(
        step(s, Input::default(), &room()).wall_speed_retention_timer,
        0.0
    );
}

#[test]
fn slip_check_is_true_with_her_hands_above_the_wall() {
    // the wall's top edge is y = 48
    let room = partial_wall(6..23);
    assert!(slip_check(&climbing(54), &room, 0));
    assert!(!slip_check(&climbing(55), &room, 0));
    assert!(!slip_check(&climbing(100), &room, 0));
}

#[test]
fn she_slips_down_when_her_hands_are_above_the_wall() {
    let room = partial_wall(6..23);
    let mut s = climbing(54);
    s = step(s, GRAB, &room);
    s = step(s, GRAB, &room);

    assert_eq!(s.speed.y, CLIMB_SLIP_SPEED);
    assert_eq!(s.mode, Mode::Climb);
}

#[test]
fn climbing_rides_the_block_in_front_of_her() {
    let beside = Rect {
        left: 80,
        top: 80,
        width: 16,
        height: 40,
    };
    assert!(is_riding(&climbing(100), &beside));
    assert!(!is_riding(&player_at(WALL_X, 100, 0.0, 0.0), &beside));
}

#[test]
fn climbing_past_the_top_of_a_wall_hops() {
    // the wall's top edge is y = 48
    let room = partial_wall(6..23);
    let s = step(climbing(54), GRAB_UP, &room);

    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.speed.y, CLIMB_HOP_Y);
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.hop_wait_x, 1);
    assert_eq!(s.force_move_x_timer, CLIMB_HOP_FORCE_TIME);
}

#[test]
fn a_hop_carries_her_onto_the_ledge() {
    let room = partial_wall(6..23);
    let mut s = step(climbing(54), GRAB_UP, &room);

    for _ in 0..40 {
        s = step(s, Input::default(), &room);
    }

    assert!(s.position.x > WALL_X);
    assert_eq!(s.position.y, 48);
    assert_eq!(s.hop_wait_x, 0);
}

#[test]
fn a_hop_with_no_wall_left_pushes_her_sideways_at_once() {
    let room = partial_wall(0..0);
    let mut s = climbing(100);
    s.speed.y = -10.0;
    s = step(s, GRAB, &room);

    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.speed.x, CLIMB_HOP_X);
    assert_eq!(s.speed.y, CLIMB_HOP_Y);
    assert_eq!(s.hop_wait_x, 0);
}

#[test]
fn losing_the_wall_without_rising_just_lets_go() {
    let room = partial_wall(0..0);
    let s = step(climbing(100), GRAB, &room);

    assert_eq!(s.mode, Mode::Normal);
    assert_eq!(s.speed.x, 0.0);
    assert_eq!(s.force_move_x_timer, 0.0);
}

#[test]
fn falling_cancels_the_wait_for_the_sideways_push() {
    let mut s = player_at(WALL_X, 100, 0.0, 10.0);
    s.hop_wait_x = 1;
    s.hop_wait_x_speed = CLIMB_HOP_X;
    s = step(s, Input::default(), &room());

    assert_eq!(s.hop_wait_x, 0);
    assert_eq!(s.speed.x, 0.0);
}

#[test]
fn dashing_cancels_the_wait_for_the_sideways_push() {
    let mut s = player_at(WALL_X, 100, 0.0, -50.0);
    s.hop_wait_x = 1;
    s.hop_wait_x_speed = CLIMB_HOP_X;
    s = step(
        s,
        Input {
            dash: true,
            ..Input::default()
        },
        &room(),
    );

    assert_eq!(s.mode, Mode::DashStart);
    assert_eq!(s.hop_wait_x, 0);
}

#[test]
fn a_hop_follows_the_moving_block_she_is_hopping_onto() {
    let mut room = room();
    room.add_zip_mover(40, 60, 16, 16, 40, 20);

    // the block was 2 px lower when she last looked
    let mut s = player_at(20, 100, 0.0, 0.0);
    s.climb_hop_solid = Some(HopSolid {
        id: 0,
        position: IVec2 { x: 40, y: 62 },
    });
    s.force_move_x_timer = CLIMB_HOP_FORCE_TIME;
    s = step(s, Input::default(), &room);

    assert_eq!(s.position.y, 98);
    assert_eq!(s.climb_hop_solid.unwrap().position, IVec2 { x: 40, y: 60 });
}

#[test]
fn she_stops_following_the_block_when_the_hop_lock_ends() {
    let mut room = room();
    room.add_zip_mover(40, 60, 16, 16, 40, 20);

    let mut s = player_at(20, 100, 0.0, 0.0);
    s.climb_hop_solid = Some(HopSolid {
        id: 0,
        position: IVec2 { x: 40, y: 62 },
    });
    s = step(s, Input::default(), &room);

    assert_eq!(s.position.y, 100);
    assert_eq!(s.climb_hop_solid, None);
}
