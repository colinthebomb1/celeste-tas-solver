use crate::entities::Entities;
use crate::entities::dash_block::DASH_BLOCK_FREEZE_TIME;
use crate::geom::Direction;
use crate::level::room::Chapter;
use crate::player::Player;
use crate::player::collision::{collides, hitbox};
use crate::player::constants::{
    CLIMB_MAX_STAMINA, DASH_COOLDOWN, HITBOX_WIDTH, JUMP_HOLD_TIME, JUMP_SPEED, WALL_SLIDE_TIME,
};
use crate::player::dash::refill_dash;
use crate::player::state::set_normal;

// frames a room change takes
pub const TRANSITION_FRAMES: usize = 40;
// how far the game looks past her center for the next room
const SIDE_REACH: f32 = 8.0;
const VERTICAL_REACH: f32 = 12.0;
// she is moved the way she is travelling until she is this far inside the new room
const ENTRY_PAD: i32 = 4;
const DOWN_ENTRY_PAD: i32 = 12;
// with no room above, she can rise this far past the top before it stops her
const TOP_MARGIN: i32 = 24;
// no transition down while there is floor this close under her
const DOWN_FLOOR_CHECK: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    // indexes into Chapter::rooms
    pub from: usize,
    pub to: usize,
    pub direction: Direction,
}

// what happens when she reaches an edge of her room. a transition starts if there is a room to
// go to. where there isn't, the left, right and top edges hold her in.
// entities are those of the room she is in
pub fn check_room_edges(
    player: &mut Player,
    chapter: &Chapter,
    room: usize,
    entities: &Entities,
) -> Option<Transition> {
    let bounds = chapter.rooms[room].bounds;
    let to = |x: f32, y: f32, direction: Direction| {
        chapter
            .room_at(x, y)
            .filter(|&to| to != room)
            .map(|to| Transition {
                from: room,
                to,
                direction,
            })
    };

    let her = hitbox(player, 0, 0);
    let (center_x, center_y) = center(player);
    // a sideways transition needs her whole height inside the room's
    let beside = her.top >= bounds.top && her.bottom() < bounds.bottom();

    if her.left < bounds.left {
        let next = to(center_x - SIDE_REACH, center_y, Direction::Left);
        if beside && next.is_some() {
            return next;
        }
        player.position.x = bounds.left + HITBOX_WIDTH / 2;
        player.speed.x = 0.0;
    } else if her.right() > bounds.right() {
        let next = to(center_x + SIDE_REACH, center_y, Direction::Right);
        if beside && next.is_some() {
            return next;
        }
        player.position.x = bounds.right() - HITBOX_WIDTH / 2;
        player.speed.x = 0.0;
    }

    // up and down are checked from where the sides left her
    let her = hitbox(player, 0, 0);
    let (center_x, center_y) = center(player);

    if center_y < bounds.top as f32 {
        let next = to(center_x, center_y - VERTICAL_REACH, Direction::Up);
        if next.is_some() {
            return next;
        }
        if her.top < bounds.top - TOP_MARGIN {
            player.position.y = bounds.top - TOP_MARGIN + her.height;
            player.speed.y = 0.0;
        }
    }

    let floor_near = collides(player, &chapter.rooms[room], entities, 0, DOWN_FLOOR_CHECK);
    if her.bottom() > bounds.bottom() && !floor_near {
        return to(center_x, center_y + VERTICAL_REACH, Direction::Down);
    }

    None
}

// the middle of her hitbox
fn center(player: &Player) -> (f32, f32) {
    let her = hitbox(player, 0, 0);
    (
        player.position.x as f32,
        player.position.y as f32 - her.height as f32 / 2.0,
    )
}

pub fn before_transition(player: &mut Player, direction: Direction) {
    match direction {
        Direction::Up => {
            player.speed.x = 0.0;
            player.speed.y = JUMP_SPEED;
            player.jump_speed = JUMP_SPEED;
            player.jump_timer = JUMP_HOLD_TIME;
            player.auto_jump = true;
            set_normal(player);
            player.dash.cooldown = DASH_COOLDOWN;
        }
        Direction::Down => {
            player.speed.y = player.speed.y.max(0.0);
            player.jump_timer = 0.0;
            player.auto_jump = false;
            set_normal(player);
        }
        Direction::Left | Direction::Right => {}
    }
}

// entities are those of the room she is leaving
pub fn finish_transition(
    player: &mut Player,
    chapter: &Chapter,
    entities: &Entities,
    transition: Transition,
) {
    let bounds = chapter.rooms[transition.to].bounds;

    // she only moves if she isn't far enough in already
    let position = &mut player.position;
    match transition.direction {
        Direction::Up => position.y = position.y.min(bounds.bottom() - ENTRY_PAD - 1),
        Direction::Down => position.y = position.y.max(bounds.top + DOWN_ENTRY_PAD),
        Direction::Right => position.x = position.x.max(bounds.left + ENTRY_PAD),
        Direction::Left => position.x = position.x.min(bounds.right() - ENTRY_PAD - 1),
    }
    if matches!(transition.direction, Direction::Left | Direction::Right) {
        position.y = position.y.min(bounds.bottom() - 1);
    }

    player.subpixel.x = 0.0;
    player.subpixel.y = 0.0;
    player.speed.x = player.speed.x.round_ties_even();
    player.speed.y = player.speed.y.round_ties_even();

    player.wall.slide_timer = WALL_SLIDE_TIME;
    player.coyote_timer = 0.0;
    player.forced_move_timer = 0.0;
    refill_dash(player);
    player.stamina = CLIMB_MAX_STAMINA;

    // the room she left is unloaded now, and every dash block still in it freezes the game
    let left_a_dash_block = entities
        .dash_blocks
        .iter()
        .any(|block| entities.solids[block.solid_id].collidable);
    if left_a_dash_block {
        player.freeze_timer = player.freeze_timer.max(DASH_BLOCK_FREEZE_TIME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::at;
    use crate::geom::IVec2;
    use crate::geom::Vec2;
    use crate::level::room::Room;
    use crate::player::State;
    use crate::player::constants::{NORMAL_DASHES, TWO_DASHES};

    const OPEN: &str = "0000\n0000\n0000\n0000";

    // four 32 px rooms around a centre one, which sits at (100, 100):
    //          up
    //   left centre right
    //         down
    fn cross() -> Chapter {
        let room = |name: &str, x: i32, y: i32| {
            let mut room = Room::from_tiles(IVec2 { x, y }, OPEN);
            room.name = name.to_string();
            room
        };
        Chapter {
            rooms: vec![
                room("centre", 100, 100),
                room("up", 100, 68),
                room("right", 132, 100),
                room("down", 100, 132),
                room("left", 68, 100),
            ],
        }
    }

    fn find(player: &Player, chapter: &Chapter, room: usize) -> Option<Transition> {
        check(&mut player.clone(), chapter, room)
    }

    fn check(player: &mut Player, chapter: &Chapter, room: usize) -> Option<Transition> {
        check_room_edges(player, chapter, room, &chapter.rooms[room].initial)
    }

    // one room at (100, 100), 32 px each way, with nothing around it
    fn alone() -> Chapter {
        let mut chapter = cross();
        chapter.rooms.truncate(1);
        chapter
    }

    #[test]
    fn the_left_edge_holds_her_in_when_there_is_no_room_there() {
        let chapter = alone();
        let mut s = at(103, 120);
        s.speed.x = -90.0;
        s.subpixel.x = 0.25;

        assert_eq!(check(&mut s, &chapter, CENTRE), None);
        assert_eq!(s.position, IVec2 { x: 104, y: 120 });
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.subpixel.x, 0.25);
    }

    #[test]
    fn the_right_edge_holds_her_in_when_there_is_no_room_there() {
        let chapter = alone();
        let mut s = at(129, 120);
        s.speed.x = 90.0;

        assert_eq!(check(&mut s, &chapter, CENTRE), None);
        assert_eq!(s.position, IVec2 { x: 128, y: 120 });
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn inside_the_room_she_is_left_alone() {
        let chapter = alone();
        let mut s = at(104, 120);
        s.speed.x = -90.0;

        assert_eq!(check(&mut s, &chapter, CENTRE), None);
        assert_eq!(s.position, IVec2 { x: 104, y: 120 });
        assert_eq!(s.speed.x, -90.0);
    }

    #[test]
    fn she_can_rise_24_px_above_a_room_with_nothing_over_it() {
        let chapter = alone();

        // the top of her hitbox is exactly 24 px above the room
        let mut s = at(116, 87);
        s.speed.y = -105.0;
        assert_eq!(check(&mut s, &chapter, CENTRE), None);
        assert_eq!(s.position.y, 87);
        assert_eq!(s.speed.y, -105.0);

        let mut s = at(116, 80);
        s.speed.y = -105.0;
        assert_eq!(check(&mut s, &chapter, CENTRE), None);
        assert_eq!(s.position.y, 87);
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn a_side_she_cannot_leave_by_holds_her_even_with_a_room_there() {
        // too high to count as beside the room on the right
        let chapter = cross();
        let mut s = at(129, 110);
        s.speed.x = 90.0;

        assert_eq!(check(&mut s, &chapter, CENTRE), None);
        assert_eq!(s.position.x, 128);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn up_is_still_checked_after_a_side_holds_her() {
        let mut chapter = cross();
        chapter.rooms.truncate(2);

        // past the left edge with no room there, and high enough to leave upward
        let mut s = at(103, 105);
        let found = check(&mut s, &chapter, CENTRE);
        assert_eq!(found.map(|t| t.direction), Some(Direction::Up));
        assert_eq!(s.position.x, 104);
    }

    fn finish(player: &mut Player, chapter: &Chapter, transition: Transition) {
        let entities = &chapter.rooms[transition.from].initial;
        finish_transition(player, chapter, entities, transition);
    }

    const CENTRE: usize = 0;
    const UP: usize = 1;
    const RIGHT: usize = 2;
    const DOWN: usize = 3;
    const LEFT: usize = 4;

    #[test]
    fn no_transition_inside_the_room() {
        assert_eq!(find(&at(116, 120), &cross(), CENTRE), None);
    }

    #[test]
    fn right_when_her_hitbox_passes_the_edge() {
        let chapter = cross();
        assert_eq!(find(&at(128, 120), &chapter, CENTRE), None);
        let found = find(&at(129, 120), &chapter, CENTRE);
        assert_eq!(
            found,
            Some(Transition {
                from: CENTRE,
                to: RIGHT,
                direction: Direction::Right
            })
        );
    }

    #[test]
    fn left_when_her_hitbox_passes_the_edge() {
        let chapter = cross();
        assert_eq!(find(&at(104, 120), &chapter, CENTRE), None);
        let found = find(&at(103, 120), &chapter, CENTRE);
        assert_eq!(
            found,
            Some(Transition {
                from: CENTRE,
                to: LEFT,
                direction: Direction::Left
            })
        );
    }

    #[test]
    fn up_when_her_centre_passes_the_edge() {
        let chapter = cross();
        assert_eq!(find(&at(116, 106), &chapter, CENTRE), None);
        let found = find(&at(116, 105), &chapter, CENTRE);
        assert_eq!(
            found,
            Some(Transition {
                from: CENTRE,
                to: UP,
                direction: Direction::Up
            })
        );
    }

    #[test]
    fn up_is_later_when_crouching() {
        let chapter = cross();
        let mut s = at(116, 103);
        s.crouching = true;
        assert_eq!(find(&s, &chapter, CENTRE), None);
        s.position.y = 102;
        assert_eq!(find(&s, &chapter, CENTRE).map(|t| t.to), Some(UP));
    }

    #[test]
    fn down_when_her_feet_pass_the_edge() {
        let chapter = cross();
        assert_eq!(find(&at(116, 132), &chapter, CENTRE), None);
        let found = find(&at(116, 133), &chapter, CENTRE);
        assert_eq!(found.map(|t| t.direction), Some(Direction::Down));
    }

    #[test]
    fn no_transition_without_a_room_there() {
        let chapter = cross();
        assert_eq!(find(&at(116, 60), &chapter, UP), None);
        assert_eq!(find(&at(170, 120), &chapter, RIGHT), None);
    }

    #[test]
    fn sideways_needs_her_whole_body_level_with_the_room() {
        let chapter = cross();
        assert_eq!(find(&at(129, 110), &chapter, CENTRE), None);
    }

    #[test]
    fn before_up_gives_her_a_jump() {
        let mut s = at(116, 105);
        s.speed = Vec2 {
            x: 200.0,
            y: -240.0,
        };
        s.state = State::Dash { timer: 0.1 };
        before_transition(&mut s, Direction::Up);
        assert_eq!(
            s.speed,
            Vec2 {
                x: 0.0,
                y: JUMP_SPEED
            }
        );
        assert_eq!(s.jump_speed, JUMP_SPEED);
        assert_eq!(s.jump_timer, JUMP_HOLD_TIME);
        assert!(s.auto_jump);
        assert_eq!(s.state, State::Normal);
        assert_eq!(s.dash.cooldown, DASH_COOLDOWN);
    }

    #[test]
    fn before_sideways_changes_nothing() {
        let mut s = at(129, 120);
        s.speed = Vec2 {
            x: 200.0,
            y: -170.0,
        };
        s.state = State::Dash { timer: 0.1 };
        let before = s;
        before_transition(&mut s, Direction::Right);
        assert_eq!(s, before);
    }

    #[test]
    fn before_down_stops_her_rising() {
        let mut s = at(116, 133);
        s.speed = Vec2 { x: 50.0, y: -20.0 };
        s.jump_timer = 0.1;
        s.auto_jump = true;
        before_transition(&mut s, Direction::Down);
        assert_eq!(s.speed, Vec2 { x: 50.0, y: 0.0 });
        assert_eq!(s.jump_timer, 0.0);
        assert!(!s.auto_jump);
    }

    #[test]
    fn finish_up_places_her_five_pixels_above_the_new_rooms_bottom() {
        let chapter = cross();
        let mut s = at(116, 105);
        s.subpixel = Vec2 { x: 0.25, y: -0.25 };
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: UP,
                direction: Direction::Up,
            },
        );
        assert_eq!(s.position, IVec2 { x: 116, y: 95 });
        assert_eq!(s.subpixel, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn finish_right_places_her_against_the_new_rooms_left_edge() {
        let chapter = cross();
        let mut s = at(129, 120);
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: RIGHT,
                direction: Direction::Right,
            },
        );
        assert_eq!(s.position, IVec2 { x: 136, y: 120 });
    }

    #[test]
    fn finish_left_and_down_use_their_own_distances() {
        let chapter = cross();
        let mut s = at(103, 120);
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: LEFT,
                direction: Direction::Left,
            },
        );
        assert_eq!(s.position, IVec2 { x: 95, y: 120 });

        let mut s = at(116, 133);
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: DOWN,
                direction: Direction::Down,
            },
        );
        assert_eq!(s.position, IVec2 { x: 116, y: 144 });
    }

    #[test]
    fn finish_leaves_her_where_she_is_if_she_is_already_far_enough_in() {
        let chapter = cross();
        let mut s = at(116, 90);
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: UP,
                direction: Direction::Up,
            },
        );
        assert_eq!(s.position, IVec2 { x: 116, y: 90 });

        let mut s = at(140, 120);
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: RIGHT,
                direction: Direction::Right,
            },
        );
        assert_eq!(s.position, IVec2 { x: 140, y: 120 });
    }

    #[test]
    fn leaving_a_room_with_a_dash_block_freezes_the_game() {
        let mut chapter = cross();
        chapter.rooms[CENTRE].initial.add_dash_block(100, 100, 8, 8);
        let up = Transition {
            from: CENTRE,
            to: UP,
            direction: Direction::Up,
        };

        let mut s = at(116, 105);
        finish(&mut s, &chapter, up);
        assert_eq!(s.freeze_timer, DASH_BLOCK_FREEZE_TIME);

        // not if it was broken first, and not for a dash block in the room she enters
        chapter.rooms[CENTRE].initial.solids[0].collidable = false;
        chapter.rooms[UP].initial.add_dash_block(100, 68, 8, 8);
        let mut s = at(116, 105);
        finish(&mut s, &chapter, up);
        assert_eq!(s.freeze_timer, 0.0);
    }

    #[test]
    fn finish_ends_a_forced_move_and_keeps_a_second_dash() {
        let chapter = cross();
        let mut s = at(129, 120);
        s.forced_move = -1;
        s.forced_move_timer = 0.1;
        s.dashes = TWO_DASHES;
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: RIGHT,
                direction: Direction::Right,
            },
        );
        assert_eq!(s.forced_move_timer, 0.0);
        assert_eq!(s.dashes, TWO_DASHES);
    }

    #[test]
    fn finish_rounds_her_speed_and_refills() {
        let chapter = cross();
        let mut s = at(129, 120);
        s.speed = Vec2 {
            x: 232.66,
            y: -24.85,
        };
        s.dashes = 0;
        s.stamina = 20.0;
        s.coyote_timer = 0.05;
        s.wall.slide_timer = 0.3;
        finish(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: RIGHT,
                direction: Direction::Right,
            },
        );
        assert_eq!(s.speed, Vec2 { x: 233.0, y: -25.0 });
        assert_eq!(s.dashes, NORMAL_DASHES);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.coyote_timer, 0.0);
        assert_eq!(s.wall.slide_timer, WALL_SLIDE_TIME);
    }
}
