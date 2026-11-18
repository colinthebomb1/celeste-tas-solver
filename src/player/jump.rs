use super::*;

pub fn wall_slide_max_fall(state: &mut PlayerState, room: &Room, input: Input) -> f32 {
    let mut max = state.max_fall;
    let into_wall = input.move_x == state.facing || (input.move_x == 0 && input.grab);

    if into_wall && input.move_y != 1 {
        if state.speed.y >= 0.0
            && state.wall_slide_timer > 0.0
            && collides(state, room, state.facing as i32, 0)
            && can_unduck(state, room)
        {
            state.ducking = false;
            state.wall_slide_dir = state.facing;
        }

        if state.wall_slide_dir != 0 {
            if input.grab {
                state.climb_trigger_dir = state.wall_slide_dir;
            }
            max = lerp(
                MAX_FALL,
                WALL_SLIDE_START_MAX,
                state.wall_slide_timer / WALL_SLIDE_TIME,
            )
        }
    }

    max
}

pub fn jump(state: &mut PlayerState, input: Input) {
    state.jump_grace_timer = 0.0;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.var_jump_timer = VAR_JUMP_TIME;
    state.speed.x += JUMP_H_BOOST * input.move_x as f32;
    state.speed.y = JUMP_SPEED;
    add_lift_boost(state);
    state.var_jump_speed = state.speed.y;
    state.auto_jump = false;
    state.wall_boost_timer = 0.0;
    state.dash_attack_timer = 0.0;
}

pub fn can_super_jump(state: &PlayerState, room: &Room, jump_pressed: bool) -> bool {
    jump_pressed
        && state.jump_grace_timer > 0.0
        && state.dash_dir.y == 0.0
        && can_unduck(state, room)
}

pub fn super_jump(state: &mut PlayerState) {
    state.jump_grace_timer = 0.0;
    state.var_jump_timer = VAR_JUMP_TIME;
    state.auto_jump = false;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.wall_boost_timer = 0.0;
    state.dash_attack_timer = 0.0;
    state.speed.x = SUPER_JUMP_H * state.facing as f32;
    state.speed.y = JUMP_SPEED;
    add_lift_boost(state);

    // hyper
    if state.ducking {
        state.ducking = false;
        state.speed.x *= DUCK_SUPER_JUMP_X_MULT;
        state.speed.y *= DUCK_SUPER_JUMP_Y_MULT;
    }

    state.var_jump_speed = state.speed.y;
    set_normal(state);
}

pub fn wall_bounce_ready(state: &PlayerState) -> bool {
    state.dash_attack_timer > 0.0 && state.dash_dir.x == 0.0 && state.dash_dir.y == -1.0
}

pub fn wall_jump_check(state: &PlayerState, room: &Room, dir: i8) -> bool {
    let dir = dir as i32;
    let mut dist = WALL_JUMP_CHECK_DIST;

    if wall_bounce_ready(state) {
        let facing_her = if dir > 0 {
            Direction::Left
        } else {
            Direction::Right
        };
        let reach = Rect {
            left: state.position.x + dir * SUPER_WALL_JUMP_CHECK_DIST - HITBOX_WIDTH / 2,
            top: state.position.y - hitbox_height(state),
            width: HITBOX_WIDTH,
            height: hitbox_height(state),
        };
        let spiked = room
            .spikes
            .iter()
            .any(|s| s.direction == facing_her && s.hitbox.overlaps(&reach));

        if !spiked {
            dist = SUPER_WALL_JUMP_CHECK_DIST;
        }
    }

    collides(state, room, dir * dist, 0)
}

pub fn wall_jump(state: &mut PlayerState, room: &Room, dir: i8, move_x: i8) {
    state.ducking = false;
    state.jump_grace_timer = 0.0;
    state.var_jump_timer = VAR_JUMP_TIME;
    state.auto_jump = false;
    state.dash_attack_timer = 0.0;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.wall_boost_timer = 0.0;

    if move_x != 0 {
        state.force_move_x = dir;
        state.force_move_x_timer = WALL_JUMP_FORCE_TIME;
    }

    if state.lift_speed == NO_LIFT && state.last_lift_speed == NO_LIFT {
        let reach = hitbox(state, WALL_JUMP_CHECK_DIST, 0);
        if let Some(wall) = room
            .solids
            .iter()
            .find(|solid| solid.collidable && solid.rect.overlaps(&reach))
        {
            set_lift_speed(state, wall.lift_speed);
        }
    }

    state.speed.x = WALL_JUMP_H_SPEED * dir as f32;
    state.speed.y = JUMP_SPEED;
    add_lift_boost(state);
    state.var_jump_speed = state.speed.y;
    set_normal(state);
}

pub fn super_wall_jump(state: &mut PlayerState, dir: i8) {
    state.ducking = false;
    state.jump_grace_timer = 0.0;
    state.var_jump_timer = SUPER_WALL_JUMP_VAR_TIME;
    state.auto_jump = false;
    state.dash_attack_timer = 0.0;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.wall_boost_timer = 0.0;

    state.speed.x = SUPER_WALL_JUMP_H * dir as f32;
    state.speed.y = SUPER_WALL_JUMP_SPEED;
    add_lift_boost(state);
    state.var_jump_speed = state.speed.y;
    set_normal(state);
}

pub fn jump_off_wall(state: &mut PlayerState, room: &Room, frame: Frame) -> bool {
    let input = frame.input;

    if !can_unduck(state, room) {
        return false;
    }

    let wall = if wall_jump_check(state, room, 1) {
        1
    } else if wall_jump_check(state, room, -1) {
        -1
    } else {
        return false;
    };

    let dashing_up = dashing(state) && state.dash_dir.x == 0.0 && state.dash_dir.y == -1.0;

    if dashing_up {
        super_wall_jump(state, -wall);
    } else if can_climb_jump(state, room, input) {
        climb_jump(state, frame);
    } else if wall_bounce_ready(state) {
        super_wall_jump(state, -wall);
    } else {
        wall_jump(state, room, -wall, input.move_x);
    }

    true
}

pub fn can_climb_jump(state: &PlayerState, room: &Room, input: Input) -> bool {
    let able = input.grab && state.stamina > 0.0;

    if wall_jump_check(state, room, 1) {
        state.facing == 1 && able
    } else if wall_jump_check(state, room, -1) {
        state.facing == -1 && able
    } else {
        false
    }
}

pub fn climb_jump(state: &mut PlayerState, frame: Frame) {
    let Frame { input, ground, .. } = frame;

    if !ground {
        state.stamina -= CLIMB_JUMP_COST;
    }

    jump(state, input);

    if input.move_x == 0 {
        state.wall_boost_dir = -state.facing;
        state.wall_boost_timer = CLIMB_JUMP_BOOST_TIME;
    }

    set_normal(state);
}

pub fn super_bounce(state: &mut PlayerState, room: &Room, from_y: i32) {
    nudge_v(state, room, (from_y - state.position.y) as f32);

    state.dashes = MAX_DASHES;
    state.stamina = CLIMB_MAX_STAMINA;
    set_normal(state);

    state.jump_grace_timer = 0.0;
    state.var_jump_timer = SUPER_BOUNCE_VAR_JUMP_TIME;
    state.auto_jump = true;
    state.dash_attack_timer = 0.0;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.wall_boost_timer = 0.0;

    state.speed.x = 0.0;
    state.speed.y = SUPER_BOUNCE_SPEED;
    state.var_jump_speed = SUPER_BOUNCE_SPEED;
}

pub fn bounce_on_springs(state: &mut PlayerState, room: &Room) {
    for spring in &room.springs {
        if state.speed.y >= 0.0 && spring.hitbox.overlaps(&hurtbox(state)) {
            super_bounce(state, room, spring.hitbox.top);
        }
    }
}
