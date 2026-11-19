use crate::entities::Solid;
use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities};
use crate::level::room::{Room, RoomState};
use crate::player::collision::hitbox;
use crate::player::constants::DELTA_TIME;
use crate::player::{Player, State};

const CRUMBLE_HEIGHT: i32 = 8;
const CRUMBLE_SHAKE_TIME: f32 = 0.2;
const CRUMBLE_SHAKES_ON_TOP: u8 = 1;
const CRUMBLE_SHAKES_CLIMBING: u8 = 3;
const CRUMBLE_HOLD_TIME: f32 = 0.4;
const CRUMBLE_RESPAWN_TIME: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrumbleStage {
    Idle,
    Shaking,
    Holding,
    Gone,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrumbleBlock {
    pub solid: usize,

    // everything below changes as it runs
    pub stage: CrumbleStage,
    pub timer: f32,
    pub shakes_left: u8,
    pub on_top: bool,
}

impl RoomState {
    // (x, y) is its top-left corner
    pub fn add_crumble_block(&mut self, x: i32, y: i32, width: i32) {
        let solid = self.add_solid(Rect::new(x, y, width, CRUMBLE_HEIGHT));
        self.crumble_blocks.push(CrumbleBlock {
            solid,
            stage: CrumbleStage::Idle,
            timer: 0.0,
            shakes_left: 0,
            on_top: false,
        });
    }
}

pub(crate) fn load_crumble_blocks(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for entity in entities(level, "crumbleBlock") {
        room.initial.add_crumble_block(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
            entity.require("width")?,
        );
    }

    Ok(())
}

pub(crate) fn update_crumble_blocks(state: &mut RoomState, player: &Player) {
    let RoomState {
        crumble_blocks,
        solids,
        ..
    } = state;

    for block in crumble_blocks {
        update_crumble_block(block, &mut solids[block.solid], player);
    }
}

fn standing_on(player: &Player, solid: &Solid) -> bool {
    !player.dead && hitbox(player, 0, 1).overlaps(&solid.rect)
}

pub(crate) fn climbing(player: &Player, solid: &Solid) -> bool {
    !player.dead
        && player.state == State::Climb
        && hitbox(player, player.facing as i32, 0).overlaps(&solid.rect)
}

fn update_crumble_block(block: &mut CrumbleBlock, solid: &mut Solid, player: &Player) {
    match block.stage {
        CrumbleStage::Idle => set_off(block, solid, player),

        CrumbleStage::Shaking => {
            if block.timer > 0.0 {
                block.timer -= DELTA_TIME;
            } else if block.shakes_left > 0 {
                block.shakes_left -= 1;
                block.timer = CRUMBLE_SHAKE_TIME;
            } else {
                block.timer = CRUMBLE_HOLD_TIME;
                hold_or_crumble(block, solid, player);
            }
        }

        CrumbleStage::Holding => {
            block.timer -= DELTA_TIME;
            hold_or_crumble(block, solid, player);
        }

        CrumbleStage::Gone => {
            if block.timer > 0.0 {
                block.timer -= DELTA_TIME;
            } else if player.dead || !hitbox(player, 0, 0).overlaps(&solid.rect) {
                solid.collidable = true;
                block.stage = CrumbleStage::Idle;
                set_off(block, solid, player);
            }
        }
    }
}

fn set_off(block: &mut CrumbleBlock, solid: &Solid, player: &Player) {
    if standing_on(player, solid) {
        block.on_top = true;
        block.shakes_left = CRUMBLE_SHAKES_ON_TOP - 1;
    } else if climbing(player, solid) {
        block.on_top = false;
        block.shakes_left = CRUMBLE_SHAKES_CLIMBING - 1;
    } else {
        return;
    }

    block.timer = CRUMBLE_SHAKE_TIME;
    block.stage = CrumbleStage::Shaking;
}

// stood on, it goes early if she steps off. climbed, it always waits the full time
fn hold_or_crumble(block: &mut CrumbleBlock, solid: &mut Solid, player: &Player) {
    let holding = block.timer > 0.0 && (!block.on_top || standing_on(player, solid));

    if holding {
        block.stage = CrumbleStage::Holding;
    } else {
        solid.collidable = false;
        block.timer = CRUMBLE_RESPAWN_TIME;
        block.stage = CrumbleStage::Gone;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::at;

    // one block, 40 px wide with its top at y = 64
    fn block() -> RoomState {
        let mut state = RoomState::default();
        state.add_crumble_block(304, 64, 40);
        state
    }

    fn solid(state: &RoomState) -> bool {
        state.solids[state.crumble_blocks[0].solid].collidable
    }

    fn on_top() -> Player {
        at(320, 64)
    }

    fn away() -> Player {
        at(100, 100)
    }

    // holding its right side
    fn climbing_it() -> Player {
        let mut s = at(348, 72);
        s.facing = -1;
        s.state = State::Climb;
        s
    }

    // frames until it is no longer solid
    fn frames_to_crumble(state: &mut RoomState, player: &Player) -> usize {
        let mut frames = 0;
        while solid(state) {
            update_crumble_blocks(state, player);
            frames += 1;
            assert!(frames < 1000);
        }
        frames
    }

    #[test]
    fn it_is_eight_pixels_tall() {
        assert_eq!(block().solids[0].rect, Rect::new(304, 64, 40, 8));
    }

    #[test]
    fn it_is_solid_until_it_crumbles() {
        let mut block = block();
        let corner = Rect::new(304, 64, 1, 1);
        assert!(block.solid(&corner));

        block.solids[0].collidable = false;
        assert!(!block.solid(&corner));
    }

    #[test]
    fn nothing_happens_with_nobody_on_it() {
        let mut block = block();
        for _ in 0..200 {
            update_crumble_blocks(&mut block, &away());
        }
        assert_eq!(block.crumble_blocks[0].stage, CrumbleStage::Idle);
    }

    #[test]
    fn standing_beside_it_does_not_set_it_off() {
        let mut block = block();
        let mut beside = climbing_it();
        beside.state = State::Normal;
        update_crumble_blocks(&mut block, &beside);
        assert_eq!(block.crumble_blocks[0].stage, CrumbleStage::Idle);
    }

    #[test]
    fn standing_on_it_sets_it_off() {
        let mut block = block();
        update_crumble_blocks(&mut block, &on_top());
        assert_eq!(block.crumble_blocks[0].stage, CrumbleStage::Shaking);
        assert!(block.crumble_blocks[0].on_top);
    }

    #[test]
    fn climbing_it_sets_it_off() {
        let mut block = block();
        update_crumble_blocks(&mut block, &climbing_it());
        assert_eq!(block.crumble_blocks[0].stage, CrumbleStage::Shaking);
        assert!(!block.crumble_blocks[0].on_top);
    }

    #[test]
    fn it_goes_right_after_the_shake_if_she_stepped_off() {
        let mut block = block();
        update_crumble_blocks(&mut block, &on_top());

        // the first frame set it off, then the shake timer runs down
        assert_eq!(1 + frames_to_crumble(&mut block, &away()), 14);
    }

    #[test]
    fn it_holds_a_little_longer_while_she_stays_on_it() {
        let mut block = block();
        let stayed = frames_to_crumble(&mut block, &on_top());

        let mut other = self::block();
        update_crumble_blocks(&mut other, &on_top());
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
        update_crumble_blocks(&mut block, &climbing_it());

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
        update_crumble_blocks(&mut block, &on_top());
        frames_to_crumble(&mut block, &away());

        let mut frames = 0;
        while !solid(&block) {
            update_crumble_blocks(&mut block, &away());
            frames += 1;
        }

        assert!((120..=123).contains(&frames), "back after {frames} frames");
        assert_eq!(block.crumble_blocks[0].stage, CrumbleStage::Idle);
    }

    #[test]
    fn it_waits_for_her_to_get_out_of_the_way() {
        let mut block = block();
        block.solids[0].collidable = false;
        block.crumble_blocks[0].stage = CrumbleStage::Gone;

        let inside = at(320, 70);
        for _ in 0..10 {
            update_crumble_blocks(&mut block, &inside);
        }
        assert!(!solid(&block));

        update_crumble_blocks(&mut block, &away());
        assert!(solid(&block));
    }

    #[test]
    fn it_can_be_set_off_again_the_frame_it_returns() {
        let mut block = block();
        block.solids[0].collidable = false;
        block.crumble_blocks[0].stage = CrumbleStage::Gone;

        update_crumble_blocks(&mut block, &on_top());
        assert!(solid(&block));
        assert_eq!(block.crumble_blocks[0].stage, CrumbleStage::Shaking);
    }
}
