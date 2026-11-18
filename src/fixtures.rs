// helpers shared by the unit tests
use crate::geom::{IVec2, Vec2};
use crate::level::room::Room;
use crate::player::{Input, Player};
use crate::sim::step_room;

// standing still at a whole pixel
pub fn at(x: i32, y: i32) -> Player {
    Player::new(IVec2 { x, y }, Vec2::default(), Vec2::default())
}

// 11 x 23 tiles of open air
pub fn open_room() -> Room {
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(23))
}

// the same, with a floor along the bottom row. the floor's top is y = 176
pub fn floor_room() -> Room {
    let tiles = format!("{}11111111111", "00000000000\n".repeat(22));
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &tiles)
}

// runs the room for some frames with no buttons held
pub fn idle(room: &mut Room, player: Player, frames: usize) -> Player {
    (0..frames).fold(player, |player, _| {
        step_room(player, Input::default(), room)
    })
}
