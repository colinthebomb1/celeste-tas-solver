use crate::entities::Entities;
use crate::entities::crumble_block::update_crumble_blocks;
use crate::entities::crystal::update_crystals;
use crate::entities::falling_block::update_falling_blocks;
use crate::entities::solid::reset_lift_speeds;
use crate::entities::zipper::update_zippers;
use crate::level::room::{Chapter, Room};
use crate::level::transition::{
    TRANSITION_FRAMES, Transition, before_transition, check_room_edges, finish_transition,
};
use crate::player::state::track_buttons;
use crate::player::{Input, Player, step};

// one frame of the room
pub fn step_room(player: Player, input: Input, room: &Room, entities: &mut Entities) -> Player {
    // entities skip a frame that began frozen, step counts the freeze down
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

// everything that changes as she plays through a chapter
#[derive(Debug, Clone, PartialEq)]
pub struct Sim {
    pub player: Player,
    // index into Chapter::rooms
    pub room: usize,
    pub entities: Entities,
    // the transition if happening, and how many frames of it are left
    transition: Option<(Transition, usize)>,
}

impl Sim {
    pub fn new(chapter: &Chapter, room: usize, player: Player) -> Sim {
        Sim {
            player,
            room,
            entities: chapter.rooms[room].initial.clone(),
            transition: None,
        }
    }

    pub fn in_transition(&self) -> bool {
        self.transition.is_some()
    }
}

// one frame of the chapter, which may start or continue a room transition
pub fn step_chapter(sim: &mut Sim, chapter: &Chapter, input: Input) {
    if sim.player.finished {
        return;
    }

    let Some((transition, frames_left)) = sim.transition else {
        sim.player = step_room(
            sim.player,
            input,
            &chapter.rooms[sim.room],
            &mut sim.entities,
        );

        if !sim.player.dead
            && let Some(found) = check_room_edges(&mut sim.player, chapter, sim.room, &sim.entities)
        {
            before_transition(&mut sim.player, found.direction);
            sim.transition = Some((found, TRANSITION_FRAMES));
        }
        return;
    };

    // she doesn't move during a transition, but buttons let go of count as presses afterwards
    track_buttons(&mut sim.player, input);

    if frames_left > 1 {
        sim.transition = Some((transition, frames_left - 1));
    } else {
        finish_transition(&mut sim.player, chapter, &sim.entities, transition);
        sim.room = transition.to;
        sim.entities = chapter.rooms[sim.room].initial.clone();
        sim.transition = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Crystal;
    use crate::fixtures::at;
    use crate::geom::IVec2;

    const RIGHT: Input = Input {
        move_x: 1,
        move_y: 0,
        jump: false,
        jump2: false,
        dash: false,
        grab: false,
        demodash: false,
    };

    // two rooms side by side, each 88 px wide with a floor whose top is y = 176
    fn two_rooms() -> Chapter {
        let tiles = format!("{}11111111111", "00000000000\n".repeat(22));
        let room = |x: i32| Room::from_tiles(IVec2 { x, y: 0 }, &tiles);
        Chapter {
            rooms: vec![room(0), room(88)],
        }
    }

    // she runs right from near the first room's edge until a transition starts
    fn at_the_start_of_a_transition(chapter: &Chapter) -> Sim {
        let mut sim = Sim::new(chapter, 0, at(80, 176));
        for _ in 0..30 {
            if sim.in_transition() {
                return sim;
            }
            step_chapter(&mut sim, chapter, RIGHT);
        }
        panic!("she never left the room");
    }

    #[test]
    fn stays_in_her_room_until_she_walks_out_of_it() {
        let chapter = two_rooms();
        let mut sim = Sim::new(&chapter, 0, at(40, 176));
        step_chapter(&mut sim, &chapter, RIGHT);
        assert!(!sim.in_transition());
        assert_eq!(sim.room, 0);
    }

    #[test]
    fn a_transition_holds_her_still_for_its_frames_then_puts_her_in_the_next_room() {
        let chapter = two_rooms();
        let mut sim = at_the_start_of_a_transition(&chapter);
        let before = sim.player.position;

        for _ in 0..TRANSITION_FRAMES - 1 {
            step_chapter(&mut sim, &chapter, RIGHT);
            assert!(sim.in_transition());
            assert_eq!(sim.room, 0);
            assert_eq!(sim.player.position, before);
        }

        step_chapter(&mut sim, &chapter, RIGHT);
        assert!(!sim.in_transition());
        assert_eq!(sim.room, 1);
        // 4 px inside the new room
        assert_eq!(sim.player.position, IVec2 { x: 92, y: 176 });
    }

    #[test]
    fn an_edge_with_no_room_past_it_holds_her_in() {
        let chapter = two_rooms();
        let left = Input {
            move_x: -1,
            ..RIGHT
        };
        let mut sim = Sim::new(&chapter, 0, at(10, 176));
        for _ in 0..30 {
            step_chapter(&mut sim, &chapter, left);
        }

        // her hitbox is 8 px wide, so its left side is on the room's edge.
        // the edge is not a wall: her speed builds up again until she next crosses it
        for _ in 0..10 {
            step_chapter(&mut sim, &chapter, left);
            assert!(!sim.in_transition());
            assert_eq!(sim.room, 0);
            assert_eq!(sim.player.position.x, 4);
        }
    }

    #[test]
    fn nothing_changes_after_she_finishes_the_chapter() {
        let mut chapter = two_rooms();
        chapter.rooms[0].finish_trigger = Some(crate::geom::Rect::new(40, 160, 16, 16));
        let mut sim = Sim::new(&chapter, 0, at(48, 176));

        step_chapter(&mut sim, &chapter, RIGHT);
        assert!(sim.player.finished);

        let done = sim.clone();
        step_chapter(&mut sim, &chapter, RIGHT);
        assert_eq!(sim, done);
    }

    #[test]
    fn a_room_starts_fresh_each_time_she_enters() {
        let mut chapter = two_rooms();
        chapter.rooms[1]
            .initial
            .crystals
            .push(Crystal::new(120, 100, false, false));
        let mut sim = at_the_start_of_a_transition(&chapter);

        // whatever happened to the room she left is dropped
        sim.entities
            .crystals
            .push(Crystal::new(40, 100, false, false));
        for _ in 0..TRANSITION_FRAMES {
            step_chapter(&mut sim, &chapter, RIGHT);
        }
        assert_eq!(sim.entities, chapter.rooms[1].initial);
    }

    #[test]
    fn buttons_are_still_read_during_a_transition() {
        let chapter = two_rooms();
        let held = Input {
            jump: true,
            ..RIGHT
        };

        // pressed during the transition and held through it: not a new press afterwards
        let mut sim = at_the_start_of_a_transition(&chapter);
        for _ in 0..TRANSITION_FRAMES + 1 {
            step_chapter(&mut sim, &chapter, held);
        }
        assert!(sim.player.speed.y >= 0.0);

        // let go of before it ends: holding it afterwards is a press, so she jumps
        let mut sim = at_the_start_of_a_transition(&chapter);
        step_chapter(&mut sim, &chapter, held);
        for _ in 0..TRANSITION_FRAMES - 1 {
            step_chapter(&mut sim, &chapter, RIGHT);
        }
        step_chapter(&mut sim, &chapter, held);
        assert!(sim.player.speed.y < 0.0);
    }
}
