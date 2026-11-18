use super::*;

// one frame of her update
pub fn step(mut player: Player, input: Input, room: &Room) -> Player {
    if player.dead {
        return player;
    }

    let mut frame = read_buttons(&mut player, input);

    // nothing but the buttons is read during a freeze
    if player.freeze_timer > 0.0 {
        player.freeze_timer = (player.freeze_timer - DELTA_TIME).max(0.0);
        return player;
    }

    frame.ground = on_ground(&player, room);

    run_timers(&mut player, frame.ground);
    apply_input(&mut player, room, &mut frame);
    update_state(&mut player, room, frame);
    settle_before_moving(&mut player, room, frame.ground);

    let (speed_x, speed_y) = (player.speed.x, player.speed.y);
    move_h(&mut player, room, speed_x * DELTA_TIME, frame.ground);
    move_v(&mut player, room, speed_y * DELTA_TIME);

    touch_hazards(&mut player, room);

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

fn run_timers(player: &mut Player, ground: bool) {
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
    } else if ground {
        player.dashes = MAX_DASHES;
    }
}

// works out the direction she is really moving and facing this frame
fn apply_input(player: &mut Player, room: &Room, frame: &mut Frame) {
    let real = frame.input;

    if player.forced_move_timer > 0.0 {
        player.forced_move_timer -= DELTA_TIME;
        frame.input.move_x = player.forced_move;
    } else {
        player.climb.hop_solid = None;
    }

    follow_hop_solid(player, room);

    player.move_x = frame.input.move_x;

    if frame.input.move_x != 0 && player.state != State::Climb {
        player.facing = frame.input.move_x;
    }

    frame.aim = aim_vector(real, player.facing);

    update_retention(player, room);
    update_hop_wait(player, room);
    tick(&mut player.jump_timer);

    if dashing(player) && player.dash.dir.y == 0.0 {
        dash_jump_thru_nudge(player, room);
    }
}

// runs whichever state she is in, which may put her in another
fn update_state(player: &mut Player, room: &Room, frame: Frame) {
    let was_normal = player.state == State::Normal;

    match player.state {
        State::Normal => {
            if try_grab(player, room, frame.input) {
                // grabbed a wall
            } else if can_dash(player, frame.dash_pressed) {
                add_liftboost(player);
                dash_begin(player, room, frame);
            } else {
                normal_update(player, room, frame);
            }
        }

        State::Climb => {
            climb_update(player, room, frame);

            if player.state != State::Climb {
                player.wall.retention_timer = 0.0;
            }
        }

        State::DashStart => {
            if !jump_out_of_dash(player, room, frame) {
                dash_launch(player, frame.aim, frame.ground);
            }
        }

        State::Dash { timer } => {
            if jump_out_of_dash(player, room, frame) {
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

fn jump_out_of_dash(player: &mut Player, room: &Room, frame: Frame) -> bool {
    if can_super_dash(player, room, frame.jump_pressed) {
        super_dash(player);
        true
    } else {
        frame.jump_pressed && jump_off_wall(player, room, frame)
    }
}

// small adjustments the game makes after her state runs and before she moves
fn settle_before_moving(player: &mut Player, room: &Room, ground: bool) {
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
        nudge_v(player, room, JUMP_THRU_ASSIST_SPEED * DELTA_TIME);
    }

    // a sideways dash just above the floor snaps down onto it
    if !ground && player.dash.attack_timer > 0.0 && player.dash.dir.y == 0.0 {
        let near = DASH_V_FLOOR_SNAP_DIST;
        if collides(player, room, 0, near) || enters_jump_thru(player, room, 0, near) {
            move_exact_v(player, room, near);
        }
    }

    // she stands up once she is falling and out of coyote time
    if player.speed.y > 0.0 && !ground && player.coyote_timer <= 0.0 && can_stand(player, room) {
        player.crouching = false;
    }
}

fn touch_hazards(player: &mut Player, room: &Room) {
    let spiked = room.spikes.iter().any(|spikes| spikes_kill(player, spikes));
    if !spiked {
        bounce_on_springs(player, room);
    }

    if spiked || fell_out(player, room) {
        player.speed = Vec2 { x: 0.0, y: 0.0 };
        player.dead = true;
    }
}

fn normal_update(player: &mut Player, room: &Room, frame: Frame) {
    let Frame {
        input,
        ground,
        jump_pressed,
        ..
    } = frame;

    // walking off a rising block keeps its upward speed
    let boost = liftboost(player);
    if boost.y < 0.0 && player.was_on_ground && !ground && player.speed.y >= 0.0 {
        player.speed.y = boost.y;
    }

    if player.crouching {
        if ground && input.move_y != 1 && can_stand(player, room) {
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
        let max_fall = wall_slide_max_fall(player, room, input);
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
            jump_off_wall(player, room, frame);
        }
    }
}
