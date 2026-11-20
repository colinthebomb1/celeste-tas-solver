use std::collections::VecDeque;

use crate::geom::{IVec2, Rect, Vec2};
use crate::level::room::Chapter;
use crate::level::transition::check_room_edges;
use crate::player::Player;
use crate::player::collision::touches_finish;
use crate::player::constants::{CROUCH_HITBOX_HEIGHT, HITBOX_WIDTH};
use crate::search::bucket_search::Goal;

// how far past the room's edges the map reaches, so it covers where she leaves from
const MARGIN: i32 = 32;
// she can be this far above a room, and this far below it before she dies
const ROOM_ABOVE: i32 = 24;
const ROOM_BELOW: i32 = 15;

// pixels along the shortest open path to the goal, from each place in a room. gravity is ignored
#[derive(Debug, Clone)]
pub struct DistanceMap {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    // None where the goal can't be reached
    steps: Vec<Option<u32>>,
}

impl DistanceMap {
    pub fn new(chapter: &Chapter, room: usize, goal: Goal) -> DistanceMap {
        let bounds = chapter.rooms[room].bounds;
        let mut map = DistanceMap {
            left: bounds.left - MARGIN,
            top: bounds.top - MARGIN,
            width: bounds.width + 2 * MARGIN,
            height: bounds.height + 2 * MARGIN,
            steps: Vec::new(),
        };
        map.steps = vec![None; (map.width * map.height) as usize];

        // the goal's own positions are zero steps away, and the rest are filled outward from them
        let mut queue = VecDeque::new();
        for y in map.top..map.top + map.height {
            for x in map.left..map.left + map.width {
                let position = IVec2 { x, y };
                if is_goal(chapter, room, goal, position) {
                    let index = map.index(position).expect("inside the map");
                    map.steps[index] = Some(0);
                    queue.push_back(position);
                }
            }
        }

        while let Some(from) = queue.pop_front() {
            let steps = map.to_goal(from).expect("queued positions have a distance") + 1;
            for (dx, dy) in NEIGHBOURS {
                let to = IVec2 {
                    x: from.x + dx,
                    y: from.y + dy,
                };
                let Some(index) = map.index(to) else {
                    continue;
                };
                if map.steps[index].is_none() && can_be_at(chapter, room, to) {
                    map.steps[index] = Some(steps);
                    queue.push_back(to);
                }
            }
        }

        map
    }

    // steps from this position to the goal. None if it can't be reached or is off the map
    pub fn to_goal(&self, position: IVec2) -> Option<u32> {
        self.steps[self.index(position)?]
    }

    fn index(&self, position: IVec2) -> Option<usize> {
        let (x, y) = (position.x - self.left, position.y - self.top);
        let inside = x >= 0 && x < self.width && y >= 0 && y < self.height;
        inside.then(|| (y * self.width + x) as usize)
    }
}

const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn standing_at(position: IVec2) -> Player {
    Player::new(position, Vec2::default(), Vec2::default())
}

// whether being here, standing or crouched, reaches the goal
fn is_goal(chapter: &Chapter, room: usize, goal: Goal, position: IVec2) -> bool {
    let crouched = Player {
        crouching: true,
        ..standing_at(position)
    };

    [standing_at(position), crouched]
        .into_iter()
        .any(|mut player| match goal {
            Goal::Finish => touches_finish(&player, &chapter.rooms[room]),
            Goal::Room(to) => {
                let entities = &chapter.rooms[room].initial;
                check_room_edges(&mut player, chapter, room, entities)
                    .is_some_and(|transition| transition.to == to)
            }
        })
}

// whether she can be here at all: inside the room's reach, and clear of tiles when crouched
fn can_be_at(chapter: &Chapter, room: usize, position: IVec2) -> bool {
    let room = &chapter.rooms[room];
    let bounds = room.bounds;
    let hitbox = Rect::new(
        position.x - HITBOX_WIDTH / 2,
        position.y - CROUCH_HITBOX_HEIGHT,
        HITBOX_WIDTH,
        CROUCH_HITBOX_HEIGHT,
    );

    hitbox.left >= bounds.left
        && hitbox.right() <= bounds.right()
        && hitbox.top >= bounds.top - ROOM_ABOVE
        && hitbox.top <= bounds.bottom() + ROOM_BELOW
        && !room.touches_tiles(&hitbox)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::room::Room;

    fn room_at(x: i32, tiles: &str) -> Room {
        Room::from_tiles(IVec2 { x, y: 0 }, tiles)
    }

    // 88 px wide, with a floor whose top is y = 176
    fn floor_tiles() -> String {
        format!("{}11111111111", "00000000000\n".repeat(22))
    }

    // the same, with a wall across the middle from y = 80 down to the floor
    fn walled_tiles() -> String {
        let open = "00000000000\n".repeat(10);
        let wall = "00000100000\n".repeat(12);
        format!("{open}{wall}11111111111")
    }

    fn two_rooms(first: &str) -> Chapter {
        Chapter {
            rooms: vec![room_at(0, first), room_at(88, &floor_tiles())],
        }
    }

    fn on_the_floor(x: i32) -> IVec2 {
        IVec2 { x, y: 176 }
    }

    #[test]
    fn it_counts_pixels_to_where_she_leaves_the_room() {
        // she leaves once her hitbox, 4 px either side of her, passes x = 88
        let chapter = two_rooms(&floor_tiles());
        let map = DistanceMap::new(&chapter, 0, Goal::Room(1));

        assert_eq!(map.to_goal(on_the_floor(85)), Some(0));
        assert_eq!(map.to_goal(on_the_floor(84)), Some(1));
        assert_eq!(map.to_goal(on_the_floor(40)), Some(45));
    }

    #[test]
    fn a_diagonal_step_counts_as_one() {
        let chapter = two_rooms(&floor_tiles());
        let map = DistanceMap::new(&chapter, 0, Goal::Room(1));

        // 45 px across and 30 px up is still 45 steps
        assert_eq!(map.to_goal(IVec2 { x: 40, y: 146 }), Some(45));
    }

    #[test]
    fn the_path_goes_around_walls() {
        let open = DistanceMap::new(&two_rooms(&floor_tiles()), 0, Goal::Room(1));
        let walled = DistanceMap::new(&two_rooms(&walled_tiles()), 0, Goal::Room(1));
        let start = on_the_floor(20);

        // over the wall means climbing 96 px to get her feet above y = 80
        assert_eq!(open.to_goal(start), Some(65));
        assert!(walled.to_goal(start) >= Some(96));
    }

    #[test]
    fn inside_a_wall_or_off_the_map_has_no_distance() {
        let chapter = two_rooms(&walled_tiles());
        let map = DistanceMap::new(&chapter, 0, Goal::Room(1));

        assert_eq!(map.to_goal(IVec2 { x: 44, y: 150 }), None);
        assert_eq!(map.to_goal(IVec2 { x: 5000, y: 100 }), None);
    }

    #[test]
    fn a_goal_she_cannot_get_to_leaves_the_map_empty() {
        // the only room is closed off by the edges, and has no finish trigger
        let chapter = Chapter {
            rooms: vec![room_at(0, &floor_tiles())],
        };
        let map = DistanceMap::new(&chapter, 0, Goal::Finish);

        assert_eq!(map.to_goal(on_the_floor(40)), None);
    }

    #[test]
    fn it_measures_to_the_finish_trigger() {
        let mut chapter = two_rooms(&floor_tiles());
        chapter.rooms[0].finish_trigger = Some(Rect::new(60, 160, 16, 16));
        let map = DistanceMap::new(&chapter, 0, Goal::Finish);

        // her hitbox's right side is at x + 4, and the trigger starts at x = 60
        assert_eq!(map.to_goal(on_the_floor(57)), Some(0));
        assert_eq!(map.to_goal(on_the_floor(40)), Some(17));
    }
}
