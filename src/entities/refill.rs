use crate::level::map::{Element, MapError};
use crate::level::room::{Rect, Room};
use crate::player::{DELTA_TIME, IVec2, PlayerState, hurtbox, use_refill};

pub const REFILL_SIZE: i32 = 16;
pub const REFILL_RESPAWN_TIME: f32 = 2.5;
pub const REFILL_FREEZE_TIME: f32 = 0.05;

// dash crystal
#[derive(Debug, Clone, PartialEq)]
pub struct Refill {
    pub hitbox: Rect,
    pub two_dashes: bool,
    pub one_use: bool,

    // everything below changes as it runs
    pub active: bool,
    pub respawn_timer: f32,
}

impl Refill {
    // (x, y) is its centre
    pub fn new(x: i32, y: i32, two_dashes: bool, one_use: bool) -> Refill {
        let hitbox = Rect {
            left: x - REFILL_SIZE / 2,
            top: y - REFILL_SIZE / 2,
            width: REFILL_SIZE,
            height: REFILL_SIZE,
        };

        Refill {
            hitbox,
            two_dashes,
            one_use,
            active: true,
            respawn_timer: 0.0,
        }
    }
}

pub fn load_refills(level: &Element, origin: IVec2) -> Result<Vec<Refill>, MapError> {
    let Some(entities) = level.child("entities") else {
        return Ok(Vec::new());
    };

    let mut refills = Vec::new();

    for entity in entities
        .children
        .iter()
        .filter(|entity| entity.name == "refill")
    {
        let x = entity.int("x").ok_or(MapError::MissingField("x"))?;
        let y = entity.int("y").ok_or(MapError::MissingField("y"))?;
        let two_dashes = entity.bool("twoDash").unwrap_or(false);
        let one_use = entity.bool("oneUse").unwrap_or(false);

        refills.push(Refill::new(origin.x + x, origin.y + y, two_dashes, one_use));
    }

    Ok(refills)
}

pub fn update_refills(room: &mut Room, state: &mut PlayerState) {
    for refill in &mut room.refills {
        let touching = refill.active && !state.dead && refill.hitbox.overlaps(&hurtbox(state));

        if touching && use_refill(state, refill.two_dashes) {
            refill.active = false;
            refill.respawn_timer = REFILL_RESPAWN_TIME;
            state.freeze_timer = state.freeze_timer.max(REFILL_FREEZE_TIME);
        }

        if refill.respawn_timer > 0.0 {
            refill.respawn_timer -= DELTA_TIME;
            if refill.respawn_timer <= 0.0 && !refill.one_use {
                refill.active = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;
    use crate::player::{CLIMB_MAX_STAMINA, CLIMB_TIRED_THRESHOLD};

    fn room_with_refill(refill: Refill) -> Room {
        let mut room = Room::from_tiles(IVec2 { x: 0, y: 0 }, "0000\n0000\n0000\n0000");
        room.refills.push(refill);
        room
    }

    fn player_at(x: i32, y: i32) -> PlayerState {
        PlayerState::new(
            IVec2 { x, y },
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 0.0, y: 0.0 },
        )
    }

    fn spent(x: i32, y: i32) -> PlayerState {
        PlayerState {
            dashes: 0,
            ..player_at(x, y)
        }
    }

    #[test]
    fn refill_hitbox_is_centred() {
        assert_eq!(
            Refill::new(100, 50, false, false).hitbox,
            Rect {
                left: 92,
                top: 42,
                width: 16,
                height: 16
            }
        );
    }

    #[test]
    fn touching_a_refill_restores_her_dash_and_stamina_and_freezes() {
        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = PlayerState {
            stamina: 50.0,
            ..spent(16, 20)
        };

        update_refills(&mut room, &mut s);

        assert_eq!(s.dashes, 1);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.freeze_timer, REFILL_FREEZE_TIME);
        assert!(!room.refills[0].active);
    }

    #[test]
    fn a_refill_is_left_alone_when_she_has_her_dash() {
        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = player_at(16, 20);

        update_refills(&mut room, &mut s);

        assert_eq!(s.freeze_timer, 0.0);
        assert!(room.refills[0].active);
    }

    #[test]
    fn low_stamina_alone_uses_a_refill() {
        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = PlayerState {
            stamina: CLIMB_TIRED_THRESHOLD - 1.0,
            ..player_at(16, 20)
        };

        update_refills(&mut room, &mut s);

        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert!(!room.refills[0].active);

        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = PlayerState {
            stamina: CLIMB_TIRED_THRESHOLD,
            ..player_at(16, 20)
        };

        update_refills(&mut room, &mut s);

        assert!(room.refills[0].active);
    }

    #[test]
    fn a_refill_touches_her_hurtbox_not_her_hitbox() {
        // refill spans y 8..24. her hurtbox stops 2 px above her feet
        let mut room = room_with_refill(Refill::new(16, 16, false, false));

        let mut s = spent(16, 10);
        update_refills(&mut room, &mut s);
        assert_eq!(s.dashes, 0);

        let mut s = spent(16, 11);
        update_refills(&mut room, &mut s);
        assert_eq!(s.dashes, 1);
    }

    #[test]
    fn a_used_refill_comes_back_after_its_respawn_time() {
        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = spent(16, 20);
        update_refills(&mut room, &mut s);

        let mut away = spent(100, 100);
        let mut frames = 1;
        while !room.refills[0].active {
            update_refills(&mut room, &mut away);
            frames += 1;
        }

        assert_eq!(frames, 150);
    }

    #[test]
    fn a_used_refill_does_nothing_until_it_is_back() {
        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = spent(16, 20);
        update_refills(&mut room, &mut s);

        let mut s = spent(16, 20);
        update_refills(&mut room, &mut s);

        assert_eq!(s.dashes, 0);
    }

    #[test]
    fn a_one_use_refill_never_comes_back() {
        let mut room = room_with_refill(Refill::new(16, 16, false, true));
        let mut s = spent(16, 20);
        update_refills(&mut room, &mut s);

        let mut away = spent(100, 100);
        for _ in 0..200 {
            update_refills(&mut room, &mut away);
        }

        assert!(!room.refills[0].active);
    }

    #[test]
    fn a_two_dash_refill_gives_two_dashes() {
        let mut room = room_with_refill(Refill::new(16, 16, true, false));
        let mut s = player_at(16, 20);

        update_refills(&mut room, &mut s);

        assert_eq!(s.dashes, 2);
        assert!(!room.refills[0].active);
    }

    #[test]
    fn the_dead_do_not_use_refills() {
        let mut room = room_with_refill(Refill::new(16, 16, false, false));
        let mut s = PlayerState {
            dead: true,
            ..spent(16, 20)
        };

        update_refills(&mut room, &mut s);

        assert_eq!(s.dashes, 0);
    }
}
