// each test file uses a different part of this
#![allow(dead_code)]

use celeste_tas_solver::geom::{IVec2, Vec2};
use celeste_tas_solver::level::map::{
    Element, find_celeste_dir, load_chapter, load_map, load_room,
};
use celeste_tas_solver::level::room::{Chapter, Room};
use celeste_tas_solver::player::{Input, Player};
use celeste_tas_solver::trace::parse_trace;

// unwraps something loaded from the game's files. without a Celeste install it ends the test
macro_rules! or_skip {
    ($loaded:expr) => {
        match $loaded {
            Some(loaded) => loaded,
            None => {
                eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
                return;
            }
        }
    };
}
pub(crate) use or_skip;

pub fn map_1a() -> Option<Element> {
    let path = find_celeste_dir()?.join("Content/Maps/1-ForsakenCity.bin");
    Some(load_map(&path).expect("failed to read 1A map"))
}

pub fn chapter_1a() -> Option<Chapter> {
    Some(load_chapter(&map_1a()?).expect("failed to load 1A"))
}

pub fn room(name: &str) -> Option<Room> {
    Some(load_room(&map_1a()?, name).expect("failed to load room"))
}

pub const NONE: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    demodash: false,
};

pub fn column(line: &str, index: usize) -> &str {
    line.split('\t').nth(index).expect("missing column")
}

// position is read as f64 so the subpixel keeps all its bits away from the origin
pub fn start_state(text: &str) -> Player {
    let first = parse_trace(text)[0];
    let row = text.lines().nth(1).expect("empty trace");
    let (x, y) = column(row, 4).split_once(", ").expect("bad position");
    let (x, y): (f64, f64) = (x.parse().unwrap(), y.parse().unwrap());
    let position = IVec2 {
        x: x.round() as i32,
        y: y.round() as i32,
    };
    let subpixel = Vec2 {
        x: (x - x.round()) as f32,
        y: (y - y.round()) as f32,
    };
    Player::new(position, subpixel, first.speed)
}

// the trace's Inputs column looks like "3/   5,R,X"
pub fn input_from_row(row: &str) -> Input {
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
            "Z" | "V" => input.demodash = true,
            "G" => input.grab = true,
            _ => {}
        }
    }
    input
}
