use celeste_tas_solver::physics::{step, IVec2, Input, Mode, PlayerState};
use celeste_tas_solver::trace::parse_trace;
use celeste_tas_solver::Vec2;

mod common;

const NONE: Input = Input { move_x: 0, move_y: 0, jump: false, dash: false };
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

// The game rounds to 17 significant digits first, then half-to-even to 12 decimals.
fn fmt(v: f64) -> String {
    let sci = format!("{:.16e}", v.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap();
    let digits: u128 = mantissa.replace('.', "").parse().unwrap();
    let shift = exp.parse::<i32>().unwrap() - 4;
    let scaled = if shift >= 0 {
        digits * 10u128.pow(shift as u32)
    } else if shift < -38 {
        0
    } else {
        let div = 10u128.pow(-shift as u32);
        let (q, r) = (digits / div, digits % div);
        if r * 2 > div || (r * 2 == div && q % 2 == 1) { q + 1 } else { q }
    };
    let sign = if v < 0.0 { "-" } else { "" };
    format!("{sign}{}.{:012}", scaled / 1_000_000_000_000, scaled % 1_000_000_000_000)
}

#[derive(Debug, PartialEq)]
struct Observed {
    position: String,
    speed: String,
    mode: String,
}

fn observe(s: &PlayerState) -> Observed {
    let px = s.position.x as f64 + s.remainder.x as f64;
    let py = s.position.y as f64 + s.remainder.y as f64;
    let mode = match s.mode {
        Mode::Normal => "StNormal",
        Mode::DashStart | Mode::Dashing { .. } => "StDash",
    };
    Observed {
        position: format!("{}, {}", fmt(px), fmt(py)),
        speed: format!("{}, {}", fmt(s.speed.x as f64), fmt(s.speed.y as f64)),
        mode: mode.to_string(),
    }
}

fn start_state(text: &str) -> PlayerState {
    let first = parse_trace(text)[0];
    let (x, y) = (first.position.x, first.position.y);
    let position = IVec2 { x: x.round() as i32, y: y.round() as i32 };
    let remainder = Vec2 { x: x - x.round(), y: y - y.round() };
    PlayerState::new(position, remainder, first.speed)
}

fn is_dead(row: &str) -> bool {
    column(row, 7).split(' ').any(|status| status == "Dead")
}

// Death isn't simulated yet, so a trace that ends in death is compared up to the frame before.
fn assert_matches_trace(name: &str, text: &str, inputs: &[Input]) {
    let rows: Vec<&str> = text.lines().skip(1).collect();
    let died = rows.last().is_some_and(|row| is_dead(row));
    assert!(
        inputs.len() + 1 == rows.len() || (died && inputs.len() + 1 > rows.len()),
        "{name}: the input list must cover every frame after the first"
    );

    let Some(room) = common::first_room() else {
        eprintln!("{name}: skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let mut state = start_state(text);
    for (i, &input) in inputs.iter().enumerate() {
        let frame = i + 1;
        if frame >= rows.len() || is_dead(rows[frame]) {
            break;
        }
        state = step(state, input, &room);
        let expected = Observed {
            position: column(rows[frame], 4).to_string(),
            speed: column(rows[frame], 5).to_string(),
            mode: column(rows[frame], 6).to_string(),
        };
        assert_eq!(
            observe(&state),
            expected,
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
