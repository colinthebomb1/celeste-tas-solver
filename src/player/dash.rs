use super::*;

pub(crate) fn dashing(player: &Player) -> bool {
    matches!(player.state, State::DashStart | State::Dash { .. })
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

pub fn can_dash(player: &Player, dash_pressed: bool) -> bool {
    dash_pressed && player.dash.cooldown <= 0.0 && player.dashes > 0
}

pub fn dash_begin(player: &mut Player, room: &Room, frame: Frame) {
    let Frame {
        input,
        ground,
        demodash_pressed,
        ..
    } = frame;

    player.dashes -= 1;
    player.freeze_timer = DASH_FREEZE_TIME;
    player.dash.cooldown = DASH_COOLDOWN;
    player.dash.refill_cooldown = DASH_REFILL_COOLDOWN;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.dash.speed_before = player.speed;
    player.dash.attack_timer = DASH_ATTACK_TIME;
    player.dash.started_on_ground = ground;
    player.speed = Vec2 { x: 0.0, y: 0.0 };
    player.dash.dir = Vec2 { x: 0.0, y: 0.0 };

    // holding down at the start of a dash crouchs
    if !ground && player.crouching && can_stand(player, room) {
        player.crouching = false;
    } else if !player.crouching && (demodash_pressed || input.move_y == 1) {
        player.crouching = true;
    }

    player.state = State::DashStart
}

pub fn dash_launch(player: &mut Player, aim: Vec2, ground: bool) {
    let mut new_speed_x = aim.x * DASH_SPEED;
    let new_speed_y = aim.y * DASH_SPEED;

    if sign(player.dash.speed_before.x) == sign(new_speed_x)
        && player.dash.speed_before.x.abs() > new_speed_x.abs()
    {
        new_speed_x = player.dash.speed_before.x;
    }

    player.speed = Vec2 {
        x: new_speed_x,
        y: new_speed_y,
    };
    player.dash.dir = aim;

    if aim.x != 0.0 {
        player.facing = sign(aim.x) as i8;
    }

    // ultra: a down diagonal dash on the ground becomes a crouched horizontal one, a bit faster
    if ground && player.dash.dir.x != 0.0 && player.dash.dir.y > 0.0 && player.speed.y > 0.0 {
        player.dash.dir = Vec2 {
            x: sign(player.dash.dir.x),
            y: 0.0,
        };
        player.speed.y = 0.0;
        player.speed.x *= ULTRA_SPEED_MULT;
        player.crouching = true;
    }

    player.state = State::Dash { timer: DASH_TIME };
}

pub fn dash_end(player: &mut Player) {
    player.auto_jump = true;

    if player.dash.dir.y <= 0.0 {
        player.speed.x = player.dash.dir.x * END_DASH_SPEED;
        player.speed.y = player.dash.dir.y * END_DASH_SPEED;
    }

    if player.speed.y < 0.0 {
        player.speed.y *= END_DASH_UP_MULT;
    }

    set_normal(player);
}

pub fn use_crystal(player: &mut Player, two_dashes: bool) -> bool {
    let dashes = if two_dashes { TWO_DASHES } else { MAX_DASHES };

    if player.dashes < dashes || player.stamina < CLIMB_TIRED_THRESHOLD {
        player.dashes = dashes;
        player.stamina = CLIMB_MAX_STAMINA;
        true
    } else {
        false
    }
}
