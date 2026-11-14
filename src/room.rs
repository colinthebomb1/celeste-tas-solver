use crate::physics::IVec2;

pub const TILE_SIZE: i32 = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    // world position of the room's top-left corner, in pixels
    pub origin: IVec2,
    // size in tiles
    pub width: usize,
    pub height: usize,
    // tile (x, y) is at index y * width + x
    solid: Vec<bool>,
}

impl Room {
    pub fn from_tiles(origin: IVec2, text: &str) -> Room {
        let height = text.lines().count();
        let width = text
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let mut solid = vec![false; width * height];
        
        for (y, line) in text.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch != '0' {
                    solid[y * width + x] = true;
                }
            }
        }

        Room {
            origin,
            width,
            height,
            solid
        }
    }

    pub fn is_solid(&self, tile_x: i32, tile_y: i32) -> bool {
        if tile_x as usize >= self.width || tile_x < 0 || tile_y as usize >= self.height || tile_y < 0 {
            return false;
        }

        self.solid[tile_y as usize * self.width + tile_x as usize]
    }

    pub fn collide_rect(&self, left: i32, top: i32, width: i32, height: i32) -> bool {
        let right = left + width - 1;
        let bottom = top + height - 1;

        // div_euclid rounds down, so pixels left of or above the room get negative tiles
        let tile_left = (left - self.origin.x).div_euclid(TILE_SIZE);
        let tile_right = (right - self.origin.x).div_euclid(TILE_SIZE);
        let tile_top = (top - self.origin.y).div_euclid(TILE_SIZE);
        let tile_bottom = (bottom - self.origin.y).div_euclid(TILE_SIZE);

        for tile_y in tile_top..=tile_bottom {
            for tile_x in tile_left..=tile_right {
                if self.is_solid(tile_x, tile_y) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_origin(text: &str) -> Room {
        Room::from_tiles(IVec2 { x: 0, y: 0 }, text)
    }

    #[test]
    fn from_tiles_measures_the_grid() {
        let room = at_origin("0010\n11\n0");
        assert_eq!(room.width, 4);
        assert_eq!(room.height, 3);
    }

    #[test]
    fn from_tiles_handles_windows_line_endings() {
        let room = at_origin("01\r\n10\r\n");
        assert_eq!(room.width, 2);
        assert_eq!(room.height, 2);
        assert!(room.is_solid(0, 1));
        assert!(!room.is_solid(1, 1));
    }

    #[test]
    fn is_solid_reads_tiles() {
        let room = at_origin("010\n001");
        assert!(room.is_solid(1, 0));
        assert!(room.is_solid(2, 1));
        assert!(!room.is_solid(0, 0));
        assert!(!room.is_solid(1, 1));
    }

    #[test]
    fn any_character_but_zero_is_solid() {
        let room = at_origin("0a3");
        assert!(!room.is_solid(0, 0));
        assert!(room.is_solid(1, 0));
        assert!(room.is_solid(2, 0));
    }

    #[test]
    fn short_rows_are_air_on_the_right() {
        let room = at_origin("0001\n1");
        assert!(room.is_solid(0, 1));
        assert!(!room.is_solid(3, 1));
    }

    #[test]
    fn outside_the_room_is_not_solid() {
        let room = at_origin("11\n11");
        assert!(!room.is_solid(-1, 0));
        assert!(!room.is_solid(0, -1));
        assert!(!room.is_solid(2, 0));
        assert!(!room.is_solid(0, 2));
    }

    fn one_block() -> Room {
        at_origin("000\n010\n000")
    }

    #[test]
    fn collide_rect_overlapping_a_solid_tile() {
        let room = one_block();
        assert!(room.collide_rect(10, 10, 2, 2));
        assert!(room.collide_rect(0, 0, 24, 24));
    }

    #[test]
    fn collide_rect_touching_edges_is_not_a_collision() {
        let room = one_block();
        assert!(!room.collide_rect(0, 8, 8, 8));
        assert!(!room.collide_rect(16, 8, 8, 8));
        assert!(!room.collide_rect(8, 0, 8, 8));
        assert!(!room.collide_rect(8, 16, 8, 8));
    }

    #[test]
    fn collide_rect_one_pixel_overlap() {
        let room = one_block();
        assert!(room.collide_rect(1, 8, 8, 8));
        assert!(room.collide_rect(15, 8, 8, 8));
        assert!(room.collide_rect(8, 1, 8, 8));
        assert!(room.collide_rect(8, 15, 8, 8));
    }

    #[test]
    fn collide_rect_uses_world_coordinates() {
        let room = Room::from_tiles(IVec2 { x: 100, y: -200 }, "000\n010\n000");
        assert!(room.collide_rect(108, -192, 1, 1));
        assert!(!room.collide_rect(8, 8, 1, 1));
    }

    #[test]
    fn collide_rect_outside_the_room() {
        let room = at_origin("1");
        assert!(!room.collide_rect(-50, -50, 8, 11));
        assert!(!room.collide_rect(1000, 1000, 8, 8));
        assert!(!room.collide_rect(-8, 0, 8, 8));
        assert!(room.collide_rect(-4, -4, 8, 8));
    }
}
