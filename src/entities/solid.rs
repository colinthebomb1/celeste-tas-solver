use crate::Vec2;
use crate::level::room::{Rect, Room};
use crate::player::{
    DELTA_TIME, IVec2, PlayerState, hitbox, is_riding, move_exact_h, move_exact_v, set_lift_speed,
};

// something she can't pass through that isn't a ground tile
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solid {
    pub rect: Rect,
    pub remainder: Vec2,
    pub lift_speed: Vec2,
    pub collidable: bool,
}

impl Solid {
    pub fn new(rect: Rect) -> Solid {
        Solid {
            rect,
            remainder: Vec2 { x: 0.0, y: 0.0 },
            lift_speed: Vec2 { x: 0.0, y: 0.0 },
            collidable: true,
        }
    }

    // top-left corner
    pub fn position(&self) -> IVec2 {
        IVec2 {
            x: self.rect.left,
            y: self.rect.top,
        }
    }
}

// a solid only has lift speed on a frame where it moved
pub fn reset_lift_speeds(room: &mut Room) {
    for solid in &mut room.solids {
        solid.lift_speed = Vec2 { x: 0.0, y: 0.0 };
    }
}

fn whole_pixels(remainder: &mut f32, amount: f32) -> i32 {
    *remainder += amount;
    // C# Math.Round
    let pixels = remainder.round_ties_even() as i32;
    *remainder -= pixels as f32;
    pixels
}

// adds a fraction of a pixel of sideways movement. returns the whole pixels it should now move
pub fn advance_h(solid: &mut Solid, amount: f32) -> i32 {
    solid.lift_speed.x = amount / DELTA_TIME;
    whole_pixels(&mut solid.remainder.x, amount)
}

pub fn advance_v(solid: &mut Solid, amount: f32) -> i32 {
    solid.lift_speed.y = amount / DELTA_TIME;
    whole_pixels(&mut solid.remainder.y, amount)
}

// moves it so its top-left corner is at (x, y), sideways first
pub fn move_solid_to(room: &mut Room, id: usize, state: &mut PlayerState, x: f32, y: f32) {
    let solid = &mut room.solids[id];
    let amount_x = x - (solid.rect.left as f32 + solid.remainder.x);
    let pixels_x = advance_h(solid, amount_x);
    if pixels_x != 0 {
        move_solid(room, id, state, pixels_x, 0);
    }

    let solid = &mut room.solids[id];
    let amount_y = y - (solid.rect.top as f32 + solid.remainder.y);
    let pixels_y = advance_v(solid, amount_y);
    if pixels_y != 0 {
        move_solid(room, id, state, 0, pixels_y);
    }
}

// moves it by whole pixels along one axis. it carries her if she is riding it and pushes her if it runs into her
pub fn move_solid(room: &mut Room, id: usize, state: &mut PlayerState, dx: i32, dy: i32) {
    let before = room.solids[id].rect;
    let riding = is_riding(state, &before);

    let solid = &mut room.solids[id];
    solid.rect = before.shifted(dx, dy);
    let (after, lift_speed) = (solid.rect, solid.lift_speed);

    if state.dead {
        return;
    }

    let her = hitbox(state, 0, 0);
    let pushed = after.overlaps(&her);
    if !pushed && !riding {
        return;
    }

    // a push moves her just far enough to clear its leading edge
    let (her_dx, her_dy) = if !pushed {
        (dx, dy)
    } else if dx < 0 {
        (dx - (her.right() - before.left), 0)
    } else if dx > 0 {
        (dx - (her.left - before.right()), 0)
    } else if dy < 0 {
        (0, dy - (her.bottom() - before.top))
    } else {
        (0, dy - (her.top - before.bottom()))
    };

    // the block moving her doesn't block her
    room.solids[id].collidable = false;
    if her_dx != 0 {
        move_exact_h(state, room, her_dx);
    }
    if her_dy != 0 {
        move_exact_v(state, room, her_dy);
    }
    room.solids[id].collidable = true;

    set_lift_speed(state, lift_speed);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_room() -> Room {
        Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(23))
    }

    fn at(x: i32, y: i32) -> PlayerState {
        PlayerState::new(
            IVec2 { x, y },
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 0.0, y: 0.0 },
        )
    }

    #[test]
    fn a_solid_blocks_her_until_it_is_not_collidable() {
        let mut room = open_room();
        let id = room.add_solid(Rect::new(8, 8, 16, 8));
        assert!(room.solid_at(10, 10));
        assert!(!room.solid_at(10, 2));

        room.solids[id].collidable = false;
        assert!(!room.solid_at(10, 10));
    }

    #[test]
    fn fractions_of_a_pixel_add_up() {
        let mut solid = Solid::new(Rect::new(0, 0, 8, 8));
        assert_eq!(advance_v(&mut solid, 0.4), 0);
        assert_eq!(advance_v(&mut solid, 0.4), 1);
        assert_eq!(solid.lift_speed.y, 0.4 / DELTA_TIME);
    }

    #[test]
    fn it_carries_her_when_she_stands_on_it() {
        let mut room = open_room();
        let id = room.add_solid(Rect::new(40, 120, 32, 16));
        let mut s = at(56, 120);

        move_solid(&mut room, id, &mut s, 3, 0);
        assert_eq!(s.position, IVec2 { x: 59, y: 120 });

        move_solid(&mut room, id, &mut s, 0, -2);
        assert_eq!(s.position, IVec2 { x: 59, y: 118 });
    }

    #[test]
    fn it_pushes_her_out_of_its_way() {
        let mut room = open_room();
        let id = room.add_solid(Rect::new(40, 120, 32, 16));

        // 2 px to its right, level with it
        let mut s = at(78, 130);
        move_solid(&mut room, id, &mut s, 5, 0);
        assert_eq!(s.position.x, 81);
    }

    #[test]
    fn it_leaves_her_alone_otherwise() {
        let mut room = open_room();
        let id = room.add_solid(Rect::new(40, 120, 32, 16));
        let mut s = at(10, 50);

        move_solid(&mut room, id, &mut s, 0, -4);
        assert_eq!(s.position, IVec2 { x: 10, y: 50 });
        assert_eq!(room.solids[id].position(), IVec2 { x: 40, y: 116 });
    }

    #[test]
    fn she_takes_its_lift_speed_when_it_moves_her() {
        let mut room = open_room();
        let id = room.add_solid(Rect::new(40, 120, 32, 16));
        room.solids[id].lift_speed = Vec2 { x: 0.0, y: -90.0 };
        let mut s = at(56, 120);

        move_solid(&mut room, id, &mut s, 0, -1);
        assert_eq!(s.lift_speed, Vec2 { x: 0.0, y: -90.0 });
    }
}
