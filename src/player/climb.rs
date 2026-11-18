use super::*;

pub fn is_tired(player: &Player) -> bool {
    let stamina = if player.wall.boost_timer > 0.0 {
        player.stamina + CLIMB_JUMP_COST
    } else {
        player.stamina
    };

    stamina < CLIMB_TIRED_THRESHOLD
}

pub fn climb_check(player: &Player, room: &Room, dir: i8, dy: i32) -> bool {
    let reach = dir as i32 * CLIMB_CHECK_DIST;
    let her = hitbox(player, reach, 0);
    let in_bounds = her.left >= room.bounds.left && her.right() < room.bounds.right();

    in_bounds && collides(player, room, reach, dy)
}

pub fn try_grab(player: &mut Player, room: &Room, input: Input) -> bool {
    if !input.grab || is_tired(player) || player.crouching {
        return false;
    }
    if player.speed.y < 0.0 || sign(player.speed.x) == -(player.facing as f32) {
        return false;
    }

    if climb_check(player, room, player.facing, 0) {
        climb_begin(player, room);
        return true;
    }

    if input.move_y < 1 {
        for up in 1..=CLIMB_UP_CHECK_DIST {
            if !collides(player, room, 0, -up) && climb_check(player, room, player.facing, -up) {
                move_exact_v(player, room, -up);
                climb_begin(player, room);
                return true;
            }
        }
    }

    false
}

pub fn climb_begin(player: &mut Player, room: &Room) {
    player.crouching = false;
    player.auto_jump = false;
    player.speed.x = 0.0;
    player.speed.y *= CLIMB_GRAB_Y_MULT;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.climb.no_move_timer = CLIMB_NO_MOVE_TIME;
    player.wall.boost_timer = 0.0;
    player.climb.last_move = 0;

    // pull her against the wall
    for _ in 0..CLIMB_CHECK_DIST {
        if collides(player, room, player.facing as i32, 0) {
            break;
        }
        player.position.x += player.facing as i32;
    }

    player.state = State::Climb;
}

fn solid_in(room: &Room, rect: &Rect) -> Option<HopSolid> {
    let id = room
        .solids
        .iter()
        .position(|solid| solid.collidable && solid.rect.overlaps(rect))?;

    Some(HopSolid {
        id,
        position: room.solids[id].position(),
    })
}

// she pops up over the ledge. the sideways push waits until the wall is out of her way
pub fn climbhop(player: &mut Player, room: &Room) {
    if collides(player, room, player.facing as i32, 0) {
        player.climb.hop_solid = solid_in(room, &hitbox(player, player.facing as i32, 0));
        player.climb.hop_wait = player.facing;
        player.climb.hop_speed = player.facing as f32 * CLIMBHOP_X;
    } else {
        player.climb.hop_solid = None;
        player.climb.hop_wait = 0;
        player.speed.x = player.facing as f32 * CLIMBHOP_X;
    }

    player.speed.y = player.speed.y.min(CLIMBHOP_Y);
    player.forced_move = 0;
    player.forced_move_timer = CLIMBHOP_FORCE_TIME;
    set_normal(player);
}

// keeps her beside a moving block while she hops onto it
pub(crate) fn follow_hop_solid(player: &mut Player, room: &Room) {
    let Some(solid) = player.climb.hop_solid else {
        return;
    };

    let now = room.solids[solid.id];

    match now.collidable.then_some(now.position()) {
        None => player.climb.hop_solid = None,
        Some(position) if position != solid.position => {
            player.climb.hop_solid = Some(HopSolid { position, ..solid });
            move_exact_h(player, room, position.x - solid.position.x);
            move_exact_v(player, room, position.y - solid.position.y);
        }
        Some(_) => {}
    }
}

pub(crate) fn update_hop_wait(player: &mut Player, room: &Room) {
    if player.climb.hop_wait == 0 {
        return;
    }

    if sign(player.speed.x) == -(player.climb.hop_wait as f32) || player.speed.y > 0.0 {
        player.climb.hop_wait = 0;
    } else if !collides(player, room, player.climb.hop_wait as i32, 0) {
        player.speed.x = player.climb.hop_speed;
        player.climb.hop_wait = 0;
    }
}

// her hands are above the top of the wall
pub fn slip_check(player: &Player, room: &Room, add_y: i32) -> bool {
    let her = hitbox(player, 0, 0);
    let x = if player.facing == 1 {
        her.right()
    } else {
        her.left - 1
    };
    let y = her.top + 4 + add_y;

    !room.solid_at(x, y) && !room.solid_at(x, y - 4 + add_y)
}

pub(crate) fn climb_update(player: &mut Player, room: &Room, frame: Frame) {
    let Frame {
        input,
        ground,
        jump_pressed,
        dash_pressed,
        ..
    } = frame;

    player.climb.no_move_timer -= DELTA_TIME;

    if ground {
        player.stamina = CLIMB_MAX_STAMINA;
    }

    if jump_pressed && can_stand(player, room) {
        if input.move_x == -player.facing {
            wall_kick(player, room, -player.facing, input.move_x);
        } else {
            climb_jump(player, frame);
        }
        return;
    }

    if can_dash(player, dash_pressed) {
        add_liftboost(player);
        dash_begin(player, room, frame);
        return;
    }

    if !input.grab {
        add_liftboost(player);
        set_normal(player);
        return;
    }

    if !collides(player, room, player.facing as i32, 0) {
        if player.speed.y < 0.0 {
            climbhop(player, room);
        } else {
            set_normal(player);
        }
        return;
    }

    let can_move = player.climb.no_move_timer <= 0.0;
    let mut target = 0.0;
    let mut may_slip = !can_move || input.move_y == 0;

    if can_move && input.move_y == -1 {
        let hop_blocked = collides(player, room, 0, -CLIMBHOP_BLOCK_DIST);

        if collides(player, room, 0, -1) || (hop_blocked && slip_check(player, room, -1)) {
            player.speed.y = player.speed.y.max(0.0);
            may_slip = true;
        } else if slip_check(player, room, 0) {
            climbhop(player, room);
            return;
        } else {
            target = CLIMB_UP_SPEED;
        }
    } else if can_move && input.move_y == 1 {
        if ground {
            player.speed.y = player.speed.y.min(0.0);
        } else {
            target = CLIMB_DOWN_SPEED;
        }
    }

    player.climb.last_move = sign(target) as i8;

    if may_slip && slip_check(player, room, 0) {
        target = CLIMB_SLIP_SPEED;
    }
    player.speed.y = approach(player.speed.y, target, CLIMB_ACCEL * DELTA_TIME);

    // nothing to slide down
    if input.move_y != 1 && player.speed.y > 0.0 && !collides(player, room, player.facing as i32, 1)
    {
        player.speed.y = 0.0;
    }

    if can_move {
        if player.climb.last_move == -1 {
            player.stamina -= CLIMB_UP_COST * DELTA_TIME;
        } else if player.climb.last_move == 0 {
            player.stamina -= CLIMB_STILL_COST * DELTA_TIME;
        }
    }

    if player.stamina <= 0.0 {
        add_liftboost(player);
        set_normal(player);
    }
}
