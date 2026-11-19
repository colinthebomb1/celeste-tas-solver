use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities_named};
use crate::level::room::Room;

const SPRING_WIDTH: i32 = 16;
const SPRING_HEIGHT: i32 = 6;

// a floor spring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spring {
    pub hitbox: Rect,
}

impl Spring {
    // (x, y) is the middle of its base
    pub fn new(x: i32, y: i32) -> Spring {
        let hitbox = Rect {
            left: x - SPRING_WIDTH / 2,
            top: y - SPRING_HEIGHT,
            width: SPRING_WIDTH,
            height: SPRING_HEIGHT,
        };

        Spring { hitbox }
    }
}

// only floor springs. the ones on walls are other entities
pub(crate) fn load_springs(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for entity in entities_named(level, "spring") {
        room.springs.push(Spring::new(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_hitbox_sits_on_its_base() {
        assert_eq!(
            Spring::new(100, 50).hitbox,
            Rect {
                left: 92,
                top: 44,
                width: 16,
                height: 6
            }
        );
    }
}
