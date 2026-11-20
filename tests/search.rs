use celeste_tas_solver::player::Input;
use celeste_tas_solver::search::{Outcome, improve, play};
use celeste_tas_solver::sim::Sim;

mod common;
use common::{input_from_row, or_skip, start_state};

// frames the community TAS takes to leave the first room. the number to match, then beat
const TAS_FRAMES_IN_ROOM_1: usize = 92;

// the first room as the chapter starts, and the community TAS's inputs for the whole chapter
fn room_1() -> Option<(celeste_tas_solver::level::room::Chapter, Sim, Vec<Input>)> {
    let chapter = common::chapter_1a()?;
    let text = include_str!("traces/community/1A_full.txt");

    let room = chapter.room_named("lvl_1").expect("no lvl_1");
    let first_two_lines: String = text
        .lines()
        .take(2)
        .map(|line| format!("{line}\n"))
        .collect();
    let start = Sim::new(&chapter, room, start_state(&first_two_lines));

    // row 0 is the starting state. each row after it is one frame's input
    let inputs = text.lines().skip(2).map(input_from_row).collect();

    Some((chapter, start, inputs))
}

#[test]
fn the_tas_leaves_room_1_on_its_frame() {
    let (chapter, start, inputs) = or_skip!(room_1());

    assert_eq!(
        play(&chapter, &start, &inputs),
        Outcome::LeftRoom {
            frames: TAS_FRAMES_IN_ROOM_1,
            to: chapter.room_named("lvl_2").expect("no lvl_2"),
        }
    );
}

#[test]
fn the_tas_cut_one_frame_short_is_still_in_room_1() {
    let (chapter, start, inputs) = or_skip!(room_1());

    let short = &inputs[..TAS_FRAMES_IN_ROOM_1 - 1];
    assert_eq!(play(&chapter, &start, short), Outcome::RanOut);
}

#[test]
fn doing_nothing_in_room_1_gets_nowhere() {
    let (chapter, start, _) = or_skip!(room_1());

    let idle = vec![Input::default(); 300];
    assert_eq!(play(&chapter, &start, &idle), Outcome::RanOut);
}

// run with: cargo test --test search improving -- --nocapture
#[test]
fn improving_the_tas_in_room_1_never_makes_it_slower() {
    let (chapter, start, inputs) = or_skip!(room_1());

    let improved = improve(&chapter, &start, &inputs[..TAS_FRAMES_IN_ROOM_1]);

    let room_2 = chapter.room_named("lvl_2").expect("no lvl_2");
    match play(&chapter, &start, &improved) {
        Outcome::LeftRoom { frames, to } if to == room_2 => {
            eprintln!("room 1: the TAS takes {TAS_FRAMES_IN_ROOM_1} frames, improve got {frames}");
            assert!(frames <= TAS_FRAMES_IN_ROOM_1);
        }
        other => panic!("she did not go into room 2: {other:?}"),
    }
}

// frames these inputs take to get from the start of room 1 into room 2
fn frames_into_room_2(
    chapter: &celeste_tas_solver::level::room::Chapter,
    start: &Sim,
    inputs: &[Input],
) -> usize {
    let room_2 = chapter.room_named("lvl_2").expect("no lvl_2");
    match play(chapter, start, inputs) {
        Outcome::LeftRoom { frames, to } if to == room_2 => frames,
        other => panic!("she did not go into room 2: {other:?}"),
    }
}

// the TAS can't be improved a frame at a time, so these two make it worse first.
// each one fails if improve hands back what it was given

#[test]
fn improve_takes_out_idle_frames_put_in_front_of_the_tas() {
    let (chapter, start, inputs) = or_skip!(room_1());

    let tas = &inputs[..TAS_FRAMES_IN_ROOM_1];
    let padded = [vec![Input::default(); 5], tas.to_vec()].concat();
    assert_eq!(
        frames_into_room_2(&chapter, &start, &padded),
        TAS_FRAMES_IN_ROOM_1 + 5
    );

    let improved = improve(&chapter, &start, &padded);
    assert_eq!(
        frames_into_room_2(&chapter, &start, &improved),
        TAS_FRAMES_IN_ROOM_1
    );
}

#[test]
fn improve_repairs_the_tas_after_one_of_its_frames_is_blanked() {
    let (chapter, start, inputs) = or_skip!(room_1());

    // letting go of everything on frame 54 costs three frames. the inputs run past the
    // TAS's exit so the slower run still has buttons to press
    let mut damaged = inputs[..TAS_FRAMES_IN_ROOM_1 + 30].to_vec();
    damaged[53] = Input::default();
    assert_eq!(
        frames_into_room_2(&chapter, &start, &damaged),
        TAS_FRAMES_IN_ROOM_1 + 3
    );

    let improved = improve(&chapter, &start, &damaged);
    assert_eq!(
        frames_into_room_2(&chapter, &start, &improved),
        TAS_FRAMES_IN_ROOM_1
    );
}
