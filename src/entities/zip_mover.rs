use crate::entities::solid::move_solid_to;
use crate::level::map::{Element, MapError, entities};
use crate::level::room::{Rect, Room};
use crate::player::{DELTA_TIME, IVec2, PlayerState, approach, is_riding};

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
    pub solid: usize,
    // top-left corners in world pixels
    pub start: IVec2,
    pub target: IVec2,

    // everything below changes as it runs
    pub stage: ZipStage,
    pub wait_timer: f32,
    // how far along the trip it is, 0 to 1
    pub at: f32,
}

impl Room {
    pub fn add_zip_mover(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        target_x: i32,
        target_y: i32,
    ) {
        let solid = self.add_solid(Rect::new(x, y, width, height));

        self.zip_movers.push(ZipMover {
            solid,
            start: IVec2 { x, y },
            target: IVec2 {
                x: target_x,
                y: target_y,
            },
            stage: ZipStage::Idle,
            wait_timer: 0.0,
            at: 0.0,
        });
    }
}

pub fn load_zip_movers(level: &Element, origin: IVec2, room: &mut Room) -> Result<(), MapError> {
    for entity in entities(level, "zipMover") {
        let node = entity.child("node").ok_or(MapError::MissingField("node"))?;

        room.add_zip_mover(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
            entity.require("width")?,
            entity.require("height")?,
            origin.x + node.require("x")?,
            origin.y + node.require("y")?,
        );
    }

    Ok(())
}

pub fn update_zip_movers(room: &mut Room, state: &mut PlayerState) {
    for index in 0..room.zip_movers.len() {
        update_zip_mover(room, index, state);
    }
}

pub fn is_ridden(room: &Room, index: usize, state: &PlayerState) -> bool {
    is_riding(state, &room.solids[room.zip_movers[index].solid].rect)
}

fn sine_in(t: f32) -> f32 {
    -(((std::f32::consts::FRAC_PI_2 * t) as f64).cos() as f32) + 1.0
}

fn lerp(from: i32, to: i32, amount: f32) -> f32 {
    from as f32 + (to - from) as f32 * amount
}

fn update_zip_mover(room: &mut Room, index: usize, state: &mut PlayerState) {
    let zip = &room.zip_movers[index];
    let (solid, start, target) = (zip.solid, zip.start, zip.target);

    match zip.stage {
        ZipStage::Idle => {
            if is_ridden(room, index, state) {
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
            move_solid_to(
                room,
                solid,
                state,
                lerp(start.x, target.x, percent),
                lerp(start.y, target.y, percent),
            );

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
            move_solid_to(
                room,
                solid,
                state,
                lerp(target.x, start.x, percent),
                lerp(target.y, start.y, percent),
            );

            if at >= 1.0 {
                let zip = &mut room.zip_movers[index];
                zip.wait_timer = ZIP_PAUSE;
                zip.stage = ZipStage::Resting;
            }
        }

        ZipStage::Resting => {
            if zip.wait_timer > 0.0 {
                room.zip_movers[index].wait_timer -= DELTA_TIME;
            } else if is_ridden(room, index, state) {
                let zip = &mut room.zip_movers[index];
                zip.wait_timer = ZIP_START_DELAY;
                zip.stage = ZipStage::Starting;
            } else {
                room.zip_movers[index].stage = ZipStage::Idle;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;
    use crate::player::{Input, LIFT_Y_CAP, lift_boost};
    use crate::sim::step_room;

    // an empty room with one block at (40, 120), 32 x 16, that travels 80 px up
    fn zip_room() -> Room {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(23));
        room.add_zip_mover(40, 120, 32, 16, 40, 40);
        room
    }

    fn at(x: i32, y: i32) -> PlayerState {
        PlayerState::new(
            IVec2 { x, y },
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 0.0, y: 0.0 },
        )
    }

    fn idle(room: &mut Room, state: PlayerState, frames: usize) -> PlayerState {
        (0..frames).fold(state, |state, _| step_room(state, Input::default(), room))
    }

    #[test]
    fn riding_means_standing_on_top() {
        let room = zip_room();
        assert!(is_ridden(&room, 0, &at(56, 120)));
        assert!(!is_ridden(&room, 0, &at(56, 119)));
        assert!(!is_ridden(&room, 0, &at(30, 120)));
    }

    #[test]
    fn stays_put_with_nobody_on_it() {
        let mut room = zip_room();
        idle(&mut room, at(10, 50), 60);
        assert_eq!(
            room.solids[room.zip_movers[0].solid].position(),
            IVec2 { x: 40, y: 120 }
        );
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
        assert!(room.solids[room.zip_movers[0].solid].lift_speed.y < 0.0);
    }

    #[test]
    fn carries_her_up_and_gives_her_lift_speed() {
        let mut room = zip_room();
        let mut s = at(56, 120);
        for _ in 0..25 {
            s = step_room(s, Input::default(), &mut room);
            assert_eq!(
                s.position.y,
                room.solids[room.zip_movers[0].solid].position().y,
                "she stays on top"
            );
        }
        assert!(room.solids[room.zip_movers[0].solid].position().y < 110);
        assert!(lift_boost(&s).y < -60.0);
    }

    #[test]
    fn stops_exactly_at_its_target_then_comes_back() {
        let mut room = zip_room();
        let mut s = idle(&mut room, at(56, 120), 39);
        assert_eq!(room.zip_movers[0].stage, ZipStage::AtTarget);
        assert_eq!(
            room.solids[room.zip_movers[0].solid].position(),
            IVec2 { x: 40, y: 40 }
        );
        assert_eq!(
            room.solids[room.zip_movers[0].solid].remainder,
            Vec2 { x: 0.0, y: 0.0 }
        );
        assert_eq!(s.position.y, 40);

        s = idle(&mut room, s, 31);
        assert_eq!(room.zip_movers[0].stage, ZipStage::Returning);

        // nobody is on it by the time it is home again, so it goes back to idle
        s.dead = true;
        idle(&mut room, s, 160);
        assert_eq!(
            room.solids[room.zip_movers[0].solid].position(),
            IVec2 { x: 40, y: 120 }
        );
        assert_eq!(room.zip_movers[0].stage, ZipStage::Idle);
    }

    #[test]
    fn lift_speed_is_capped_when_it_is_fast() {
        let mut room = zip_room();
        let s = idle(&mut room, at(56, 120), 35);
        assert!(room.solids[room.zip_movers[0].solid].lift_speed.y < LIFT_Y_CAP);
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
        let top = room.solids[room.zip_movers[0].solid].position().y;
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
        room.add_zip_mover(8, 120, 24, 16, 56, 120);
        let s = idle(&mut room, at(20, 120), 30);
        let moved = room.solids[room.zip_movers[0].solid].position().x - 8;
        assert!(moved > 10);
        assert_eq!(
            s.position,
            IVec2 {
                x: 20 + moved,
                y: 120
            }
        );
        assert!(lift_boost(&s).x > 0.0);
    }

    #[test]
    fn freeze_frames_stop_it_too() {
        let mut room = zip_room();
        room.zip_movers[0].stage = ZipStage::Going;
        room.zip_movers[0].at = 0.5;
        let before = room.clone();

        let mut s = at(10, 50);
        s.freeze_timer = 0.05;
        step_room(s, Input::default(), &mut room);
        assert_eq!(room, before);
    }
}
