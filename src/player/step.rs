use super::climb::{climb_update, follow_hop_solid, try_grab, update_hop_wait};
use super::collision::{
    can_stand, collides, enters_jump_thru, fell_out, on_ground, spikes_kill, touches_jump_thru,
    touches_spikes,
};
use super::constants::*;
use super::dash::{aim_vector, can_dash, dash_begin, dash_end, dash_launch, dashing, refill_dash};
use super::jump::{
    bounce_on_springs, can_super_dash, jump, jump_off_wall, super_dash, wall_slide_max_fall,
};
use super::movement::{
    add_liftboost, approach, dash_jump_thru_nudge, liftboost, move_exact_v, move_h, move_v,
    nudge_v, update_retention,
};
use super::state::{Frame, Input, Player, State, jump_pressed, track_buttons};
use crate::geom::Vec2;
use crate::level::room::{Room, RoomState};

// one frame of her update
pub fn step(mut player: Player, input: Input, room: &Room, state: &RoomState) -> Player {
    if player.dead {
        return player;
    }

    let mut frame = read_buttons(&mut player, input);

    // nothing but the buttons is read during a freeze
    if player.freeze_timer > 0.0 {
        player.freeze_timer = (player.freeze_timer - DELTA_TIME).max(0.0);
        return player;
    }

    frame.ground = on_ground(&player, room, state);

    run_timers(&mut player, room, frame.ground);
    apply_input(&mut player, room, state, &mut frame);
    update_state(&mut player, room, state, frame);
    settle_before_moving(&mut player, room, state, frame.ground);

    let (speed_x, speed_y) = (player.speed.x, player.speed.y);
    move_h(&mut player, room, state, speed_x * DELTA_TIME, frame.ground);
    move_v(&mut player, room, state, speed_y * DELTA_TIME);

    touch_hazards(&mut player, room, state);

    player.was_on_ground = frame.ground;
    player
}

fn tick(timer: &mut f32) {
    if *timer > 0.0 {
        *timer -= DELTA_TIME;
    }
}

// a press is a button that is down now and wasn't last frame
fn read_buttons(player: &mut Player, input: Input) -> Frame {
    let demodash_pressed = input.demodash && !player.held.demodash;

    let frame = Frame {
        input,
        jump_pressed: jump_pressed(player, input),
        dash_pressed: demodash_pressed || (input.dash && !player.held.dash),
        demodash_pressed,
        ..Frame::default()
    };

    track_buttons(player, input);
    frame
}

fn run_timers(player: &mut Player, room: &Room, ground: bool) {
    player.climb.trigger_dir = 0;

    if player.wall.slide_dir != 0 {
        player.wall.slide_timer = (player.wall.slide_timer - DELTA_TIME).max(0.0);
        player.wall.slide_dir = 0;
    }

    if player.wall.boost_timer > 0.0 {
        player.wall.boost_timer -= DELTA_TIME;

        if player.move_x == player.wall.boost_dir {
            player.speed.x = WALL_KICK_SPEED * player.move_x as f32;
            player.stamina += CLIMB_JUMP_COST;
            player.wall.boost_timer = 0.0;
        }
    }

    if ground && player.state != State::Climb {
        player.wall.slide_timer = WALL_SLIDE_TIME;
        player.stamina = CLIMB_MAX_STAMINA;
        player.auto_jump = false;
    }

    if ground {
        player.coyote_timer = COYOTE_TIME;
    } else {
        tick(&mut player.coyote_timer);
    }

    tick(&mut player.dash.attack_timer);
    tick(&mut player.dash.cooldown);

    if player.dash.refill_cooldown > 0.0 {
        player.dash.refill_cooldown -= DELTA_TIME;
    } else if ground && !touches_spikes(player, room) {
        refill_dash(player);
    }
}

// works out the direction she is really moving and facing this frame
fn apply_input(player: &mut Player, room: &Room, state: &RoomState, frame: &mut Frame) {
    let real = frame.input;

    if player.forced_move_timer > 0.0 {
        player.forced_move_timer -= DELTA_TIME;
        frame.input.move_x = player.forced_move;
    } else {
        player.climb.hop_solid = None;
    }

    follow_hop_solid(player, room, state);

    player.move_x = frame.input.move_x;

    if frame.input.move_x != 0 && player.state != State::Climb {
        player.facing = frame.input.move_x;
    }

    frame.aim = aim_vector(real, player.facing);

    update_retention(player, room, state);
    update_hop_wait(player, room, state);
    tick(&mut player.jump_timer);

    if dashing(player) && player.dash.dir.y == 0.0 {
        dash_jump_thru_nudge(player, room, state);
    }
}

// runs whichever state she is in, which may put her in another
fn update_state(player: &mut Player, room: &Room, state: &RoomState, frame: Frame) {
    let was_normal = player.state == State::Normal;

    match player.state {
        State::Normal => {
            walk_off_liftboost(player, frame.ground);

            if try_grab(player, room, state, frame.input) {
                // grabbed a wall
            } else if can_dash(player, frame.dash_pressed) {
                add_liftboost(player);
                dash_begin(player, room, state, frame);
            } else {
                normal_update(player, room, state, frame);
            }
        }

        State::Climb => {
            climb_update(player, room, state, frame);

            if player.state != State::Climb {
                player.wall.retention_timer = 0.0;
            }
        }

        State::DashStart => {
            if !jump_out_of_dash(player, room, state, frame) {
                dash_launch(player, frame.aim, frame.ground);
            }
        }

        State::Dash { timer } => {
            if jump_out_of_dash(player, room, state, frame) {
                // the jump ended the dash
            } else if timer > 0.0 {
                player.state = State::Dash {
                    timer: timer - DELTA_TIME,
                };
            } else {
                dash_end(player);
            }
        }
    }

    if was_normal && player.state != State::Normal {
        player.wall.boost_timer = 0.0;
        player.wall.retention_timer = 0.0;
        player.climb.hop_wait = 0;
    }
}

// walking off a rising block keeps its upward speed. she is moving up then, so she can't grab
fn walk_off_liftboost(player: &mut Player, ground: bool) {
    let boost = liftboost(player);
    if boost.y < 0.0 && player.was_on_ground && !ground && player.speed.y >= 0.0 {
        player.speed.y = boost.y;
    }
}

fn jump_out_of_dash(player: &mut Player, room: &Room, state: &RoomState, frame: Frame) -> bool {
    if can_super_dash(player, room, state, frame.jump_pressed) {
        super_dash(player);
        true
    } else {
        frame.jump_pressed && jump_off_wall(player, room, state, frame)
    }
}

// small adjustments the game makes after her state runs and before she moves
fn settle_before_moving(player: &mut Player, room: &Room, state: &RoomState, ground: bool) {
    // lift speed lasts a bit after the block stops moving
    player.lift.speed = NO_LIFT;
    if player.lift.timer > 0.0 {
        player.lift.timer -= DELTA_TIME;
        if player.lift.timer <= 0.0 {
            player.lift.last_speed = NO_LIFT;
        }
    }

    // rising inside a jump-through lifts her through it
    let climbing_still = player.state == State::Climb && player.climb.last_move != -1;
    if !ground && player.speed.y <= 0.0 && !climbing_still && touches_jump_thru(player, room, 0, 0)
    {
        nudge_v(player, room, state, JUMP_THRU_ASSIST_SPEED * DELTA_TIME);
    }

    // a sideways dash just above the floor snaps down onto it
    if !ground && player.dash.attack_timer > 0.0 && player.dash.dir.y == 0.0 {
        let near = DASH_V_FLOOR_SNAP_DIST;
        if collides(player, room, state, 0, near) || enters_jump_thru(player, room, 0, near) {
            move_exact_v(player, room, state, near);
        }
    }

    // she stands up once she is falling and out of coyote time
    if player.speed.y > 0.0
        && !ground
        && player.coyote_timer <= 0.0
        && can_stand(player, room, state)
    {
        player.crouching = false;
    }
}

fn touch_hazards(player: &mut Player, room: &Room, state: &RoomState) {
    let spiked = room.spikes.iter().any(|spikes| spikes_kill(player, spikes));
    if !spiked {
        bounce_on_springs(player, room, state);
    }

    if spiked || fell_out(player, room) {
        player.speed = Vec2 { x: 0.0, y: 0.0 };
        player.dead = true;
    }
}

fn normal_update(player: &mut Player, room: &Room, state: &RoomState, frame: Frame) {
    let Frame {
        input,
        ground,
        jump_pressed,
        ..
    } = frame;

    if player.crouching {
        if ground && input.move_y != 1 && can_stand(player, room, state) {
            player.crouching = false;
        }
    } else if ground && input.move_y == 1 && player.speed.y >= 0.0 {
        player.crouching = true;
    }

    let mult = if ground { GROUND_MULT } else { AIR_MULT };
    let move_x = input.move_x as f32;
    let target = MAX_RUN * move_x;

    let max_move_x =
        if player.speed.x.abs() > MAX_RUN && (player.speed.x.signum() as i8) == input.move_x {
            RUN_REDUCE * DELTA_TIME * mult
        } else {
            RUN_ACCEL * DELTA_TIME * mult
        };
    if player.crouching && ground {
        player.speed.x = approach(player.speed.x, 0.0, CROUCH_FRICTION * DELTA_TIME);
    } else {
        player.speed.x = approach(player.speed.x, target, max_move_x);
    }

    let fastfall = input.move_y == 1 && player.speed.y >= MAX_FALL;
    let fall_limit = if fastfall { FASTFALL_SPEED } else { MAX_FALL };
    player.max_fall = approach(player.max_fall, fall_limit, FASTFALL_ACCEL * DELTA_TIME);

    if !ground {
        let max_fall = wall_slide_max_fall(player, room, state, input);
        let grav_mult = if player.speed.y.abs() < HALF_GRAV_THRESHOLD
            && (input.jump_down() || player.auto_jump)
        {
            0.5
        } else {
            1.0
        };
        player.speed.y = approach(player.speed.y, max_fall, GRAVITY * grav_mult * DELTA_TIME);
    }

    if player.jump_timer > 0.0 {
        if input.jump_down() || player.auto_jump {
            player.speed.y = player.speed.y.min(player.jump_speed);
        } else {
            player.jump_timer = 0.0;
        }
    }

    if jump_pressed {
        if player.coyote_timer > 0.0 {
            jump(player, input);
        } else {
            jump_off_wall(player, room, state, frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::fixtures::*;
    use crate::player::state::set_normal;

    #[test]
    fn no_dash_refill_on_the_ground_while_touching_spikes() {
        let mut s = standing();
        s.dashes = 0;
        run_timers(&mut s, &spiked_floor(), true);
        assert_eq!(s.dashes, 0);

        run_timers(&mut s, &room(), true);
        assert_eq!(s.dashes, MAX_DASHES);
    }

    #[test]
    fn standing_on_the_ground_keeps_a_second_dash() {
        let mut s = standing();
        s.dashes = TWO_DASHES;
        run_timers(&mut s, &room(), true);
        assert_eq!(s.dashes, TWO_DASHES);
    }

    #[test]
    fn walking_off_a_rising_block_beside_a_wall_does_not_grab() {
        let room = room();
        let mut s = lifted(falling_against_wall(), 0.0, -90.0);
        s.speed.y = 0.0;
        s.was_on_ground = true;

        let s = step(s, GRAB, &room, &room.initial);
        assert_eq!(s.state, State::Normal);
        assert!(s.speed.y < 0.0);
    }

    #[test]
    fn not_on_ground_above_floor() {
        let room = room();
        let mut s = standing();
        s.position.y -= 1;
        assert!(!on_ground(&s, &room, &room.initial));
    }

    #[test]
    fn not_on_ground_while_moving_up() {
        let room = room();
        let mut s = standing();
        s.speed.y = -10.0;
        assert!(!on_ground(&s, &room, &room.initial));
    }

    #[test]
    fn wall_at_detects_wall_to_the_right_only() {
        let room = room();
        let mut s = at_rest();
        s.position.x = WALL_X;
        assert!(collides(&s, &room, &room.initial, 1, 0));
        assert!(!collides(&s, &room, &room.initial, -1, 0));
        s.position.x = WALL_X - 1;
        assert!(!collides(&s, &room, &room.initial, 1, 0));
    }

    #[test]
    fn not_on_ground_past_ledge() {
        let room = room();
        let mut s = standing();
        s.position.x = LEDGE_X;
        assert!(on_ground(&s, &room, &room.initial));
        s.position.x = LEDGE_X + 1;
        assert!(!on_ground(&s, &room, &room.initial));
    }

    #[test]
    fn frozen_frames_do_nothing() {
        let room = room();
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
            &room,
            &room.initial,
        );
        assert_eq!(s.position, before.position);
        assert_eq!(s.speed, before.speed);
        assert!(s.freeze_timer < DASH_FREEZE_TIME);
    }

    #[test]
    fn step_kills_when_she_falls_out() {
        let room = room();
        let mut s = player_at(60, 190, 0.0, 160.0);
        s = step(s, Input::default(), &room, &room.initial);
        assert!(!s.dead);
        s = step(s, Input::default(), &room, &room.initial);
        s = step(s, Input::default(), &room, &room.initial);
        assert!(s.dead);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn falling_inside_the_room_is_not_death() {
        let room = room();
        let s = step(
            player_at(60, 150, 0.0, 160.0),
            Input::default(),
            &room,
            &room.initial,
        );
        assert!(!s.dead);
    }

    #[test]
    fn step_tracks_each_button() {
        let room = room();
        let s = step(standing(), JUMP_2, &room, &room.initial);
        assert!(s.held.jump2);
        assert!(!s.held.jump);
        let s = step(s, JUMP_1, &room, &room.initial);
        assert!(s.held.jump);
        assert!(!s.held.jump2);
    }

    #[test]
    fn step_refills_stamina_on_the_ground() {
        let room = room();
        let mut s = standing();
        s.stamina = 10.0;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_does_not_refill_stamina_in_the_air() {
        let room = room();
        let mut s = at_rest();
        s.stamina = 10.0;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.stamina, 10.0);
    }

    #[test]
    fn step_gives_retained_speed_back_before_moving() {
        let room = room();
        let s = step(
            retaining(WALL_X - 20, 0.0, 120.0),
            Input {
                move_x: 1,
                ..Default::default()
            },
            &room,
            &room.initial,
        );
        assert!(s.speed.x > 100.0);
        assert!(s.position.x > WALL_X - 20);
        assert_eq!(s.wall.retention_timer, 0.0);
    }

    #[test]
    fn step_remembers_the_direction_held() {
        let room = room();
        assert_eq!(step(at_rest(), LEFT, &room, &room.initial).move_x, -1);
        assert_eq!(
            step(at_rest(), Input::default(), &room, &room.initial).move_x,
            0
        );
    }

    #[test]
    fn step_forced_direction_runs_out() {
        let room = room();
        let mut s = player_at(40, 100, 0.0, -100.0);
        s.forced_move = -1;
        s.forced_move_timer = 0.01;
        let right = Input {
            move_x: 1,
            ..Input::default()
        };
        s = step(s, right, &room, &room.initial);
        assert_eq!(s.facing, -1);
        s = step(s, right, &room, &room.initial);
        assert_eq!(s.facing, 1);
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
        assert!(!s.held.jump);
        assert!(s.held.jump2);
        assert!(s.held.dash);

        track_buttons(&mut s, Input::default());
        assert!(!s.held.jump2);
        assert!(!s.held.dash);
    }

    #[test]
    fn step_walking_off_a_rising_block_keeps_its_speed() {
        let room = room();
        let mut s = lifted(player_at(60, 100, 0.0, 0.0), 0.0, -70.0);
        s.was_on_ground = true;
        s = step(s, Input::default(), &room, &room.initial);
        assert!(s.speed.y < -50.0);
    }

    #[test]
    fn step_remembers_whether_she_was_on_the_ground() {
        let room = room();
        assert!(step(standing(), Input::default(), &room, &room.initial).was_on_ground);
        assert!(!step(at_rest(), Input::default(), &room, &room.initial).was_on_ground);
    }

    #[test]
    fn step_holding_down_at_full_fall_speed_falls_faster() {
        let room = room();
        let mut s = step(falling(MAX_FALL), DOWN, &room, &room.initial);
        assert_eq!(s.max_fall, MAX_FALL + FASTFALL_ACCEL * DELTA_TIME);
        assert_eq!(s.speed.y, s.max_fall);

        // 2 px a frame for long enough would leave the room, so check the limit directly
        s.max_fall = FASTFALL_SPEED - 1.0;
        s.speed.y = FASTFALL_SPEED - 1.0;
        s.position.y = 20;
        s = step(s, DOWN, &room, &room.initial);
        assert_eq!(s.max_fall, FASTFALL_SPEED);
        assert_eq!(s.speed.y, FASTFALL_SPEED);
    }

    #[test]
    fn step_no_fastfall_below_full_fall_speed() {
        let room = room();
        let s = step(falling(100.0), DOWN, &room, &room.initial);
        assert_eq!(s.max_fall, MAX_FALL);
    }

    #[test]
    fn step_no_fastfall_without_down() {
        let room = room();
        let s = step(falling(MAX_FALL), Input::default(), &room, &room.initial);
        assert_eq!(s.max_fall, MAX_FALL);
        assert_eq!(s.speed.y, MAX_FALL);
    }

    #[test]
    fn step_letting_go_of_down_slows_her_back() {
        let room = room();
        let mut s = falling(FASTFALL_SPEED);
        s.max_fall = FASTFALL_SPEED;
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.max_fall, FASTFALL_SPEED - FASTFALL_ACCEL * DELTA_TIME);
        assert_eq!(s.speed.y, s.max_fall);
    }

    #[test]
    fn set_normal_resets_the_fall_limit_only_when_leaving_another_state() {
        let mut s = falling(FASTFALL_SPEED);
        s.max_fall = FASTFALL_SPEED;
        set_normal(&mut s);
        assert_eq!(s.max_fall, FASTFALL_SPEED);

        s.state = State::Dash { timer: 0.0 };
        set_normal(&mut s);
        assert_eq!(s.max_fall, MAX_FALL);
        assert_eq!(s.state, State::Normal);
    }

    #[test]
    fn dead_player_does_not_change() {
        let room = room();
        let mut before = standing();
        before.dead = true;
        let after = step(
            before,
            Input {
                move_x: 1,
                jump: true,
                ..Default::default()
            },
            &room,
            &room.initial,
        );
        assert_eq!(after, before);
    }

    #[test]
    fn holding_still_costs_a_little_stamina() {
        let room = room();
        let s = step(climbing(100), GRAB, &room, &room.initial);

        assert_eq!(s.position.y, 100);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_STILL_COST * DELTA_TIME);
    }

    #[test]
    fn losing_the_wall_without_rising_just_lets_go() {
        let room = partial_wall(0..0);
        let s = step(climbing(100), GRAB, &room, &room.initial);

        assert_eq!(s.state, State::Normal);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.forced_move_timer, 0.0);
    }

    #[test]
    fn falling_cancels_the_wait_for_the_sideways_push() {
        let room = room();
        let mut s = player_at(WALL_X, 100, 0.0, 10.0);
        s.climb.hop_wait = 1;
        s.climb.hop_speed = CLIMBHOP_X;
        s = step(s, Input::default(), &room, &room.initial);

        assert_eq!(s.climb.hop_wait, 0);
        assert_eq!(s.speed.x, 0.0);
    }
}
