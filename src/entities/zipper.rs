use crate::entities::Entities;
use crate::entities::solid::move_solid_to;
use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities_named};
use crate::level::room::Room;
use crate::player::Player;
use crate::player::collision::is_riding;
use crate::player::constants::DELTA_TIME;
use crate::player::movement::{approach, lerp};

const ZIP_START_DELAY: f32 = 0.1;
const ZIP_PAUSE: f32 = 0.5;
// progress per second, where 1.0 is the whole trip
const ZIP_GO_RATE: f32 = 2.0;
const ZIP_RETURN_RATE: f32 = 0.5;

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
pub struct Zipper {
    pub solid_id: usize,
    // top-left corners in world pixels
    pub start: IVec2,
    pub target: IVec2,

    // everything below changes as it runs
    pub stage: ZipStage,
    pub wait_timer: f32,
    // how far along the trip it is, 0 to 1
    pub at: f32,
}

impl Entities {
    pub fn add_zipper(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        target_x: i32,
        target_y: i32,
    ) {
        let solid_id = self.add_solid(Rect::new(x, y, width, height));

        self.zippers.push(Zipper {
            solid_id,
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

pub(crate) fn load_zippers(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for entity in entities_named(level, "zipMover") {
        let node = entity.child("node").ok_or(MapError::MissingField("node"))?;

        room.initial.add_zipper(
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

pub(crate) fn update_zippers(room: &Room, entities: &mut Entities, player: &mut Player) {
    for index in 0..entities.zippers.len() {
        update_zipper(room, entities, index, player);
    }
}

fn is_ridden(entities: &Entities, index: usize, player: &Player) -> bool {
    is_riding(
        player,
        &entities.solids[entities.zippers[index].solid_id].rect,
    )
}

fn sine_in(t: f32) -> f32 {
    -(((std::f32::consts::FRAC_PI_2 * t) as f64).cos() as f32) + 1.0
}

fn update_zipper(room: &Room, entities: &mut Entities, index: usize, player: &mut Player) {
    let zip = &entities.zippers[index];
    let (id, start, target) = (zip.solid_id, zip.start, zip.target);

    match zip.stage {
        ZipStage::Idle => {
            if is_ridden(entities, index, player) {
                let zip = &mut entities.zippers[index];
                zip.wait_timer = ZIP_START_DELAY;
                zip.stage = ZipStage::Starting;
            }
        }

        ZipStage::Starting => {
            let zip = &mut entities.zippers[index];
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
            entities.zippers[index].at = at;
            move_solid_to(
                room,
                entities,
                id,
                player,
                lerp(start.x as f32, target.x as f32, percent),
                lerp(start.y as f32, target.y as f32, percent),
            );

            if at >= 1.0 {
                let zip = &mut entities.zippers[index];
                zip.wait_timer = ZIP_PAUSE;
                zip.stage = ZipStage::AtTarget;
            }
        }

        ZipStage::AtTarget => {
            let zip = &mut entities.zippers[index];
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
            entities.zippers[index].at = at;
            move_solid_to(
                room,
                entities,
                id,
                player,
                lerp(target.x as f32, start.x as f32, percent),
                lerp(target.y as f32, start.y as f32, percent),
            );

            if at >= 1.0 {
                let zip = &mut entities.zippers[index];
                zip.wait_timer = ZIP_PAUSE;
                zip.stage = ZipStage::Resting;
            }
        }

        ZipStage::Resting => {
            if zip.wait_timer > 0.0 {
                entities.zippers[index].wait_timer -= DELTA_TIME;
            } else if is_ridden(entities, index, player) {
                let zip = &mut entities.zippers[index];
                zip.wait_timer = ZIP_START_DELAY;
                zip.stage = ZipStage::Starting;
            } else {
                entities.zippers[index].stage = ZipStage::Idle;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{at, idle, open_room};
    use crate::geom::Vec2;
    use crate::player::Input;
    use crate::player::constants::LIFT_Y_CAP;
    use crate::player::movement::liftboost;
    use crate::sim::step_room;

    // an empty room with one block at (40, 120), 32 x 16, that travels 80 px up
    fn zip_room() -> Room {
        let mut room = open_room();
        room.initial.add_zipper(40, 120, 32, 16, 40, 40);
        room
    }

    #[test]
    fn riding_means_standing_on_top() {
        let room = zip_room();
        assert!(is_ridden(&room.initial, 0, &at(56, 120)));
        assert!(!is_ridden(&room.initial, 0, &at(56, 119)));
        assert!(!is_ridden(&room.initial, 0, &at(30, 120)));
    }

    #[test]
    fn stays_put_with_nobody_on_it() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        idle(&room, &mut entities, at(10, 50), 60);
        assert_eq!(entities.solids[0].position(), IVec2 { x: 40, y: 120 });
        assert_eq!(entities.zippers[0].stage, ZipStage::Idle);
    }

    #[test]
    fn she_can_stand_on_it() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        let s = step_room(at(56, 120), Input::default(), &room, &mut entities);
        assert_eq!(s.position, IVec2 { x: 56, y: 120 });
        assert_eq!(entities.zippers[0].stage, ZipStage::Starting);
    }

    #[test]
    fn first_moves_on_the_eighth_frame_after_she_steps_on() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        let s = idle(&room, &mut entities, at(56, 120), 8);
        assert_eq!(entities.zippers[0].stage, ZipStage::Going);
        assert_eq!(entities.zippers[0].at, 0.0);

        idle(&room, &mut entities, s, 1);
        assert!(entities.zippers[0].at > 0.0);
        assert!(entities.solids[0].lift_speed.y < 0.0);
    }

    #[test]
    fn carries_her_up_and_gives_her_lift_speed() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        let mut s = at(56, 120);
        for _ in 0..25 {
            s = step_room(s, Input::default(), &room, &mut entities);
            assert_eq!(
                s.position.y,
                entities.solids[0].position().y,
                "she stays on top"
            );
        }
        assert!(entities.solids[0].position().y < 110);
        assert!(liftboost(&s).y < -60.0);
    }

    #[test]
    fn stops_exactly_at_its_target_then_comes_back() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        let mut s = idle(&room, &mut entities, at(56, 120), 39);
        assert_eq!(entities.zippers[0].stage, ZipStage::AtTarget);
        assert_eq!(entities.solids[0].position(), IVec2 { x: 40, y: 40 });
        assert_eq!(entities.solids[0].subpixel, Vec2 { x: 0.0, y: 0.0 });
        assert_eq!(s.position.y, 40);

        s = idle(&room, &mut entities, s, 31);
        assert_eq!(entities.zippers[0].stage, ZipStage::Returning);

        // nobody is on it by the time it is home again, so it goes back to idle
        s.dead = true;
        idle(&room, &mut entities, s, 160);
        assert_eq!(entities.solids[0].position(), IVec2 { x: 40, y: 120 });
        assert_eq!(entities.zippers[0].stage, ZipStage::Idle);
    }

    #[test]
    fn lift_speed_is_capped_when_it_is_fast() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        let s = idle(&room, &mut entities, at(56, 120), 35);
        assert!(entities.solids[0].lift_speed.y < LIFT_Y_CAP);
        assert_eq!(liftboost(&s).y, LIFT_Y_CAP);
    }

    #[test]
    fn pushes_her_up_when_it_rises_into_her() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        entities.zippers[0].stage = ZipStage::Going;
        entities.zippers[0].at = 0.5;

        // hanging 1 px above it, so not riding
        let mut s = at(56, 119);
        update_zippers(&room, &mut entities, &mut s);
        let top = entities.solids[0].position().y;
        assert!(top < 118, "it moved more than the gap");
        assert_eq!(s.position.y, top);
        assert!(liftboost(&s).y < 0.0);
    }

    #[test]
    fn leaves_her_alone_when_she_is_not_on_it() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        entities.zippers[0].stage = ZipStage::Going;
        entities.zippers[0].at = 0.5;

        let mut s = at(10, 50);
        update_zippers(&room, &mut entities, &mut s);
        assert_eq!(s.position, IVec2 { x: 10, y: 50 });
        assert_eq!(liftboost(&s), Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn carries_her_sideways() {
        let room = open_room();
        let mut entities = room.initial.clone();
        entities.add_zipper(8, 120, 24, 16, 56, 120);
        let s = idle(&room, &mut entities, at(20, 120), 30);
        let moved = entities.solids[0].position().x - 8;
        assert!(moved > 10);
        assert_eq!(
            s.position,
            IVec2 {
                x: 20 + moved,
                y: 120
            }
        );
        assert!(liftboost(&s).x > 0.0);
    }

    #[test]
    fn freeze_frames_stop_it_too() {
        let room = zip_room();
        let mut entities = room.initial.clone();
        entities.zippers[0].stage = ZipStage::Going;
        entities.zippers[0].at = 0.5;
        let before = entities.clone();

        let mut s = at(10, 50);
        s.freeze_timer = 0.05;
        step_room(s, Input::default(), &room, &mut entities);
        assert_eq!(entities, before);
    }
}
