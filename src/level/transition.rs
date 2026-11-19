use crate::entities::dash_block::DASH_BLOCK_FREEZE_TIME;
use crate::geom::Direction;
use crate::level::room::Chapter;
use crate::player::Player;
use crate::player::collision::{collides, hitbox_height};
use crate::player::constants::{
    CLIMB_MAX_STAMINA, DASH_COOLDOWN, HITBOX_WIDTH, JUMP_HOLD_TIME, JUMP_SPEED, MAX_DASHES,
    WALL_SLIDE_TIME,
};
use crate::player::state::set_normal;

pub const TRANSITION_FRAMES: usize = 40;
// how far the game looks past her center for the next room
const SIDE_REACH: f32 = 8.0;
const VERTICAL_REACH: f32 = 12.0;
// she is moved the way she is travelling until she is this far inside the new room
const ENTRY_PAD: i32 = 4;
const DOWN_ENTRY_PAD: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    // indexes into Chapter::rooms
    pub from: usize,
    pub to: usize,
    pub direction: Direction,
}

pub fn find_transition(player: &Player, chapter: &Chapter, room: usize) -> Option<Transition> {
    let bounds = chapter.rooms[room].bounds;
    let height = hitbox_height(player);

    let left = player.position.x - HITBOX_WIDTH / 2;
    let right = player.position.x + HITBOX_WIDTH / 2;
    let top = player.position.y - height;
    let bottom = player.position.y;
    let center_x = player.position.x as f32;
    let center_y = player.position.y as f32 - height as f32 / 2.0;

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
    let beside = top >= bounds.top && bottom < bounds.bottom();

    if left < bounds.left {
        if beside {
            return to(center_x - SIDE_REACH, center_y, Direction::Left);
        }
    } else if right > bounds.right() && beside {
        return to(center_x + SIDE_REACH, center_y, Direction::Right);
    }

    if center_y < bounds.top as f32 {
        return to(center_x, center_y - VERTICAL_REACH, Direction::Up);
    }

    if bottom > bounds.bottom() && !collides(player, &chapter.rooms[room], 0, 4) {
        return to(center_x, center_y + VERTICAL_REACH, Direction::Down);
    }

    None
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

pub fn finish_transition(player: &mut Player, chapter: &Chapter, transition: Transition) {
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
    player.dashes = MAX_DASHES;
    player.stamina = CLIMB_MAX_STAMINA;

    // the room she left is unloaded now, and every dash block still in it freezes the game
    let left = &chapter.rooms[transition.from];
    let left_a_dash_block = left
        .dash_blocks
        .iter()
        .any(|block| left.solids[block.solid].collidable);
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

    const CENTRE: usize = 0;
    const UP: usize = 1;
    const RIGHT: usize = 2;
    const DOWN: usize = 3;
    const LEFT: usize = 4;

    #[test]
    fn no_transition_inside_the_room() {
        assert_eq!(find_transition(&at(116, 120), &cross(), CENTRE), None);
    }

    #[test]
    fn right_when_her_hitbox_passes_the_edge() {
        let chapter = cross();
        assert_eq!(find_transition(&at(128, 120), &chapter, CENTRE), None);
        let found = find_transition(&at(129, 120), &chapter, CENTRE);
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
        assert_eq!(find_transition(&at(104, 120), &chapter, CENTRE), None);
        let found = find_transition(&at(103, 120), &chapter, CENTRE);
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
        assert_eq!(find_transition(&at(116, 106), &chapter, CENTRE), None);
        let found = find_transition(&at(116, 105), &chapter, CENTRE);
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
        assert_eq!(find_transition(&s, &chapter, CENTRE), None);
        s.position.y = 102;
        assert_eq!(
            find_transition(&s, &chapter, CENTRE).map(|t| t.to),
            Some(UP)
        );
    }

    #[test]
    fn down_when_her_feet_pass_the_edge() {
        let chapter = cross();
        assert_eq!(find_transition(&at(116, 132), &chapter, CENTRE), None);
        let found = find_transition(&at(116, 133), &chapter, CENTRE);
        assert_eq!(found.map(|t| t.direction), Some(Direction::Down));
    }

    #[test]
    fn no_transition_without_a_room_there() {
        let chapter = cross();
        assert_eq!(find_transition(&at(116, 60), &chapter, UP), None);
        assert_eq!(find_transition(&at(170, 120), &chapter, RIGHT), None);
    }

    #[test]
    fn sideways_needs_her_whole_body_level_with_the_room() {
        let chapter = cross();
        assert_eq!(find_transition(&at(129, 110), &chapter, CENTRE), None);
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
        finish_transition(
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
        finish_transition(
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
        finish_transition(
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
        finish_transition(
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
        finish_transition(
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
        finish_transition(
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
        chapter.rooms[CENTRE].add_dash_block(100, 100, 8, 8);
        let up = Transition {
            from: CENTRE,
            to: UP,
            direction: Direction::Up,
        };

        let mut s = at(116, 105);
        finish_transition(&mut s, &chapter, up);
        assert_eq!(s.freeze_timer, DASH_BLOCK_FREEZE_TIME);

        // not if it was broken first, and not for a dash block in the room she enters
        chapter.rooms[CENTRE].solids[0].collidable = false;
        chapter.rooms[UP].add_dash_block(100, 68, 8, 8);
        let mut s = at(116, 105);
        finish_transition(&mut s, &chapter, up);
        assert_eq!(s.freeze_timer, 0.0);
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
        finish_transition(
            &mut s,
            &chapter,
            Transition {
                from: CENTRE,
                to: RIGHT,
                direction: Direction::Right,
            },
        );
        assert_eq!(s.speed, Vec2 { x: 233.0, y: -25.0 });
        assert_eq!(s.dashes, MAX_DASHES);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.coyote_timer, 0.0);
        assert_eq!(s.wall.slide_timer, WALL_SLIDE_TIME);
    }
}
