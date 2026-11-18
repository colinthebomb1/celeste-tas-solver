use super::*;

pub fn is_tired(state: &PlayerState) -> bool {
    let stamina = if state.wall_boost_timer > 0.0 {
        state.stamina + CLIMB_JUMP_COST
    } else {
        state.stamina
    };

    stamina < CLIMB_TIRED_THRESHOLD
}

pub fn climb_check(state: &PlayerState, room: &Room, dir: i8, dy: i32) -> bool {
    let reach = dir as i32 * CLIMB_CHECK_DIST;
    let her = hitbox(state, reach, 0);
    let in_bounds = her.left >= room.bounds.left && her.right() < room.bounds.right();

    in_bounds && collides(state, room, reach, dy)
}

pub fn try_grab(state: &mut PlayerState, room: &Room, input: Input) -> bool {
    if !input.grab || is_tired(state) || state.ducking {
        return false;
    }
    if state.speed.y < 0.0 || sign(state.speed.x) == -(state.facing as f32) {
        return false;
    }

    if climb_check(state, room, state.facing, 0) {
        climb_begin(state, room);
        return true;
    }

    if input.move_y < 1 {
        for up in 1..=CLIMB_UP_CHECK_DIST {
            if !collides(state, room, 0, -up) && climb_check(state, room, state.facing, -up) {
                move_exact_v(state, room, -up);
                climb_begin(state, room);
                return true;
            }
        }
    }

    false
}

pub fn climb_begin(state: &mut PlayerState, room: &Room) {
    state.ducking = false;
    state.auto_jump = false;
    state.speed.x = 0.0;
    state.speed.y *= CLIMB_GRAB_Y_MULT;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.climb_no_move_timer = CLIMB_NO_MOVE_TIME;
    state.wall_boost_timer = 0.0;
    state.last_climb_move = 0;

    // pull her against the wall
    for _ in 0..CLIMB_CHECK_DIST {
        if collides(state, room, state.facing as i32, 0) {
            break;
        }
        state.position.x += state.facing as i32;
    }

    state.mode = Mode::Climb;
}

pub(crate) fn moving_solid_at(room: &Room, rect: &Rect) -> Option<HopSolid> {
    let zip = room
        .zip_movers
        .iter()
        .position(|zip| zip.collidable && zip.rect().overlaps(rect));
    let falling = room
        .falling_blocks
        .iter()
        .position(|block| block.collidable && block.rect().overlaps(rect));

    if let Some(index) = zip {
        Some(HopSolid {
            kind: MovingSolid::ZipMover,
            index,
            position: room.zip_movers[index].position,
        })
    } else {
        falling.map(|index| HopSolid {
            kind: MovingSolid::FallingBlock,
            index,
            position: room.falling_blocks[index].position,
        })
    }
}

pub(crate) fn moving_solid_position(room: &Room, solid: HopSolid) -> Option<IVec2> {
    let (position, collidable) = match solid.kind {
        MovingSolid::ZipMover => (
            room.zip_movers[solid.index].position,
            room.zip_movers[solid.index].collidable,
        ),
        MovingSolid::FallingBlock => (
            room.falling_blocks[solid.index].position,
            room.falling_blocks[solid.index].collidable,
        ),
    };

    collidable.then_some(position)
}

// she pops up over the ledge. the sideways push waits until the wall is out of her way
pub fn climb_hop(state: &mut PlayerState, room: &Room) {
    if collides(state, room, state.facing as i32, 0) {
        state.climb_hop_solid = moving_solid_at(room, &hitbox(state, state.facing as i32, 0));
        state.hop_wait_x = state.facing;
        state.hop_wait_x_speed = state.facing as f32 * CLIMB_HOP_X;
    } else {
        state.climb_hop_solid = None;
        state.hop_wait_x = 0;
        state.speed.x = state.facing as f32 * CLIMB_HOP_X;
    }

    state.speed.y = state.speed.y.min(CLIMB_HOP_Y);
    state.force_move_x = 0;
    state.force_move_x_timer = CLIMB_HOP_FORCE_TIME;
    set_normal(state);
}

// keeps her beside a moving block while she hops onto it
pub(crate) fn follow_hop_solid(state: &mut PlayerState, room: &Room) {
    let Some(solid) = state.climb_hop_solid else {
        return;
    };

    match moving_solid_position(room, solid) {
        None => state.climb_hop_solid = None,
        Some(position) if position != solid.position => {
            state.climb_hop_solid = Some(HopSolid { position, ..solid });
            move_exact_h(state, room, position.x - solid.position.x);
            move_exact_v(state, room, position.y - solid.position.y);
        }
        Some(_) => {}
    }
}

pub(crate) fn update_hop_wait(state: &mut PlayerState, room: &Room) {
    if state.hop_wait_x == 0 {
        return;
    }

    if sign(state.speed.x) == -(state.hop_wait_x as f32) || state.speed.y > 0.0 {
        state.hop_wait_x = 0;
    } else if !collides(state, room, state.hop_wait_x as i32, 0) {
        state.speed.x = state.hop_wait_x_speed;
        state.hop_wait_x = 0;
    }
}

// her hands are above the top of the wall
pub fn slip_check(state: &PlayerState, room: &Room, add_y: i32) -> bool {
    let her = hitbox(state, 0, 0);
    let x = if state.facing == 1 {
        her.right()
    } else {
        her.left - 1
    };
    let y = her.top + 4 + add_y;

    !room.solid_at(x, y) && !room.solid_at(x, y - 4 + add_y)
}

pub(crate) fn climb_update(
    state: &mut PlayerState,
    room: &Room,
    input: Input,
    ground: bool,
    jump_pressed: bool,
    dash_pressed: bool,
    crouch: bool,
) {
    state.climb_no_move_timer -= DELTA_TIME;

    if ground {
        state.stamina = CLIMB_MAX_STAMINA;
    }

    if jump_pressed && can_unduck(state, room) {
        if input.move_x == -state.facing {
            wall_jump(state, room, -state.facing, input.move_x);
        } else {
            climb_jump(state, input, ground);
        }
        return;
    }

    if can_dash(state, dash_pressed) {
        add_lift_boost(state);
        dash_begin(state, room, input, ground, crouch);
        return;
    }

    if !input.grab {
        add_lift_boost(state);
        set_normal(state);
        return;
    }

    if !collides(state, room, state.facing as i32, 0) {
        if state.speed.y < 0.0 {
            climb_hop(state, room);
        } else {
            set_normal(state);
        }
        return;
    }

    let can_move = state.climb_no_move_timer <= 0.0;
    let mut target = 0.0;
    let mut may_slip = !can_move || input.move_y == 0;

    if can_move && input.move_y == -1 {
        let hop_blocked = collides(state, room, 0, -CLIMB_HOP_BLOCK_DIST);

        if collides(state, room, 0, -1) || (hop_blocked && slip_check(state, room, -1)) {
            state.speed.y = state.speed.y.max(0.0);
            may_slip = true;
        } else if slip_check(state, room, 0) {
            climb_hop(state, room);
            return;
        } else {
            target = CLIMB_UP_SPEED;
        }
    } else if can_move && input.move_y == 1 {
        if ground {
            state.speed.y = state.speed.y.min(0.0);
        } else {
            target = CLIMB_DOWN_SPEED;
        }
    }

    state.last_climb_move = sign(target) as i8;

    if may_slip && slip_check(state, room, 0) {
        target = CLIMB_SLIP_SPEED;
    }
    state.speed.y = approach(state.speed.y, target, CLIMB_ACCEL * DELTA_TIME);

    // nothing to slide down
    if input.move_y != 1 && state.speed.y > 0.0 && !collides(state, room, state.facing as i32, 1) {
        state.speed.y = 0.0;
    }

    if can_move {
        if state.last_climb_move == -1 {
            state.stamina -= CLIMB_UP_COST * DELTA_TIME;
        } else if state.last_climb_move == 0 {
            state.stamina -= CLIMB_STILL_COST * DELTA_TIME;
        }
    }

    if state.stamina <= 0.0 {
        add_lift_boost(state);
        set_normal(state);
    }
}
