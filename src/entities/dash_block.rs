use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities};
use crate::level::room::Room;

// the game freezes this long when it breaks,
// also when she leaves a room that still has one
pub const DASH_BLOCK_FREEZE_TIME: f32 = 0.05;

// a solid block that a dash breaks. broken means its solid is no longer collidable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashBlock {
    pub solid: usize,
}

impl Room {
    // (x, y) is its top-left corner
    pub fn add_dash_block(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let solid = self.add_solid(Rect::new(x, y, width, height));
        self.dash_blocks.push(DashBlock { solid });
    }
}

pub fn load_dash_blocks(level: &Element, origin: IVec2, room: &mut Room) -> Result<(), MapError> {
    for entity in entities(level, "dashBlock") {
        room.add_dash_block(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
            entity.require("width")?,
            entity.require("height")?,
        );
    }

    Ok(())
}
