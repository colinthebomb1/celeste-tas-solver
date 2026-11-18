use super::*;

// one frame of her update
pub fn step(mut state: PlayerState, input: Input, room: &Room) -> PlayerState {
    if state.dead {
        return state;
    }

    let mut frame = read_buttons(&mut state, input);

    // nothing but the buttons is read during a freeze
    if state.freeze_timer > 0.0 {
        state.freeze_timer = (state.freeze_timer - DELTA_TIME).max(0.0);
        return state;
    }

    frame.ground = on_ground(&state, room);

    run_timers(&mut state, frame.ground);
    apply_input(&mut state, room, &mut frame);
    update_mode(&mut state, room, frame);
    settle_before_moving(&mut state, room, frame.ground);

    let (speed_x, speed_y) = (state.speed.x, state.speed.y);
    move_h(&mut state, room, speed_x * DELTA_TIME, frame.ground);
    move_v(&mut state, room, speed_y * DELTA_TIME);

    touch_hazards(&mut state, room);

    state.was_on_ground = frame.ground;
    state
}

fn tick(timer: &mut f32) {
    if *timer > 0.0 {
        *timer -= DELTA_TIME;
    }
}

// a press is a button that is down now and wasn't last frame
fn read_buttons(state: &mut PlayerState, input: Input) -> Frame {
    let crouch_dash_pressed = input.crouch_dash && !state.crouch_dash_held;

    let frame = Frame {
        input,
        jump_pressed: jump_pressed(state, input),
        dash_pressed: crouch_dash_pressed || (input.dash && !state.dash_held),
        crouch_dash_pressed,
        ..Frame::default()
    };

    track_buttons(state, input);
    frame
}

fn run_timers(state: &mut PlayerState, ground: bool) {
    state.climb_trigger_dir = 0;

    if state.wall_slide_dir != 0 {
        state.wall_slide_timer = (state.wall_slide_timer - DELTA_TIME).max(0.0);
        state.wall_slide_dir = 0;
    }

    if state.wall_boost_timer > 0.0 {
        state.wall_boost_timer -= DELTA_TIME;

        if state.move_x == state.wall_boost_dir {
            state.speed.x = WALL_JUMP_H_SPEED * state.move_x as f32;
            state.stamina += CLIMB_JUMP_COST;
            state.wall_boost_timer = 0.0;
        }
    }

    if ground && state.mode != Mode::Climb {
        state.wall_slide_timer = WALL_SLIDE_TIME;
        state.stamina = CLIMB_MAX_STAMINA;
        state.auto_jump = false;
    }

    if ground {
        state.jump_grace_timer = JUMP_GRACE_TIME;
    } else {
        tick(&mut state.jump_grace_timer);
    }

    tick(&mut state.dash_attack_timer);
    tick(&mut state.dash_cooldown_timer);

    if state.dash_refill_cooldown_timer > 0.0 {
        state.dash_refill_cooldown_timer -= DELTA_TIME;
    } else if ground {
        state.dashes = MAX_DASHES;
    }
}

// works out the direction she is really moving and facing this frame
fn apply_input(state: &mut PlayerState, room: &Room, frame: &mut Frame) {
    let real = frame.input;

    if state.force_move_x_timer > 0.0 {
        state.force_move_x_timer -= DELTA_TIME;
        frame.input.move_x = state.force_move_x;
    } else {
        state.climb_hop_solid = None;
    }

    follow_hop_solid(state, room);

    state.move_x = frame.input.move_x;

    if frame.input.move_x != 0 && state.mode != Mode::Climb {
        state.facing = frame.input.move_x;
    }

    frame.aim = aim_vector(real, state.facing);

    update_wall_speed_retention(state, room);
    update_hop_wait(state, room);
    tick(&mut state.var_jump_timer);

    if dashing(state) && state.dash_dir.y == 0.0 {
        dash_jump_thru_nudge(state, room);
    }
}

// runs whichever state she is in, which may put her in another
fn update_mode(state: &mut PlayerState, room: &Room, frame: Frame) {
    let was_normal = state.mode == Mode::Normal;

    match state.mode {
        Mode::Normal => {
            if try_grab(state, room, frame.input) {
                // grabbed a wall
            } else if can_dash(state, frame.dash_pressed) {
                add_lift_boost(state);
                dash_begin(state, room, frame);
            } else {
                normal_update(state, room, frame);
            }
        }

        Mode::Climb => {
            climb_update(state, room, frame);

            if state.mode != Mode::Climb {
                state.wall_speed_retention_timer = 0.0;
            }
        }

        Mode::DashStart => {
            if !jump_out_of_dash(state, room, frame) {
                dash_launch(state, frame.aim, frame.ground);
            }
        }

        Mode::Dashing { timer } => {
            if jump_out_of_dash(state, room, frame) {
                // the jump ended the dash
            } else if timer > 0.0 {
                state.mode = Mode::Dashing {
                    timer: timer - DELTA_TIME,
                };
            } else {
                dash_end(state);
            }
        }
    }

    if was_normal && state.mode != Mode::Normal {
        state.wall_boost_timer = 0.0;
        state.wall_speed_retention_timer = 0.0;
        state.hop_wait_x = 0;
    }
}

fn jump_out_of_dash(state: &mut PlayerState, room: &Room, frame: Frame) -> bool {
    if can_super_jump(state, room, frame.jump_pressed) {
        super_jump(state);
        true
    } else {
        frame.jump_pressed && jump_off_wall(state, room, frame)
    }
}

// small adjustments the game makes after her state runs and before she moves
fn settle_before_moving(state: &mut PlayerState, room: &Room, ground: bool) {
    // lift speed lasts a bit after the block stops moving
    state.lift_speed = NO_LIFT;
    if state.lift_speed_timer > 0.0 {
        state.lift_speed_timer -= DELTA_TIME;
        if state.lift_speed_timer <= 0.0 {
            state.last_lift_speed = NO_LIFT;
        }
    }

    // rising inside a jump-through lifts her through it
    let climbing_still = state.mode == Mode::Climb && state.last_climb_move != -1;
    if !ground && state.speed.y <= 0.0 && !climbing_still && touches_jump_thru(state, room, 0, 0) {
        nudge_v(state, room, JUMP_THRU_ASSIST_SPEED * DELTA_TIME);
    }

    // a sideways dash just above the floor snaps down onto it
    if !ground && state.dash_attack_timer > 0.0 && state.dash_dir.y == 0.0 {
        let near = DASH_V_FLOOR_SNAP_DIST;
        if collides(state, room, 0, near) || enters_jump_thru(state, room, 0, near) {
            move_exact_v(state, room, near);
        }
    }

    // she stands up once she is falling and out of coyote time
    if state.speed.y > 0.0 && !ground && state.jump_grace_timer <= 0.0 && can_unduck(state, room) {
        state.ducking = false;
    }
}

fn touch_hazards(state: &mut PlayerState, room: &Room) {
    let spiked = room.spikes.iter().any(|spikes| spikes_kill(state, spikes));
    if !spiked {
        bounce_on_springs(state, room);
    }

    if spiked || fell_out(state, room) {
        state.speed = Vec2 { x: 0.0, y: 0.0 };
        state.dead = true;
    }
}

fn normal_update(state: &mut PlayerState, room: &Room, frame: Frame) {
    let Frame {
        input,
        ground,
        jump_pressed,
        ..
    } = frame;

    // walking off a rising block keeps its upward speed
    let boost = lift_boost(state);
    if boost.y < 0.0 && state.was_on_ground && !ground && state.speed.y >= 0.0 {
        state.speed.y = boost.y;
    }

    if state.ducking {
        if ground && input.move_y != 1 && can_unduck(state, room) {
            state.ducking = false;
        }
    } else if ground && input.move_y == 1 && state.speed.y >= 0.0 {
        state.ducking = true;
    }

    let mult = if ground { GROUND_MULT } else { AIR_MULT };
    let move_x = input.move_x as f32;
    let target = MAX_RUN * move_x;

    let max_move_x =
        if state.speed.x.abs() > MAX_RUN && (state.speed.x.signum() as i8) == input.move_x {
            RUN_REDUCE * DELTA_TIME * mult
        } else {
            RUN_ACCEL * DELTA_TIME * mult
        };
    if state.ducking && ground {
        state.speed.x = approach(state.speed.x, 0.0, DUCK_FRICTION * DELTA_TIME);
    } else {
        state.speed.x = approach(state.speed.x, target, max_move_x);
    }

    let fast_fall = input.move_y == 1 && state.speed.y >= MAX_FALL;
    let fall_limit = if fast_fall { FAST_MAX_FALL } else { MAX_FALL };
    state.max_fall = approach(state.max_fall, fall_limit, FAST_MAX_ACCEL * DELTA_TIME);

    if !ground {
        let max_fall = wall_slide_max_fall(state, room, input);
        let grav_mult = if state.speed.y.abs() < HALF_GRAV_THRESHOLD
            && (input.jump_down() || state.auto_jump)
        {
            0.5
        } else {
            1.0
        };
        state.speed.y = approach(state.speed.y, max_fall, GRAVITY * grav_mult * DELTA_TIME);
    }

    if state.var_jump_timer > 0.0 {
        if input.jump_down() || state.auto_jump {
            state.speed.y = state.speed.y.min(state.var_jump_speed);
        } else {
            state.var_jump_timer = 0.0;
        }
    }

    if jump_pressed {
        if state.jump_grace_timer > 0.0 {
            jump(state, input);
        } else {
            jump_off_wall(state, room, frame);
        }
    }
}
