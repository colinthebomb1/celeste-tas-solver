use super::*;

pub fn hitbox_height(state: &PlayerState) -> i32 {
    if state.ducking { DUCK_HITBOX_HEIGHT } else { HITBOX_HEIGHT }
}

// her feet are at her position, in the middle of the box
fn box_at_feet(state: &PlayerState, height: i32) -> Rect {
    Rect::new(state.position.x - HITBOX_WIDTH / 2, state.position.y - height, HITBOX_WIDTH, height)
}

pub fn hitbox(state: &PlayerState, dx: i32, dy: i32) -> Rect {
    box_at_feet(state, hitbox_height(state)).shifted(dx, dy)
}

// same top as the hitbox, but it stops short of her feet
pub fn hurtbox(state: &PlayerState) -> Rect {
    let height = if state.ducking { DUCK_HURTBOX_HEIGHT } else { HURTBOX_HEIGHT };
    Rect { height, ..hitbox(state, 0, 0) }
}

// whether her hitbox would be inside something solid if she moved this far
pub fn collides(state: &PlayerState, room: &Room, dx: i32, dy: i32) -> bool {
    room.solid(&hitbox(state, dx, dy))
}

// like collides, but a jump-through under her also counts when she is moving down onto it
pub fn blocked(state: &PlayerState, room: &Room, dx: i32, dy: i32) -> bool {
    collides(state, room, dx, dy) || (dy > 0 && enters_jump_thru(state, room, dx, dy))
}

pub fn on_ground(state: &PlayerState, room: &Room) -> bool {
    state.speed.y >= 0.0 && blocked(state, room, 0, 1)
}

pub fn can_unduck(state: &PlayerState, room: &Room) -> bool {
    !state.ducking || !room.solid(&box_at_feet(state, HITBOX_HEIGHT))
}

// room to be ducked this far to the side
pub fn duck_free_at(state: &PlayerState, room: &Room, dx: i32) -> bool {
    !room.solid(&box_at_feet(state, DUCK_HITBOX_HEIGHT).shifted(dx, 0))
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

