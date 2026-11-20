use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::level::room::Chapter;
use crate::player::collision::on_ground;
use crate::player::constants::DELTA_TIME;
use crate::player::{Input, State};
use crate::search::candidate_inputs;
use crate::search::distance::DistanceMap;
use crate::sim::{Sim, step_chapter};

// position buckets per pixel. any coarser and a slow climb looks like standing still
const POSITION_STEPS_PER_PIXEL: f32 = 8.0;
// speed bucket size in px/s
const SPEED_BUCKET: f32 = 5.0;

// where a search is trying to get to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    // an index into Chapter::rooms
    Room(usize),
    Finish,
}

// when a search gives up
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_frames: usize,
    // most states carried from one frame to the next
    pub max_states: usize,
    pub cut: Option<Cut>,
}

// drops any state that could not reach the goal in time, even on the shortest path at top speed
#[derive(Debug, Clone, Copy)]
pub struct Cut {
    // the frame the goal has to be reached by
    pub frames: usize,
    // the fastest she is assumed to travel, in px/s
    pub top_speed: f32,
}

impl Cut {
    // fewest frames it takes to cover this many pixels
    fn frames_for(self, pixels: u32) -> usize {
        (pixels as f32 / (self.top_speed * DELTA_TIME)).ceil() as usize
    }

    // whether a state this far from the goal after this many frames can no longer make it
    fn too_late(self, frame: usize, to_goal: Option<u32>) -> bool {
        match to_goal {
            Some(pixels) => frame + self.frames_for(pixels) > self.frames,
            None => true,
        }
    }
}

// what a search found, and how much work it did
#[derive(Debug, Clone)]
pub struct Report {
    // the fastest inputs found that reach the goal
    pub inputs: Option<Vec<Input>>,
    // how many states were alive after each frame
    pub states_per_frame: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Bucket {
    room: usize,
    x: i32,
    y: i32,
    speed_x: i32,
    speed_y: i32,
    dashes: u8,
    dash_ready: bool,
    state: u8,
    dash_frames_left: i32,
    freeze_frames_left: i32,
    on_ground: bool,
    crouching: bool,
    facing: i8,
    // both zero unless speed is being held against a wall
    retained_speed: i32,
    retention_frames_left: i32,
    // both zero unless a wallboost can still happen
    wallboost_dir: i8,
    wallboost_frames_left: i32,
}

fn frames(seconds: f32) -> i32 {
    (seconds / DELTA_TIME).round() as i32
}

fn bucket(chapter: &Chapter, sim: &Sim) -> Bucket {
    let player = &sim.player;
    let (state, dash_frames_left) = match player.state {
        State::Normal => (0, 0),
        State::Climb => (1, 0),
        State::DashStart => (2, 0),
        State::Dash { timer } => (3, frames(timer)),
    };
    let wall = player.wall;
    let retaining = wall.retention_timer > 0.0;
    let can_wallboost = wall.boost_timer > 0.0;
    let exact_x = player.position.x as f32 + player.subpixel.x;
    let exact_y = player.position.y as f32 + player.subpixel.y;

    Bucket {
        room: sim.room,
        x: (exact_x * POSITION_STEPS_PER_PIXEL).floor() as i32,
        y: (exact_y * POSITION_STEPS_PER_PIXEL).floor() as i32,
        speed_x: (player.speed.x / SPEED_BUCKET).floor() as i32,
        speed_y: (player.speed.y / SPEED_BUCKET).floor() as i32,
        dashes: player.dashes,
        dash_ready: player.dash.cooldown <= 0.0,
        state,
        dash_frames_left,
        freeze_frames_left: frames(player.freeze_timer),
        on_ground: on_ground(player, &chapter.rooms[sim.room], &sim.entities),
        crouching: player.crouching,
        facing: player.facing,
        retained_speed: if retaining {
            (wall.retained_speed / SPEED_BUCKET).floor() as i32
        } else {
            0
        },
        retention_frames_left: if retaining {
            frames(wall.retention_timer)
        } else {
            0
        },
        wallboost_dir: if can_wallboost { wall.boost_dir } else { 0 },
        wallboost_frames_left: if can_wallboost {
            frames(wall.boost_timer)
        } else {
            0
        },
    }
}

// how a state was reached: the link of the state before it, and the input pressed
#[derive(Clone, Copy)]
struct Link {
    parent: Option<usize>,
    input: Input,
}

fn route_to(links: &[Link], last: Link) -> Vec<Input> {
    let mut inputs = vec![last.input];
    let mut parent = last.parent;
    while let Some(index) = parent {
        inputs.push(links[index].input);
        parent = links[index].parent;
    }
    inputs.reverse();
    inputs
}

// the states a search is carrying, and every bucket it has been in
struct Frontier {
    candidates: Vec<Input>,
    // the frame each bucket was first reached on
    seen: HashMap<Bucket, usize>,
    links: Vec<Link>,
    // each state alive, with the link that reached it
    alive: Vec<(Sim, Option<usize>)>,
}

impl Frontier {
    fn new(chapter: &Chapter, start: &Sim) -> Frontier {
        Frontier {
            candidates: candidate_inputs(),
            seen: HashMap::from([(bucket(chapter, start), 0)]),
            links: Vec::new(),
            alive: vec![(start.clone(), None)],
        }
    }

    // steps every state alive by every input. returns the route of the first to reach the goal
    fn advance(
        &mut self,
        chapter: &Chapter,
        goal: Option<Goal>,
        cut: Option<(Cut, &DistanceMap)>,
        frame: usize,
    ) -> Option<Vec<Input>> {
        let mut next = Vec::new();

        for (sim, parent) in &self.alive {
            for &input in &self.candidates {
                let mut after = sim.clone();
                step_chapter(&mut after, chapter, input);
                let link = Link {
                    parent: *parent,
                    input,
                };

                let reached = if after.player.finished {
                    Some(Goal::Finish)
                } else {
                    after.entering().map(Goal::Room)
                };
                if reached.is_some() && reached == goal {
                    return Some(route_to(&self.links, link));
                }

                // dead, or out by a way that is not the goal
                if reached.is_some() || after.player.dead {
                    continue;
                }

                let too_late = cut.is_some_and(|(cut, distances)| {
                    cut.too_late(frame, distances.to_goal(after.player.position))
                });
                if too_late {
                    continue;
                }

                if let Entry::Vacant(unseen) = self.seen.entry(bucket(chapter, &after)) {
                    unseen.insert(frame);
                    self.links.push(link);
                    next.push((after, Some(self.links.len() - 1)));
                }
            }
        }

        self.alive = next;
        None
    }

    fn can_go_on(&self, limits: Limits) -> bool {
        !self.alive.is_empty() && self.alive.len() <= limits.max_states
    }
}

// tries every input one frame at a time. a state is dropped if its bucket was reached before
pub fn bucket_search(chapter: &Chapter, start: &Sim, goal: Goal, limits: Limits) -> Report {
    let mut frontier = Frontier::new(chapter, start);
    let mut states_per_frame = Vec::new();
    let distances = limits
        .cut
        .map(|cut| (cut, DistanceMap::new(chapter, start.room, goal)));

    for frame in 1..=limits.max_frames {
        let cut = distances.as_ref().map(|(cut, map)| (*cut, map));
        let found = frontier.advance(chapter, Some(goal), cut, frame);
        states_per_frame.push(frontier.alive.len());

        if found.is_some() || !frontier.can_go_on(limits) {
            return Report {
                inputs: found,
                states_per_frame,
            };
        }
    }

    Report {
        inputs: None,
        states_per_frame,
    }
}

// how the search stands against one frame of a known route
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    // the search was in the route's bucket sooner
    Ahead,
    // it got there on this frame
    Level,
    // it never got there, so the pruning lost this line
    Lost,
}

// plays a known route beside the search to see whether the pruning loses it
pub fn follow_route(chapter: &Chapter, start: &Sim, route: &[Input], limits: Limits) -> Vec<Pace> {
    let mut frontier = Frontier::new(chapter, start);
    let mut on_route = start.clone();
    let mut paces = Vec::new();

    for (frame, &input) in (1..=limits.max_frames).zip(route) {
        frontier.advance(chapter, None, None, frame);
        step_chapter(&mut on_route, chapter, input);

        let over = on_route.player.dead || on_route.player.finished || on_route.in_transition();
        if over {
            break;
        }

        paces.push(match frontier.seen.get(&bucket(chapter, &on_route)) {
            Some(&reached) if reached < frame => Pace::Ahead,
            Some(_) => Pace::Level,
            None => Pace::Lost,
        });

        if !frontier.can_go_on(limits) {
            break;
        }
    }

    paces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::at;
    use crate::geom::{IVec2, Rect};
    use crate::level::room::Room;
    use crate::search::{Outcome, play};

    const NONE: Input = Input {
        move_x: 0,
        move_y: 0,
        jump: false,
        jump2: false,
        dash: false,
        grab: false,
        demodash: false,
    };
    const RIGHT: Input = Input { move_x: 1, ..NONE };
    const LIMITS: Limits = Limits {
        max_frames: 30,
        max_states: 1_000_000,
        cut: None,
    };

    // 88 px wide, with a floor whose top is y = 176
    fn floor_room_at(x: i32) -> Room {
        let tiles = format!("{}11111111111", "00000000000\n".repeat(22));
        Room::from_tiles(IVec2 { x, y: 0 }, &tiles)
    }

    // three rooms in a row
    fn row_of_rooms() -> Chapter {
        Chapter {
            rooms: vec![floor_room_at(0), floor_room_at(88), floor_room_at(176)],
        }
    }

    #[test]
    fn it_finds_a_route_that_really_reaches_the_room() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let report = bucket_search(&chapter, &start, Goal::Room(1), LIMITS);

        let inputs = report.inputs.expect("no route found");
        assert_eq!(
            play(&chapter, &start, &inputs),
            Outcome::LeftRoom {
                frames: inputs.len(),
                to: 1
            }
        );
    }

    #[test]
    fn it_is_at_least_as_fast_as_running() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let Outcome::LeftRoom {
            frames: running, ..
        } = play(&chapter, &start, &[RIGHT; 60])
        else {
            panic!("running should get her out");
        };

        let report = bucket_search(&chapter, &start, Goal::Room(1), LIMITS);
        assert!(report.inputs.expect("no route found").len() <= running);
    }

    #[test]
    fn it_only_stops_at_the_room_it_was_asked_for() {
        // she starts in the middle room, 6 px from the left one and 76 px from the right one
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 1, at(98, 176));
        let report = bucket_search(&chapter, &start, Goal::Room(0), LIMITS);

        let inputs = report.inputs.expect("no route found");
        assert!(matches!(
            play(&chapter, &start, &inputs),
            Outcome::LeftRoom { to: 0, .. }
        ));
    }

    #[test]
    fn it_can_aim_for_the_finish_trigger() {
        let mut chapter = row_of_rooms();
        chapter.rooms[0].finish_trigger = Some(Rect::new(60, 160, 16, 16));
        let start = Sim::new(&chapter, 0, at(50, 176));
        let report = bucket_search(&chapter, &start, Goal::Finish, LIMITS);

        let inputs = report.inputs.expect("no route found");
        assert_eq!(
            play(&chapter, &start, &inputs),
            Outcome::Finished {
                frames: inputs.len()
            }
        );
    }

    #[test]
    fn it_gives_up_when_it_runs_out_of_frames() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(40, 176));
        let limits = Limits {
            max_frames: 3,
            ..LIMITS
        };
        let report = bucket_search(&chapter, &start, Goal::Room(1), limits);

        assert!(report.inputs.is_none());
        assert_eq!(report.states_per_frame.len(), 3);
    }

    #[test]
    fn a_frozen_state_is_not_mistaken_for_one_already_seen() {
        // the frames of a dash's freeze are in the same place at the same speed
        let chapter = row_of_rooms();
        let mut sim = Sim::new(&chapter, 0, at(40, 176));
        let dash = Input {
            dash: true,
            ..RIGHT
        };

        step_chapter(&mut sim, &chapter, dash);
        let first = bucket(&chapter, &sim);
        step_chapter(&mut sim, &chapter, dash);
        assert_ne!(bucket(&chapter, &sim), first);
    }

    #[test]
    fn retained_speed_only_counts_while_it_is_being_held() {
        let chapter = row_of_rooms();
        let mut sim = Sim::new(&chapter, 0, at(40, 176));
        let plain = bucket(&chapter, &sim);

        // left over from an old wall hit: the timer has run out, so it can't come back
        sim.player.wall.retained_speed = 200.0;
        assert_eq!(bucket(&chapter, &sim), plain);

        sim.player.wall.retention_timer = 0.05;
        let holding_200 = bucket(&chapter, &sim);
        assert_ne!(holding_200, plain);

        sim.player.wall.retained_speed = 250.0;
        assert_ne!(bucket(&chapter, &sim), holding_200);
    }

    #[test]
    fn a_wallboost_only_counts_while_it_can_still_happen() {
        let chapter = row_of_rooms();
        let mut sim = Sim::new(&chapter, 0, at(40, 176));
        let plain = bucket(&chapter, &sim);

        sim.player.wall.boost_dir = -1;
        assert_eq!(bucket(&chapter, &sim), plain);

        sim.player.wall.boost_timer = 0.2;
        assert_ne!(bucket(&chapter, &sim), plain);
    }

    #[test]
    fn facing_the_other_way_is_a_different_bucket() {
        let chapter = row_of_rooms();
        let mut sim = Sim::new(&chapter, 0, at(40, 176));
        let facing_right = bucket(&chapter, &sim);

        sim.player.facing = -1;
        assert_ne!(bucket(&chapter, &sim), facing_right);
    }

    #[test]
    fn standing_still_stays_in_one_bucket() {
        let chapter = row_of_rooms();
        let mut sim = Sim::new(&chapter, 0, at(40, 176));
        step_chapter(&mut sim, &chapter, NONE);
        let first = bucket(&chapter, &sim);
        step_chapter(&mut sim, &chapter, NONE);
        assert_eq!(bucket(&chapter, &sim), first);
    }

    #[test]
    fn the_search_keeps_up_with_a_route_it_could_have_found() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(40, 176));
        let paces = follow_route(&chapter, &start, &[RIGHT; 5], LIMITS);

        assert_eq!(paces, vec![Pace::Level; 5]);
    }

    #[test]
    fn the_search_is_ahead_of_a_route_that_wastes_frames() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(40, 176));
        let dawdling = [vec![NONE; 2], vec![RIGHT; 3]].concat();
        let paces = follow_route(&chapter, &start, &dawdling, LIMITS);

        // the search made the same moves two frames sooner
        assert_eq!(paces, vec![Pace::Ahead; 5]);
    }

    #[test]
    fn following_stops_when_the_route_leaves_the_room() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let Outcome::LeftRoom { frames, .. } = play(&chapter, &start, &[RIGHT; 60]) else {
            panic!("running should get her out");
        };

        let paces = follow_route(&chapter, &start, &[RIGHT; 60], LIMITS);
        assert_eq!(paces.len(), frames - 1);
    }

    #[test]
    fn following_stops_at_the_frame_limit() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(40, 176));
        let limits = Limits {
            max_frames: 3,
            ..LIMITS
        };
        assert_eq!(follow_route(&chapter, &start, &[RIGHT; 5], limits).len(), 3);
    }

    fn cut(frames: usize) -> Limits {
        Limits {
            cut: Some(Cut {
                frames,
                top_speed: 480.0,
            }),
            ..LIMITS
        }
    }

    #[test]
    fn top_speed_sets_how_many_frames_a_distance_takes() {
        // 480 px/s is 8 px a frame
        let cut = Cut {
            frames: 10,
            top_speed: 480.0,
        };
        assert_eq!(cut.frames_for(0), 0);
        assert_eq!(cut.frames_for(8), 1);
        assert_eq!(cut.frames_for(9), 2);

        assert!(!cut.too_late(8, Some(16)));
        assert!(cut.too_late(9, Some(16)));
        assert!(cut.too_late(1, None));
    }

    #[test]
    fn a_cut_the_best_route_fits_inside_finds_the_same_route() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let free = bucket_search(&chapter, &start, Goal::Room(1), LIMITS);
        let frames = free.inputs.as_ref().expect("no route found").len();

        let with_cut = bucket_search(&chapter, &start, Goal::Room(1), cut(frames));
        assert_eq!(with_cut.inputs, free.inputs);
    }

    #[test]
    fn a_cut_carries_fewer_states() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let free = bucket_search(&chapter, &start, Goal::Room(1), LIMITS);
        let frames = free.inputs.as_ref().expect("no route found").len();

        // at 2 px a frame, anything still far from the exit near the end is too late
        let slow = Limits {
            cut: Some(Cut {
                frames,
                top_speed: 120.0,
            }),
            ..LIMITS
        };
        let with_cut = bucket_search(&chapter, &start, Goal::Room(1), slow);

        let total = |report: &Report| report.states_per_frame.iter().sum::<usize>();
        assert!(total(&with_cut) < total(&free));
    }

    #[test]
    fn a_cut_too_tight_for_any_route_finds_nothing() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let free = bucket_search(&chapter, &start, Goal::Room(1), LIMITS);
        let frames = free.inputs.expect("no route found").len();

        let report = bucket_search(&chapter, &start, Goal::Room(1), cut(frames - 1));
        assert!(report.inputs.is_none());
    }
}
