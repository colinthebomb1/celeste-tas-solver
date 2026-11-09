use celeste_tas_solver::trace::{parse_trace, TraceFrame};
use celeste_tas_solver::Vec2;

fn f(s: &str) -> f32 {
    s.parse().unwrap()
}

fn v(x: &str, y: &str) -> Vec2 {
    Vec2 { x: f(x), y: f(y) }
}

const RUN: &str = include_str!("traces/01_run.txt");
const JUMP: &str = include_str!("traces/02_jump_full.txt");

#[test]
fn run_trace_has_one_frame_per_row() {
    // 1 idle frame + 10 frames holding R + 25 released = 36
    assert_eq!(parse_trace(RUN).len(), 36);
}

#[test]
fn run_trace_first_frame_is_the_idle_start_state() {
    let frames = parse_trace(RUN);
    assert_eq!(
        frames[0],
        TraceFrame {
            position: v("19.000000000000", "144.000000000000"),
            speed: v("0.000000000000", "0.000000000000"),
        }
    );
}

#[test]
fn run_trace_first_frames_of_acceleration() {
    let frames = parse_trace(RUN);
    assert_eq!(frames[1].position, v("19.277778863907", "144.000000000000"));
    assert_eq!(frames[1].speed, v("16.666698455811", "0.000000000000"));
    assert_eq!(frames[2].position, v("19.833336591721", "144.000000000000"));
    assert_eq!(frames[2].speed, v("33.333396911621", "0.000000000000"));
}

#[test]
fn run_trace_first_frame_after_releasing_right() {
    let frames = parse_trace(RUN);
    assert_eq!(frames[11].position, v("31.888921856880", "144.000000000000"));
    assert_eq!(frames[11].speed, v("73.333297729492", "0.000000000000"));
}

#[test]
fn run_trace_last_frame_is_stopped() {
    let frames = parse_trace(RUN);
    let last = frames.last().unwrap();
    assert_eq!(last.position, v("34.000029459596", "144.000000000000"));
    assert_eq!(last.speed, v("0.000000000000", "0.000000000000"));
}

#[test]
fn jump_trace_parses_negative_speeds() {
    let frames = parse_trace(JUMP);
    assert_eq!(frames.len(), 56);
    assert_eq!(frames[1].position, v("19.000000000000", "142.249996542931"));
    assert_eq!(frames[1].speed, v("0.000000000000", "-105.000000000000"));
}
