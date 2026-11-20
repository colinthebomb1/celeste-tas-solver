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

// prints how many states each frame keeps. slow without --release:
// cargo test --release --test search bucket_search_on_room_1 -- --ignored --nocapture
#[test]
#[ignore]
fn bucket_search_on_room_1() {
    use celeste_tas_solver::search::bucket_search::{Goal, Limits, bucket_search};

    let (chapter, start, _) = or_skip!(room_1());
    let room_2 = chapter.room_named("lvl_2").expect("no lvl_2");
    let limits = Limits {
        max_frames: 120,
        max_states: 300_000,
        cut: None,
        max_late: None,
        nearest: None,
    };

    let began = std::time::Instant::now();
    let report = bucket_search(&chapter, &start, Goal::Room(room_2), limits);
    let seconds = began.elapsed().as_secs_f64();

    for (frame, states) in report.states_per_frame.iter().enumerate() {
        eprintln!("frame {:3}: {states} states", frame + 1);
    }
    match &report.inputs {
        Some(inputs) => eprintln!(
            "room 1: the TAS takes {TAS_FRAMES_IN_ROOM_1} frames, the search got {} in {seconds:.1} s",
            inputs.len()
        ),
        None => eprintln!("room 1: no route found in {seconds:.1} s"),
    }
}

// follows the TAS ten frames at a time, since the search can't run all 92 yet. slow without --release:
// cargo test --release --test search the_search_keeps_the_tas_line -- --ignored --nocapture
#[test]
#[ignore]
fn the_search_keeps_the_tas_line_through_room_1() {
    use celeste_tas_solver::search::bucket_search::{Goal, Limits, Pace, follow_route};
    use celeste_tas_solver::sim::step_chapter;

    const STRETCH: usize = 10;

    let (chapter, start, inputs) = or_skip!(room_1());
    let tas = &inputs[..TAS_FRAMES_IN_ROOM_1];
    let goal = Goal::Room(chapter.room_named("lvl_2").expect("no lvl_2"));
    let limits = Limits {
        max_frames: STRETCH,
        max_states: 2_000_000,
        cut: None,
        max_late: None,
        nearest: None,
    };

    let mut sim = start;
    let mut lost = Vec::new();
    for from in (0..TAS_FRAMES_IN_ROOM_1).step_by(STRETCH) {
        let paces = follow_route(&chapter, &sim, &tas[from..], goal, limits);

        // one letter per frame: = level, + the search was there sooner, X lost
        let line: String = paces
            .iter()
            .map(|pace| match pace {
                Pace::Level => '=',
                Pace::Ahead => '+',
                Pace::Lost => 'X',
            })
            .collect();
        eprintln!("frames {:2} to {:2}: {line}", from + 1, from + paces.len());

        for (offset, pace) in paces.iter().enumerate() {
            if *pace == Pace::Lost {
                lost.push(from + offset + 1);
            }
        }
        for &input in tas.iter().skip(from).take(STRETCH) {
            step_chapter(&mut sim, &chapter, input);
        }
    }

    assert!(
        lost.is_empty(),
        "the search lost the TAS on frames {lost:?}"
    );
}

// how fast the TAS would have to be allowed to go for the cut to keep it:
// cargo test --release --test search the_cut_on_room_1 -- --ignored --nocapture
#[test]
#[ignore]
fn the_cut_on_room_1() {
    use celeste_tas_solver::search::bucket_search::{Cut, Goal, Limits, bucket_search};
    use celeste_tas_solver::search::distance::DistanceMap;
    use celeste_tas_solver::sim::step_chapter;

    let (chapter, start, inputs) = or_skip!(room_1());
    let room_2 = chapter.room_named("lvl_2").expect("no lvl_2");
    let goal = Goal::Room(room_2);
    let map = DistanceMap::new(&chapter, start.room, goal);

    // the pace she must keep from each frame of the TAS to be out by frame 92
    let mut sim = start.clone();
    let mut needed: f64 = 0.0;
    eprintln!(
        "start: {:?} px from the exit",
        map.to_goal(sim.player.position)
    );
    for (frame, &input) in (1..TAS_FRAMES_IN_ROOM_1).zip(&inputs) {
        step_chapter(&mut sim, &chapter, input);
        let pixels = map
            .to_goal(sim.player.position)
            .expect("the TAS is on the map");
        let pace = pixels as f64 * 60.0 / (TAS_FRAMES_IN_ROOM_1 - frame) as f64;
        if frame % 10 == 0 {
            eprintln!("frame {frame:2}: {pixels:3} px to go, needs {pace:.0} px/s from here");
        }
        needed = needed.max(pace);
    }
    eprintln!("the TAS needs a top speed of at least {needed:.0} px/s to survive the cut");

    for top_speed in [480.0, 360.0] {
        let limits = Limits {
            max_frames: 120,
            max_states: 400_000,
            cut: Some(Cut {
                frames: TAS_FRAMES_IN_ROOM_1,
                top_speed,
            }),
            max_late: None,
            nearest: None,
        };
        let began = std::time::Instant::now();
        let report = bucket_search(&chapter, &start, goal, limits);
        let last = report.states_per_frame.len();
        eprintln!(
            "top speed {top_speed}: {} after {last} frames ({} states on the last), {:.0} s",
            match &report.inputs {
                Some(inputs) => format!("a route of {} frames", inputs.len()),
                None => "no route".to_string(),
            },
            report.states_per_frame[last - 1],
            began.elapsed().as_secs_f64()
        );
    }
}

// the whole of room 1, dropping states that arrive late:
// cargo test --release --test search late_arrivals_on_room_1 -- --ignored --nocapture
#[test]
#[ignore]
fn late_arrivals_on_room_1() {
    use celeste_tas_solver::search::bucket_search::{Goal, Limits, bucket_search};

    let (chapter, start, _) = or_skip!(room_1());
    let room_2 = chapter.room_named("lvl_2").expect("no lvl_2");

    for max_late in [0, 1, 2, 4] {
        let limits = Limits {
            max_frames: 130,
            max_states: 150_000,
            cut: None,
            max_late: Some(max_late),
            nearest: None,
        };
        let began = std::time::Instant::now();
        let report = bucket_search(&chapter, &start, Goal::Room(room_2), limits);

        let result = match &report.inputs {
            Some(inputs) => format!("a route of {} frames", inputs.len()),
            None => format!("no route after {} frames", report.states_per_frame.len()),
        };
        let every_tenth: Vec<usize> = report
            .states_per_frame
            .iter()
            .copied()
            .step_by(10)
            .collect();
        eprintln!(
            "at most {max_late} late: {result}, {:.0} s. states every 10 frames: {every_tenth:?}",
            began.elapsed().as_secs_f64()
        );
    }
}

// the whole of room 1, keeping only the states nearest the exit:
// cargo test --release --test search the_capped_search_on_room_1 -- --ignored --nocapture
#[test]
#[ignore]
fn the_capped_search_on_room_1() {
    use celeste_tas_solver::search::bucket_search::{
        Goal, Limits, Pace, bucket_search, follow_route,
    };

    let (chapter, start, inputs) = or_skip!(room_1());
    let tas = &inputs[..TAS_FRAMES_IN_ROOM_1];
    let goal = Goal::Room(chapter.room_named("lvl_2").expect("no lvl_2"));

    let nearest = 5_000;
    let limits = Limits {
        max_frames: 200,
        max_states: usize::MAX,
        cut: None,
        max_late: Some(4),
        nearest: Some(nearest),
    };

    let began = std::time::Instant::now();
    let report = bucket_search(&chapter, &start, goal, limits);
    let result = match &report.inputs {
        Some(inputs) => format!("a route of {} frames", inputs.len()),
        None => format!("no route after {} frames", report.states_per_frame.len()),
    };
    eprintln!(
        "keeping {nearest}: {result}, {:.0} s",
        began.elapsed().as_secs_f64()
    );

    // the route has to be real, and single-frame tweaks may shorten it
    if let Some(found) = &report.inputs {
        assert_eq!(frames_into_room_2(&chapter, &start, found), found.len());
        let polished = improve(&chapter, &start, found);
        eprintln!("  after improve: {} frames", polished.len());

        // chapter 1 opens with 1 frame and then 88 before she can move
        let file = format!(
            "# room 1 in {} frames, found by the search\nconsole load 1\n   1\n\n#Start\n  88\n\n#lvl_1\n{}  60\n",
            polished.len(),
            celeste_tas_solver::tas::tas_lines(&polished)
        );
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/routes/1A_lvl_1.tas");
        std::fs::create_dir_all(path.parent().expect("the path has a folder"))
            .expect("could not make the routes folder");
        std::fs::write(&path, file).expect("could not write the route");
        eprintln!("  written to {}", path.display());
    }

    // one letter per frame of the TAS: = level, + the search was there sooner, X lost
    let line: String = follow_route(&chapter, &start, tas, goal, limits)
        .iter()
        .map(|pace| match pace {
            Pace::Level => '=',
            Pace::Ahead => '+',
            Pace::Lost => 'X',
        })
        .collect();
    eprintln!("  the TAS: {line}");
}
