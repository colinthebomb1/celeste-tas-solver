use crate::entities::{
    CrumbleBlock, Crystal, DashBlock, FallingBlock, JumpThru, Solid, Spikes, Spring, Zipper,
};
use crate::player::IVec2;

pub const TILE_SIZE: i32 = 8;
pub const SCREEN_HEIGHT: i32 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect {
            left,
            top,
            width,
            height,
        }
    }

    pub fn shifted(&self, dx: i32, dy: i32) -> Rect {
        Rect {
            left: self.left + dx,
            top: self.top + dy,
            ..*self
        }
    }

    pub fn right(&self) -> i32 {
        self.left + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.top + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.left as f32
            && x < self.right() as f32
            && y >= self.top as f32
            && y < self.bottom() as f32
    }

    // touching edges don't count
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left < other.left + other.width
            && other.left < self.left + self.width
            && self.top < other.top + other.height
            && other.top < self.top + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
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
    pub jump_thrus: Vec<JumpThru>,
    pub springs: Vec<Spring>,
    pub crystals: Vec<Crystal>,
    pub solids: Vec<Solid>,
    pub dash_blocks: Vec<DashBlock>,
    pub crumble_blocks: Vec<CrumbleBlock>,
    pub falling_blocks: Vec<FallingBlock>,
    pub zippers: Vec<Zipper>,
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
            bounds: Rect {
                left: origin.x,
                top: origin.y,
                width: 0,
                height: 0,
            },
            origin,
            width,
            height,
            solid,
            spikes: Vec::new(),
            jump_thrus: Vec::new(),
            springs: Vec::new(),
            crystals: Vec::new(),
            solids: Vec::new(),
            dash_blocks: Vec::new(),
            crumble_blocks: Vec::new(),
            falling_blocks: Vec::new(),
            zippers: Vec::new(),
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

    pub fn tile_solid(&self, tile_x: i32, tile_y: i32) -> bool {
        if tile_x as usize >= self.width
            || tile_x < 0
            || tile_y as usize >= self.height
            || tile_y < 0
        {
            return false;
        }

        self.solid[tile_y as usize * self.width + tile_x as usize]
    }

    // ground tiles only, not blocks
    pub fn tiles_solid(&self, rect: &Rect) -> bool {
        // div_euclid rounds down, so pixels left of or above the room get negative tiles
        let tile_left = (rect.left - self.origin.x).div_euclid(TILE_SIZE);
        let tile_right = (rect.right() - 1 - self.origin.x).div_euclid(TILE_SIZE);
        let tile_top = (rect.top - self.origin.y).div_euclid(TILE_SIZE);
        let tile_bottom = (rect.bottom() - 1 - self.origin.y).div_euclid(TILE_SIZE);

        (tile_top..=tile_bottom)
            .any(|tile_y| (tile_left..=tile_right).any(|tile_x| self.tile_solid(tile_x, tile_y)))
    }

    // anything she can't pass through: tiles and whichever blocks are solid right now
    pub fn solid(&self, rect: &Rect) -> bool {
        self.tiles_solid(rect)
            || self
                .solids
                .iter()
                .any(|solid| solid.collidable && solid.rect.overlaps(rect))
    }

    // returns its place in the list
    pub fn add_solid(&mut self, rect: Rect) -> usize {
        self.solids.push(Solid::new(rect));
        self.solids.len() - 1
    }

    // one pixel
    pub fn solid_at(&self, x: i32, y: i32) -> bool {
        self.solid(&Rect::new(x, y, 1, 1))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    pub rooms: Vec<Room>,
}

impl Chapter {
    pub fn room_at(&self, x: f32, y: f32) -> Option<usize> {
        self.rooms
            .iter()
            .position(|room| room.bounds.contains(x, y))
    }

    pub fn room_named(&self, name: &str) -> Option<usize> {
        self.rooms.iter().position(|room| room.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collide_rect_hits_a_zipper_where_it_is_now() {
        let mut room = at_origin("0000\n0000\n0000\n0000");
        room.add_zipper(8, 8, 16, 8, 8, 0);
        assert!(room.solid(&Rect::new(10, 10, 2, 2)));
        assert!(!room.solid(&Rect::new(10, 2, 2, 2)));

        room.solids[0].rect.top = 0;
        assert!(!room.solid(&Rect::new(10, 10, 2, 2)));
        assert!(room.solid(&Rect::new(10, 2, 2, 2)));
    }

    #[test]
    fn collide_rect_skips_a_zipper_that_is_not_collidable() {
        let mut room = at_origin("0000\n0000\n0000\n0000");
        room.add_zipper(8, 8, 16, 8, 8, 0);
        room.solids[room.zippers[0].solid].collidable = false;
        assert!(!room.solid(&Rect::new(10, 10, 2, 2)));
    }

    #[test]
    fn collide_rect_hits_a_dash_block_until_it_is_broken() {
        let mut room = at_origin("0000\n0000\n0000\n0000");
        room.add_dash_block(8, 8, 16, 8);
        assert!(room.solid(&Rect::new(10, 10, 2, 2)));
        assert!(!room.solid(&Rect::new(2, 2, 2, 2)));

        room.solids[room.dash_blocks[0].solid].collidable = false;
        assert!(!room.solid(&Rect::new(10, 10, 2, 2)));
    }

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
        Rect {
            left,
            top,
            width,
            height,
        }
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
        assert!(room.tile_solid(0, 1));
        assert!(!room.tile_solid(1, 1));
    }

    #[test]
    fn is_solid_reads_tiles() {
        let room = at_origin("010\n001");
        assert!(room.tile_solid(1, 0));
        assert!(room.tile_solid(2, 1));
        assert!(!room.tile_solid(0, 0));
        assert!(!room.tile_solid(1, 1));
    }

    #[test]
    fn any_character_but_zero_is_solid() {
        let room = at_origin("0a3");
        assert!(!room.tile_solid(0, 0));
        assert!(room.tile_solid(1, 0));
        assert!(room.tile_solid(2, 0));
    }

    #[test]
    fn short_rows_are_air_on_the_right() {
        let room = at_origin("0001\n1");
        assert!(room.tile_solid(0, 1));
        assert!(!room.tile_solid(3, 1));
    }

    #[test]
    fn outside_the_room_is_not_solid() {
        let room = at_origin("11\n11");
        assert!(!room.tile_solid(-1, 0));
        assert!(!room.tile_solid(0, -1));
        assert!(!room.tile_solid(2, 0));
        assert!(!room.tile_solid(0, 2));
    }

    fn one_block() -> Room {
        at_origin("000\n010\n000")
    }

    #[test]
    fn collide_rect_overlapping_a_solid_tile() {
        let room = one_block();
        assert!(room.solid(&Rect::new(10, 10, 2, 2)));
        assert!(room.solid(&Rect::new(0, 0, 24, 24)));
    }

    #[test]
    fn collide_rect_touching_edges_is_not_a_collision() {
        let room = one_block();
        assert!(!room.solid(&Rect::new(0, 8, 8, 8)));
        assert!(!room.solid(&Rect::new(16, 8, 8, 8)));
        assert!(!room.solid(&Rect::new(8, 0, 8, 8)));
        assert!(!room.solid(&Rect::new(8, 16, 8, 8)));
    }

    #[test]
    fn collide_rect_one_pixel_overlap() {
        let room = one_block();
        assert!(room.solid(&Rect::new(1, 8, 8, 8)));
        assert!(room.solid(&Rect::new(15, 8, 8, 8)));
        assert!(room.solid(&Rect::new(8, 1, 8, 8)));
        assert!(room.solid(&Rect::new(8, 15, 8, 8)));
    }

    #[test]
    fn collide_rect_uses_world_coordinates() {
        let room = Room::from_tiles(IVec2 { x: 100, y: -200 }, "000\n010\n000");
        assert!(room.solid_at(108, -192));
        assert!(!room.solid_at(8, 8));
    }

    #[test]
    fn collide_rect_outside_the_room() {
        let room = at_origin("1");
        assert!(!room.solid(&Rect::new(-50, -50, 8, 11)));
        assert!(!room.solid(&Rect::new(1000, 1000, 8, 8)));
        assert!(!room.solid(&Rect::new(-8, 0, 8, 8)));
        assert!(room.solid(&Rect::new(-4, -4, 8, 8)));
    }
}
