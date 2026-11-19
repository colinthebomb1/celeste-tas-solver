use celeste_tas_solver::player::Input;
use celeste_tas_solver::search::{Outcome, improve, play};
use celeste_tas_solver::sim::Sim;

mod common;
use common::{input_from_row, start_state};

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
    let Some((chapter, start, inputs)) = room_1() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };

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
    let Some((chapter, start, inputs)) = room_1() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };

    let short = &inputs[..TAS_FRAMES_IN_ROOM_1 - 1];
    assert_eq!(play(&chapter, &start, short), Outcome::RanOut);
}

#[test]
fn doing_nothing_in_room_1_gets_nowhere() {
    let Some((chapter, start, _)) = room_1() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };

    let idle = vec![Input::default(); 300];
    assert_eq!(play(&chapter, &start, &idle), Outcome::RanOut);
}

// run with: cargo test --test search improving -- --nocapture
#[test]
fn improving_the_tas_in_room_1_never_makes_it_slower() {
    let Some((chapter, start, inputs)) = room_1() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };

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
