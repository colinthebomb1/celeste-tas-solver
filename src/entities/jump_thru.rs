use crate::map::{Element, MapError};
use crate::physics::IVec2;
use crate::room::Rect;

pub const JUMP_THRU_HEIGHT: i32 = 5;

// a platform she can jump up through and stand on. (x, y) is its top-left corner
pub fn jump_thru(x: i32, y: i32, width: i32) -> Rect {
    Rect { left: x, top: y, width, height: JUMP_THRU_HEIGHT }
}

pub fn load_jump_thrus(level: &Element, origin: IVec2) -> Result<Vec<Rect>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut jump_thrus = Vec::new();

    for entity in entities.children.iter().filter(|entity| entity.name == "jumpThru") {
        let x = entity.int("x").ok_or(MapError::MissingField("x"))?;
        let y = entity.int("y").ok_or(MapError::MissingField("y"))?;
        let width = entity.int("width").ok_or(MapError::MissingField("width"))?;

        jump_thrus.push(jump_thru(origin.x + x, origin.y + y, width));
    }

    Ok(jump_thrus)
}
