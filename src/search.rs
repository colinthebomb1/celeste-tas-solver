use crate::level::room::Chapter;
use crate::player::Input;
use crate::sim::{Sim, step_chapter};

// how a run of inputs ended, frames counts the inputs used, so the first frame is 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    // a transition into room `to` began on this frame. to is an index into Chapter::rooms
    LeftRoom { frames: usize, to: usize },
    // she touched the finish trigger on this frame
    Finished { frames: usize },
    Died { frames: usize },
    // every input was used and she is still in the room
    RanOut,
}

// plays these inputs from a starting state, one per frame, and stops at the first frame she
// leaves the room, finishes or dies
pub fn play(chapter: &Chapter, start: &Sim, inputs: &[Input]) -> Outcome {
    let mut sim = start.clone();

    for (frame, &input) in (1..).zip(inputs) {
        step_chapter(&mut sim, chapter, input);

        if sim.player.finished {
            return Outcome::Finished { frames: frame };
        }

        if sim.player.dead {
            return Outcome::Died { frames: frame };
        }

        if let Some(to) = sim.entering() {
            return Outcome::LeftRoom { frames: frame, to };
        }
    }

    Outcome::RanOut
}

// (jump, jump2) both at once is left out
const JUMP_BUTTONS: [(bool, bool); 3] = [(false, false), (true, false), (false, true)];
// (dash, demodash) both at once is left out
const DASH_BUTTONS: [(bool, bool); 3] = [(false, false), (true, false), (false, true)];

// every input that can do something different on a single frame
pub fn candidate_inputs() -> Vec<Input> {
    let mut inputs = Vec::new();

    for move_x in [-1, 0, 1] {
        for move_y in [-1, 0, 1] {
            for (jump, jump2) in JUMP_BUTTONS {
                for (dash, demodash) in DASH_BUTTONS {
                    for grab in [false, true] {
                        inputs.push(Input {
                            move_x,
                            move_y,
                            jump,
                            jump2,
                            dash,
                            demodash,
                            grab,
                        });
                    }
                }
            }
        }
    }

    inputs
}

// frames these inputs take to end the way `goal` did: into the same room, or at the finish.
// None if they end any other way
fn frames_to_reach(
    chapter: &Chapter,
    start: &Sim,
    inputs: &[Input],
    goal: Outcome,
) -> Option<usize> {
    match (play(chapter, start, inputs), goal) {
        (Outcome::LeftRoom { frames, to }, Outcome::LeftRoom { to: wanted, .. })
            if to == wanted =>
        {
            Some(frames)
        }
        (Outcome::Finished { frames }, Outcome::Finished { .. }) => Some(frames),
        _ => None,
    }
}

// the first small change to one frame that reaches the goal sooner
// dropping the frame or pressing something else on it
fn faster_tweak(
    chapter: &Chapter,
    start: &Sim,
    inputs: &[Input],
    index: usize,
    goal: Outcome,
    frames_to_beat: usize,
) -> Option<(Vec<Input>, usize)> {
    let faster = |tweaked: Vec<Input>| {
        let frames = frames_to_reach(chapter, start, &tweaked, goal)?;
        (frames < frames_to_beat).then_some((tweaked, frames))
    };

    let mut dropped = inputs.to_vec();
    dropped.remove(index);
    if let Some(found) = faster(dropped) {
        return Some(found);
    }

    for input in candidate_inputs() {
        if input == inputs[index] {
            continue;
        }

        let mut swapped = inputs.to_vec();
        swapped[index] = input;
        if let Some(found) = faster(swapped) {
            return Some(found);
        }
    }

    None
}

// tries small changes to these inputs and keeps each one that gets her out sooner, until no
// change to any single frame helps. Wthe inputs given must already get her out
pub fn improve(chapter: &Chapter, start: &Sim, inputs: &[Input]) -> Vec<Input> {
    let goal = play(chapter, start, inputs);
    let mut best_frames = frames_to_reach(chapter, start, inputs, goal)
        .expect("improve needs inputs that leave the room or finish the chapter");

    // inputs after the frame she gets out on are never played
    let mut best = inputs[..best_frames].to_vec();

    loop {
        let frames_before = best_frames;

        let mut index = 0;
        while index < best.len() {
            match faster_tweak(chapter, start, &best, index, goal, best_frames) {
                // look at this frame again, what is there now may improve too
                Some((tweaked, frames)) => {
                    best = tweaked;
                    best.truncate(frames);
                    best_frames = frames;
                }
                None => index += 1,
            }
        }

        // a whole pass with nothing faster
        if best_frames == frames_before {
            return best;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::at;
    use crate::geom::{IVec2, Rect};
    use crate::level::room::Room;

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

    // 88 px wide, with a floor whose top is y = 176
    fn floor_room_at(x: i32) -> Room {
        let tiles = format!("{}11111111111", "00000000000\n".repeat(22));
        Room::from_tiles(IVec2 { x, y: 0 }, &tiles)
    }

    fn two_rooms() -> Chapter {
        Chapter {
            rooms: vec![floor_room_at(0), floor_room_at(88)],
        }
    }

    // she stands on the floor, 8 px from the first room's right edge
    fn near_the_exit(chapter: &Chapter) -> Sim {
        Sim::new(chapter, 0, at(80, 176))
    }

    #[test]
    fn running_out_of_the_room_reports_the_frame_she_left() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        assert_eq!(
            play(&chapter, &start, &[RIGHT; 60]),
            Outcome::LeftRoom { frames: 6, to: 1 }
        );
    }

    #[test]
    fn it_stops_at_the_exit_even_with_inputs_left_over() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        assert_eq!(
            play(&chapter, &start, &[RIGHT; 60]),
            play(&chapter, &start, &[RIGHT; 600])
        );
    }

    #[test]
    fn too_few_inputs_to_get_out_runs_out() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        assert_eq!(play(&chapter, &start, &[RIGHT; 3]), Outcome::RanOut);
        assert_eq!(play(&chapter, &start, &[NONE; 60]), Outcome::RanOut);
        assert_eq!(play(&chapter, &start, &[]), Outcome::RanOut);
    }

    #[test]
    fn falling_out_of_the_bottom_reports_the_frame_she_died() {
        // a single room with no floor and nothing under it
        let chapter = Chapter {
            rooms: vec![Room::from_tiles(
                IVec2 { x: 0, y: 0 },
                &"00000000000\n".repeat(23),
            )],
        };
        let start = Sim::new(&chapter, 0, at(40, 170));
        assert_eq!(
            play(&chapter, &start, &[NONE; 60]),
            Outcome::Died { frames: 15 }
        );
    }

    #[test]
    fn touching_the_finish_trigger_reports_the_frame_she_finished() {
        let mut chapter = two_rooms();
        chapter.rooms[0].finish_trigger = Some(Rect::new(40, 160, 16, 16));
        let start = Sim::new(&chapter, 0, at(48, 176));
        assert_eq!(
            play(&chapter, &start, &[NONE; 60]),
            Outcome::Finished { frames: 1 }
        );
    }

    #[test]
    fn the_starting_state_is_left_as_it_was() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        let before = start.clone();

        let first = play(&chapter, &start, &[RIGHT; 60]);
        assert_eq!(start, before);
        assert_eq!(play(&chapter, &start, &[RIGHT; 60]), first);
    }

    #[test]
    fn candidate_inputs_cover_the_basic_moves() {
        let inputs = candidate_inputs();
        let left = Input { move_x: -1, ..NONE };

        assert!(inputs.contains(&NONE), "standing still");
        assert!(inputs.contains(&RIGHT));
        assert!(inputs.contains(&left));
        assert!(inputs.iter().any(|input| input.jump || input.jump2));
        assert!(inputs.iter().any(|input| input.dash || input.demodash));
    }

    #[test]
    fn candidate_inputs_use_every_button_but_never_two_that_do_the_same() {
        let inputs = candidate_inputs();
        // 3 ways sideways, 3 up or down, 3 jump choices, 3 dash choices, grab or not
        assert_eq!(inputs.len(), 3 * 3 * 3 * 3 * 2);

        assert!(inputs.iter().any(|input| input.jump));
        assert!(inputs.iter().any(|input| input.jump2));
        assert!(inputs.iter().any(|input| input.dash));
        assert!(inputs.iter().any(|input| input.demodash));
        assert!(inputs.iter().any(|input| input.grab));

        assert!(!inputs.iter().any(|input| input.jump && input.jump2));
        assert!(!inputs.iter().any(|input| input.dash && input.demodash));
    }

    #[test]
    fn candidate_inputs_has_no_repeats() {
        let inputs = candidate_inputs();
        for (index, input) in inputs.iter().enumerate() {
            assert!(!inputs[..index].contains(input), "{input:?} is in twice");
        }
    }

    // the frame she leaves the room on. fails the test if she doesn't
    fn frames_to_leave(chapter: &Chapter, start: &Sim, inputs: &[Input]) -> usize {
        match play(chapter, start, inputs) {
            Outcome::LeftRoom { frames, .. } => frames,
            other => panic!("she did not leave the room: {other:?}"),
        }
    }

    // on the floor, 18 px from the first room's right edge
    fn a_short_run_from_the_exit(chapter: &Chapter) -> Sim {
        Sim::new(chapter, 0, at(70, 176))
    }

    fn running() -> Vec<Input> {
        vec![RIGHT; 30]
    }

    // five frames standing still, then the same run
    fn dawdling() -> Vec<Input> {
        [vec![NONE; 5], running()].concat()
    }

    #[test]
    fn improved_inputs_still_get_her_out() {
        let chapter = two_rooms();
        let start = a_short_run_from_the_exit(&chapter);
        let improved = improve(&chapter, &start, &dawdling());
        frames_to_leave(&chapter, &start, &improved);
    }

    #[test]
    fn improve_never_makes_it_slower() {
        let chapter = two_rooms();
        let start = a_short_run_from_the_exit(&chapter);
        for inputs in [running(), dawdling()] {
            let before = frames_to_leave(&chapter, &start, &inputs);
            let improved = improve(&chapter, &start, &inputs);
            assert!(frames_to_leave(&chapter, &start, &improved) <= before);
        }
    }

    #[test]
    fn improve_finds_something_faster_than_dawdling() {
        let chapter = two_rooms();
        let start = a_short_run_from_the_exit(&chapter);
        let before = frames_to_leave(&chapter, &start, &dawdling());
        let improved = improve(&chapter, &start, &dawdling());
        assert!(frames_to_leave(&chapter, &start, &improved) < before);
    }

    // this one assumes improve repeats until no small change helps. drop it if you stop sooner
    #[test]
    fn improve_wins_back_every_frame_spent_standing_still() {
        let chapter = two_rooms();
        let start = a_short_run_from_the_exit(&chapter);
        let just_running = frames_to_leave(&chapter, &start, &running());
        let improved = improve(&chapter, &start, &dawdling());
        assert!(frames_to_leave(&chapter, &start, &improved) <= just_running);
    }

    // three rooms in a row. she starts in the middle one, 6 px from its left edge
    fn between_two_rooms() -> (Chapter, Sim) {
        let chapter = Chapter {
            rooms: vec![floor_room_at(0), floor_room_at(88), floor_room_at(176)],
        };
        let start = Sim::new(&chapter, 1, at(98, 176));
        (chapter, start)
    }

    #[test]
    fn leaving_reports_the_room_she_went_into() {
        let (chapter, start) = between_two_rooms();
        let left = Input { move_x: -1, ..NONE };

        assert!(matches!(
            play(&chapter, &start, &[left; 60]),
            Outcome::LeftRoom { to: 0, .. }
        ));
        assert!(matches!(
            play(&chapter, &start, &[RIGHT; 120]),
            Outcome::LeftRoom { to: 2, .. }
        ));
    }

    #[test]
    fn only_the_same_way_out_counts_as_reaching_the_goal() {
        let (chapter, start) = between_two_rooms();
        let left = Input { move_x: -1, ..NONE };
        let to_the_right = play(&chapter, &start, &[RIGHT; 120]);

        // the left room is much nearer, but it is not where the goal run went
        assert_eq!(
            frames_to_reach(&chapter, &start, &[left; 60], to_the_right),
            None
        );
        assert!(frames_to_reach(&chapter, &start, &[RIGHT; 120], to_the_right).is_some());
        assert_eq!(
            frames_to_reach(&chapter, &start, &[NONE; 60], to_the_right),
            None
        );
    }

    #[test]
    fn improved_inputs_go_into_the_same_room() {
        let (chapter, start) = between_two_rooms();
        let inputs = [vec![NONE; 5], vec![RIGHT; 120]].concat();
        let improved = improve(&chapter, &start, &inputs);

        assert!(matches!(
            play(&chapter, &start, &improved),
            Outcome::LeftRoom { to: 2, .. }
        ));
    }

    #[test]
    fn improve_works_toward_the_finish_trigger_too() {
        let mut chapter = two_rooms();
        chapter.rooms[0].finish_trigger = Some(Rect::new(60, 160, 16, 16));
        let start = Sim::new(&chapter, 0, at(40, 176));
        let inputs = [vec![NONE; 5], vec![RIGHT; 40]].concat();

        let Outcome::Finished { frames: before } = play(&chapter, &start, &inputs) else {
            panic!("the test's own inputs should finish");
        };
        let improved = improve(&chapter, &start, &inputs);
        let Outcome::Finished { frames: after } = play(&chapter, &start, &improved) else {
            panic!("the improved inputs should still finish");
        };
        assert!(after < before);
    }

    #[test]
    fn improved_inputs_end_on_the_frame_she_gets_out() {
        let chapter = two_rooms();
        let start = a_short_run_from_the_exit(&chapter);
        let improved = improve(&chapter, &start, &dawdling());
        assert_eq!(improved.len(), frames_to_leave(&chapter, &start, &improved));
    }

    #[test]
    #[should_panic(expected = "improve needs inputs")]
    fn improve_refuses_inputs_that_do_not_get_her_out() {
        let chapter = two_rooms();
        let start = a_short_run_from_the_exit(&chapter);
        improve(&chapter, &start, &[NONE; 10]);
    }
}
