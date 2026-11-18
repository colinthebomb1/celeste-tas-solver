use crate::level::map::{Element, MapError};
use crate::level::room::Rect;
use crate::player::IVec2;

pub const JUMP_THRU_HEIGHT: i32 = 5;

// a platform she can jump up through and stand on. (x, y) is its top-left corner
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpThru {
    pub hitbox: Rect,
}

impl JumpThru {
    pub fn new(x: i32, y: i32, width: i32) -> JumpThru {
        JumpThru {
            hitbox: Rect {
                left: x,
                top: y,
                width,
                height: JUMP_THRU_HEIGHT,
            },
        }
    }
}

pub fn load_jump_thrus(level: &Element, origin: IVec2) -> Result<Vec<JumpThru>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut jump_thrus = Vec::new();

    for entity in entities
        .children
        .iter()
        .filter(|entity| entity.name == "jumpThru")
    {
        let x = entity.int("x").ok_or(MapError::MissingField("x"))?;
        let y = entity.int("y").ok_or(MapError::MissingField("y"))?;
        let width = entity.int("width").ok_or(MapError::MissingField("width"))?;

        jump_thrus.push(JumpThru::new(origin.x + x, origin.y + y, width));
    }

    Ok(jump_thrus)
}
