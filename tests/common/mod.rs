// each test file uses a different part of this
#![allow(dead_code)]

use celeste_tas_solver::map::{find_celeste_dir, load_chapter, load_map, load_room, Element};
use celeste_tas_solver::room::{Chapter, Room};

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
