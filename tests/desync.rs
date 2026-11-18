use celeste_tas_solver::player::{step, track_buttons, IVec2, Input, Mode, PlayerState};
use celeste_tas_solver::sim::step_room;
use celeste_tas_solver::trace::parse_trace;
use celeste_tas_solver::level::transition::{before_transition, find_transition, finish_transition, TRANSITION_FRAMES};
use celeste_tas_solver::Vec2;

mod common;

const NONE: Input = Input { move_x: 0, move_y: 0, jump: false, jump2: false, dash: false, grab: false, crouch_dash: false };
const RIGHT: Input = Input { move_x: 1, ..NONE };
const JUMP: Input = Input { jump: true, ..NONE };
const RIGHT_JUMP: Input = Input { move_x: 1, jump: true, ..NONE };
const RIGHT_DASH: Input = Input { move_x: 1, dash: true, ..NONE };
const UP_DASH: Input = Input { move_y: -1, dash: true, ..NONE };
const UP_RIGHT_DASH: Input = Input { move_x: 1, move_y: -1, dash: true, ..NONE };

fn expand(lines: &[(usize, Input)]) -> Vec<Input> {
    lines
        .iter()
        .flat_map(|&(count, input)| std::iter::repeat_n(input, count))
        .collect()
}

fn column(line: &str, index: usize) -> &str {
    line.split('\t').nth(index).expect("missing column")
}

// The game prints Math.Round(v, 12): scale by 1e12 as a double, round half to even.
fn fmt(v: f64) -> String {
    let scaled = (v.abs() * 1e12).round_ties_even() as u64;
    let sign = if v < 0.0 { "-" } else { "" };
    format!("{sign}{}.{:012}", scaled / 1_000_000_000_000, scaled % 1_000_000_000_000)
}

#[derive(Debug, PartialEq)]
struct Observed {
    position: String,
    speed: String,
    mode: String,
    dead: bool,
}

fn observe(s: &PlayerState) -> Observed {
    let px = s.position.x as f64 + s.remainder.x as f64;
    let py = s.position.y as f64 + s.remainder.y as f64;
    let mode = match s.mode {
        Mode::Normal => "StNormal",
        Mode::Climb => "StClimb",
        Mode::DashStart | Mode::Dashing { .. } => "StDash",
    };
    Observed {
        position: format!("{}, {}", fmt(px), fmt(py)),
        speed: format!("{}, {}", fmt(s.speed.x as f64), fmt(s.speed.y as f64)),
        mode: mode.to_string(),
        dead: s.dead,
    }
}

// position is read as f64 so the remainder keeps all its bits away from the origin
fn start_state(text: &str) -> PlayerState {
    let first = parse_trace(text)[0];
    let row = text.lines().nth(1).expect("empty trace");
    let (x, y) = column(row, 4).split_once(", ").expect("bad position");
    let (x, y): (f64, f64) = (x.parse().unwrap(), y.parse().unwrap());
    let position = IVec2 { x: x.round() as i32, y: y.round() as i32 };
    let remainder = Vec2 { x: (x - x.round()) as f32, y: (y - y.round()) as f32 };
    PlayerState::new(position, remainder, first.speed)
}

fn game_row(row: &str) -> Observed {
    Observed {
        position: column(row, 4).to_string(),
        speed: column(row, 5).to_string(),
        mode: column(row, 6).to_string(),
        dead: is_dead(row),
    }
}

fn is_dead(row: &str) -> bool {
    column(row, 7).split(' ').any(|status| status == "Dead")
}

// A trace that ends in death stops at the frame she dies, so it can be shorter than its inputs.
fn assert_matches_trace(name: &str, text: &str, inputs: &[Input]) {
    assert_matches_trace_in("lvl_1", name, text, inputs);
}

fn assert_matches_trace_in(room_name: &str, name: &str, text: &str, inputs: &[Input]) {
    let rows: Vec<&str> = text.lines().skip(1).collect();
    let died = rows.last().is_some_and(|row| is_dead(row));
    assert!(
        inputs.len() + 1 == rows.len() || (died && inputs.len() + 1 > rows.len()),
        "{name}: the input list must cover every frame after the first"
    );

    let Some(room) = common::room(room_name) else {
        eprintln!("{name}: skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let mut state = start_state(text);
    for (i, &input) in inputs.iter().enumerate() {
        let frame = i + 1;
        if frame >= rows.len() {
            break;
        }
        state = step(state, input, &room);
        assert_eq!(
            observe(&state),
            game_row(rows[frame]),
            "\n{name}: desync on frame {frame} (input {input:?})\n  \
             (left = simulator, right = the game)\n  state: {state:?}\n"
        );
    }
}

#[test]
fn run_right_then_release() {
    let inputs = expand(&[(10, RIGHT), (25, NONE)]);
    assert_matches_trace("01_run", include_str!("traces/01_run.txt"), &inputs);
}

#[test]
fn full_jump() {
    let inputs = expand(&[(15, JUMP), (40, NONE)]);
    assert_matches_trace("02_jump_full", include_str!("traces/02_jump_full.txt"), &inputs);
}

#[test]
fn short_jump() {
    let inputs = expand(&[(1, JUMP), (40, NONE)]);
    assert_matches_trace("03_jump_short", include_str!("traces/03_jump_short.txt"), &inputs);
}

#[test]
fn run_jump() {
    let inputs = expand(&[(10, RIGHT), (12, RIGHT_JUMP), (25, RIGHT)]);
    assert_matches_trace("04_run_jump", include_str!("traces/04_run_jump.txt"), &inputs);
}

#[test]
fn dash_right() {
    let inputs = expand(&[(5, RIGHT_DASH), (35, NONE)]);
    assert_matches_trace("05_dash_right", include_str!("traces/05_dash_right.txt"), &inputs);
}

#[test]
fn dash_up() {
    let inputs = expand(&[(5, UP_DASH), (45, NONE)]);
    assert_matches_trace("06_dash_up", include_str!("traces/06_dash_up.txt"), &inputs);
}

#[test]
fn dash_up_right() {
    let inputs = expand(&[(5, UP_RIGHT_DASH), (45, NONE)]);
    assert_matches_trace("07_dash_upright", include_str!("traces/07_dash_upright.txt"), &inputs);
}

#[test]
fn dash_held_right_into_wall() {
    let inputs = expand(&[(30, RIGHT_DASH)]);
    assert_matches_trace("08_dash_held_right", include_str!("traces/08_dash_held_right.txt"), &inputs);
}

#[test]
fn dash_direction_read_late() {
    let inputs = expand(&[(1, UP_DASH), (40, NONE)]);
    assert_matches_trace("09_dash_dir_read_late", include_str!("traces/09_dash_dir_read_late.txt"), &inputs);
}

#[test]
fn dash_up_held_one_frame_short() {
    let inputs = expand(&[(4, UP_DASH), (46, NONE)]);
    assert_matches_trace("10_dash_up_4_frames", include_str!("traces/10_dash_up_4_frames.txt"), &inputs);
}

#[test]
fn fall_out_of_the_room() {
    let inputs = expand(&[(48, RIGHT)]);
    assert_matches_trace_in("lvl_3", "11_fall_out", include_str!("traces/11_fall_out.txt"), &inputs);
}

// the trace's Inputs column looks like "3/   5,R,X"
fn input_from_row(row: &str) -> Input {
    let (_, line) = column(row, 1).split_once('/').expect("bad inputs column");
    let mut input = NONE;
    for key in line.split(',').skip(1) {
        match key.trim() {
            "L" => input.move_x = -1,
            "R" => input.move_x = 1,
            "U" => input.move_y = -1,
            "D" => input.move_y = 1,
            "J" => input.jump = true,
            "K" => input.jump2 = true,
            "X" | "C" => input.dash = true,
            "Z" | "V" => input.crouch_dash = true,
            "G" => input.grab = true,
            _ => {}
        }
    }
    input
}

// the community TAS's first room, up to where the room transition takes control away
#[test]
fn community_lvl_1() {
    let text: String = include_str!("traces/community/1A_lvl_1.txt")
        .lines()
        .take_while(|row| !row.contains("NoControl"))
        .map(|row| format!("{row}\n"))
        .collect();
    let inputs: Vec<Input> = text.lines().skip(2).map(input_from_row).collect();
    assert_matches_trace_in("lvl_1", "community lvl_1", &text, &inputs);
}

// one simulation of the community TAS through every room of 1A.
// run with: cargo test --test desync chapter -- --nocapture
#[test]
fn chapter_1a() {
    let text = include_str!("traces/community/1A_full.txt");
    let Some(mut chapter) = common::chapter_1a() else {
        eprintln!("chapter 1A: skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };

    // the run ends when the ending cutscene takes over
    let in_cutscene = |row: &str| column(row, 7).split(' ').any(|status| status == "Cutscene");
    let rows: Vec<&str> = text.lines().skip(1).take_while(|row| !in_cutscene(row)).collect();
    let total = rows.len() - 1;
    let first_two_lines: String = text.lines().take(2).map(|line| format!("{line}\n")).collect();

    let mut state = start_state(&first_two_lines);
    let mut room = chapter.room_named("lvl_1").expect("no lvl_1");
    let mut matched = 0;
    let mut frame = 1;

    let desync = |frame: usize, room: &str, state: &PlayerState| {
        eprintln!(
            "chapter 1A: desync on frame {frame} in {} (tas line {}, inputs {}, time {})\n  simulator: {:?}\n  game:      {:?}",
            room,
            column(rows[frame], 0).trim(),
            column(rows[frame], 1).trim(),
            column(rows[frame], 3).trim(),
            observe(state),
            game_row(rows[frame]),
        );
    };

    while frame < rows.len() {
        state = step_room(state, input_from_row(rows[frame]), &mut chapter.rooms[room]);

        let transition = if state.dead { None } else { find_transition(&state, &chapter, room) };
        if let Some(transition) = transition {
            before_transition(&mut state, transition.direction);
        }

        if observe(&state) != game_row(rows[frame]) {
            desync(frame, &chapter.rooms[room].name, &state);
            break;
        }
        matched = frame;
        frame += 1;

        // nothing is simulated during a transition. the next row compared is the one where it ends
        if let Some(transition) = transition {
            finish_transition(&mut state, &chapter, transition);
            room = transition.to;

            // buttons let go of during the transition count as presses afterwards
            let end = (frame + TRANSITION_FRAMES - 1).min(rows.len() - 1);
            for &row in &rows[frame..=end] {
                track_buttons(&mut state, input_from_row(row));
            }
            frame += TRANSITION_FRAMES - 1;

            if frame >= rows.len() {
                break;
            }
            if observe(&state) != game_row(rows[frame]) {
                desync(frame, &chapter.rooms[room].name, &state);
                break;
            }
            matched = frame;
            frame += 1;
        }
    }

    eprintln!("chapter 1A: matched {matched} of {total} frames");
    assert_eq!(matched, total, "chapter 1A: desynced after frame {matched}");
}
