use super::*;

pub fn step(mut state: PlayerState, input: Input, room: &Room) -> PlayerState {
    if state.dead {
        return state;
    }

    let jump_pressed = jump_pressed(&state, input);
    let dash_pressed = input.dash && !state.dash_held;
    let crouch_dash_pressed = input.crouch_dash && !state.crouch_dash_held;
    track_buttons(&mut state, input);

    if state.freeze_timer > 0.0 {
        state.freeze_timer = (state.freeze_timer - DELTA_TIME).max(0.0);
        return state;
    }

    let ground = on_ground(&state, room);
    state.climb_trigger_dir = 0;

    if state.wall_slide_dir != 0 {
        state.wall_slide_timer = (state.wall_slide_timer - DELTA_TIME).max(0.0);
        state.wall_slide_dir = 0;
    }

    // wall boost
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
    } else if state.jump_grace_timer > 0.0 {
        state.jump_grace_timer -= DELTA_TIME;
    }

    if state.dash_attack_timer > 0.0 {
        state.dash_attack_timer -= DELTA_TIME;
    }

    if state.dash_cooldown_timer > 0.0 {
        state.dash_cooldown_timer -= DELTA_TIME;
    }

    if state.dash_refill_cooldown_timer > 0.0 {
        state.dash_refill_cooldown_timer -= DELTA_TIME;
    } else if ground {
        state.dashes = MAX_DASHES;
    }

    // a wall jump overrides left and right for a while. aiming a dash still uses the real input
    let aim_input = input;
    let input = if state.force_move_x_timer > 0.0 {
        state.force_move_x_timer -= DELTA_TIME;
        Input { move_x: state.force_move_x, ..input }
    } else {
        state.climb_hop_solid = None;
        input
    };

    follow_hop_solid(&mut state, room);

    state.move_x = input.move_x;

    if input.move_x != 0 && state.mode != Mode::Climb {
        state.facing = input.move_x;
    }
    
    let aim = aim_vector(aim_input, state.facing);

    update_wall_speed_retention(&mut state, room);
    update_hop_wait(&mut state, room);

    if state.var_jump_timer > 0.0 {
        state.var_jump_timer -= DELTA_TIME;
    }

    if dashing(&state) && state.dash_dir.y == 0.0 {
        dash_jump_thru_nudge(&mut state, room);
    }

    let was_normal = state.mode == Mode::Normal;

    match state.mode {
        Mode::Normal => {
            if try_grab(&mut state, room, input) {
                // grabbed a wall
            } else if can_dash(&state, dash_pressed || crouch_dash_pressed) {
                add_lift_boost(&mut state);
                dash_begin(&mut state, room, input, ground, crouch_dash_pressed);
            } else {
                normal_update(&mut state, room, input, ground, jump_pressed);
            }
        }

        Mode::Climb => {
            climb_update(&mut state, room, input, ground, jump_pressed, dash_pressed || crouch_dash_pressed, crouch_dash_pressed);

            if state.mode != Mode::Climb {
                state.wall_speed_retention_timer = 0.0;
            }
        }

        Mode::DashStart => {
            if can_super_jump(&state, room, jump_pressed) {
                super_jump(&mut state);
            } else if jump_pressed && jump_off_wall(&mut state, room, input, ground) {
                // jumped out of the dash
            } else {
                dash_launch(&mut state, aim, ground);
            }
        }
        
        Mode::Dashing { timer } => {
            if can_super_jump(&state, room, jump_pressed) {
                super_jump(&mut state);
            } else if jump_pressed && jump_off_wall(&mut state, room, input, ground) {
                // jumped out of the dash
            } else if timer > 0.0 {
                state.mode = Mode::Dashing { timer: timer - DELTA_TIME };
            } else {
                dash_end(&mut state);
            }
        }
    }

    if was_normal && state.mode != Mode::Normal {
        state.wall_boost_timer = 0.0;
        state.wall_speed_retention_timer = 0.0;
        state.hop_wait_x = 0;
    }

    // lift speed lasts a bit after the block stops moving
    state.lift_speed = NO_LIFT;
    if state.lift_speed_timer > 0.0 {
        state.lift_speed_timer -= DELTA_TIME;
        if state.lift_speed_timer <= 0.0 {
            state.last_lift_speed = NO_LIFT;
        }
    }

    let climbing_still = state.mode == Mode::Climb && state.last_climb_move != -1;
    if !ground && state.speed.y <= 0.0 && !climbing_still && touches_jump_thru(&state, room, 0, 0) {
        nudge_v(&mut state, room, JUMP_THRU_ASSIST_SPEED * DELTA_TIME);
    }

    if !ground && state.dash_attack_timer > 0.0 && state.dash_dir.y == 0.0 {
        let near = DASH_V_FLOOR_SNAP_DIST;
        if collides(&state, room, 0, near) || enters_jump_thru(&state, room, 0, near) {
            move_exact_v(&mut state, room, near);
        }
    }

    if state.speed.y > 0.0 && !ground && state.jump_grace_timer <= 0.0 && can_unduck(&state, room) {
        state.ducking = false;
    }

    let (speed_x, speed_y) = (state.speed.x, state.speed.y);
    move_h(&mut state, room, speed_x * DELTA_TIME, ground);
    move_v(&mut state, room, speed_y * DELTA_TIME);

    let spiked = room.spikes.iter().any(|s| spikes_kill(&state, s));
    if !spiked {
        bounce_on_springs(&mut state, room);
    }

    if spiked || fell_out(&state, room) {
        state.speed = Vec2 { x: 0.0, y: 0.0 };
        state.dead = true;
    }

    state.was_on_ground = ground;

    state
}

pub(crate) fn normal_update(state: &mut PlayerState, room: &Room, input: Input, ground: bool, jump_pressed: bool) {
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

    let mult = if ground {
        GROUND_MULT
    } else {
        AIR_MULT
    };
    let move_x = input.move_x as f32;
    let target = MAX_RUN * move_x;

    let max_move_x = if state.speed.x.abs() > MAX_RUN && (state.speed.x.signum() as i8) == input.move_x {
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
        let grav_mult = if state.speed.y.abs() < HALF_GRAV_THRESHOLD && (input.jump_down() || state.auto_jump) {
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
            jump_off_wall(state, room, input, ground);
        }
    }
}
