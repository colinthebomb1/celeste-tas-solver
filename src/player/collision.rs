use super::*;

pub fn collide_at(room: &Room, x: i32, y: i32) -> bool {
    room.collide_rect(x - HITBOX_WIDTH / 2, y - HITBOX_HEIGHT, HITBOX_WIDTH, HITBOX_HEIGHT)
}

pub fn hitbox_height(state: &PlayerState) -> i32 {
    if state.ducking { DUCK_HITBOX_HEIGHT } else { HITBOX_HEIGHT }
}

pub fn collides(state: &PlayerState, room: &Room, dx: i32, dy: i32) -> bool {
    let height = hitbox_height(state);

    room.collide_rect(
        state.position.x + dx - HITBOX_WIDTH / 2,
        state.position.y + dy - height,
        HITBOX_WIDTH,
        height,
    )
}

pub fn can_unduck(state: &PlayerState, room: &Room) -> bool {
    !state.ducking || !collide_at(room, state.position.x, state.position.y)
}

pub fn hurtbox(state: &PlayerState) -> Rect {
    let (top, height) = if state.ducking {
        (state.position.y - DUCK_HITBOX_HEIGHT, DUCK_HURTBOX_HEIGHT)
    } else {
        (state.position.y - HITBOX_HEIGHT, HURTBOX_HEIGHT)
    };

    Rect {
        left: state.position.x - HITBOX_WIDTH / 2,
        top,
        width: HITBOX_WIDTH,
        height,
    }
}

pub fn spikes_kill(state: &PlayerState, spikes: &Spikes) -> bool {
    let madeline_hurtbox = &hurtbox(state);

    if !spikes.hitbox.overlaps(madeline_hurtbox) {
        return false;
    }

    match spikes.direction {
        Direction::Up => state.speed.y >= 0.0 && madeline_hurtbox.top + madeline_hurtbox.height <= spikes.hitbox.top + spikes.hitbox.height,
        Direction::Down => state.speed.y <= 0.0,
        Direction::Left => state.speed.x >= 0.0,
        Direction::Right => state.speed.x <= 0.0
    }
}

// top of hitbox is more than 4 px below the room's bottom edge
pub fn fell_out(state: &PlayerState, room: &Room) -> bool {
    state.position.y - HITBOX_HEIGHT > room.bottom() + 4
}

pub fn hitbox(state: &PlayerState, dx: i32, dy: i32) -> Rect {
    let height = hitbox_height(state);

    Rect {
        left: state.position.x + dx - HITBOX_WIDTH / 2,
        top: state.position.y + dy - height,
        width: HITBOX_WIDTH,
        height,
    }
}

pub fn touches_jump_thru(state: &PlayerState, room: &Room, dx: i32, dy: i32) -> bool {
    let moved = hitbox(state, dx, dy);
    room.jump_thrus.iter().any(|jump_thru| jump_thru.hitbox.overlaps(&moved))
}

pub fn enters_jump_thru(state: &PlayerState, room: &Room, dx: i32, dy: i32) -> bool {
    let now = hitbox(state, 0, 0);
    let moved = hitbox(state, dx, dy);
    room.jump_thrus
        .iter()
        .any(|jump_thru| !jump_thru.hitbox.overlaps(&now) && jump_thru.hitbox.overlaps(&moved))
}

pub fn floor_below(state: &PlayerState, room: &Room, dx: i32) -> bool {
    collides(state, room, dx, 1) || enters_jump_thru(state, room, dx, 1)
}

// whether a moving block carries her along
pub fn is_riding(state: &PlayerState, solid: &Rect) -> bool {
    if state.dead {
        false
    } else if state.mode == Mode::Climb {
        hitbox(state, state.facing as i32, 0).overlaps(solid)
    } else if state.climb_trigger_dir != 0 {
        hitbox(state, state.climb_trigger_dir as i32, 0).overlaps(solid)
    } else {
        hitbox(state, 0, 1).overlaps(solid)
    }
}

pub fn on_ground(state: &PlayerState, room: &Room) -> bool {
    state.speed.y >= 0.0 && floor_below(state, room, 0)
}

pub fn duck_free_at(state: &PlayerState, room: &Room, dx: i32) -> bool {
    !room.collide_rect(
        state.position.x + dx - HITBOX_WIDTH / 2,
        state.position.y - DUCK_HITBOX_HEIGHT,
        HITBOX_WIDTH,
        DUCK_HITBOX_HEIGHT,
    )
}

// jump-throughs only block her on the way down
pub(crate) fn blocked_v(state: &PlayerState, room: &Room, step: i32) -> bool {
    collides(state, room, 0, step) || (step > 0 && enters_jump_thru(state, room, 0, step))
}

pub fn wall_at(state: &PlayerState, room: &Room, dir: i8) -> bool {
    collides(state, room, dir as i32, 0)
}

pub(crate) fn solid_at(room: &Room, x: i32, y: i32) -> bool {
    room.collide_rect(x, y, 1, 1)
}
