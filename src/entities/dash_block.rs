use crate::level::map::{Element, MapError};
use crate::player::IVec2;
use crate::level::room::Rect;

// the game freezes this long when it breaks,
// also when she leaves a room that still has one
pub const DASH_BLOCK_FREEZE_TIME: f32 = 0.05;

// a solid block that a dash breaks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashBlock {
    pub hitbox: Rect,
    pub broken: bool,
}

impl DashBlock {
    // (x, y) is its top-left corner
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> DashBlock {
        DashBlock { hitbox: Rect { left: x, top: y, width, height }, broken: false }
    }
}

pub fn load_dash_blocks(level: &Element, origin: IVec2) -> Result<Vec<DashBlock>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut dash_blocks = Vec::new();

    for entity in entities.children.iter().filter(|entity| entity.name == "dashBlock") {
        let int = |key: &'static str| entity.int(key).ok_or(MapError::MissingField(key));

        dash_blocks.push(DashBlock::new(origin.x + int("x")?, origin.y + int("y")?, int("width")?, int("height")?));
    }

    Ok(dash_blocks)
}
