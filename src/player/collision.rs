use super::constants::*;
use super::state::{Player, State};
use crate::entities::Entities;
use crate::entities::Spikes;
use crate::geom::{Direction, Rect};
use crate::level::room::Room;

pub(crate) fn hitbox_height(player: &Player) -> i32 {
    if player.crouching {
        CROUCH_HITBOX_HEIGHT
    } else {
        HITBOX_HEIGHT
    }
}

// her position is the middle of the box's bottom edge
fn box_at_feet(player: &Player, height: i32) -> Rect {
    Rect::new(
        player.position.x - HITBOX_WIDTH / 2,
        player.position.y - height,
        HITBOX_WIDTH,
        height,
    )
}

pub(crate) fn hitbox(player: &Player, dx: i32, dy: i32) -> Rect {
    box_at_feet(player, hitbox_height(player)).shifted(dx, dy)
}

// same top as the hitbox, but it stops short of her feet
pub(crate) fn hurtbox(player: &Player) -> Rect {
    let height = if player.crouching {
        CROUCH_HURTBOX_HEIGHT
    } else {
        HURTBOX_HEIGHT
    };
    Rect {
        height,
        ..hitbox(player, 0, 0)
    }
}

// whether her hitbox would be inside something solid if she moved this far
pub(crate) fn collides(
    player: &Player,
    room: &Room,
    entities: &Entities,
    dx: i32,
    dy: i32,
) -> bool {
    room.is_solid(entities, &hitbox(player, dx, dy))
}

// like collides, but a jump-through under her also counts when she is moving down onto it
pub(crate) fn blocked(player: &Player, room: &Room, entities: &Entities, dx: i32, dy: i32) -> bool {
    collides(player, room, entities, dx, dy) || (dy > 0 && enters_jump_thru(player, room, dx, dy))
}

pub(crate) fn on_ground(player: &Player, room: &Room, entities: &Entities) -> bool {
    player.speed.y >= 0.0 && blocked(player, room, entities, 0, 1)
}

pub(crate) fn can_stand(player: &Player, room: &Room, entities: &Entities) -> bool {
    !player.crouching || !room.is_solid(entities, &box_at_feet(player, HITBOX_HEIGHT))
}

// room to be crouched this far to the side
pub(crate) fn crouch_free_at(player: &Player, room: &Room, entities: &Entities, dx: i32) -> bool {
    !room.is_solid(
        entities,
        &box_at_feet(player, CROUCH_HITBOX_HEIGHT).shifted(dx, 0),
    )
}

pub(crate) fn spikes_kill(player: &Player, spikes: &Spikes) -> bool {
    let madeline_hurtbox = &hurtbox(player);

    if !spikes.hitbox.overlaps(madeline_hurtbox) {
        return false;
    }

    match spikes.direction {
        Direction::Up => {
            player.speed.y >= 0.0
                && madeline_hurtbox.top + madeline_hurtbox.height
                    <= spikes.hitbox.top + spikes.hitbox.height
        }
        Direction::Down => player.speed.y <= 0.0,
        Direction::Left => player.speed.x >= 0.0,
        Direction::Right => player.speed.x <= 0.0,
    }
}

// the top of her hitbox is more than 4 px below the room's bottom edge
pub(crate) fn fell_out(player: &Player, room: &Room) -> bool {
    hitbox(player, 0, 0).top > room.bottom() + 4
}

// spikes she is never eased into or onto
pub(crate) fn spikes_block(room: &Room, rect: &Rect) -> bool {
    room.spikes
        .iter()
        .any(|spikes| spikes.direction != Direction::Down && spikes.hitbox.overlaps(rect))
}

// whether a dash correction that moves her this far would put her hurtbox in spikes
pub(crate) fn correction_spiked(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    spikes_block(room, &hurtbox(player).shifted(dx, dy))
}

pub(crate) fn touches_finish(player: &Player, room: &Room) -> bool {
    room.finish_trigger
        .is_some_and(|trigger| trigger.overlaps(&hitbox(player, 0, 0)))
}

// any spikes, whichever way they point
pub(crate) fn touches_spikes(player: &Player, room: &Room) -> bool {
    let her = hitbox(player, 0, 0);
    room.spikes
        .iter()
        .any(|spikes| spikes.hitbox.overlaps(&her))
}

pub(crate) fn touches_jump_thru(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    let moved = hitbox(player, dx, dy);
    room.jump_thrus
        .iter()
        .any(|jump_thru| jump_thru.hitbox.overlaps(&moved))
}

pub(crate) fn enters_jump_thru(player: &Player, room: &Room, dx: i32, dy: i32) -> bool {
    let now = hitbox(player, 0, 0);
    let moved = hitbox(player, dx, dy);
    room.jump_thrus
        .iter()
        .any(|jump_thru| !jump_thru.hitbox.overlaps(&now) && jump_thru.hitbox.overlaps(&moved))
}

// whether a moving block carries her along
pub(crate) fn is_riding(player: &Player, solid: &Rect) -> bool {
    if player.dead {
        false
    } else if player.state == State::Climb {
        hitbox(player, player.facing as i32, 0).overlaps(solid)
    } else if player.climb.trigger_dir != 0 {
        hitbox(player, player.climb.trigger_dir as i32, 0).overlaps(solid)
    } else {
        hitbox(player, 0, 1).overlaps(solid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::{IVec2, Vec2};
    use crate::player::fixtures::*;
    use crate::player::state::Input;
    use crate::player::step::step;

    #[test]
    fn collide_at_uses_her_hitbox() {
        let room = room();
        let inside =
            |x: i32, y: i32| collides(&player_at(x, y, 0.0, 0.0), &room, &room.initial, 0, 0);

        assert!(!inside(0, FLOOR_Y));
        assert!(inside(0, FLOOR_Y + 1));
        assert!(inside(LEDGE_X, FLOOR_Y + 1));
        assert!(!inside(LEDGE_X + 1, FLOOR_Y + 1));
        assert!(!inside(WALL_X, 64));
        assert!(inside(WALL_X + 1, 64));
        assert!(!inside(60, FLOOR_Y + 12));
    }

    #[test]
    fn auto_jump_cleared_on_ground() {
        let room = room();
        let mut s = standing();
        s.auto_jump = true;
        s = step(s, Input::default(), &room, &room.initial);
        assert!(!s.auto_jump);
    }
    #[test]
    fn hurtbox_is_two_pixels_shorter_than_the_hitbox() {
        let mut s = at_rest();
        s.position = IVec2 { x: 100, y: 50 };
        assert_eq!(
            hurtbox(&s),
            Rect {
                left: 96,
                top: 39,
                width: 8,
                height: 9
            }
        );
    }

    #[test]
    fn up_spikes_kill_when_standing_or_falling_on_them() {
        let spikes = Spikes::new(Direction::Up, 0, 100, 16);
        assert!(spikes_kill(&player_at(8, 100, 0.0, 0.0), &spikes));
        assert!(spikes_kill(&player_at(8, 100, 0.0, 160.0), &spikes));
        assert!(!spikes_kill(&player_at(8, 100, 0.0, -10.0), &spikes));
    }

    #[test]
    fn up_spikes_need_the_hurtbox_to_overlap() {
        let spikes = Spikes::new(Direction::Up, 0, 100, 16);
        assert!(!spikes_kill(&player_at(8, 99, 0.0, 0.0), &spikes));
        assert!(spikes_kill(&player_at(19, 100, 0.0, 0.0), &spikes));
        assert!(!spikes_kill(&player_at(20, 100, 0.0, 0.0), &spikes));
    }

    #[test]
    fn up_spikes_spare_her_when_her_hurtbox_reaches_below_them() {
        let spikes = Spikes::new(Direction::Up, 0, 100, 16);
        assert!(spikes_kill(&player_at(8, 102, 0.0, 0.0), &spikes));
        assert!(!spikes_kill(&player_at(8, 103, 0.0, 0.0), &spikes));
    }

    #[test]
    fn down_spikes_kill_unless_moving_down() {
        let spikes = Spikes::new(Direction::Down, 0, 100, 16);
        assert!(spikes_kill(&player_at(8, 111, 0.0, 0.0), &spikes));
        assert!(spikes_kill(&player_at(8, 111, 0.0, -50.0), &spikes));
        assert!(!spikes_kill(&player_at(8, 111, 0.0, 50.0), &spikes));
        assert!(!spikes_kill(&player_at(8, 114, 0.0, -50.0), &spikes));
    }

    #[test]
    fn left_spikes_kill_unless_moving_left() {
        let spikes = Spikes::new(Direction::Left, 100, 0, 16);
        assert!(spikes_kill(&player_at(94, 12, 0.0, 0.0), &spikes));
        assert!(spikes_kill(&player_at(94, 12, 50.0, 0.0), &spikes));
        assert!(!spikes_kill(&player_at(94, 12, -50.0, 0.0), &spikes));
        assert!(!spikes_kill(&player_at(93, 12, 50.0, 0.0), &spikes));
    }

    #[test]
    fn right_spikes_kill_unless_moving_right() {
        let spikes = Spikes::new(Direction::Right, 100, 0, 16);
        assert!(spikes_kill(&player_at(106, 12, 0.0, 0.0), &spikes));
        assert!(spikes_kill(&player_at(106, 12, -50.0, 0.0), &spikes));
        assert!(!spikes_kill(&player_at(106, 12, 50.0, 0.0), &spikes));
        assert!(!spikes_kill(&player_at(107, 12, -50.0, 0.0), &spikes));
    }

    #[test]
    fn step_kills_on_spikes() {
        let room = spiked_floor();
        let s = step(standing(), Input::default(), &room, &room.initial);
        assert!(s.dead);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn step_kills_on_the_frame_she_lands_on_spikes() {
        let room = spiked_floor();
        let mut s = player_at(0, FLOOR_Y - 5, 0.0, 160.0);
        s = step(s, Input::default(), &room, &room.initial);
        assert!(!s.dead);
        s = step(s, Input::default(), &room, &room.initial);
        assert!(s.dead);
        assert_eq!(s.position.y, FLOOR_Y);
    }

    #[test]
    fn no_spikes_no_death() {
        let room = room();
        let s = step(standing(), Input::default(), &room, &room.initial);
        assert!(!s.dead);
    }

    #[test]
    fn fell_out_once_her_head_is_5_px_below_the_room() {
        let room = room();
        assert_eq!(room.bottom(), 180);
        assert!(!fell_out(&player_at(60, 180, 0.0, 0.0), &room));
        assert!(!fell_out(&player_at(60, 195, 0.0, 0.0), &room));
        assert!(fell_out(&player_at(60, 196, 0.0, 0.0), &room));
    }

    #[test]
    fn fell_out_sooner_when_crouched() {
        // her crouched hitbox is 5 px shorter, so its top passes the line 5 px earlier
        let room = room();
        assert!(!fell_out(&crouching_at(60, 190), &room));
        assert!(fell_out(&crouching_at(60, 191), &room));
    }

    #[test]
    fn spikes_block_unless_they_point_down() {
        let mut room = room();
        room.spikes.push(up_spikes(0, FLOOR_Y, 40));
        room.spikes.push(Spikes::new(Direction::Down, 0, 20, 40));

        assert!(spikes_block(&room, &Rect::new(10, FLOOR_Y - 2, 4, 4)));
        assert!(!spikes_block(&room, &Rect::new(10, 20, 4, 4)));
        assert!(!spikes_block(&room, &Rect::new(10, 60, 4, 4)));
    }

    #[test]
    fn touches_spikes_that_overlap_her_hitbox() {
        let room = spiked_floor();
        assert!(touches_spikes(&player_at(20, FLOOR_Y, 0.0, 0.0), &room));
        assert!(!touches_spikes(
            &player_at(20, FLOOR_Y - 3, 0.0, 0.0),
            &room
        ));
        assert!(!touches_spikes(&player_at(60, FLOOR_Y, 0.0, 0.0), &room));
    }

    #[test]
    fn can_stand_when_not_crouching() {
        let room = tunnel_room();
        assert!(can_stand(
            &player_at(40, FLOOR_Y, 0.0, 0.0),
            &room,
            &room.initial
        ));
    }

    #[test]
    fn can_stand_with_room_overhead() {
        let room = tunnel_room();
        assert!(can_stand(&crouching_at(8, FLOOR_Y), &room, &room.initial));
    }

    #[test]
    fn can_stand_not_under_a_low_ceiling() {
        let room = tunnel_room();
        assert!(!can_stand(&crouching_at(40, FLOOR_Y), &room, &room.initial));
    }

    #[test]
    fn hurtbox_is_shorter_when_crouching() {
        assert_eq!(
            hurtbox(&crouching_at(100, 50)),
            Rect {
                left: 96,
                top: 44,
                width: 8,
                height: 4
            }
        );
    }

    #[test]
    fn crouch_free_at_checks_the_crouching_hitbox() {
        let room = tunnel_room();
        let s = player_at(20, FLOOR_Y, 0.0, 0.0);
        assert!(crouch_free_at(&s, &room, &room.initial, 1));
        assert!(collides(&s, &room, &room.initial, 1, 0));
        assert!(!crouch_free_at(
            &player_at(20, FLOOR_Y - 4, 0.0, 0.0),
            &room,
            &room.initial,
            1
        ));
    }

    #[test]
    fn climbing_rides_the_block_in_front_of_her() {
        let beside = Rect {
            left: 80,
            top: 80,
            width: 16,
            height: 40,
        };
        assert!(is_riding(&climbing(100), &beside));
        assert!(!is_riding(&player_at(WALL_X, 100, 0.0, 0.0), &beside));
    }
}
