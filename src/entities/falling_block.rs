use crate::Vec2;
use crate::level::map::{Element, MapError};
use crate::level::room::{Rect, Room};
use crate::player::{
    DELTA_TIME, IVec2, PlayerState, approach, hitbox, is_riding, move_exact_v, set_lift_speed,
};

pub const FALL_SHAKE_TIME: f32 = 0.2;
pub const FALL_WAIT_TIME: f32 = 0.4;
pub const FALL_MAX_SPEED: f32 = 160.0;
pub const FALL_ACCEL: f32 = 500.0;
pub const FALL_LAND_TIME: f32 = 0.2;
pub const FALL_RECHECK_TIME: f32 = 0.1;
pub const FALL_OUT_MARGIN: i32 = 16;

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
    pub width: i32,
    pub height: i32,
    pub climb_fall: bool,

    // everything below changes as it runs
    pub position: IVec2,
    pub remainder: f32,
    pub speed: f32,
    pub stage: FallStage,
    pub timer: f32,
    pub collidable: bool,
}

impl FallingBlock {
    pub fn new(x: i32, y: i32, width: i32, height: i32, climb_fall: bool) -> FallingBlock {
        FallingBlock {
            width,
            height,
            climb_fall,
            position: IVec2 { x, y },
            remainder: 0.0,
            speed: 0.0,
            stage: FallStage::Idle,
            timer: 0.0,
            collidable: true,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect {
            left: self.position.x,
            top: self.position.y,
            width: self.width,
            height: self.height,
        }
    }
}

pub fn load_falling_blocks(level: &Element, origin: IVec2) -> Result<Vec<FallingBlock>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut falling_blocks = Vec::new();

    for entity in entities
        .children
        .iter()
        .filter(|entity| entity.name == "fallingBlock")
    {
        let int = |key: &'static str| entity.int(key).ok_or(MapError::MissingField(key));
        let climb_fall = entity.bool("climbFall").unwrap_or(true);

        falling_blocks.push(FallingBlock::new(
            origin.x + int("x")?,
            origin.y + int("y")?,
            int("width")?,
            int("height")?,
            climb_fall,
        ));
    }

    Ok(falling_blocks)
}

pub fn update_falling_blocks(room: &mut Room, state: &mut PlayerState) {
    for index in 0..room.falling_blocks.len() {
        update_falling_block(room, index, state);
    }
}

fn sets_it_off(state: &PlayerState, block: &FallingBlock) -> bool {
    if block.climb_fall {
        is_riding(state, &block.rect())
    } else {
        !state.dead && hitbox(state, 0, 1).overlaps(&block.rect())
    }
}

// once it's shaking, standing beside it also holds it up
fn holds_it_up(state: &PlayerState, block: &FallingBlock) -> bool {
    let beside = !state.dead
        && (hitbox(state, 1, 0).overlaps(&block.rect())
            || hitbox(state, -1, 0).overlaps(&block.rect()));

    sets_it_off(state, block) || (block.climb_fall && beside)
}

fn solid_below(room: &mut Room, index: usize) -> bool {
    let below = room.falling_blocks[index].rect().shifted(0, 1);

    room.falling_blocks[index].collidable = false;
    let solid = room.solid(&below);
    room.falling_blocks[index].collidable = true;

    solid
}

fn tiles_below(room: &Room, index: usize) -> bool {
    let block = room.falling_blocks[index].rect();

    room.tiles_solid(&Rect::new(block.left, block.bottom(), block.width, 1))
}

fn jump_thru_below(room: &Room, index: usize) -> bool {
    let block = &room.falling_blocks[index];
    let (here, below) = (block.rect(), block.rect().shifted(0, 1));

    room.jump_thrus
        .iter()
        .any(|thru| thru.hitbox.overlaps(&below) && !thru.hitbox.overlaps(&here))
}

fn update_falling_block(room: &mut Room, index: usize, state: &mut PlayerState) {
    let block = &room.falling_blocks[index];

    match block.stage {
        FallStage::Idle => {
            if sets_it_off(state, block) {
                start_shaking(&mut room.falling_blocks[index]);
            }
        }

        FallStage::Shaking => {
            if block.timer > 0.0 {
                room.falling_blocks[index].timer -= DELTA_TIME;
            } else {
                room.falling_blocks[index].timer = FALL_WAIT_TIME;
                wait_or_fall(room, index, state);
            }
        }

        FallStage::Waiting => {
            room.falling_blocks[index].timer -= DELTA_TIME;
            wait_or_fall(room, index, state);
        }

        FallStage::Falling => fall(room, index, state),

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

fn wait_or_fall(room: &mut Room, index: usize, state: &mut PlayerState) {
    let block = &room.falling_blocks[index];

    if block.timer > 0.0 && holds_it_up(state, block) {
        room.falling_blocks[index].stage = FallStage::Waiting;
    } else {
        let block = &mut room.falling_blocks[index];
        block.speed = 0.0;
        block.stage = FallStage::Falling;
        fall(room, index, state);
    }
}

fn wait_while_propped(room: &mut Room, index: usize) {
    let on_jump_thru = {
        let below = room.falling_blocks[index].rect().shifted(0, 1);
        room.jump_thrus
            .iter()
            .any(|thru| thru.hitbox.overlaps(&below))
    };

    if solid_below(room, index) || on_jump_thru {
        let block = &mut room.falling_blocks[index];
        block.timer = FALL_RECHECK_TIME;
        block.stage = FallStage::Propped;
    } else {
        start_shaking(&mut room.falling_blocks[index]);
    }
}

fn fall(room: &mut Room, index: usize, state: &mut PlayerState) {
    let block = &mut room.falling_blocks[index];
    block.speed = approach(block.speed, FALL_MAX_SPEED, FALL_ACCEL * DELTA_TIME);
    let amount = block.speed * DELTA_TIME;

    block.remainder += amount;
    let pixels = block.remainder.round_ties_even() as i32;
    block.remainder -= pixels as f32;

    if pixels != 0 && move_down(room, index, state, pixels, amount / DELTA_TIME) {
        let block = &mut room.falling_blocks[index];
        block.timer = FALL_LAND_TIME;
        block.stage = FallStage::Landed;
        return;
    }

    let top = room.falling_blocks[index].position.y;
    let fell_out = top > room.bottom() + FALL_OUT_MARGIN
        || (top > room.bottom() - 1 && solid_below(room, index));
    if fell_out {
        let block = &mut room.falling_blocks[index];
        block.collidable = false;
        block.stage = FallStage::Gone;
    }
}

fn move_down(
    room: &mut Room,
    index: usize,
    state: &mut PlayerState,
    pixels: i32,
    lift_speed: f32,
) -> bool {
    let mut moved = 0;
    let mut stopped = false;

    while moved < pixels {
        let below = room.falling_blocks[index].rect().shifted(0, moved + 1);
        for dash_block in room
            .dash_blocks
            .iter_mut()
            .filter(|dash_block| dash_block.hitbox.overlaps(&below))
        {
            dash_block.broken = true;
        }

        room.falling_blocks[index].position.y += moved;
        stopped = solid_below(room, index) || jump_thru_below(room, index);
        room.falling_blocks[index].position.y -= moved;

        if stopped {
            break;
        }
        moved += 1;
    }

    if moved != 0 {
        carry_down(room, index, state, moved, lift_speed);
    }

    stopped
}

fn carry_down(
    room: &mut Room,
    index: usize,
    state: &mut PlayerState,
    pixels: i32,
    lift_speed: f32,
) {
    let riding = is_riding(state, &room.falling_blocks[index].rect());
    let before = room.falling_blocks[index].rect();

    room.falling_blocks[index].position.y += pixels;

    if state.dead {
        return;
    }

    let her = hitbox(state, 0, 0);
    let pushed = room.falling_blocks[index].rect().overlaps(&her);
    if !pushed && !riding {
        return;
    }

    let her_pixels = if pushed {
        pixels - (her.top - before.bottom())
    } else {
        pixels
    };

    // the block moving her doesn't block her
    room.falling_blocks[index].collidable = false;
    move_exact_v(state, room, her_pixels);
    room.falling_blocks[index].collidable = true;

    set_lift_speed(
        state,
        Vec2 {
            x: 0.0,
            y: lift_speed,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{DashBlock, JumpThru};
    use crate::player::Input;
    use crate::sim::step_room;

    // 11 x 23 tiles, open except for a floor along the bottom row. the floor's top is y = 176
    fn floor_room() -> Room {
        let tiles = format!("{}11111111111", "00000000000\n".repeat(22));
        Room::from_tiles(IVec2 { x: 0, y: 0 }, &tiles)
    }

    // a 16 x 16 block at (40, 80)
    fn block_room() -> Room {
        let mut room = floor_room();
        room.falling_blocks
            .push(FallingBlock::new(40, 80, 16, 16, true));
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

    fn update(room: &mut Room, state: &mut PlayerState, frames: usize) {
        for _ in 0..frames {
            update_falling_blocks(room, state);
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
        assert_eq!(room.falling_blocks[0].position, IVec2 { x: 40, y: 80 });
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
        let mut s = PlayerState {
            climb_trigger_dir: -1,
            ..at(60, 96)
        };
        update(&mut room, &mut s, 1);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Shaking);
    }

    #[test]
    fn grabbing_its_side_does_nothing_when_climbing_does_not_count() {
        let mut room = floor_room();
        room.falling_blocks
            .push(FallingBlock::new(40, 80, 16, 16, false));
        let mut s = PlayerState {
            climb_trigger_dir: -1,
            ..at(60, 96)
        };
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

        assert_eq!(room.falling_blocks[0].position, IVec2 { x: 40, y: 160 });
        assert_eq!(room.falling_blocks[0].stage, FallStage::Resting);
        assert_eq!(s.position, IVec2 { x: 48, y: 160 });
        assert!(!s.dead);
    }

    #[test]
    fn it_speeds_up_to_its_top_speed() {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(60));
        room.falling_blocks
            .push(FallingBlock::new(40, 0, 16, 16, true));
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

        assert_eq!(room.falling_blocks[0].position.y, 83);
        assert_eq!(s.position.y, 83);
        assert_eq!(s.lift_speed.y, FALL_MAX_SPEED * DELTA_TIME / DELTA_TIME);
    }

    #[test]
    fn it_pushes_her_down_when_it_lands_on_her_head() {
        let mut room = block_room();
        room.falling_blocks[0].stage = FallStage::Falling;
        room.falling_blocks[0].speed = FALL_MAX_SPEED;

        // her head is 1 px under the block
        let mut s = at(48, 108);
        update(&mut room, &mut s, 1);

        assert_eq!(room.falling_blocks[0].position.y, 83);
        assert_eq!(s.position.y, 110);
    }

    #[test]
    fn it_disappears_below_the_room() {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, &"00000000000\n".repeat(23));
        room.falling_blocks
            .push(FallingBlock::new(40, 80, 16, 16, true));
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 120);

        assert_eq!(room.falling_blocks[0].stage, FallStage::Gone);
        assert!(room.falling_blocks[0].position.y > room.bottom() + FALL_OUT_MARGIN);
        assert!(!room.falling_blocks[0].collidable);
    }

    #[test]
    fn it_lands_on_a_jump_thru() {
        let mut room = block_room();
        room.jump_thrus.push(JumpThru::new(40, 120, 16));
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 60);

        assert_eq!(room.falling_blocks[0].position.y, 104);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Propped);
    }

    #[test]
    fn it_breaks_dash_blocks_it_falls_on() {
        let mut room = block_room();
        room.dash_blocks.push(DashBlock::new(40, 120, 16, 16));
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 120);

        assert!(room.dash_blocks[0].broken);
        assert_eq!(room.falling_blocks[0].position.y, 160);
    }

    #[test]
    fn it_falls_again_when_the_block_under_it_goes() {
        let mut room = block_room();
        room.falling_blocks
            .push(FallingBlock::new(40, 120, 16, 16, true));
        room.falling_blocks[0].stage = FallStage::Falling;

        let mut away = at(8, 8);
        update(&mut room, &mut away, 60);
        assert_eq!(room.falling_blocks[0].position.y, 104);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Propped);

        room.falling_blocks[1].collidable = false;
        update(&mut room, &mut away, 120);
        assert_eq!(room.falling_blocks[0].position.y, 160);
        assert_eq!(room.falling_blocks[0].stage, FallStage::Resting);
    }
}
