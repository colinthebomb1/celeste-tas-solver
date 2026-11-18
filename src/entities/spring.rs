use crate::level::map::{Element, MapError};
use crate::level::room::Rect;
use crate::player::IVec2;

pub const SPRING_WIDTH: i32 = 16;
pub const SPRING_HEIGHT: i32 = 6;

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
pub fn load_springs(level: &Element, origin: IVec2) -> Result<Vec<Spring>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut springs = Vec::new();

    for entity in entities
        .children
        .iter()
        .filter(|entity| entity.name == "spring")
    {
        let x = entity.int("x").ok_or(MapError::MissingField("x"))?;
        let y = entity.int("y").ok_or(MapError::MissingField("y"))?;

        springs.push(Spring::new(origin.x + x, origin.y + y));
    }

    Ok(springs)
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
