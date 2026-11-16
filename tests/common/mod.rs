use celeste_tas_solver::map::{find_celeste_dir, load_map, load_room, Element};
use celeste_tas_solver::room::Room;

pub fn map_1a() -> Option<Element> {
    let path = find_celeste_dir()?.join("Content/Maps/1-ForsakenCity.bin");
    Some(load_map(&path).expect("failed to read 1A map"))
}

pub fn room(name: &str) -> Option<Room> {
    Some(load_room(&map_1a()?, name).expect("failed to load room"))
}
