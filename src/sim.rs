use crate::entities::zip_mover::update_zip_movers;
use crate::physics::{step, Input, PlayerState};
use crate::room::Room;

// one frame of the room: her update, then the moving blocks. a freeze frame stops both
pub fn step_room(state: PlayerState, input: Input, room: &mut Room) -> PlayerState {
    let frozen = state.freeze_timer > 0.0;
    let mut state = step(state, input, room);

    if !frozen {
        update_zip_movers(room, &mut state);
    }

    state
}
