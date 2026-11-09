use celeste_tas_solver::physics::{step, IVec2, Input, PlayerState};
use celeste_tas_solver::trace::parse_trace;
use celeste_tas_solver::Vec2;

const RUN: &str = include_str!("traces/01_run.txt");

const NONE: Input = Input { move_x: 0 };
const RIGHT: Input = Input { move_x: 1 };

fn expand(lines: &[(usize, Input)]) -> Vec<Input> {
    lines
        .iter()
        .flat_map(|&(count, input)| std::iter::repeat_n(input, count))
        .collect()
}

fn exact_positions(text: &str) -> Vec<(IVec2, Vec2)> {
    text.lines()
        .skip(1)
        .map(|line| {
            let column = line.split('\t').nth(4).expect("missing Position column");
            let (x, y) = column.split_once(", ").expect("Position is not 'x, y'");
            let (x, y): (f64, f64) = (x.parse().unwrap(), y.parse().unwrap());
            let (ix, iy) = (x.round(), y.round());
            (
                IVec2 { x: ix as i32, y: iy as i32 },
                Vec2 { x: (x - ix) as f32, y: (y - iy) as f32 },
            )
        })
        .collect()
}

fn assert_matches_trace(name: &str, text: &str, inputs: &[Input]) {
    let speeds: Vec<Vec2> = parse_trace(text).iter().map(|f| f.speed).collect();
    let positions = exact_positions(text);
    assert_eq!(
        inputs.len() + 1,
        positions.len(),
        "{name}: the input list must cover every frame after the first"
    );

    let expected = |i: usize| PlayerState {
        position: positions[i].0,
        remainder: positions[i].1,
        speed: speeds[i],
    };

    let mut state = expected(0);
    for (i, &input) in inputs.iter().enumerate() {
        let frame = i + 1;
        state = step(state, input);
        let want = expected(frame);
        assert_eq!(
            state, want,
            "\n{name}: desync on frame {frame} (input {input:?})\n  \
             (left = your simulator, right = the game)\n"
        );
    }
}

#[test]
fn run_right_then_release() {
    // Matches golden_traces.tas Test_01_run: 10,R then 25
    let inputs = expand(&[(10, RIGHT), (25, NONE)]);
    assert_matches_trace("01_run", RUN, &inputs);
}
