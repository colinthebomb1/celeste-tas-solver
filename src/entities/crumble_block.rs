use crate::level::map::{Element, MapError};
use crate::level::room::{Rect, Room};
use crate::player::{DELTA_TIME, IVec2, Mode, PlayerState, hitbox};

pub const CRUMBLE_HEIGHT: i32 = 8;
pub const CRUMBLE_SHAKE_TIME: f32 = 0.2;
pub const CRUMBLE_SHAKES_ON_TOP: u8 = 1;
pub const CRUMBLE_SHAKES_CLIMBING: u8 = 3;
pub const CRUMBLE_HOLD_TIME: f32 = 0.4;
pub const CRUMBLE_RESPAWN_TIME: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrumbleStage {
    Idle,
    Shaking,
    Holding,
    Gone,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrumbleBlock {
    pub hitbox: Rect,

    // everything below changes as it runs
    pub stage: CrumbleStage,
    pub timer: f32,
    pub shakes_left: u8,
    pub on_top: bool,
    pub collidable: bool,
}

impl CrumbleBlock {
    pub fn new(x: i32, y: i32, width: i32) -> CrumbleBlock {
        CrumbleBlock {
            hitbox: Rect {
                left: x,
                top: y,
                width,
                height: CRUMBLE_HEIGHT,
            },
            stage: CrumbleStage::Idle,
            timer: 0.0,
            shakes_left: 0,
            on_top: false,
            collidable: true,
        }
    }
}

pub fn load_crumble_blocks(level: &Element, origin: IVec2) -> Result<Vec<CrumbleBlock>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut crumble_blocks = Vec::new();

    for entity in entities
        .children
        .iter()
        .filter(|entity| entity.name == "crumbleBlock")
    {
        let int = |key: &'static str| entity.int(key).ok_or(MapError::MissingField(key));

        crumble_blocks.push(CrumbleBlock::new(
            origin.x + int("x")?,
            origin.y + int("y")?,
            int("width")?,
        ));
    }

    Ok(crumble_blocks)
}

pub fn update_crumble_blocks(room: &mut Room, state: &PlayerState) {
    for block in &mut room.crumble_blocks {
        update_crumble_block(block, state);
    }
}

fn standing_on(state: &PlayerState, block: &CrumbleBlock) -> bool {
    !state.dead && hitbox(state, 0, 1).overlaps(&block.hitbox)
}

fn climbing(state: &PlayerState, block: &CrumbleBlock) -> bool {
    !state.dead
        && state.mode == Mode::Climb
        && hitbox(state, state.facing as i32, 0).overlaps(&block.hitbox)
}

fn update_crumble_block(block: &mut CrumbleBlock, state: &PlayerState) {
    match block.stage {
        CrumbleStage::Idle => set_off(block, state),

        CrumbleStage::Shaking => {
            if block.timer > 0.0 {
                block.timer -= DELTA_TIME;
            } else if block.shakes_left > 0 {
                block.shakes_left -= 1;
                block.timer = CRUMBLE_SHAKE_TIME;
            } else {
                block.timer = CRUMBLE_HOLD_TIME;
                hold_or_crumble(block, state);
            }
        }

        CrumbleStage::Holding => {
            block.timer -= DELTA_TIME;
            hold_or_crumble(block, state);
        }

        CrumbleStage::Gone => {
            if block.timer > 0.0 {
                block.timer -= DELTA_TIME;
            } else if state.dead || !hitbox(state, 0, 0).overlaps(&block.hitbox) {
                block.collidable = true;
                block.stage = CrumbleStage::Idle;
                set_off(block, state);
            }
        }
    }
}

fn set_off(block: &mut CrumbleBlock, state: &PlayerState) {
    if standing_on(state, block) {
        block.on_top = true;
        block.shakes_left = CRUMBLE_SHAKES_ON_TOP - 1;
    } else if climbing(state, block) {
        block.on_top = false;
        block.shakes_left = CRUMBLE_SHAKES_CLIMBING - 1;
    } else {
        return;
    }

    block.timer = CRUMBLE_SHAKE_TIME;
    block.stage = CrumbleStage::Shaking;
}

// stood on, it goes early if she steps off. climbed, it always waits the full time
fn hold_or_crumble(block: &mut CrumbleBlock, state: &PlayerState) {
    let holding = block.timer > 0.0 && (!block.on_top || standing_on(state, block));

    if holding {
        block.stage = CrumbleStage::Holding;
    } else {
        block.collidable = false;
        block.timer = CRUMBLE_RESPAWN_TIME;
        block.stage = CrumbleStage::Gone;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;

    // 40 px wide with its top at y = 64
    fn block() -> CrumbleBlock {
        CrumbleBlock::new(304, 64, 40)
    }

    fn at(x: i32, y: i32) -> PlayerState {
        PlayerState::new(
            IVec2 { x, y },
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 0.0, y: 0.0 },
        )
    }

    fn on_top() -> PlayerState {
        at(320, 64)
    }

    fn away() -> PlayerState {
        at(100, 100)
    }

    // holding its right side
    fn climbing_it() -> PlayerState {
        let mut s = at(348, 72);
        s.facing = -1;
        s.mode = Mode::Climb;
        s
    }

    // frames until it is no longer solid
    fn frames_to_crumble(block: &mut CrumbleBlock, state: &PlayerState) -> usize {
        let mut frames = 0;
        while block.collidable {
            update_crumble_block(block, state);
            frames += 1;
            assert!(frames < 1000);
        }
        frames
    }

    #[test]
    fn it_is_eight_pixels_tall() {
        assert_eq!(
            block().hitbox,
            Rect {
                left: 304,
                top: 64,
                width: 40,
                height: 8
            }
        );
    }

    #[test]
    fn it_is_solid_until_it_crumbles() {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, "0000\n0000");
        room.crumble_blocks.push(CrumbleBlock::new(8, 8, 16));
        assert!(room.solid_at(8, 8));

        room.crumble_blocks[0].collidable = false;
        assert!(!room.solid_at(8, 8));
    }

    #[test]
    fn nothing_happens_with_nobody_on_it() {
        let mut block = block();
        for _ in 0..200 {
            update_crumble_block(&mut block, &away());
        }
        assert_eq!(block.stage, CrumbleStage::Idle);
    }

    #[test]
    fn standing_beside_it_does_not_set_it_off() {
        let mut block = block();
        let mut beside = climbing_it();
        beside.mode = Mode::Normal;
        update_crumble_block(&mut block, &beside);
        assert_eq!(block.stage, CrumbleStage::Idle);
    }

    #[test]
    fn standing_on_it_sets_it_off() {
        let mut block = block();
        update_crumble_block(&mut block, &on_top());
        assert_eq!(block.stage, CrumbleStage::Shaking);
        assert!(block.on_top);
    }

    #[test]
    fn climbing_it_sets_it_off() {
        let mut block = block();
        update_crumble_block(&mut block, &climbing_it());
        assert_eq!(block.stage, CrumbleStage::Shaking);
        assert!(!block.on_top);
    }

    #[test]
    fn it_goes_right_after_the_shake_if_she_stepped_off() {
        let mut block = block();
        update_crumble_block(&mut block, &on_top());

        // the first frame set it off, then the shake timer runs down
        assert_eq!(1 + frames_to_crumble(&mut block, &away()), 14);
    }

    #[test]
    fn it_holds_a_little_longer_while_she_stays_on_it() {
        let mut block = block();
        let stayed = frames_to_crumble(&mut block, &on_top());

        let mut other = self::block();
        update_crumble_block(&mut other, &on_top());
        let left = 1 + frames_to_crumble(&mut other, &away());

        assert!(
            (36..=40).contains(&stayed),
            "crumbled after {stayed} frames"
        );
        assert!(stayed > left);
    }

    #[test]
    fn climbing_it_takes_three_shakes_and_the_full_hold() {
        let mut block = block();
        update_crumble_block(&mut block, &climbing_it());

        // letting go makes no difference
        let frames = 1 + frames_to_crumble(&mut block, &away());
        assert!(
            (62..=68).contains(&frames),
            "crumbled after {frames} frames"
        );
    }

    #[test]
    fn it_comes_back_after_two_seconds() {
        let mut block = block();
        update_crumble_block(&mut block, &on_top());
        frames_to_crumble(&mut block, &away());

        let mut frames = 0;
        while !block.collidable {
            update_crumble_block(&mut block, &away());
            frames += 1;
        }

        assert!((120..=123).contains(&frames), "back after {frames} frames");
        assert_eq!(block.stage, CrumbleStage::Idle);
    }

    #[test]
    fn it_waits_for_her_to_get_out_of_the_way() {
        let mut block = block();
        block.collidable = false;
        block.stage = CrumbleStage::Gone;

        let inside = at(320, 70);
        for _ in 0..10 {
            update_crumble_block(&mut block, &inside);
        }
        assert!(!block.collidable);

        update_crumble_block(&mut block, &away());
        assert!(block.collidable);
    }

    #[test]
    fn it_can_be_set_off_again_the_frame_it_returns() {
        let mut block = block();
        block.collidable = false;
        block.stage = CrumbleStage::Gone;

        update_crumble_block(&mut block, &on_top());
        assert!(block.collidable);
        assert_eq!(block.stage, CrumbleStage::Shaking);
    }
}
