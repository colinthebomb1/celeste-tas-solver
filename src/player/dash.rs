use super::*;

pub(crate) fn dashing(state: &PlayerState) -> bool {
    matches!(state.mode, Mode::DashStart | Mode::Dashing { .. })
}

pub fn aim_vector(input: Input, facing: i8) -> Vec2 {
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

pub fn can_dash(state: &PlayerState, dash_pressed: bool) -> bool {
    dash_pressed && state.dash_cooldown_timer <= 0.0 && state.dashes > 0
}

pub fn dash_begin(state: &mut PlayerState, room: &Room, frame: Frame) {
    let Frame {
        input,
        ground,
        crouch_dash_pressed,
        ..
    } = frame;

    state.dashes -= 1;
    state.freeze_timer = DASH_FREEZE_TIME;
    state.dash_cooldown_timer = DASH_COOLDOWN;
    state.dash_refill_cooldown_timer = DASH_REFILL_COOLDOWN;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.before_dash_speed = state.speed;
    state.dash_attack_timer = DASH_ATTACK_TIME;
    state.dash_started_on_ground = ground;
    state.speed = Vec2 { x: 0.0, y: 0.0 };
    state.dash_dir = Vec2 { x: 0.0, y: 0.0 };

    // holding down at the start of a dash ducks
    if !ground && state.ducking && can_unduck(state, room) {
        state.ducking = false;
    } else if !state.ducking && (crouch_dash_pressed || input.move_y == 1) {
        state.ducking = true;
    }

    state.mode = Mode::DashStart
}

pub fn dash_launch(state: &mut PlayerState, aim: Vec2, ground: bool) {
    let mut new_speed_x = aim.x * DASH_SPEED;
    let new_speed_y = aim.y * DASH_SPEED;

    if sign(state.before_dash_speed.x) == sign(new_speed_x)
        && state.before_dash_speed.x.abs() > new_speed_x.abs()
    {
        new_speed_x = state.before_dash_speed.x;
    }

    state.speed = Vec2 {
        x: new_speed_x,
        y: new_speed_y,
    };
    state.dash_dir = aim;

    if aim.x != 0.0 {
        state.facing = sign(aim.x) as i8;
    }

    // dash slide - a down diagonal dash on the ground becomes a ducked horizontal one
    if ground && state.dash_dir.x != 0.0 && state.dash_dir.y > 0.0 && state.speed.y > 0.0 {
        state.dash_dir = Vec2 {
            x: sign(state.dash_dir.x),
            y: 0.0,
        };
        state.speed.y = 0.0;
        state.speed.x *= DODGE_SLIDE_SPEED_MULT;
        state.ducking = true;
    }

    state.mode = Mode::Dashing { timer: DASH_TIME };
}

pub fn dash_end(state: &mut PlayerState) {
    state.auto_jump = true;

    if state.dash_dir.y <= 0.0 {
        state.speed.x = state.dash_dir.x * END_DASH_SPEED;
        state.speed.y = state.dash_dir.y * END_DASH_SPEED;
    }

    if state.speed.y < 0.0 {
        state.speed.y *= END_DASH_UP_MULT;
    }

    set_normal(state);
}

pub fn use_refill(state: &mut PlayerState, two_dashes: bool) -> bool {
    let dashes = if two_dashes { TWO_DASHES } else { MAX_DASHES };

    if state.dashes < dashes || state.stamina < CLIMB_TIRED_THRESHOLD {
        state.dashes = dashes;
        state.stamina = CLIMB_MAX_STAMINA;
        true
    } else {
        false
    }
}
