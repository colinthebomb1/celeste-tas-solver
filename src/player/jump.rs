use super::climb::climb_in_bounds;
use super::collision::{can_stand, collides, hitbox, hitbox_height, hurtbox};
use super::constants::*;
use super::dash::{dashing, refill_dash};
use super::movement::{add_liftboost, lerp, nudge_v, set_lift_speed};
use super::state::{Frame, Input, Player, set_normal};
use crate::entities::Entities;
use crate::geom::{Direction, Rect};
use crate::level::room::Room;

// starts or keeps up a wall slide. returns her top fall speed for this frame
pub(crate) fn update_wall_slide(
    player: &mut Player,
    room: &Room,
    entities: &Entities,
    input: Input,
) -> f32 {
    let mut max = player.max_fall;
    let into_wall = input.move_x == player.facing || (input.move_x == 0 && input.grab);

    if into_wall && input.move_y != 1 {
        if player.speed.y >= 0.0
            && player.wall.slide_timer > 0.0
            && climb_in_bounds(player, room, player.facing)
            && collides(player, room, entities, player.facing as i32, 0)
            && can_stand(player, room, entities)
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

// what every kind of jump resets. hold_time is how long holding jump keeps her rising
fn begin_jump(player: &mut Player, hold_time: f32) {
    player.coyote_timer = 0.0;
    player.jump_timer = hold_time;
    player.auto_jump = false;
    player.dash.attack_timer = 0.0;
    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.wall.boost_timer = 0.0;
}

pub(crate) fn jump(player: &mut Player, input: Input) {
    begin_jump(player, JUMP_HOLD_TIME);
    player.speed.x += JUMP_X_BOOST * input.move_x as f32;
    player.speed.y = JUMP_SPEED;
    add_liftboost(player);
    player.jump_speed = player.speed.y;
}

pub(crate) fn can_super(
    player: &Player,
    room: &Room,
    entities: &Entities,
    jump_pressed: bool,
) -> bool {
    jump_pressed
        && player.coyote_timer > 0.0
        && player.dash.dir.y == 0.0
        && can_stand(player, room, entities)
}

pub(crate) fn super_or_hyper(player: &mut Player) {
    begin_jump(player, JUMP_HOLD_TIME);
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

fn can_wallbounce(player: &Player) -> bool {
    player.dash.attack_timer > 0.0 && player.dash.dir.x == 0.0 && player.dash.dir.y == -1.0
}

fn can_wall_kick(player: &Player, room: &Room, entities: &Entities, dir: i8) -> bool {
    let dir = dir as i32;
    let mut dist = WALL_KICK_REACH;

    if can_wallbounce(player) {
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

    collides(player, room, entities, dir * dist, 0)
}

pub(crate) fn wall_kick(player: &mut Player, entities: &Entities, dir: i8, move_x: i8) {
    player.crouching = false;
    begin_jump(player, JUMP_HOLD_TIME);

    if move_x != 0 {
        player.forced_move = dir;
        player.forced_move_timer = WALL_KICK_FORCE_TIME;
    }

    if player.lift.speed == NO_LIFT && player.lift.last_speed == NO_LIFT {
        // dir is the way she kicks, so the wall is the other way
        let reach = hitbox(player, -(dir as i32) * WALL_KICK_REACH, 0);
        if let Some(wall) = entities
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

fn wallbounce(player: &mut Player, dir: i8) {
    player.crouching = false;
    begin_jump(player, WALLBOUNCE_JUMP_TIME);

    player.speed.x = WALLBOUNCE_SPEED_X * dir as f32;
    player.speed.y = WALLBOUNCE_SPEED_Y;
    add_liftboost(player);
    player.jump_speed = player.speed.y;
    set_normal(player);
}

pub(crate) fn jump_off_wall(
    player: &mut Player,
    room: &Room,
    entities: &Entities,
    frame: Frame,
) -> bool {
    let input = frame.input;

    if !can_stand(player, room, entities) {
        return false;
    }

    let wall = if can_wall_kick(player, room, entities, 1) {
        1
    } else if can_wall_kick(player, room, entities, -1) {
        -1
    } else {
        return false;
    };

    let dashing_up = dashing(player) && player.dash.dir.x == 0.0 && player.dash.dir.y == -1.0;

    if dashing_up {
        wallbounce(player, -wall);
    } else if can_climb_jump(player, room, entities, input) {
        climb_jump(player, frame);
    } else if can_wallbounce(player) {
        wallbounce(player, -wall);
    } else {
        wall_kick(player, entities, -wall, input.move_x);
    }

    true
}

fn can_climb_jump(player: &Player, room: &Room, entities: &Entities, input: Input) -> bool {
    let able = input.grab && player.stamina > 0.0;

    if can_wall_kick(player, room, entities, 1) {
        player.facing == 1 && able
    } else if can_wall_kick(player, room, entities, -1) {
        player.facing == -1 && able
    } else {
        false
    }
}

pub(crate) fn climb_jump(player: &mut Player, frame: Frame) {
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

fn spring_bounce(player: &mut Player, room: &Room, entities: &Entities, from_y: i32) {
    nudge_v(player, room, entities, (from_y - player.position.y) as f32);

    refill_dash(player);
    player.stamina = CLIMB_MAX_STAMINA;
    set_normal(player);

    begin_jump(player, SPRING_JUMP_TIME);
    // she keeps rising as if jump were held
    player.auto_jump = true;

    player.speed.x = 0.0;
    player.speed.y = SPRING_SPEED;
    player.jump_speed = SPRING_SPEED;
}

pub(crate) fn bounce_on_springs(player: &mut Player, room: &Room, entities: &Entities) {
    for spring in &room.springs {
        if player.speed.y >= 0.0 && spring.hitbox.overlaps(&hurtbox(player)) {
            spring_bounce(player, room, entities, spring.hitbox.top);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Spikes;
    use crate::geom::{IVec2, Vec2};
    use crate::player::collision::on_ground;
    use crate::player::fixtures::*;
    use crate::player::state::{State, jump_pressed};
    use crate::player::step::step;

    #[test]
    fn jump_sets_speed_and_timers() {
        let mut s = standing();
        s.coyote_timer = COYOTE_TIME;
        jump(
            &mut s,
            Input {
                move_x: 0,
                jump: true,
                ..Default::default()
            },
        );
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.jump_speed, JUMP_SPEED);
        assert_eq!(s.jump_timer, JUMP_HOLD_TIME);
        assert_eq!(s.coyote_timer, 0.0);
    }

    #[test]
    fn jump_resets_wall_slide_timer() {
        let mut s = standing();
        s.wall.slide_timer = 0.3;
        jump(
            &mut s,
            Input {
                move_x: 0,
                jump: true,
                ..Default::default()
            },
        );
        assert_eq!(s.wall.slide_timer, WALL_SLIDE_TIME);
    }

    #[test]
    fn wall_slide_starts_at_full_timer() {
        let room = room();
        let mut s = falling_against_wall();
        let max = update_wall_slide(
            &mut s,
            &room,
            &room.initial,
            Input {
                move_x: 1,
                jump: false,
                ..Default::default()
            },
        );
        assert_eq!(max, WALL_SLIDE_START_MAX);
        assert_eq!(s.wall.slide_dir, 1);
    }

    #[test]
    fn wall_slide_max_rises_as_timer_runs_out() {
        let room = room();
        let mut s = falling_against_wall();
        s.wall.slide_timer = WALL_SLIDE_TIME * 0.5;
        let max = update_wall_slide(
            &mut s,
            &room,
            &room.initial,
            Input {
                move_x: 1,
                jump: false,
                ..Default::default()
            },
        );
        assert_eq!(max, lerp(MAX_FALL, WALL_SLIDE_START_MAX, 0.5));
    }

    #[test]
    fn no_wall_slide_without_holding_toward_wall() {
        let room = room();
        let mut s = falling_against_wall();
        assert_eq!(
            update_wall_slide(
                &mut s,
                &room,
                &room.initial,
                Input {
                    move_x: 0,
                    jump: false,
                    ..Default::default()
                }
            ),
            MAX_FALL
        );
        assert_eq!(s.wall.slide_dir, 0);
    }

    #[test]
    fn no_wall_slide_while_moving_up() {
        let room = room();
        let mut s = falling_against_wall();
        s.speed.y = -50.0;
        assert_eq!(
            update_wall_slide(
                &mut s,
                &room,
                &room.initial,
                Input {
                    move_x: 1,
                    jump: false,
                    ..Default::default()
                }
            ),
            MAX_FALL
        );
        assert_eq!(s.wall.slide_dir, 0);
    }

    #[test]
    fn no_wall_slide_away_from_wall() {
        let room = room();
        let mut s = falling_against_wall();
        s.position.x = WALL_X - 5;
        assert_eq!(
            update_wall_slide(
                &mut s,
                &room,
                &room.initial,
                Input {
                    move_x: 1,
                    jump: false,
                    ..Default::default()
                }
            ),
            MAX_FALL
        );
        assert_eq!(s.wall.slide_dir, 0);
    }

    #[test]
    fn jump_adds_horizontal_boost() {
        let mut s = standing();
        s.speed.x = 90.0;
        jump(
            &mut s,
            Input {
                move_x: 1,
                jump: true,
                ..Default::default()
            },
        );
        assert_eq!(s.speed.x, 90.0 + JUMP_X_BOOST);
    }

    #[test]
    fn can_super_during_a_horizontal_dash() {
        let room = room();
        assert!(can_super(&dashing_right(), &room, &room.initial, true));
    }

    #[test]
    fn can_super_needs_a_jump_press() {
        let room = room();
        assert!(!can_super(&dashing_right(), &room, &room.initial, false));
    }

    #[test]
    fn can_super_needs_coyote_time() {
        let room = room();
        let mut s = dashing_right();
        s.coyote_timer = 0.0;
        assert!(!can_super(&s, &room, &room.initial, true));
    }

    #[test]
    fn can_super_not_in_a_vertical_or_diagonal_dash() {
        let room = room();
        let d = 1.0 / 2f32.sqrt();
        let mut s = dashing_right();
        s.dash.dir = Vec2 { x: 0.0, y: -1.0 };
        assert!(!can_super(&s, &room, &room.initial, true));
        s.dash.dir = Vec2 { x: d, y: -d };
        assert!(!can_super(&s, &room, &room.initial, true));
        s.dash.dir = Vec2 { x: 0.0, y: 1.0 };
        assert!(!can_super(&s, &room, &room.initial, true));
    }

    #[test]
    fn super_or_hyper_launches_the_way_she_faces() {
        let mut s = dashing_right();
        super_or_hyper(&mut s);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED,
                y: JUMP_SPEED
            }
        );

        let mut s = dashing_right();
        s.facing = -1;
        super_or_hyper(&mut s);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -SUPER_SPEED,
                y: JUMP_SPEED
            }
        );
    }

    #[test]
    fn super_or_hyper_ends_the_dash_and_starts_a_variable_jump() {
        let mut s = dashing_right();
        s.auto_jump = true;
        s.wall.slide_timer = 0.5;
        super_or_hyper(&mut s);
        assert_eq!(s.state, State::Normal);
        assert_eq!(s.coyote_timer, 0.0);
        assert_eq!(s.jump_timer, JUMP_HOLD_TIME);
        assert_eq!(s.jump_speed, JUMP_SPEED);
        assert_eq!(s.wall.slide_timer, WALL_SLIDE_TIME);
        assert!(!s.auto_jump);
    }

    #[test]
    fn jump_down_with_either_button() {
        assert!(JUMP_1.jump_down());
        assert!(JUMP_2.jump_down());
        assert!(JUMP_BOTH.jump_down());
        assert!(!Input::default().jump_down());
    }

    #[test]
    fn jump_pressed_when_a_button_goes_down() {
        assert!(jump_pressed(&standing(), JUMP_1));
        assert!(jump_pressed(&standing(), JUMP_2));
        assert!(!jump_pressed(&standing(), Input::default()));
    }

    #[test]
    fn jump_pressed_not_while_the_same_button_stays_down() {
        let mut s = standing();
        s.held.jump = true;
        assert!(!jump_pressed(&s, JUMP_1));
        s.held.jump = false;
        s.held.jump2 = true;
        assert!(!jump_pressed(&s, JUMP_2));
    }

    #[test]
    fn jump_pressed_by_one_button_while_the_other_is_held() {
        let mut s = standing();
        s.held.jump = true;
        assert!(jump_pressed(&s, JUMP_BOTH));
        assert!(jump_pressed(&s, JUMP_2));
    }

    #[test]
    fn step_second_button_jumps_while_the_first_is_held() {
        let room = room();
        let mut s = standing();
        s.held.jump = true;
        s = step(s, JUMP_1, &room, &room.initial);
        assert_eq!(s.speed.y, 0.0);
        s = step(s, JUMP_2, &room, &room.initial);
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn step_second_button_holds_a_variable_jump() {
        let room = room();
        let mut s = step(standing(), JUMP_1, &room, &room.initial);
        for _ in 0..5 {
            s = step(s, JUMP_2, &room, &room.initial);
        }
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn can_wall_kick_reaches_two_pixels_of_gap() {
        let room = room();
        assert!(can_wall_kick(
            &player_at(WALL_X, 100, 0.0, 0.0),
            &room,
            &room.initial,
            1
        ));
        assert!(can_wall_kick(
            &player_at(WALL_X - 2, 100, 0.0, 0.0),
            &room,
            &room.initial,
            1
        ));
        assert!(!can_wall_kick(
            &player_at(WALL_X - 3, 100, 0.0, 0.0),
            &room,
            &room.initial,
            1
        ));
    }

    #[test]
    fn can_wall_kick_only_on_the_side_asked() {
        assert!(!can_wall_kick(
            &player_at(WALL_X, 100, 0.0, 0.0),
            &room(),
            &room().initial,
            -1
        ));

        let room = left_wall_room();
        assert!(can_wall_kick(
            &player_at(LEFT_WALL_X, 100, 0.0, 0.0),
            &room,
            &room.initial,
            -1
        ));
        assert!(can_wall_kick(
            &player_at(LEFT_WALL_X + 2, 100, 0.0, 0.0),
            &room,
            &room.initial,
            -1
        ));
        assert!(!can_wall_kick(
            &player_at(LEFT_WALL_X + 3, 100, 0.0, 0.0),
            &room,
            &room.initial,
            -1
        ));
        assert!(!can_wall_kick(
            &player_at(LEFT_WALL_X, 100, 0.0, 0.0),
            &room,
            &room.initial,
            1
        ));
    }

    #[test]
    fn can_climb_jump_facing_a_wall_with_grab_held() {
        let room = room();
        let left_wall = left_wall_room();
        assert!(can_climb_jump(
            &player_at(WALL_X, 100, 0.0, 0.0),
            &room,
            &room.initial,
            JUMP_GRAB
        ));

        let mut s = player_at(LEFT_WALL_X, 100, 0.0, 0.0);
        s.facing = -1;
        assert!(can_climb_jump(
            &s,
            &left_wall,
            &left_wall.initial,
            JUMP_GRAB
        ));
    }

    #[test]
    fn can_climb_jump_needs_grab() {
        let room = room();
        let no_grab = Input {
            grab: false,
            ..JUMP_GRAB
        };
        assert!(!can_climb_jump(
            &player_at(WALL_X, 100, 0.0, 0.0),
            &room,
            &room.initial,
            no_grab
        ));
    }

    #[test]
    fn can_climb_jump_needs_stamina() {
        let room = room();
        let mut s = player_at(WALL_X, 100, 0.0, 0.0);
        s.stamina = 0.0;
        assert!(!can_climb_jump(&s, &room, &room.initial, JUMP_GRAB));
    }

    #[test]
    fn can_climb_jump_needs_a_wall() {
        let room = room();
        assert!(!can_climb_jump(
            &player_at(WALL_X - 3, 100, 0.0, 0.0),
            &room,
            &room.initial,
            JUMP_GRAB
        ));
    }

    #[test]
    fn can_climb_jump_not_facing_away_from_the_wall() {
        let room = room();
        let left_wall = left_wall_room();
        let mut s = player_at(WALL_X, 100, 0.0, 0.0);
        s.facing = -1;
        assert!(!can_climb_jump(&s, &room, &room.initial, JUMP_GRAB));

        let s = player_at(LEFT_WALL_X, 100, 0.0, 0.0);
        assert!(!can_climb_jump(
            &s,
            &left_wall,
            &left_wall.initial,
            JUMP_GRAB
        ));
    }

    #[test]
    fn climb_jump_in_the_air_costs_stamina() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.state = State::Dash { timer: 0.1 };
        climb_jump(
            &mut s,
            frame(
                Input {
                    move_x: 1,
                    ..JUMP_GRAB
                },
                false,
            ),
        );
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
        assert_eq!(
            s.speed,
            Vec2 {
                x: JUMP_X_BOOST,
                y: JUMP_SPEED
            }
        );
        assert_eq!(s.jump_timer, JUMP_HOLD_TIME);
        assert_eq!(s.state, State::Normal);
    }

    #[test]
    fn climb_jump_on_the_ground_is_free() {
        let mut s = standing();
        climb_jump(&mut s, frame(JUMP_GRAB, true));
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn super_or_hyper_while_crouching_is_a_hyper() {
        let mut s = dashing_right();
        s.crouching = true;
        super_or_hyper(&mut s);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED * HYPER_X_MULT,
                y: JUMP_SPEED * HYPER_Y_MULT
            }
        );
        assert_eq!(s.jump_speed, JUMP_SPEED * HYPER_Y_MULT);
        assert!(!s.crouching);
    }

    #[test]
    fn can_super_needs_room_to_stand_up() {
        let room = tunnel_room();
        let mut s = crouching_at(40, FLOOR_Y);
        s.state = State::Dash { timer: DASH_TIME };
        s.dash.dir = Vec2 { x: 1.0, y: 0.0 };
        s.coyote_timer = COYOTE_TIME;
        assert!(!can_super(&s, &room, &room.initial, true));
        s.position.x = 8;
        assert!(can_super(&s, &room, &room.initial, true));
    }

    #[test]
    fn step_stays_crouched_when_falling_in_coyote_time() {
        let room = room();
        let mut s = crouching_at(0, 64);
        s.speed.y = 50.0;
        s.coyote_timer = COYOTE_TIME;
        assert!(step(s, Input::default(), &room, &room.initial).crouching);
    }

    #[test]
    fn climb_jump_without_a_direction_arms_the_wallboost() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        climb_jump(&mut s, frame(JUMP_GRAB, false));
        assert_eq!(s.wall.boost_dir, -1);
        assert_eq!(s.wall.boost_timer, WALLBOOST_TIME);

        let mut s = player_at(LEFT_WALL_X, 100, 0.0, 30.0);
        s.facing = -1;
        climb_jump(&mut s, frame(JUMP_GRAB, false));
        assert_eq!(s.wall.boost_dir, 1);
    }

    #[test]
    fn climb_jump_with_a_direction_does_not_arm_the_wallboost() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        climb_jump(
            &mut s,
            frame(
                Input {
                    move_x: 1,
                    ..JUMP_GRAB
                },
                false,
            ),
        );
        assert_eq!(s.wall.boost_timer, 0.0);
    }

    #[test]
    fn jumps_cancel_the_wallboost() {
        let mut s = standing();
        s.wall.boost_timer = 0.1;
        jump(&mut s, Input::default());
        assert_eq!(s.wall.boost_timer, 0.0);

        let mut s = dashing_right();
        s.wall.boost_timer = 0.1;
        super_or_hyper(&mut s);
        assert_eq!(s.wall.boost_timer, 0.0);
    }

    #[test]
    fn step_wallboost_ignores_the_other_direction() {
        let room = room();
        let mut s = step(
            wallboosting(),
            Input { move_x: 1, ..LEFT },
            &room,
            &room.initial,
        );
        s = step(s, Input::default(), &room, &room.initial);
        assert_eq!(s.stamina, 50.0);
        assert!(s.wall.boost_timer > 0.0);
    }

    #[test]
    fn step_wallboost_runs_out() {
        let room = room();
        let mut s = wallboosting();
        s.wall.boost_timer = 0.01;
        s = step(s, LEFT, &room, &room.initial);
        s = step(s, LEFT, &room, &room.initial);
        assert_eq!(s.stamina, 50.0);
    }

    #[test]
    fn wallbounce_is_faster_and_higher_than_a_jump() {
        let mut s = bouncing_at(LEFT_WALL_X);
        s.crouching = true;
        s.auto_jump = true;
        wallbounce(&mut s, 1);
        assert_eq!(
            s.speed,
            Vec2 {
                x: WALLBOUNCE_SPEED_X,
                y: WALLBOUNCE_SPEED_Y
            }
        );
        assert_eq!(s.jump_speed, WALLBOUNCE_SPEED_Y);
        assert_eq!(s.jump_timer, WALLBOUNCE_JUMP_TIME);
        assert_eq!(s.dash.attack_timer, 0.0);
        assert_eq!(s.state, State::Normal);
        assert!(!s.crouching);
        assert!(!s.auto_jump);
    }

    #[test]
    fn can_wall_kick_reaches_further_for_a_wallbounce() {
        let room = room();
        assert!(can_wall_kick(
            &bouncing_at(WALL_X - 4),
            &room,
            &room.initial,
            1
        ));
        assert!(!can_wall_kick(
            &bouncing_at(WALL_X - 5),
            &room,
            &room.initial,
            1
        ));
        assert!(!can_wall_kick(
            &player_at(WALL_X - 4, 100, 0.0, 0.0),
            &room,
            &room.initial,
            1
        ));
    }

    #[test]
    fn can_wall_kick_is_short_again_when_the_wall_has_spikes() {
        let mut room = room();
        room.spikes
            .push(Spikes::new(Direction::Left, WALL_X + 4, 88, 16));
        assert!(!can_wall_kick(
            &bouncing_at(WALL_X - 4),
            &room,
            &room.initial,
            1
        ));
        assert!(can_wall_kick(
            &bouncing_at(WALL_X - 2),
            &room,
            &room.initial,
            1
        ));
    }

    #[test]
    fn wall_kick_pushes_off_the_wall() {
        let mut s = crouching_at(WALL_X, 100);
        s.state = State::Dash { timer: 0.1 };
        s.dash.attack_timer = 0.2;
        s.auto_jump = true;
        wall_kick(&mut s, &room().initial, -1, 0);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALL_KICK_SPEED,
                y: JUMP_SPEED
            }
        );
        assert_eq!(s.jump_speed, JUMP_SPEED);
        assert_eq!(s.jump_timer, JUMP_HOLD_TIME);
        assert_eq!(s.state, State::Normal);
        assert_eq!(s.dash.attack_timer, 0.0);
        assert!(!s.crouching);
        assert!(!s.auto_jump);
    }

    #[test]
    fn wall_kick_forces_her_direction_only_if_one_is_held() {
        let mut s = player_at(WALL_X, 100, 0.0, 0.0);
        wall_kick(&mut s, &room().initial, -1, 0);
        assert_eq!(s.forced_move_timer, 0.0);

        let mut s = player_at(WALL_X, 100, 0.0, 0.0);
        wall_kick(&mut s, &room().initial, -1, 1);
        assert_eq!(s.forced_move, -1);
        assert_eq!(s.forced_move_timer, WALL_KICK_FORCE_TIME);
    }

    #[test]
    fn step_no_wall_kick_away_from_walls() {
        let room = room();
        let s = step(player_at(40, 100, 0.0, 30.0), JUMP, &room, &room.initial);
        assert_eq!(s.speed.x, 0.0);
        assert!(s.speed.y > 0.0);
    }

    #[test]
    fn standing_on_a_jump_thru_is_on_the_ground() {
        let room = platform_room();
        assert!(on_ground(
            &player_at(60, PLATFORM_Y, 0.0, 0.0),
            &room,
            &room.initial
        ));
        assert!(!on_ground(
            &player_at(60, PLATFORM_Y - 1, 0.0, 0.0),
            &room,
            &room.initial
        ));
        assert!(!on_ground(
            &player_at(30, PLATFORM_Y, 0.0, 0.0),
            &room,
            &room.initial
        ));
    }

    #[test]
    fn inside_a_jump_thru_is_not_on_the_ground() {
        let room = platform_room();
        assert!(!on_ground(
            &player_at(60, PLATFORM_Y + 2, 0.0, 0.0),
            &room,
            &room.initial
        ));
    }

    #[test]
    fn step_rising_through_a_jump_thru_is_pushed_up() {
        let platform = platform_room();
        let room = room();
        let start = player_at(60, PLATFORM_Y + 3, 0.0, -50.0);
        let with = step(start, Input::default(), &platform, &platform.initial);
        let without = step(start, Input::default(), &room, &room.initial);
        let push = exact_y(&with) - exact_y(&without);
        assert!(
            (push - JUMP_THRU_ASSIST_SPEED * DELTA_TIME).abs() < 1e-4,
            "pushed {push}"
        );
    }

    #[test]
    fn step_no_push_up_through_a_jump_thru_with_spikes_on_it() {
        let spiked = spiked_platform_room();
        let room = room();
        let start = player_at(60, PLATFORM_Y + 3, 0.0, -50.0);
        let with = step(start, Input::default(), &spiked, &spiked.initial);
        let without = step(start, Input::default(), &room, &room.initial);
        assert!(!with.dead);
        assert_eq!(exact_y(&with), exact_y(&without));
    }

    #[test]
    fn step_no_push_when_falling_through_a_jump_thru() {
        let platform = platform_room();
        let room = room();
        let start = player_at(60, PLATFORM_Y + 3, 0.0, 50.0);
        let with = step(start, Input::default(), &platform, &platform.initial);
        let without = step(start, Input::default(), &room, &room.initial);
        assert_eq!(with, without);
    }

    #[test]
    fn step_stands_on_a_jump_thru() {
        let room = platform_room();
        let mut s = player_at(60, PLATFORM_Y, 0.0, 0.0);
        for _ in 0..10 {
            s = step(s, Input::default(), &room, &room.initial);
        }
        assert_eq!(
            s.position,
            IVec2 {
                x: 60,
                y: PLATFORM_Y
            }
        );
        assert_eq!(s.coyote_timer, COYOTE_TIME);
    }

    #[test]
    fn step_jumps_off_a_jump_thru() {
        let room = platform_room();
        let s = step(
            player_at(60, PLATFORM_Y, 0.0, 0.0),
            JUMP,
            &room,
            &room.initial,
        );
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn jumps_add_liftboost() {
        let mut s = lifted(standing(), 50.0, -70.0);
        jump(&mut s, Input::default());
        assert_eq!(
            s.speed,
            Vec2 {
                x: 50.0,
                y: JUMP_SPEED - 70.0
            }
        );
        assert_eq!(s.jump_speed, JUMP_SPEED - 70.0);

        let mut s = lifted(dashing_right(), 0.0, -70.0);
        super_or_hyper(&mut s);
        assert_eq!(
            s.speed,
            Vec2 {
                x: SUPER_SPEED,
                y: JUMP_SPEED - 70.0
            }
        );

        let mut s = lifted(player_at(WALL_X, 100, 0.0, 0.0), 0.0, -70.0);
        wall_kick(&mut s, &room().initial, -1, 0);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALL_KICK_SPEED,
                y: JUMP_SPEED - 70.0
            }
        );

        let mut s = lifted(bouncing_at(WALL_X), 0.0, -70.0);
        wallbounce(&mut s, -1);
        assert_eq!(
            s.speed,
            Vec2 {
                x: -WALLBOUNCE_SPEED_X,
                y: WALLBOUNCE_SPEED_Y - 70.0
            }
        );
    }

    #[test]
    fn hyper_halves_the_lift_too() {
        let mut s = lifted(dashing_right(), 0.0, -70.0);
        s.crouching = true;
        super_or_hyper(&mut s);
        assert_eq!(s.speed.y, (JUMP_SPEED - 70.0) * HYPER_Y_MULT);
    }

    #[test]
    fn wall_kick_takes_lift_from_a_moving_wall_on_her_right() {
        let mut room = room();
        room.initial.add_zipper(60, 80, 16, 40, 60, 0);
        room.initial.solids[room.initial.zippers[0].solid_id].lift_speed =
            Vec2 { x: 0.0, y: -90.0 };

        let mut s = player_at(54, 100, 0.0, 0.0);
        wall_kick(&mut s, &room.initial, -1, 0);
        assert_eq!(s.speed.y, JUMP_SPEED - 90.0);
    }

    // a rising block whose right edge is 2 px from her left side
    fn rising_block_on_her_left() -> (Entities, Player) {
        let mut entities = Entities::default();
        let id = entities.add_solid(Rect::new(40, 80, 16, 40));
        entities.solids[id].lift_speed = Vec2 { x: 0.0, y: -90.0 };
        (entities, player_at(62, 100, 0.0, 0.0))
    }

    #[test]
    fn wall_kick_takes_lift_from_a_moving_wall_on_her_left() {
        let (entities, mut s) = rising_block_on_her_left();
        wall_kick(&mut s, &entities, 1, 0);
        assert_eq!(s.speed.y, JUMP_SPEED - 90.0);
    }

    #[test]
    fn wall_kick_takes_no_lift_from_a_block_behind_her() {
        let (entities, mut s) = rising_block_on_her_left();
        wall_kick(&mut s, &entities, -1, 0);
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn no_wall_slide_on_a_wall_at_the_rooms_edge() {
        // the room is 88 px wide, and this block starts 1 px inside its right edge
        let mut room = crate::fixtures::open_room();
        room.initial.add_solid(Rect::new(87, 0, 8, 184));
        let mut s = falling_against_wall();
        s.position.x = 83;

        let toward = Input {
            move_x: 1,
            ..Default::default()
        };
        let max = update_wall_slide(&mut s, &room, &room.initial, toward);
        assert_eq!(max, s.max_fall);
        assert_eq!(s.wall.slide_dir, 0);
    }

    #[test]
    fn spring_bounce_keeps_a_second_dash() {
        let room = spring_room();
        let mut s = dashing_at(20, FLOOR_Y, 200.0, 50.0);
        s.dashes = TWO_DASHES;
        spring_bounce(&mut s, &room, &room.initial, FLOOR_Y - 6);
        assert_eq!(s.dashes, TWO_DASHES);
    }

    #[test]
    fn spring_bounce_launches_her_straight_up() {
        let room = spring_room();
        let mut s = dashing_at(20, FLOOR_Y, 200.0, 50.0);
        s.dashes = 0;
        s.stamina = 20.0;
        s.dash.attack_timer = 0.2;
        spring_bounce(&mut s, &room, &room.initial, FLOOR_Y - 6);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: SPRING_SPEED
            }
        );
        assert_eq!(s.jump_speed, SPRING_SPEED);
        assert_eq!(s.jump_timer, SPRING_JUMP_TIME);
        assert_eq!(s.state, State::Normal);
        assert_eq!(s.dashes, NORMAL_DASHES);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.dash.attack_timer, 0.0);
        assert_eq!(s.coyote_timer, 0.0);
        assert!(s.auto_jump);
    }

    #[test]
    fn spring_bounce_lifts_her_to_the_springs_top_and_keeps_her_subpixel() {
        let room = spring_room();
        let mut s = player_at(20, FLOOR_Y - 2, 0.0, 50.0);
        s.subpixel.y = -0.25;
        spring_bounce(&mut s, &room, &room.initial, FLOOR_Y - 6);
        assert_eq!(s.position.y, FLOOR_Y - 6);
        assert_eq!(s.subpixel.y, -0.25);
    }

    #[test]
    fn step_landing_on_a_spring_bounces() {
        let room = spring_room();
        let s = step(
            player_at(20, FLOOR_Y - 4, 0.0, 120.0),
            Input::default(),
            &room,
            &room.initial,
        );
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: SPRING_SPEED
            }
        );
        assert_eq!(s.position.y, FLOOR_Y - 6);
        assert!(!s.dead);
    }

    #[test]
    fn step_running_into_a_spring_bounces() {
        let room = spring_room();
        let mut s = standing();
        let right = Input {
            move_x: 1,
            ..Input::default()
        };
        let mut frames = 0;
        while s.speed.y == 0.0 && frames < 30 {
            s = step(s, right, &room, &room.initial);
            frames += 1;
        }
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: SPRING_SPEED
            }
        );
    }

    #[test]
    fn step_rising_past_a_spring_does_not_bounce() {
        let room = spring_room();
        let s = step(
            player_at(20, FLOOR_Y - 2, 0.0, -100.0),
            Input::default(),
            &room,
            &room.initial,
        );
        assert!(s.speed.y > SPRING_SPEED);
        assert_eq!(s.jump_timer, 0.0);
    }

    #[test]
    fn step_no_bounce_beside_a_spring() {
        let room = spring_room();
        let s = step(
            player_at(34, FLOOR_Y - 4, 0.0, 120.0),
            Input::default(),
            &room,
            &room.initial,
        );
        assert!(s.speed.y >= 0.0);
    }

    #[test]
    fn step_spring_bounce_holds_its_height_without_jump_held() {
        let room = spring_room();
        let mut s = step(
            player_at(20, FLOOR_Y - 4, 0.0, 120.0),
            Input::default(),
            &room,
            &room.initial,
        );
        for _ in 0..5 {
            s = step(s, Input::default(), &room, &room.initial);
        }
        assert_eq!(s.speed.y, SPRING_SPEED);
    }

    #[test]
    fn no_wall_slide_while_holding_down() {
        let room = room();
        let mut s = player_at(WALL_X, 100, 0.0, 50.0);
        let max = update_wall_slide(&mut s, &room, &room.initial, Input { move_x: 1, ..DOWN });
        assert_eq!(max, MAX_FALL);
        assert_eq!(s.wall.slide_dir, 0);
    }

    #[test]
    fn wall_slide_with_grab_held_and_no_direction() {
        let room = room();
        let mut s = player_at(WALL_X, 100, 0.0, 50.0);
        let max = update_wall_slide(
            &mut s,
            &room,
            &room.initial,
            Input {
                grab: true,
                ..Input::default()
            },
        );
        assert_eq!(s.wall.slide_dir, 1);
        assert_eq!(max, WALL_SLIDE_START_MAX);
    }

    #[test]
    fn wall_slide_stands_her_up() {
        let room = room();
        let mut s = crouching_at(WALL_X, 100);
        s.speed.y = 50.0;
        update_wall_slide(
            &mut s,
            &room,
            &room.initial,
            Input {
                move_x: 1,
                ..Input::default()
            },
        );
        assert!(!s.crouching);
    }
}
