use celeste_tas_solver::trace::parse_trace;

fn main() {
    let text = std::fs::read_to_string("tests/traces/01_run.txt").unwrap();
    let frames = parse_trace(&text);

    println!("{} frames", frames.len());
    for (i, frame) in frames.iter().take(3).enumerate() {
        println!("{i}: {frame:?}");
    }
}
