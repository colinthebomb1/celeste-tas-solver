use celeste_tas_solver::physics::{step, IVec2, Input, PlayerState};
use celeste_tas_solver::trace::parse_trace;
use celeste_tas_solver::Vec2;

const NONE: Input = Input { move_x: 0, jump: false };
const RIGHT: Input = Input { move_x: 1, jump: false };
const JUMP: Input = Input { move_x: 0, jump: true };

fn expand(lines: &[(usize, Input)]) -> Vec<Input> {
    lines
        .iter()
        .flat_map(|&(count, input)| std::iter::repeat_n(input, count))
        .collect()
}

fn column(line: &str, index: usize) -> &str {
    line.split('\t').nth(index).expect("missing column")
}

fn fmt(v: f64) -> String {
    format!("{:.12}", v + 0.0)
}

#[derive(Debug, PartialEq)]
struct Observed {
    position: String,
    speed: String,
}

fn observe(s: &PlayerState) -> Observed {
    let px = s.position.x as f64 + s.remainder.x as f64;
    let py = s.position.y as f64 + s.remainder.y as f64;
    Observed {
        position: format!("{}, {}", fmt(px), fmt(py)),
        speed: format!("{}, {}", fmt(s.speed.x as f64), fmt(s.speed.y as f64)),
    }
}

fn start_state(text: &str) -> PlayerState {
    let first = parse_trace(text)[0];
    let (x, y) = (first.position.x, first.position.y);
    let position = IVec2 { x: x.round() as i32, y: y.round() as i32 };
    let remainder = Vec2 { x: x - x.round(), y: y - y.round() };
    PlayerState::new(position, remainder, first.speed)
}

fn assert_matches_trace(name: &str, text: &str, inputs: &[Input]) {
    let rows: Vec<&str> = text.lines().skip(1).collect();
    assert_eq!(
        inputs.len() + 1,
        rows.len(),
        "{name}: the input list must cover every frame after the first"
    );

    let mut state = start_state(text);
    for (i, &input) in inputs.iter().enumerate() {
        let frame = i + 1;
        state = step(state, input);
        let expected = Observed {
            position: column(rows[frame], 4).to_string(),
            speed: column(rows[frame], 5).to_string(),
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
