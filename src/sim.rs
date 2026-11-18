use crate::entities::crumble_block::update_crumble_blocks;
use crate::entities::falling_block::update_falling_blocks;
use crate::entities::refill::update_refills;
use crate::entities::solid::reset_lift_speeds;
use crate::entities::zip_mover::update_zip_movers;
use crate::level::room::Room;
use crate::player::{Input, PlayerState, step};

// one frame of the room
pub fn step_room(state: PlayerState, input: Input, room: &mut Room) -> PlayerState {
    let frozen = state.freeze_timer > 0.0;
    let mut state = step(state, input, room);

    if !frozen {
        reset_lift_speeds(room);
        update_refills(room, &mut state);
        update_zip_movers(room, &mut state);
        update_falling_blocks(room, &mut state);
        update_crumble_blocks(room, &state);
    }

    state
}
