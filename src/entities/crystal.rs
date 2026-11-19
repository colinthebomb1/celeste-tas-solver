use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, entities};
use crate::level::room::Room;
use crate::player::Player;
use crate::player::collision::hurtbox;
use crate::player::constants::DELTA_TIME;
use crate::player::dash::use_crystal;

const CRYSTAL_SIZE: i32 = 16;
const CRYSTAL_RESPAWN_TIME: f32 = 2.5;
const CRYSTAL_FREEZE_TIME: f32 = 0.05;

// dash crystal
#[derive(Debug, Clone, PartialEq)]
pub struct Crystal {
    pub hitbox: Rect,
    pub two_dashes: bool,
    pub one_use: bool,

    // everything below changes as it runs
    pub active: bool,
    pub respawn_timer: f32,
}

impl Crystal {
    // (x, y) is its centre
    pub fn new(x: i32, y: i32, two_dashes: bool, one_use: bool) -> Crystal {
        let hitbox = Rect {
            left: x - CRYSTAL_SIZE / 2,
            top: y - CRYSTAL_SIZE / 2,
            width: CRYSTAL_SIZE,
            height: CRYSTAL_SIZE,
        };

        Crystal {
            hitbox,
            two_dashes,
            one_use,
            active: true,
            respawn_timer: 0.0,
        }
    }
}

pub(crate) fn load_crystals(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for entity in entities(level, "refill") {
        let two_dashes = entity.bool("twoDash").unwrap_or(false);
        let one_use = entity.bool("oneUse").unwrap_or(false);

        room.crystals.push(Crystal::new(
            origin.x + entity.require("x")?,
            origin.y + entity.require("y")?,
            two_dashes,
            one_use,
        ));
    }

    Ok(())
}

pub(crate) fn update_crystals(room: &mut Room, player: &mut Player) {
    for crystal in &mut room.crystals {
        let touching = crystal.active && !player.dead && crystal.hitbox.overlaps(&hurtbox(player));

        if touching && use_crystal(player, crystal.two_dashes) {
            crystal.active = false;
            crystal.respawn_timer = CRYSTAL_RESPAWN_TIME;
            player.freeze_timer = player.freeze_timer.max(CRYSTAL_FREEZE_TIME);
        }

        if crystal.respawn_timer > 0.0 {
            crystal.respawn_timer -= DELTA_TIME;
            if crystal.respawn_timer <= 0.0 && !crystal.one_use {
                crystal.active = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::at;
    use crate::player::constants::{CLIMB_MAX_STAMINA, CLIMB_TIRED_THRESHOLD};

    fn room_with_crystal(crystal: Crystal) -> Room {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, "0000\n0000\n0000\n0000");
        room.crystals.push(crystal);
        room
    }

    fn spent(x: i32, y: i32) -> Player {
        Player {
            dashes: 0,
            ..at(x, y)
        }
    }

    #[test]
    fn crystal_hitbox_is_centred() {
        assert_eq!(
            Crystal::new(100, 50, false, false).hitbox,
            Rect {
                left: 92,
                top: 42,
                width: 16,
                height: 16
            }
        );
    }

    #[test]
    fn touching_a_crystal_restores_her_dash_and_stamina_and_freezes() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = Player {
            stamina: 50.0,
            ..spent(16, 20)
        };

        update_crystals(&mut room, &mut s);

        assert_eq!(s.dashes, 1);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.freeze_timer, CRYSTAL_FREEZE_TIME);
        assert!(!room.crystals[0].active);
    }

    #[test]
    fn a_crystal_is_left_alone_when_she_has_her_dash() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = at(16, 20);

        update_crystals(&mut room, &mut s);

        assert_eq!(s.freeze_timer, 0.0);
        assert!(room.crystals[0].active);
    }

    #[test]
    fn low_stamina_alone_uses_a_crystal() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = Player {
            stamina: CLIMB_TIRED_THRESHOLD - 1.0,
            ..at(16, 20)
        };

        update_crystals(&mut room, &mut s);

        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert!(!room.crystals[0].active);

        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = Player {
            stamina: CLIMB_TIRED_THRESHOLD,
            ..at(16, 20)
        };

        update_crystals(&mut room, &mut s);

        assert!(room.crystals[0].active);
    }

    #[test]
    fn a_crystal_touches_her_hurtbox_not_her_hitbox() {
        // crystal spans y 8..24. her hurtbox stops 2 px above her feet
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));

        let mut s = spent(16, 10);
        update_crystals(&mut room, &mut s);
        assert_eq!(s.dashes, 0);

        let mut s = spent(16, 11);
        update_crystals(&mut room, &mut s);
        assert_eq!(s.dashes, 1);
    }

    #[test]
    fn a_used_crystal_comes_back_after_its_respawn_time() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = spent(16, 20);
        update_crystals(&mut room, &mut s);

        let mut away = spent(100, 100);
        let mut frames = 1;
        while !room.crystals[0].active {
            update_crystals(&mut room, &mut away);
            frames += 1;
        }

        assert_eq!(frames, 150);
    }

    #[test]
    fn a_used_crystal_does_nothing_until_it_is_back() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = spent(16, 20);
        update_crystals(&mut room, &mut s);

        let mut s = spent(16, 20);
        update_crystals(&mut room, &mut s);

        assert_eq!(s.dashes, 0);
    }

    #[test]
    fn a_one_use_crystal_never_comes_back() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, true));
        let mut s = spent(16, 20);
        update_crystals(&mut room, &mut s);

        let mut away = spent(100, 100);
        for _ in 0..200 {
            update_crystals(&mut room, &mut away);
        }

        assert!(!room.crystals[0].active);
    }

    #[test]
    fn a_two_dash_refill_gives_two_dashes() {
        let mut room = room_with_crystal(Crystal::new(16, 16, true, false));
        let mut s = at(16, 20);

        update_crystals(&mut room, &mut s);

        assert_eq!(s.dashes, 2);
        assert!(!room.crystals[0].active);
    }

    #[test]
    fn the_dead_do_not_use_crystals() {
        let mut room = room_with_crystal(Crystal::new(16, 16, false, false));
        let mut s = Player {
            dead: true,
            ..spent(16, 20)
        };

        update_crystals(&mut room, &mut s);

        assert_eq!(s.dashes, 0);
    }
}
