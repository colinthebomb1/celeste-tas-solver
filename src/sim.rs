use crate::entities::Entities;
use crate::entities::crumble_block::update_crumble_blocks;
use crate::entities::crystal::update_crystals;
use crate::entities::falling_block::update_falling_blocks;
use crate::entities::solid::reset_lift_speeds;
use crate::entities::zipper::update_zippers;
use crate::level::room::Room;
use crate::player::{Input, Player, step};

// one frame of the room
pub fn step_room(player: Player, input: Input, room: &Room, entities: &mut Entities) -> Player {
    let frozen = player.freeze_timer > 0.0;
    let mut player = step(player, input, room, entities);

    if !frozen {
        reset_lift_speeds(entities);
        update_crystals(entities, &mut player);
        update_zippers(room, entities, &mut player);
        update_falling_blocks(room, entities, &mut player);
        update_crumble_blocks(entities, &player);
    }

    player
}
