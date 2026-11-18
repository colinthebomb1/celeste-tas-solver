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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::fixtures::*;

    #[test]
    fn step_climb_jumps_next_to_a_wall() {
        let s = step(player_at(WALL_X, 100, 0.0, -30.0), JUMP_GRAB, &room());
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
    }

    #[test]
    fn step_no_climb_jump_without_grab() {
        let no_grab = Input {
            grab: false,
            ..JUMP_GRAB
        };
        let s = step(player_at(WALL_X, 100, 0.0, 30.0), no_grab, &room());
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_ground_jump_wins_over_climb_jump() {
        let mut s = player_at(36, FLOOR_Y, 0.0, 0.0);
        s.stamina = 50.0;
        s = step(
            s,
            Input {
                move_x: 1,
                ..JUMP_GRAB
            },
            &room(),
        );
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_climb_jumps_out_of_a_dash() {
        let mut s = player_at(WALL_X - 2, 100, DASH_SPEED, 0.0);
        s.state = State::Dash { timer: 0.1 };
        s.dash.dir = Vec2 { x: 1.0, y: 0.0 };
        s = step(
            s,
            Input {
                move_x: 1,
                ..JUMP_GRAB
            },
            &room(),
        );
        assert_eq!(s.state, State::Normal);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn step_wallbounce_during_an_upward_dash_ignores_grab() {
        let mut s = bouncing_at(WALL_X);
        s.speed.y = -DASH_SPEED;
        s.state = State::Dash { timer: 0.1 };
        s = step(s, JUMP_GRAB, &room());
        assert_eq!(s.state, State::Normal);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALLBOUNCE_SPEED_X,
                y: WALLBOUNCE_SPEED_Y
            }
        );
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_holding_grab_without_pressing_jump_does_not_climb_jump() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.held.jump = true;
        s = step(s, JUMP_GRAB, &room());
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert!(s.speed.y > 0.0);
    }

    #[test]
    fn step_wall_kicks_without_grab() {
        let s = step(player_at(WALL_X - 2, 100, 0.0, 30.0), JUMP, &room());
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALL_KICK_SPEED,
                y: JUMP_SPEED
            }
        );
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);

        let s = step(
            player_at(LEFT_WALL_X + 2, 100, 0.0, 30.0),
            JUMP,
            &left_wall_room(),
        );
        assert_eq!(
            s.speed,
            Vec2 {
                x: WALL_KICK_SPEED,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn step_wall_kicks_when_facing_away_even_with_grab() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.facing = -1;
        s = step(s, JUMP_GRAB, &room());
        assert_eq!(s.speed.x, -WALL_KICK_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn grabbing_a_wall_while_falling_starts_a_climb() {
        let mut s = player_at(WALL_X, 100, 50.0, 30.0);
        s.facing = 1;
        s = step(s, GRAB, &room());

        assert_eq!(s.state, State::Climb);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.speed.y, 30.0 * CLIMB_GRAB_Y_MULT);
        assert_eq!(s.climb.no_move_timer, CLIMB_NO_MOVE_TIME);
    }

    #[test]
    fn no_grab_while_rising() {
        let mut s = player_at(WALL_X, 100, 0.0, -30.0);
        s.facing = 1;
        assert_eq!(step(s, GRAB, &room()).state, State::Normal);
    }

    #[test]
    fn no_grab_without_the_button() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.facing = 1;
        assert_eq!(step(s, Input::default(), &room()).state, State::Normal);
    }

    #[test]
    fn no_grab_when_tired() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.facing = 1;
        s.stamina = CLIMB_TIRED_THRESHOLD - 1.0;
        assert_eq!(step(s, GRAB, &room()).state, State::Normal);
    }

    #[test]
    fn a_pending_wallboost_counts_towards_not_being_tired() {
        let mut s = at_rest();
        s.stamina = CLIMB_TIRED_THRESHOLD - 1.0;
        assert!(is_tired(&s));

        s.wall.boost_timer = 0.1;
        assert!(!is_tired(&s));
    }

    #[test]
    fn no_grab_while_moving_away_from_the_wall() {
        let mut s = player_at(WALL_X, 100, -50.0, 30.0);
        s.facing = 1;
        assert!(!try_grab(&mut s, &room(), GRAB));
    }

    #[test]
    fn no_grab_while_crouching() {
        let mut s = crouching_at(WALL_X, 100);
        s.facing = 1;
        assert!(!try_grab(&mut s, &room(), GRAB));
    }

    #[test]
    fn grabbing_pulls_her_in_from_one_pixel_away() {
        let mut s = player_at(WALL_X - 1, 100, 0.0, 0.0);
        s.facing = 1;
        assert!(try_grab(&mut s, &room(), GRAB));
        assert_eq!(s.position.x, WALL_X);

        let mut s = player_at(WALL_X - 2, 100, 0.0, 0.0);
        s.facing = 1;
        assert!(!try_grab(&mut s, &room(), GRAB));
    }

    #[test]
    fn grabbing_reaches_two_pixels_up() {
        // the wall's bottom edge is y = 48. her head is 11 px above her feet
        let room = partial_wall(0..6);

        let mut s = player_at(WALL_X, 60, 0.0, 0.0);
        s.facing = 1;
        assert!(try_grab(&mut s, &room, GRAB));
        assert_eq!(s.position.y, 58);

        let mut s = player_at(WALL_X, 61, 0.0, 0.0);
        s.facing = 1;
        assert!(!try_grab(&mut s, &room, GRAB));
    }

    #[test]
    fn no_reaching_up_while_holding_down() {
        let room = partial_wall(0..6);
        let mut s = player_at(WALL_X, 60, 0.0, 0.0);
        s.facing = 1;
        assert!(!try_grab(&mut s, &room, GRAB_DOWN));
    }

    #[test]
    fn she_cannot_move_along_the_wall_right_after_grabbing() {
        let mut s = climbing(100);
        s.climb.no_move_timer = CLIMB_NO_MOVE_TIME;
        s = step(s, GRAB_UP, &room());

        assert_eq!(s.speed.y, 0.0);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn climbing_up_speeds_up_and_costs_stamina() {
        let mut s = step(climbing(100), GRAB_UP, &room());
        assert_eq!(s.speed.y, -CLIMB_ACCEL * DELTA_TIME);
        assert_eq!(s.climb.last_move, -1);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_UP_COST * DELTA_TIME);

        for _ in 0..10 {
            s = step(s, GRAB_UP, &room());
        }
        assert_eq!(s.speed.y, CLIMB_UP_SPEED);
    }

    #[test]
    fn climbing_down_is_free() {
        let mut s = climbing(100);
        for _ in 0..10 {
            s = step(s, GRAB_DOWN, &room());
        }

        assert_eq!(s.speed.y, CLIMB_DOWN_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn a_ceiling_stops_her_climbing_up() {
        let mut room = room();
        room.add_dash_block(64, 80, 16, 9);
        let s = step(climbing(100), GRAB_UP, &room);

        assert_eq!(s.speed.y, 0.0);
        assert_eq!(s.climb.last_move, 0);
    }

    #[test]
    fn letting_go_ends_the_climb() {
        let s = step(climbing(100), Input::default(), &room());
        assert_eq!(s.state, State::Normal);
    }

    #[test]
    fn running_out_of_stamina_ends_the_climb() {
        let mut s = climbing(100);
        s.stamina = 0.1;
        assert_eq!(step(s, GRAB, &room()).state, State::Normal);
    }

    #[test]
    fn jumping_while_climbing_is_a_climb_jump() {
        let s = step(climbing(100), JUMP_GRAB, &room());

        assert_eq!(s.state, State::Normal);
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
    }

    #[test]
    fn jumping_away_while_climbing_is_a_wall_kick() {
        let away = Input {
            move_x: -1,
            ..JUMP_GRAB
        };
        let s = step(climbing(100), away, &room());

        assert_eq!(s.state, State::Normal);
        assert_eq!(s.speed.x, -WALL_KICK_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn dashing_out_of_a_climb() {
        let dash = Input { dash: true, ..GRAB };
        assert_eq!(step(climbing(100), dash, &room()).state, State::DashStart);
    }

    #[test]
    fn she_keeps_facing_the_wall_while_climbing() {
        let away = Input { move_x: -1, ..GRAB };
        let s = step(climbing(100), away, &room());

        assert_eq!(s.facing, 1);
        assert_eq!(s.state, State::Climb);
    }

    #[test]
    fn leaving_a_climb_drops_any_retained_wall_speed() {
        let mut s = climbing(100);
        s.wall.retained_speed = 100.0;
        s.wall.retention_timer = RETENTION_TIME;

        assert_eq!(step(s, Input::default(), &room()).wall.retention_timer, 0.0);
    }

    #[test]
    fn slip_check_is_true_with_her_hands_above_the_wall() {
        // the wall's top edge is y = 48
        let room = partial_wall(6..23);
        assert!(slip_check(&climbing(54), &room, 0));
        assert!(!slip_check(&climbing(55), &room, 0));
        assert!(!slip_check(&climbing(100), &room, 0));
    }

    #[test]
    fn she_slips_down_when_her_hands_are_above_the_wall() {
        let room = partial_wall(6..23);
        let mut s = climbing(54);
        s = step(s, GRAB, &room);
        s = step(s, GRAB, &room);

        assert_eq!(s.speed.y, CLIMB_SLIP_SPEED);
        assert_eq!(s.state, State::Climb);
    }

    #[test]
    fn climbing_past_the_top_of_a_wall_hops() {
        // the wall's top edge is y = 48
        let room = partial_wall(6..23);
        let s = step(climbing(54), GRAB_UP, &room);

        assert_eq!(s.state, State::Normal);
        assert_eq!(s.speed.y, CLIMBHOP_Y);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.climb.hop_wait, 1);
        assert_eq!(s.forced_move_timer, CLIMBHOP_FORCE_TIME);
    }

    #[test]
    fn a_hop_carries_her_onto_the_ledge() {
        let room = partial_wall(6..23);
        let mut s = step(climbing(54), GRAB_UP, &room);

        for _ in 0..40 {
            s = step(s, Input::default(), &room);
        }

        assert!(s.position.x > WALL_X);
        assert_eq!(s.position.y, 48);
        assert_eq!(s.climb.hop_wait, 0);
    }

    #[test]
    fn a_hop_with_no_wall_left_pushes_her_sideways_at_once() {
        let room = partial_wall(0..0);
        let mut s = climbing(100);
        s.speed.y = -10.0;
        s = step(s, GRAB, &room);

        assert_eq!(s.state, State::Normal);
        assert_eq!(s.speed.x, CLIMBHOP_X);
        assert_eq!(s.speed.y, CLIMBHOP_Y);
        assert_eq!(s.climb.hop_wait, 0);
    }

    #[test]
    fn a_hop_follows_the_moving_block_she_is_hopping_onto() {
        let mut room = room();
        room.add_zipper(40, 60, 16, 16, 40, 20);

        // the block was 2 px lower when she last looked
        let mut s = player_at(20, 100, 0.0, 0.0);
        s.climb.hop_solid = Some(HopSolid {
            id: 0,
            position: IVec2 { x: 40, y: 62 },
        });
        s.forced_move_timer = CLIMBHOP_FORCE_TIME;
        s = step(s, Input::default(), &room);

        assert_eq!(s.position.y, 98);
        assert_eq!(s.climb.hop_solid.unwrap().position, IVec2 { x: 40, y: 60 });
    }

    #[test]
    fn she_stops_following_the_block_when_the_hop_lock_ends() {
        let mut room = room();
        room.add_zipper(40, 60, 16, 16, 40, 20);

        let mut s = player_at(20, 100, 0.0, 0.0);
        s.climb.hop_solid = Some(HopSolid {
            id: 0,
            position: IVec2 { x: 40, y: 62 },
        });
        s = step(s, Input::default(), &room);

        assert_eq!(s.position.y, 100);
        assert_eq!(s.climb.hop_solid, None);
    }
}
