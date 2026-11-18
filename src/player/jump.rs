use super::*;

pub fn wall_slide_max_fall(player: &mut Player, room: &Room, input: Input) -> f32 {
    let mut max = player.max_fall;
    let into_wall = input.move_x == player.facing || (input.move_x == 0 && input.grab);

    if into_wall && input.move_y != 1 {
        if player.speed.y >= 0.0
            && player.wall.slide_timer > 0.0
            && collides(player, room, player.facing as i32, 0)
            && can_stand(player, room)
        {
            player.crouching = false;
            player.wall.slide_dir = player.facing;
        }

        if player.wall.slide_dir != 0 {
            if input.grab {
                player.climb.trigger_dir = player.wall.slide_dir;
            }
            max = lerp(
                MAX_FALL,
                WALL_SLIDE_START_MAX,
                player.wall.slide_timer / WALL_SLIDE_TIME,
            )
        }
    }

    max
}

pub fn jump(player: &mut Player, input: Input) {
    player.coyote_timer = 0.0;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.jump_timer = JUMP_HOLD_TIME;
    player.speed.x += JUMP_H_BOOST * input.move_x as f32;
    player.speed.y = JUMP_SPEED;
    add_liftboost(player);
    player.jump_speed = player.speed.y;
    player.auto_jump = false;
    player.wall.boost_timer = 0.0;
    player.dash.attack_timer = 0.0;
}

pub fn can_super_dash(player: &Player, room: &Room, jump_pressed: bool) -> bool {
    jump_pressed && player.coyote_timer > 0.0 && player.dash.dir.y == 0.0 && can_stand(player, room)
}

pub fn super_dash(player: &mut Player) {
    player.coyote_timer = 0.0;
    player.jump_timer = JUMP_HOLD_TIME;
    player.auto_jump = false;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.wall.boost_timer = 0.0;
    player.dash.attack_timer = 0.0;
    player.speed.x = SUPER_SPEED * player.facing as f32;
    player.speed.y = JUMP_SPEED;
    add_liftboost(player);

    // hyper
    if player.crouching {
        player.crouching = false;
        player.speed.x *= HYPER_X_MULT;
        player.speed.y *= HYPER_Y_MULT;
    }

    player.jump_speed = player.speed.y;
    set_normal(player);
}

pub fn wallbounce_ready(player: &Player) -> bool {
    player.dash.attack_timer > 0.0 && player.dash.dir.x == 0.0 && player.dash.dir.y == -1.0
}

pub fn wall_kick_check(player: &Player, room: &Room, dir: i8) -> bool {
    let dir = dir as i32;
    let mut dist = WALL_KICK_REACH;

    if wallbounce_ready(player) {
        let facing_her = if dir > 0 {
            Direction::Left
        } else {
            Direction::Right
        };
        let reach = Rect {
            left: player.position.x + dir * WALLBOUNCE_REACH - HITBOX_WIDTH / 2,
            top: player.position.y - hitbox_height(player),
            width: HITBOX_WIDTH,
            height: hitbox_height(player),
        };
        let spiked = room
            .spikes
            .iter()
            .any(|s| s.direction == facing_her && s.hitbox.overlaps(&reach));

        if !spiked {
            dist = WALLBOUNCE_REACH;
        }
    }

    collides(player, room, dir * dist, 0)
}

pub fn wall_kick(player: &mut Player, room: &Room, dir: i8, move_x: i8) {
    player.crouching = false;
    player.coyote_timer = 0.0;
    player.jump_timer = JUMP_HOLD_TIME;
    player.auto_jump = false;
    player.dash.attack_timer = 0.0;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.wall.boost_timer = 0.0;

    if move_x != 0 {
        player.forced_move = dir;
        player.forced_move_timer = WALL_KICK_FORCE_TIME;
    }

    if player.lift.speed == NO_LIFT && player.lift.last_speed == NO_LIFT {
        let reach = hitbox(player, WALL_KICK_REACH, 0);
        if let Some(wall) = room
            .solids
            .iter()
            .find(|solid| solid.collidable && solid.rect.overlaps(&reach))
        {
            set_lift_speed(player, wall.lift_speed);
        }
    }

    player.speed.x = WALL_KICK_SPEED * dir as f32;
    player.speed.y = JUMP_SPEED;
    add_liftboost(player);
    player.jump_speed = player.speed.y;
    set_normal(player);
}

pub fn wallbounce(player: &mut Player, dir: i8) {
    player.crouching = false;
    player.coyote_timer = 0.0;
    player.jump_timer = WALLBOUNCE_JUMP_TIME;
    player.auto_jump = false;
    player.dash.attack_timer = 0.0;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.wall.boost_timer = 0.0;

    player.speed.x = WALLBOUNCE_SPEED_X * dir as f32;
    player.speed.y = WALLBOUNCE_SPEED_Y;
    add_liftboost(player);
    player.jump_speed = player.speed.y;
    set_normal(player);
}

pub fn jump_off_wall(player: &mut Player, room: &Room, frame: Frame) -> bool {
    let input = frame.input;

    if !can_stand(player, room) {
        return false;
    }

    let wall = if wall_kick_check(player, room, 1) {
        1
    } else if wall_kick_check(player, room, -1) {
        -1
    } else {
        return false;
    };

    let dashing_up = dashing(player) && player.dash.dir.x == 0.0 && player.dash.dir.y == -1.0;

    if dashing_up {
        wallbounce(player, -wall);
    } else if can_climb_jump(player, room, input) {
        climb_jump(player, frame);
    } else if wallbounce_ready(player) {
        wallbounce(player, -wall);
    } else {
        wall_kick(player, room, -wall, input.move_x);
    }

    true
}

pub fn can_climb_jump(player: &Player, room: &Room, input: Input) -> bool {
    let able = input.grab && player.stamina > 0.0;

    if wall_kick_check(player, room, 1) {
        player.facing == 1 && able
    } else if wall_kick_check(player, room, -1) {
        player.facing == -1 && able
    } else {
        false
    }
}

pub fn climb_jump(player: &mut Player, frame: Frame) {
    let Frame { input, ground, .. } = frame;

    if !ground {
        player.stamina -= CLIMB_JUMP_COST;
    }

    jump(player, input);

    if input.move_x == 0 {
        player.wall.boost_dir = -player.facing;
        player.wall.boost_timer = WALLBOOST_TIME;
    }

    set_normal(player);
}

pub fn spring_bounce(player: &mut Player, room: &Room, from_y: i32) {
    nudge_v(player, room, (from_y - player.position.y) as f32);

    player.dashes = MAX_DASHES;
    player.stamina = CLIMB_MAX_STAMINA;
    set_normal(player);

    player.coyote_timer = 0.0;
    player.jump_timer = SPRING_JUMP_TIME;
    player.auto_jump = true;
    player.dash.attack_timer = 0.0;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.wall.boost_timer = 0.0;

    player.speed.x = 0.0;
    player.speed.y = SPRING_SPEED;
    player.jump_speed = SPRING_SPEED;
}

pub fn bounce_on_springs(player: &mut Player, room: &Room) {
    for spring in &room.springs {
        if player.speed.y >= 0.0 && spring.hitbox.overlaps(&hurtbox(player)) {
            spring_bounce(player, room, spring.hitbox.top);
        }
    }
}
