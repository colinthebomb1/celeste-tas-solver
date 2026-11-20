use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};

use crate::level::room::Chapter;
use crate::player::collision::on_ground;
use crate::player::constants::{CLIMB_JUMP_COST, DELTA_TIME};
use crate::player::{Input, State};
use crate::search::candidate_inputs;
use crate::search::distance::DistanceMap;
use crate::sim::{Sim, step_chapter};

// position buckets per pixel. any coarser and a slow climb looks like standing still
const POSITION_STEPS_PER_PIXEL: f32 = 8.0;
// speed bucket size in px/s
const SPEED_BUCKET: f32 = 5.0;
// she has 0, 1 or 2 dashes
const DASH_COUNTS: usize = 3;

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
    // drops a state that reaches a pixel more than this many frames after the first state
    // to get there with at least as many dashes
    pub max_late: Option<usize>,
    // keeps only this many states each frame, the ones nearest the goal
    pub nearest: Option<usize>,
    // keeps at most this many states for each pixel, dash count and stamina level
    pub per_place: Option<usize>,
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

// the frame each pixel was first reached on, for each number of dashes left
type FirstArrivals = HashMap<(i32, i32), [Option<usize>; DASH_COUNTS]>;

// notes that a state is here on this frame. returns how many frames ago the first state
// with at least as many dashes was
fn frames_behind(first_at: &mut FirstArrivals, sim: &Sim, frame: usize) -> usize {
    let place = (sim.player.position.x, sim.player.position.y);
    let dashes = (sim.player.dashes as usize).min(DASH_COUNTS - 1);
    let first = first_at.entry(place).or_default();

    first[dashes].get_or_insert(frame);
    let earliest = first[dashes..].iter().flatten().min();
    frame - earliest.copied().unwrap_or(frame)
}

fn pixels_to_goal(distances: &Option<DistanceMap>, sim: &Sim) -> Option<u32> {
    distances.as_ref()?.to_goal(sim.player.position)
}

// where she is and what she has left. states that match on these share a limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Place {
    x: i32,
    y: i32,
    dashes: u8,
    // how many climb jumps her stamina still pays for
    climb_jumps_left: i32,
}

fn place(sim: &Sim) -> Place {
    let player = &sim.player;
    Place {
        x: player.position.x,
        y: player.position.y,
        dashes: player.dashes,
        climb_jumps_left: (player.stamina / CLIMB_JUMP_COST).floor() as i32,
    }
}

// keeps at most this many states for each place, spread from the slowest to the fastest
fn keep_variety(states: Vec<(Sim, Link)>, most: usize) -> Vec<(Sim, Link)> {
    // in order of place, so the result does not change from run to run
    let mut by_place: BTreeMap<Place, Vec<(Sim, Link)>> = BTreeMap::new();
    for state in states {
        by_place.entry(place(&state.0)).or_default().push(state);
    }

    let mut kept = Vec::new();
    for mut group in by_place.into_values() {
        if group.len() <= most {
            kept.append(&mut group);
            continue;
        }

        group.sort_by(|a, b| a.0.player.speed.x.total_cmp(&b.0.player.speed.x));
        let last = group.len() - 1;
        let picks: Vec<usize> = (0..most)
            .map(|pick| match most {
                1 => last / 2,
                _ => pick * last / (most - 1),
            })
            .collect();

        for (index, state) in group.into_iter().enumerate() {
            if picks.contains(&index) {
                kept.push(state);
            }
        }
    }

    kept
}

// keeps the states nearest the goal, up to this many
fn keep_nearest(states: &mut Vec<(Sim, Link)>, most: usize, distances: &Option<DistanceMap>) {
    if states.len() <= most {
        return;
    }

    // a place with no way to the goal sorts last
    states.select_nth_unstable_by_key(most, |(sim, _)| {
        pixels_to_goal(distances, sim).unwrap_or(u32::MAX)
    });
    states.truncate(most);
}

// the states a search is carrying, and every bucket it has been in
struct Frontier {
    candidates: Vec<Input>,
    limits: Limits,
    // pixels to the goal from each place. only made when a limit uses it
    distances: Option<DistanceMap>,
    // the frame each bucket was first reached on
    seen: HashMap<Bucket, usize>,
    links: Vec<Link>,
    // each state alive, with the link that reached it
    alive: Vec<(Sim, Option<usize>)>,
    first_at: FirstArrivals,
}

impl Frontier {
    fn new(chapter: &Chapter, start: &Sim, goal: Goal, limits: Limits) -> Frontier {
        let uses_distances = limits.cut.is_some() || limits.nearest.is_some();

        Frontier {
            candidates: candidate_inputs(),
            limits,
            distances: uses_distances.then(|| DistanceMap::new(chapter, start.room, goal)),
            seen: HashMap::from([(bucket(chapter, start), 0)]),
            links: Vec::new(),
            alive: vec![(start.clone(), None)],
            first_at: FirstArrivals::new(),
        }
    }

    // steps every state alive by every input. returns the route of the first to reach stop_at
    fn advance(
        &mut self,
        chapter: &Chapter,
        stop_at: Option<Goal>,
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
                if reached.is_some() && reached == stop_at {
                    return Some(route_to(&self.links, link));
                }

                // dead, or out by a way that is not the goal
                if reached.is_some() || after.player.dead {
                    continue;
                }

                let too_late = self.limits.cut.is_some_and(|cut| {
                    cut.too_late(frame, pixels_to_goal(&self.distances, &after))
                });
                if too_late {
                    continue;
                }

                // the freeze at the start of a dash holds her still without her being behind
                let held_still =
                    after.player.freeze_timer > 0.0 || after.player.state == State::DashStart;
                if !held_still {
                    let behind = frames_behind(&mut self.first_at, &after, frame);
                    if self.limits.max_late.is_some_and(|most| behind > most) {
                        continue;
                    }
                }

                if let Entry::Vacant(unseen) = self.seen.entry(bucket(chapter, &after)) {
                    unseen.insert(frame);
                    next.push((after, link));
                }
            }
        }

        if let Some(most) = self.limits.per_place {
            next = keep_variety(next, most);
        }
        if let Some(most) = self.limits.nearest {
            keep_nearest(&mut next, most, &self.distances);
        }

        // only the states kept get a link
        self.alive.clear();
        for (sim, link) in next {
            self.links.push(link);
            self.alive.push((sim, Some(self.links.len() - 1)));
        }
        None
    }

    fn can_go_on(&self) -> bool {
        !self.alive.is_empty() && self.alive.len() <= self.limits.max_states
    }
}

// tries every input one frame at a time. a state is dropped if its bucket was reached before
pub fn bucket_search(chapter: &Chapter, start: &Sim, goal: Goal, limits: Limits) -> Report {
    let mut frontier = Frontier::new(chapter, start, goal, limits);
    let mut states_per_frame = Vec::new();

    for frame in 1..=limits.max_frames {
        let found = frontier.advance(chapter, Some(goal), frame);
        states_per_frame.push(frontier.alive.len());

        if found.is_some() || !frontier.can_go_on() {
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
pub fn follow_route(
    chapter: &Chapter,
    start: &Sim,
    route: &[Input],
    goal: Goal,
    limits: Limits,
) -> Vec<Pace> {
    let mut frontier = Frontier::new(chapter, start, goal, limits);
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

        if !frontier.can_go_on() {
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
        max_late: None,
        nearest: None,
        per_place: None,
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
        let paces = follow_route(&chapter, &start, &[RIGHT; 5], Goal::Room(1), LIMITS);

        assert_eq!(paces, vec![Pace::Level; 5]);
    }

    #[test]
    fn the_search_is_ahead_of_a_route_that_wastes_frames() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(40, 176));
        let dawdling = [vec![NONE; 2], vec![RIGHT; 3]].concat();
        let paces = follow_route(&chapter, &start, &dawdling, Goal::Room(1), LIMITS);

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

        let paces = follow_route(&chapter, &start, &[RIGHT; 60], Goal::Room(1), LIMITS);
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
        assert_eq!(
            follow_route(&chapter, &start, &[RIGHT; 5], Goal::Room(1), limits).len(),
            3
        );
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

    #[test]
    fn a_state_is_behind_by_the_frames_since_the_first_one_there() {
        let chapter = row_of_rooms();
        let sim = Sim::new(&chapter, 0, at(40, 176));
        let mut first_at = FirstArrivals::new();

        assert_eq!(frames_behind(&mut first_at, &sim, 3), 0);
        assert_eq!(frames_behind(&mut first_at, &sim, 3), 0);
        assert_eq!(frames_behind(&mut first_at, &sim, 7), 4);
    }

    #[test]
    fn a_state_is_not_behind_one_that_had_fewer_dashes() {
        let chapter = row_of_rooms();
        let with_dash = Sim::new(&chapter, 0, at(40, 176));
        let mut without = with_dash.clone();
        without.player.dashes = 0;
        let mut first_at = FirstArrivals::new();

        // the first one here had used its dash, so a later one that still has it is not behind
        assert_eq!(frames_behind(&mut first_at, &without, 2), 0);
        assert_eq!(frames_behind(&mut first_at, &with_dash, 6), 0);
        // and from then on a dashless one is behind the earlier of the two
        assert_eq!(frames_behind(&mut first_at, &without, 9), 7);
    }

    #[test]
    fn dropping_late_states_still_finds_the_route_with_fewer_states() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let free = bucket_search(&chapter, &start, Goal::Room(1), LIMITS);
        let strict = Limits {
            max_late: Some(0),
            ..LIMITS
        };
        let pruned = bucket_search(&chapter, &start, Goal::Room(1), strict);

        let frames = |report: &Report| report.inputs.as_ref().map(Vec::len);
        let total = |report: &Report| report.states_per_frame.iter().sum::<usize>();
        assert_eq!(frames(&pruned), frames(&free));
        assert!(total(&pruned) < total(&free));
    }

    #[test]
    fn keeping_the_nearest_drops_the_ones_furthest_from_the_goal() {
        let chapter = row_of_rooms();
        let distances = Some(DistanceMap::new(&chapter, 0, Goal::Room(1)));
        let link = Link {
            parent: None,
            input: NONE,
        };
        let mut states: Vec<(Sim, Link)> = [20, 70, 40, 60]
            .into_iter()
            .map(|x| (Sim::new(&chapter, 0, at(x, 176)), link))
            .collect();

        keep_nearest(&mut states, 2, &distances);
        let mut kept: Vec<i32> = states
            .iter()
            .map(|(sim, _)| sim.player.position.x)
            .collect();
        kept.sort();
        assert_eq!(kept, vec![60, 70]);
    }

    #[test]
    fn the_search_never_carries_more_than_it_is_told_to_keep() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(80, 176));
        let narrow = Limits {
            nearest: Some(50),
            ..LIMITS
        };
        let report = bucket_search(&chapter, &start, Goal::Room(1), narrow);

        assert!(report.inputs.is_some());
        assert!(report.states_per_frame.iter().all(|&states| states <= 50));
    }

    #[test]
    fn a_dash_is_not_dropped_as_late_while_its_freeze_holds_her_still() {
        let chapter = row_of_rooms();
        let start = Sim::new(&chapter, 0, at(20, 176));
        let dash = Input {
            dash: true,
            ..RIGHT
        };
        let strict = Limits {
            max_frames: 7,
            max_late: Some(0),
            ..LIMITS
        };

        let paces = follow_route(&chapter, &start, &[dash; 7], Goal::Room(1), strict);
        assert!(!paces.contains(&Pace::Lost), "{paces:?}");
    }

    fn state_at(chapter: &Chapter, x: i32, speed_x: f32) -> (Sim, Link) {
        let mut sim = Sim::new(chapter, 0, at(x, 176));
        sim.player.speed.x = speed_x;
        let link = Link {
            parent: None,
            input: NONE,
        };
        (sim, link)
    }

    fn speeds(states: &[(Sim, Link)]) -> Vec<f32> {
        let mut speeds: Vec<f32> = states.iter().map(|(sim, _)| sim.player.speed.x).collect();
        speeds.sort_by(f32::total_cmp);
        speeds
    }

    #[test]
    fn one_place_keeps_a_spread_from_its_slowest_state_to_its_fastest() {
        let chapter = row_of_rooms();
        let states: Vec<_> = [40.0, 10.0, 60.0, 20.0, 50.0, 30.0]
            .into_iter()
            .map(|speed| state_at(&chapter, 40, speed))
            .collect();

        assert_eq!(speeds(&keep_variety(states, 3)), vec![10.0, 30.0, 60.0]);
    }

    #[test]
    fn a_place_with_few_states_keeps_them_all() {
        let chapter = row_of_rooms();
        let states = vec![state_at(&chapter, 40, 10.0), state_at(&chapter, 40, 20.0)];

        assert_eq!(keep_variety(states, 3).len(), 2);
    }

    #[test]
    fn what_she_has_left_makes_it_a_different_place() {
        let chapter = row_of_rooms();
        let fresh = state_at(&chapter, 40, 10.0);
        let mut no_dash = state_at(&chapter, 40, 20.0);
        no_dash.0.player.dashes = 0;
        let mut tired = state_at(&chapter, 40, 30.0);
        tired.0.player.stamina = 50.0;
        let elsewhere = state_at(&chapter, 41, 40.0);

        // one each, though all but the last stand on the same pixel
        let kept = keep_variety(vec![fresh, no_dash, tired, elsewhere], 1);
        assert_eq!(kept.len(), 4);
    }
}
