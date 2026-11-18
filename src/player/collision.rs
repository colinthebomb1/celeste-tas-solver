use super::*;

pub fn hitbox_height(player: &Player) -> i32 {
    if player.crouching {
        CROUCH_HITBOX_HEIGHT
    } else {
        HITBOX_HEIGHT
    }
}

// her feet are at her position, in the middle of the box
fn box_at_feet(player: &Player, height: i32) -> Rect {
    Rect::new(
        player.position.x - HITBOX_WIDTH / 2,
        player.position.y - height,
        HITBOX_WIDTH,
        height,
    )
}

pub fn hitbox(player: &Player, dx: i32, dy: i32) -> Rect {
    box_at_feet(player, hitbox_height(player)).shifted(dx, dy)
}

// same top as the hitbox, but it stops short of her feet
pub fn hurtbox(player: &Player) -> Rect {
    let height = if player.crouching {
        CROUCH_HURTBOX_HEIGHT
    } else {
        HURTBOX_HEIGHT
    };
    Rect {
        height,
        ..hitbox(player, 0, 0)
    }
}

// whether her hitbox would be inside something solid if she moved this far
pub fn collides(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    room.solid(&hitbox(player, dx, dy))
}

// like collides, but a jump-through under her also counts when she is moving down onto it
pub fn blocked(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    collides(player, room, dx, dy) || (dy > 0 && enters_jump_thru(player, room, dx, dy))
}

pub fn on_ground(player: &Player, room: &Room) -> bool {
    player.speed.y >= 0.0 && blocked(player, room, 0, 1)
}

pub fn can_stand(player: &Player, room: &Room) -> bool {
    !player.crouching || !room.solid(&box_at_feet(player, HITBOX_HEIGHT))
}

// room to be crouched this far to the side
pub fn crouch_free_at(player: &Player, room: &Room, dx: i32) -> bool {
    !room.solid(&box_at_feet(player, CROUCH_HITBOX_HEIGHT).shifted(dx, 0))
}

pub fn spikes_kill(player: &Player, spikes: &Spikes) -> bool {
    let madeline_hurtbox = &hurtbox(player);

    if !spikes.hitbox.overlaps(madeline_hurtbox) {
        return false;
    }

    match spikes.direction {
        Direction::Up => {
            player.speed.y >= 0.0
                && madeline_hurtbox.top + madeline_hurtbox.height
                    <= spikes.hitbox.top + spikes.hitbox.height
        }
        Direction::Down => player.speed.y <= 0.0,
        Direction::Left => player.speed.x >= 0.0,
        Direction::Right => player.speed.x <= 0.0,
    }
}

// top of hitbox is more than 4 px below the room's bottom edge
pub fn fell_out(player: &Player, room: &Room) -> bool {
    player.position.y - HITBOX_HEIGHT > room.bottom() + 4
}

pub fn touches_jump_thru(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    let moved = hitbox(player, dx, dy);
    room.jump_thrus
        .iter()
        .any(|jump_thru| jump_thru.hitbox.overlaps(&moved))
}

pub fn enters_jump_thru(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    let now = hitbox(player, 0, 0);
    let moved = hitbox(player, dx, dy);
    room.jump_thrus
        .iter()
        .any(|jump_thru| !jump_thru.hitbox.overlaps(&now) && jump_thru.hitbox.overlaps(&moved))
}

// whether a moving block carries her along
pub fn is_riding(player: &Player, solid: &Rect) -> bool {
    if player.dead {
        false
    } else if player.state == State::Climb {
        hitbox(player, player.facing as i32, 0).overlaps(solid)
    } else if player.climb.trigger_dir != 0 {
        hitbox(player, player.climb.trigger_dir as i32, 0).overlaps(solid)
    } else {
        hitbox(player, 0, 1).overlaps(solid)
    }
}
