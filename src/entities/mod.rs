pub mod jump_thru;
pub mod spikes;
pub mod zip_mover;

pub use jump_thru::jump_thru;
pub use spikes::Spikes;
pub use zip_mover::{ZipMover, ZipStage};

use crate::physics::{step, Input, PlayerState};
use crate::room::Room;
use zip_mover::update_zip_movers;

// one frame of the room: her update, then the moving blocks. a freeze frame stops both
pub fn step_room(state: PlayerState, input: Input, room: &mut Room) -> PlayerState {
    let frozen = state.freeze_timer > 0.0;
    let mut state = step(state, input, room);

    if !frozen {
        update_zip_movers(room, &mut state);
    }

    state
}
