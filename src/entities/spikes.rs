use crate::level::map::{Element, MapError};
use crate::player::IVec2;
use crate::level::room::{Direction, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spikes {
    pub direction: Direction,
    pub hitbox: Rect,
}

impl Spikes {
    pub fn new(direction: Direction, x: i32, y: i32, size: i32) -> Spikes {
        let hitbox = match direction {
            Direction::Up => Rect { 
                left: x,
                top: y - 3,
                width: size,
                height: 3
            },
            Direction::Down => Rect { 
                left: x,
                top: y,
                width: size,
                height: 3
            },
            Direction::Right => Rect { 
                left: x,
                top: y,
                width: 3,
                height: size
            },
            Direction::Left => Rect { 
                left: x - 3,
                top: y,
                width: 3,
                height: size
            },
        };

        Spikes { direction, hitbox }
    }
}

// The spikes among a level's "entities" children: elements named "spikesUp", "spikesDown",
// "spikesLeft" and "spikesRight". "x" and "y" are relative to the room's origin. The length
// is "width" for Up and Down and "height" for Left and Right. A level with no "entities"
// child has no spikes.
pub fn load_spikes(level: &Element, origin: IVec2) -> Result<Vec<Spikes>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut spikes = Vec::new();

    for entity in entities.children.iter() {
        let (direction, length_attribute) = match entity.name.as_str() {
            "spikesUp" => (Direction::Up, "width"),
            "spikesDown" => (Direction::Down, "width"),
            "spikesLeft" => (Direction::Left, "height"),
            "spikesRight" => (Direction::Right, "height"),
            _ => continue,
        };

        let x = entity.int("x").ok_or(MapError::MissingField("x"))?;
        let y = entity.int("y").ok_or(MapError::MissingField("y"))?;
        let size = entity.int(length_attribute).ok_or(MapError::MissingField(length_attribute))?;

        spikes.push(Spikes::new(direction, origin.x + x, origin.y + y, size));
    }

    Ok(spikes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect { left, top, width, height }
    }

    #[test]
    fn spikes_hitbox_sits_on_the_side_the_points_face() {
        assert_eq!(Spikes::new(Direction::Up, 40, 168, 24).hitbox, rect(40, 165, 24, 3));
        assert_eq!(Spikes::new(Direction::Down, 40, 168, 24).hitbox, rect(40, 168, 24, 3));
        assert_eq!(Spikes::new(Direction::Left, 40, 168, 24).hitbox, rect(37, 168, 3, 24));
        assert_eq!(Spikes::new(Direction::Right, 40, 168, 24).hitbox, rect(40, 168, 3, 24));
    }

    #[test]
    fn spikes_remember_their_direction() {
        assert_eq!(Spikes::new(Direction::Left, 0, 0, 8).direction, Direction::Left);
    }
}
