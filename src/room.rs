use crate::physics::IVec2;

pub const TILE_SIZE: i32 = 8;
pub const SCREEN_HEIGHT: i32 = 180;
pub const JUMP_THRU_HEIGHT: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn right(&self) -> i32 {
        self.left + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.top + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.left as f32 && x < self.right() as f32 && y >= self.top as f32 && y < self.bottom() as f32
    }

    // touching edges don't count
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left < other.left + other.width && other.left < self.left + self.width && self.top < other.top + other.height && other.top < self.top + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub name: String,
    // the room's area in world pixels. a 184 px tall room counts as 180, as in the game
    pub bounds: Rect,
    // world position of the room's top-left corner, in pixels
    pub origin: IVec2,
    // size in tiles
    pub width: usize,
    pub height: usize,
    // tile (x, y) is at index y * width + x
    solid: Vec<bool>,
    pub spikes: Vec<Spikes>,
    // platforms she can jump up through and stand on
    pub jump_thrus: Vec<Rect>,
}

pub fn jump_thru(x: i32, y: i32, width: i32) -> Rect {
    Rect { left: x, top: y, width, height: JUMP_THRU_HEIGHT }
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

        let mut room = Room {
            name: String::new(),
            bounds: Rect { left: origin.x, top: origin.y, width: 0, height: 0 },
            origin,
            width,
            height,
            solid,
            spikes: Vec::new(),
            jump_thrus: Vec::new(),
        };
        room.set_size(width as i32 * TILE_SIZE, height as i32 * TILE_SIZE);
        room
    }

    // size in pixels 
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.bounds.width = width;
        // the game treats a 184 px tall room as 180
        self.bounds.height = if height == 184 { SCREEN_HEIGHT } else { height };
    }

    // world y of the room's bottom edge
    pub fn bottom(&self) -> i32 {
        self.bounds.bottom()
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

#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    pub rooms: Vec<Room>,
}

impl Chapter {
    pub fn room_at(&self, x: f32, y: f32) -> Option<usize> {
        self.rooms.iter().position(|room| room.bounds.contains(x, y))
    }

    pub fn room_named(&self, name: &str) -> Option<usize> {
        self.rooms.iter().position(|room| room.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_tiles_bounds_cover_the_grid() {
        let room = Room::from_tiles(IVec2 { x: 100, y: -200 }, "000\n000");
        assert_eq!(room.bounds, rect(100, -200, 24, 16));
    }

    #[test]
    fn set_size_applies_the_180_rule() {
        let mut room = at_origin("0");
        room.set_size(320, 184);
        assert_eq!(room.bounds, rect(0, 0, 320, 180));
        room.set_size(392, 192);
        assert_eq!(room.bounds, rect(0, 0, 392, 192));
    }

    #[test]
    fn rect_contains_excludes_the_right_and_bottom_edges() {
        let r = rect(10, 20, 30, 40);
        assert!(r.contains(10.0, 20.0));
        assert!(r.contains(39.5, 59.5));
        assert!(!r.contains(40.0, 30.0));
        assert!(!r.contains(20.0, 60.0));
        assert!(!r.contains(9.5, 30.0));
    }

    #[test]
    fn chapter_finds_rooms_by_point_and_name() {
        let mut a = Room::from_tiles(IVec2 { x: 0, y: 0 }, "00\n00");
        a.name = "a".to_string();
        let mut b = Room::from_tiles(IVec2 { x: 16, y: 0 }, "00\n00");
        b.name = "b".to_string();
        let chapter = Chapter { rooms: vec![a, b] };
        assert_eq!(chapter.room_at(15.5, 8.0), Some(0));
        assert_eq!(chapter.room_at(16.0, 8.0), Some(1));
        assert_eq!(chapter.room_at(40.0, 8.0), None);
        assert_eq!(chapter.room_named("b"), Some(1));
        assert_eq!(chapter.room_named("c"), None);
    }

    fn at_origin(text: &str) -> Room {
        Room::from_tiles(IVec2 { x: 0, y: 0 }, text)
    }

    fn rect(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect { left, top, width, height }
    }

    #[test]
    fn rects_overlap() {
        let a = rect(10, 10, 8, 8);
        assert!(a.overlaps(&rect(17, 17, 8, 8)));
        assert!(a.overlaps(&rect(3, 3, 8, 8)));
        assert!(a.overlaps(&rect(12, 12, 2, 2)));
        assert!(a.overlaps(&rect(0, 0, 100, 100)));
        assert!(a.overlaps(&a));
    }

    #[test]
    fn rects_touching_edges_do_not_overlap() {
        let a = rect(10, 10, 8, 8);
        assert!(!a.overlaps(&rect(18, 10, 8, 8)));
        assert!(!a.overlaps(&rect(2, 10, 8, 8)));
        assert!(!a.overlaps(&rect(10, 18, 8, 8)));
        assert!(!a.overlaps(&rect(10, 2, 8, 8)));
    }

    #[test]
    fn rects_apart_do_not_overlap() {
        let a = rect(10, 10, 8, 8);
        assert!(!a.overlaps(&rect(100, 10, 8, 8)));
        assert!(!a.overlaps(&rect(10, -50, 8, 8)));
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

    #[test]
    fn from_tiles_has_no_spikes() {
        assert!(at_origin("01").spikes.is_empty());
    }

    #[test]
    fn bottom_is_in_world_pixels() {
        let room = Room::from_tiles(IVec2 { x: 100, y: -200 }, "0\n0\n0");
        assert_eq!(room.bottom(), -176);
    }

    #[test]
    fn bottom_of_a_184_px_room_is_at_180() {
        let room = Room::from_tiles(IVec2 { x: 0, y: -200 }, &"0\n".repeat(23));
        assert_eq!(room.bottom(), -20);
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
