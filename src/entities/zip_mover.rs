use crate::map::{Element, MapError};
use crate::physics::{approach, hitbox, move_exact_h, move_exact_v, set_lift_speed, IVec2, PlayerState, DELTA_TIME};
use crate::room::{Rect, Room};
use crate::Vec2;

pub const ZIP_START_DELAY: f32 = 0.1;
pub const ZIP_PAUSE: f32 = 0.5;
// progress per second, where 1.0 is the whole trip
pub const ZIP_GO_RATE: f32 = 2.0;
pub const ZIP_RETURN_RATE: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipStage {
    Idle,
    Starting,
    Going,
    AtTarget,
    Returning,
    Resting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZipMover {
    // top-left corners in world pixels
    pub start: IVec2,
    pub target: IVec2,
    pub width: i32,
    pub height: i32,

    // everything below changes as it runs
    pub position: IVec2,
    pub remainder: Vec2,
    pub lift_speed: Vec2,
    pub stage: ZipStage,
    pub wait_timer: f32,
    // how far along the trip it is, 0 to 1
    pub at: f32,
    // off while it is moving her, so it doesn't block her
    pub collidable: bool,
}

impl ZipMover {
    pub fn new(x: i32, y: i32, width: i32, height: i32, target_x: i32, target_y: i32) -> ZipMover {
        ZipMover {
            start: IVec2 { x, y },
            target: IVec2 { x: target_x, y: target_y },
            width,
            height,
            position: IVec2 { x, y },
            remainder: Vec2 { x: 0.0, y: 0.0 },
            lift_speed: Vec2 { x: 0.0, y: 0.0 },
            stage: ZipStage::Idle,
            wait_timer: 0.0,
            at: 0.0,
            collidable: true,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect { left: self.position.x, top: self.position.y, width: self.width, height: self.height }
    }
}

pub fn load_zip_movers(level: &Element, origin: IVec2) -> Result<Vec<ZipMover>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut zip_movers = Vec::new();

    for entity in entities.children.iter().filter(|entity| entity.name == "zipMover") {
        let int = |element: &Element, key: &'static str| element.int(key).ok_or(MapError::MissingField(key));
        let node = entity.child("node").ok_or(MapError::MissingField("node"))?;

        zip_movers.push(ZipMover::new(
            origin.x + int(entity, "x")?,
            origin.y + int(entity, "y")?,
            int(entity, "width")?,
            int(entity, "height")?,
            origin.x + int(node, "x")?,
            origin.y + int(node, "y")?,
        ));
    }

    Ok(zip_movers)
}

pub fn update_zip_movers(room: &mut Room, state: &mut PlayerState) {
    for index in 0..room.zip_movers.len() {
        update_zip_mover(room, index, state);
    }
}

pub fn is_riding(state: &PlayerState, room: &Room, index: usize) -> bool {
    !state.dead && hitbox(state, 0, 1).overlaps(&room.zip_movers[index].rect())
}

fn sine_in(t: f32) -> f32 {
    -(((std::f32::consts::FRAC_PI_2 * t) as f64).cos() as f32) + 1.0
}

fn lerp(from: i32, to: i32, amount: f32) -> f32 {
    from as f32 + (to - from) as f32 * amount
}

fn update_zip_mover(room: &mut Room, index: usize, state: &mut PlayerState) {
    let zip = &room.zip_movers[index];
    let (start, target) = (zip.start, zip.target);

    match zip.stage {
        ZipStage::Idle => {
            if is_riding(state, room, index) {
                let zip = &mut room.zip_movers[index];
                zip.wait_timer = ZIP_START_DELAY;
                zip.stage = ZipStage::Starting;
            }
        }

        ZipStage::Starting => {
            let zip = &mut room.zip_movers[index];
            if zip.wait_timer > 0.0 {
                zip.wait_timer -= DELTA_TIME;
            } else {
                zip.at = 0.0;
                zip.stage = ZipStage::Going;
            }
        }

        ZipStage::Going => {
            let at = approach(zip.at, 1.0, ZIP_GO_RATE * DELTA_TIME);
            let percent = sine_in(at);
            room.zip_movers[index].at = at;
            move_to(room, index, state, lerp(start.x, target.x, percent), lerp(start.y, target.y, percent));

            if at >= 1.0 {
                let zip = &mut room.zip_movers[index];
                zip.wait_timer = ZIP_PAUSE;
                zip.stage = ZipStage::AtTarget;
            }
        }

        ZipStage::AtTarget => {
            let zip = &mut room.zip_movers[index];
            if zip.wait_timer > 0.0 {
                zip.wait_timer -= DELTA_TIME;
            } else {
                zip.at = 0.0;
                zip.stage = ZipStage::Returning;
            }
        }

        ZipStage::Returning => {
            let at = approach(zip.at, 1.0, ZIP_RETURN_RATE * DELTA_TIME);
            let percent = sine_in(at);
            room.zip_movers[index].at = at;
            move_to(room, index, state, lerp(target.x, start.x, percent), lerp(target.y, start.y, percent));

            if at >= 1.0 {
                let zip = &mut room.zip_movers[index];
                zip.wait_timer = ZIP_PAUSE;
                zip.stage = ZipStage::Resting;
            }
        }

        ZipStage::Resting => {
            if zip.wait_timer > 0.0 {
                room.zip_movers[index].wait_timer -= DELTA_TIME;
            } else if is_riding(state, room, index) {
                let zip = &mut room.zip_movers[index];
                zip.wait_timer = ZIP_START_DELAY;
                zip.stage = ZipStage::Starting;
            } else {
                room.zip_movers[index].stage = ZipStage::Idle;
            }
        }
    }
}

fn move_to(room: &mut Room, index: usize, state: &mut PlayerState, x: f32, y: f32) {
    let zip = &mut room.zip_movers[index];
    let amount_x = x - (zip.position.x as f32 + zip.remainder.x);
    zip.lift_speed.x = amount_x / DELTA_TIME;
    zip.remainder.x += amount_x;
    let pixels_x = zip.remainder.x.round_ties_even() as i32;
    zip.remainder.x -= pixels_x as f32;
    if pixels_x != 0 {
        move_pixels(room, index, state, pixels_x, 0);
    }

    let zip = &mut room.zip_movers[index];
    let amount_y = y - (zip.position.y as f32 + zip.remainder.y);
    zip.lift_speed.y = amount_y / DELTA_TIME;
    zip.remainder.y += amount_y;
    let pixels_y = zip.remainder.y.round_ties_even() as i32;
    zip.remainder.y -= pixels_y as f32;
    if pixels_y != 0 {
        move_pixels(room, index, state, 0, pixels_y);
    }
}

fn move_pixels(room: &mut Room, index: usize, state: &mut PlayerState, dx: i32, dy: i32) {
    let riding = is_riding(state, room, index);
    let before = room.zip_movers[index].rect();

    let zip = &mut room.zip_movers[index];
    zip.position.x += dx;
    zip.position.y += dy;
    let lift_speed = zip.lift_speed;

    if state.dead {
        return;
    }

    let her = hitbox(state, 0, 0);
    let pushed = zip.rect().overlaps(&her);
    if !pushed && !riding {
        return;
    }

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
    room.zip_movers[index].collidable = false;
    if her_dx != 0 {
        move_exact_h(state, room, her_dx);
    }
    if her_dy != 0 {
        move_exact_v(state, room, her_dy);
    }
    room.zip_movers[index].collidable = true;

    set_lift_speed(state, lift_speed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::step_room;
    use crate::physics::{lift_boost, Input, LIFT_Y_CAP};

    // an empty room with one block at (40, 120), 32 x 16, that travels 80 px up
    fn zip_room() -> Room {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(23));
        room.zip_movers.push(ZipMover::new(40, 120, 32, 16, 40, 40));
        room
    }

    fn at(x: i32, y: i32) -> PlayerState {
        PlayerState::new(IVec2 { x, y }, Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 0.0, y: 0.0 })
    }

    fn idle(room: &mut Room, state: PlayerState, frames: usize) -> PlayerState {
        (0..frames).fold(state, |state, _| step_room(state, Input::default(), room))
    }

    #[test]
    fn riding_means_standing_on_top() {
        let room = zip_room();
        assert!(is_riding(&at(56, 120), &room, 0));
        assert!(!is_riding(&at(56, 119), &room, 0));
        assert!(!is_riding(&at(30, 120), &room, 0));
    }

    #[test]
    fn stays_put_with_nobody_on_it() {
        let mut room = zip_room();
        idle(&mut room, at(10, 50), 60);
        assert_eq!(room.zip_movers[0].position, IVec2 { x: 40, y: 120 });
        assert_eq!(room.zip_movers[0].stage, ZipStage::Idle);
    }

    #[test]
    fn she_can_stand_on_it() {
        let mut room = zip_room();
        let s = step_room(at(56, 120), Input::default(), &mut room);
        assert_eq!(s.position, IVec2 { x: 56, y: 120 });
        assert_eq!(room.zip_movers[0].stage, ZipStage::Starting);
    }

    #[test]
    fn first_moves_on_the_eighth_frame_after_she_steps_on() {
        let mut room = zip_room();
        let s = idle(&mut room, at(56, 120), 8);
        assert_eq!(room.zip_movers[0].stage, ZipStage::Going);
        assert_eq!(room.zip_movers[0].at, 0.0);

        idle(&mut room, s, 1);
        assert!(room.zip_movers[0].at > 0.0);
        assert!(room.zip_movers[0].lift_speed.y < 0.0);
    }

    #[test]
    fn carries_her_up_and_gives_her_lift_speed() {
        let mut room = zip_room();
        let mut s = at(56, 120);
        for _ in 0..25 {
            s = step_room(s, Input::default(), &mut room);
            assert_eq!(s.position.y, room.zip_movers[0].position.y, "she stays on top");
        }
        assert!(room.zip_movers[0].position.y < 110);
        assert!(lift_boost(&s).y < -60.0);
    }

    #[test]
    fn stops_exactly_at_its_target_then_comes_back() {
        let mut room = zip_room();
        let mut s = idle(&mut room, at(56, 120), 39);
        assert_eq!(room.zip_movers[0].stage, ZipStage::AtTarget);
        assert_eq!(room.zip_movers[0].position, IVec2 { x: 40, y: 40 });
        assert_eq!(room.zip_movers[0].remainder, Vec2 { x: 0.0, y: 0.0 });
        assert_eq!(s.position.y, 40);

        s = idle(&mut room, s, 31);
        assert_eq!(room.zip_movers[0].stage, ZipStage::Returning);

        // nobody is on it by the time it is home again, so it goes back to idle
        s.dead = true;
        idle(&mut room, s, 160);
        assert_eq!(room.zip_movers[0].position, IVec2 { x: 40, y: 120 });
        assert_eq!(room.zip_movers[0].stage, ZipStage::Idle);
    }

    #[test]
    fn lift_speed_is_capped_when_it_is_fast() {
        let mut room = zip_room();
        let s = idle(&mut room, at(56, 120), 35);
        assert!(room.zip_movers[0].lift_speed.y < LIFT_Y_CAP);
        assert_eq!(lift_boost(&s).y, LIFT_Y_CAP);
    }

    #[test]
    fn pushes_her_up_when_it_rises_into_her() {
        let mut room = zip_room();
        room.zip_movers[0].stage = ZipStage::Going;
        room.zip_movers[0].at = 0.5;

        // hanging 1 px above it, so not riding
        let mut s = at(56, 119);
        update_zip_movers(&mut room, &mut s);
        let top = room.zip_movers[0].position.y;
        assert!(top < 118, "it moved more than the gap");
        assert_eq!(s.position.y, top);
        assert!(lift_boost(&s).y < 0.0);
    }

    #[test]
    fn leaves_her_alone_when_she_is_not_on_it() {
        let mut room = zip_room();
        room.zip_movers[0].stage = ZipStage::Going;
        room.zip_movers[0].at = 0.5;

        let mut s = at(10, 50);
        update_zip_movers(&mut room, &mut s);
        assert_eq!(s.position, IVec2 { x: 10, y: 50 });
        assert_eq!(lift_boost(&s), Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn carries_her_sideways() {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(23));
        room.zip_movers.push(ZipMover::new(8, 120, 24, 16, 56, 120));
        let s = idle(&mut room, at(20, 120), 30);
        let moved = room.zip_movers[0].position.x - 8;
        assert!(moved > 10);
        assert_eq!(s.position, IVec2 { x: 20 + moved, y: 120 });
        assert!(lift_boost(&s).x > 0.0);
    }

    #[test]
    fn freeze_frames_stop_it_too() {
        let mut room = zip_room();
        room.zip_movers[0].stage = ZipStage::Going;
        room.zip_movers[0].at = 0.5;
        let before = room.zip_movers[0].clone();

        let mut s = at(10, 50);
        s.freeze_timer = 0.05;
        step_room(s, Input::default(), &mut room);
        assert_eq!(room.zip_movers[0], before);
    }
}
