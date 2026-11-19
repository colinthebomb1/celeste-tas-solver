use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities_named};
use crate::level::room::Room;

const JUMP_THRU_HEIGHT: i32 = 5;

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

pub(crate) fn load_jump_thrus(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for entity in entities_named(level, "jumpThru") {
        room.jump_thrus.push(JumpThru::new(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
            entity.require("width")?,
        ));
    }

    Ok(())
}
