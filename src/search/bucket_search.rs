use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::level::room::Chapter;
use crate::player::collision::on_ground;
use crate::player::constants::DELTA_TIME;
use crate::player::{Input, State};
use crate::search::candidate_inputs;
use crate::sim::{Sim, step_chapter};

// position buckets per pixel. any coarser and a slow climb looks like standing still
const POSITION_STEPS_PER_PIXEL: f32 = 8.0;
// speed bucket size in px/s
const SPEED_BUCKET: f32 = 10.0;

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

    for frame in 1..=limits.max_frames {
        let found = frontier.advance(chapter, Some(goal), frame);
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
        frontier.advance(chapter, None, frame);
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
}
