use crate::entities::solid::{advance_v, move_solid};
use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities};
use crate::level::room::Room;
use crate::player::Player;
use crate::player::collision::{hitbox, is_riding};
use crate::player::constants::DELTA_TIME;
use crate::player::movement::approach;

const FALL_SHAKE_TIME: f32 = 0.2;
const FALL_WAIT_TIME: f32 = 0.4;
const FALL_MAX_SPEED: f32 = 160.0;
const FALL_ACCEL: f32 = 500.0;
const FALL_LAND_TIME: f32 = 0.2;
const FALL_RECHECK_TIME: f32 = 0.1;
const FALL_OUT_MARGIN: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallStage {
    Idle,
    Shaking,
    Waiting,
    Falling,
    Landed,
    Propped,
    Resting,
    Gone,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallingBlock {
    pub solid: usize,
    pub climb_fall: bool,

    // everything below changes as it runs
    pub speed: f32,
    pub stage: FallStage,
    pub timer: f32,
}

impl Room {
    // (x, y) is its top-left corner
    pub fn add_falling_block(&mut self, x: i32, y: i32, width: i32, height: i32, climb_fall: bool) {
        let solid = self.add_solid(Rect::new(x, y, width, height));
        self.falling_blocks.push(FallingBlock {
            solid,
            climb_fall,
            speed: 0.0,
            stage: FallStage::Idle,
            timer: 0.0,
        });
    }
}

pub(crate) fn load_falling_blocks(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for entity in entities(level, "fallingBlock") {
        let climb_fall = entity.bool("climbFall").unwrap_or(true);

        room.add_falling_block(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
            entity.require("width")?,
            entity.require("height")?,
            climb_fall,
        );
    }

    Ok(())
}

pub(crate) fn update_falling_blocks(room: &mut Room, player: &mut Player) {
    for index in 0..room.falling_blocks.len() {
        update_falling_block(room, index, player);
    }
}

pub(crate) fn rect(room: &Room, index: usize) -> Rect {
    room.solids[room.falling_blocks[index].solid].rect
}

fn sets_it_off(room: &Room, index: usize, player: &Player) -> bool {
    if room.falling_blocks[index].climb_fall {
        is_riding(player, &rect(room, index))
    } else {
        !player.dead && hitbox(player, 0, 1).overlaps(&rect(room, index))
    }
}

// once it's shaking, standing beside it also holds it up
fn holds_it_up(room: &Room, index: usize, player: &Player) -> bool {
    let block = rect(room, index);
    let beside = !player.dead
        && (hitbox(player, 1, 0).overlaps(&block) || hitbox(player, -1, 0).overlaps(&block));

    sets_it_off(room, index, player) || (room.falling_blocks[index].climb_fall && beside)
}

// whether something solid other than itself is right under it, counting from this far down
fn solid_below(room: &mut Room, index: usize, down: i32) -> bool {
    let id = room.falling_blocks[index].solid;
    let below = room.solids[id].rect.shifted(0, down + 1);

    room.solids[id].collidable = false;
    let solid = room.solid(&below);
    room.solids[id].collidable = true;

    solid
}

fn tiles_below(room: &Room, index: usize) -> bool {
    let block = rect(room, index);

    room.tiles_solid(&Rect::new(block.left, block.bottom(), block.width, 1))
}

// a platform it would land on, as opposed to one it is already inside
fn jump_thru_below(room: &Room, index: usize, down: i32) -> bool {
    let here = rect(room, index).shifted(0, down);
    let below = here.shifted(0, 1);

    room.jump_thrus
        .iter()
        .any(|thru| thru.hitbox.overlaps(&below) && !thru.hitbox.overlaps(&here))
}

fn update_falling_block(room: &mut Room, index: usize, player: &mut Player) {
    let block = &room.falling_blocks[index];

    match block.stage {
        FallStage::Idle => {
            if sets_it_off(room, index, player) {
                start_shaking(&mut room.falling_blocks[index]);
            }
        }

        FallStage::Shaking => {
            if block.timer > 0.0 {
                room.falling_blocks[index].timer -= DELTA_TIME;
            } else {
                room.falling_blocks[index].timer = FALL_WAIT_TIME;
                wait_or_fall(room, index, player);
            }
        }

        FallStage::Waiting => {
            room.falling_blocks[index].timer -= DELTA_TIME;
            wait_or_fall(room, index, player);
        }

        FallStage::Falling => fall(room, index, player),

        FallStage::Landed => {
            if block.timer > 0.0 {
                room.falling_blocks[index].timer -= DELTA_TIME;
            } else if tiles_below(room, index) {
                room.falling_blocks[index].stage = FallStage::Resting;
            } else {
                wait_while_propped(room, index);
            }
        }

        FallStage::Propped => {
            if block.timer > 0.0 {
                room.falling_blocks[index].timer -= DELTA_TIME;
            } else {
                wait_while_propped(room, index);
            }
        }

        FallStage::Resting | FallStage::Gone => {}
    }
}

fn start_shaking(block: &mut FallingBlock) {
    block.timer = FALL_SHAKE_TIME;
    block.stage = FallStage::Shaking;
}

fn wait_or_fall(room: &mut Room, index: usize, player: &mut Player) {
    if room.falling_blocks[index].timer > 0.0 && holds_it_up(room, index, player) {
        room.falling_blocks[index].stage = FallStage::Waiting;
    } else {
        let block = &mut room.falling_blocks[index];
        block.speed = 0.0;
        block.stage = FallStage::Falling;
        fall(room, index, player);
    }
}

fn wait_while_propped(room: &mut Room, index: usize) {
    let below = rect(room, index).shifted(0, 1);
    let on_jump_thru = room
        .jump_thrus
        .iter()
        .any(|thru| thru.hitbox.overlaps(&below));

    if solid_below(room, index, 0) || on_jump_thru {
        let block = &mut room.falling_blocks[index];
        block.timer = FALL_RECHECK_TIME;
        block.stage = FallStage::Propped;
    } else {
        start_shaking(&mut room.falling_blocks[index]);
    }
}

fn fall(room: &mut Room, index: usize, player: &mut Player) {
    let block = &mut room.falling_blocks[index];
    block.speed = approach(block.speed, FALL_MAX_SPEED, FALL_ACCEL * DELTA_TIME);
    let (id, amount) = (block.solid, block.speed * DELTA_TIME);

    let pixels = advance_v(&mut room.solids[id], amount);

    if pixels != 0 && move_down(room, index, player, pixels) {
        let block = &mut room.falling_blocks[index];
        block.timer = FALL_LAND_TIME;
        block.stage = FallStage::Landed;
        return;
    }

    let top = room.solids[id].rect.top;
    let fell_out = top > room.bottom() + FALL_OUT_MARGIN
        || (top > room.bottom() - 1 && solid_below(room, index, 0));
    if fell_out {
        room.solids[id].collidable = false;
        room.falling_blocks[index].stage = FallStage::Gone;
    }
}

// true if something stopped it. it breaks dash blocks on the way
fn move_down(room: &mut Room, index: usize, player: &mut Player, pixels: i32) -> bool {
    let id = room.falling_blocks[index].solid;
    let mut moved = 0;
    let mut stopped = false;

    while moved < pixels {
        let below = room.solids[id].rect.shifted(0, moved + 1);
        for dash_block in 0..room.dash_blocks.len() {
            let dash_solid = room.dash_blocks[dash_block].solid;
            if room.solids[dash_solid].rect.overlaps(&below) {
                room.solids[dash_solid].collidable = false;
            }
        }

        stopped = solid_below(room, index, moved) || jump_thru_below(room, index, moved);
        if stopped {
            break;
        }
        moved += 1;
    }

    if moved != 0 {
        move_solid(room, id, player, 0, moved);
    }

    stopped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::JumpThru;
    use crate::fixtures::{at, floor_room, idle, open_room};
    use crate::player::Input;
    use crate::sim::step_room;

    // a 16 x 16 block at (40, 80)
    fn block_room() -> Room {
        let mut room = floor_room();
        room.add_falling_block(40, 80, 16, 16, true);
        room
    }

    fn update(room: &mut Room, player: &mut Player, frames: usize) {
        for _ in 0..frames {
            update_falling_blocks(room, player);
        }
    }

    #[test]
    fn it_is_solid() {
        let room = block_room();
        assert!(room.solid_at(40, 80));
        assert!(room.solid_at(55, 95));
        assert!(!room.solid_at(56, 80));
    }

    #[test]
    fn stays_put_with_nobody_on_it() {
        let mut room = block_room();
        idle(&mut room, at(100, 176), 120);
        assert_eq!(room.solids[0].position(), IVec2 { x: 40, y: 80 });
        assert_eq!(room.falling_blocks[0].stage, FallStage::Idle);
    }

    #[test]
    fn standing_on_it_sets_it_off() {
        let mut room = block_room();
        let s = step_room(at(48, 80), Input::default(), &mut room);
        assert_eq!(s.position, IVec2 { x: 48, y: 80 });
        assert_eq!(room.falling_blocks[0].stage, FallStage::Shaking);
    }

    #[test]
    fn standing_beside_it_does_not_set_it_off() {
        let mut room = block_room();
        let mut s = at(60, 96);
        update(&mut room, &mut s, 1);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Idle);
    }

    #[test]
    fn grabbing_its_side_sets_it_off() {
        let mut room = block_room();
        let mut s = at(60, 96);
        s.climb.trigger_dir = -1;
        update(&mut room, &mut s, 1);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Shaking);
    }

    #[test]
    fn grabbing_its_side_does_nothing_when_climbing_does_not_count() {
        let mut room = floor_room();
        room.add_falling_block(40, 80, 16, 16, false);
        let mut s = at(60, 96);
        s.climb.trigger_dir = -1;
        update(&mut room, &mut s, 1);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Idle);
    }

    #[test]
    fn it_falls_as_soon_as_the_shake_ends_if_she_has_left() {
        let mut room = block_room();
        let mut s = at(48, 80);
        update(&mut room, &mut s, 1);

        let mut away = at(100, 176);
        let mut frames = 0;
        while room.falling_blocks[0].stage == FallStage::Shaking {
            update(&mut room, &mut away, 1);
            frames += 1;
        }

        // the shake timer runs down, then the next frame moves it
        assert_eq!(frames, 13);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Falling);
        assert_eq!(room.falling_blocks[0].speed, FALL_ACCEL * DELTA_TIME);
    }

    #[test]
    fn it_holds_on_longer_while_she_stays() {
        let mut room = block_room();
        let mut s = at(48, 80);
        update(&mut room, &mut s, 14);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Waiting);

        let mut frames = 14;
        while room.falling_blocks[0].stage == FallStage::Waiting {
            s = step_room(s, Input::default(), &mut room);
            frames += 1;
        }

        assert_eq!(room.falling_blocks[0].stage, FallStage::Falling);
        assert!((36..=40).contains(&frames), "fell after {frames} frames");
    }

    #[test]
    fn standing_beside_it_holds_it_up_once_it_shakes() {
        let mut room = block_room();
        let mut s = at(48, 80);
        update(&mut room, &mut s, 1);

        let mut beside = at(60, 96);
        update(&mut room, &mut beside, 14);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Waiting);
    }

    #[test]
    fn it_carries_her_down_and_lands_on_the_floor() {
        let mut room = block_room();
        let s = idle(&mut room, at(48, 80), 120);

        assert_eq!(room.solids[0].position(), IVec2 { x: 40, y: 160 });
        assert_eq!(room.falling_blocks[0].stage, FallStage::Resting);
        assert_eq!(s.position, IVec2 { x: 48, y: 160 });
        assert!(!s.dead);
    }

    #[test]
    fn it_speeds_up_to_its_top_speed() {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(60));
        room.add_falling_block(40, 0, 16, 16, true);
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 30);

        assert_eq!(room.falling_blocks[0].speed, FALL_MAX_SPEED);
    }

    #[test]
    fn riding_it_down_gives_her_its_lift_speed() {
        let mut room = block_room();
        room.falling_blocks[0].stage = FallStage::Falling;
        room.falling_blocks[0].speed = FALL_MAX_SPEED;

        let mut s = at(48, 80);
        update(&mut room, &mut s, 1);

        assert_eq!(room.solids[0].position().y, 83);
        assert_eq!(s.position.y, 83);
        assert_eq!(s.lift.speed.y, FALL_MAX_SPEED * DELTA_TIME / DELTA_TIME);
    }

    #[test]
    fn it_pushes_her_down_when_it_lands_on_her_head() {
        let mut room = block_room();
        room.falling_blocks[0].stage = FallStage::Falling;
        room.falling_blocks[0].speed = FALL_MAX_SPEED;

        // her head is 1 px under the block
        let mut s = at(48, 108);
        update(&mut room, &mut s, 1);

        assert_eq!(room.solids[0].position().y, 83);
        assert_eq!(s.position.y, 110);
    }

    #[test]
    fn it_disappears_below_the_room() {
        let mut room = open_room();
        room.add_falling_block(40, 80, 16, 16, true);
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 120);

        assert_eq!(room.falling_blocks[0].stage, FallStage::Gone);
        assert!(room.solids[0].position().y > room.bottom() + FALL_OUT_MARGIN);
        assert!(!room.solids[0].collidable);
    }

    #[test]
    fn it_lands_on_a_jump_thru() {
        let mut room = block_room();
        room.jump_thrus.push(JumpThru::new(40, 120, 16));
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 60);

        assert_eq!(room.solids[0].position().y, 104);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Propped);
    }

    #[test]
    fn it_breaks_dash_blocks_it_falls_on() {
        let mut room = block_room();
        room.add_dash_block(40, 120, 16, 16);
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 120);

        assert!(!room.solids[room.dash_blocks[0].solid].collidable);
        assert_eq!(room.solids[0].position().y, 160);
    }

    #[test]
    fn it_falls_again_when_the_block_under_it_goes() {
        let mut room = block_room();
        room.add_falling_block(40, 120, 16, 16, true);
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 60);
        assert_eq!(room.solids[0].position().y, 104);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Propped);

        room.solids[room.falling_blocks[1].solid].collidable = false;
        update(&mut room, &mut away, 120);
        assert_eq!(room.solids[0].position().y, 160);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Resting);
    }
}
