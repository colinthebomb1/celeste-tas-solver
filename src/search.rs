use crate::level::room::Chapter;
use crate::player::Input;
use crate::sim::{Sim, step_chapter};

// how a run of inputs ended, frames counts the inputs used, so the first frame is 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    // a transition into another room began on this frame
    LeftRoom { frames: usize },
    // she touched the finish trigger on this frame
    Finished { frames: usize },
    Died { frames: usize },
    // every input was used and she is still in the room
    RanOut,
}

// plays these inputs from a starting state, one per frame, and stops at the first frame she
// leaves the room, finishes or dies
pub fn play(chapter: &Chapter, start: &Sim, inputs: &[Input]) -> Outcome {
    let mut sim = start.clone();

    for (frame, &input) in (1..).zip(inputs) {
        step_chapter(&mut sim, chapter, input);

        if sim.player.finished {
            return Outcome::Finished { frames: frame };
        }

        if sim.player.dead {
            return Outcome::Died { frames: frame };
        }

        if sim.in_transition() {
            return Outcome::LeftRoom { frames: frame };
        }
    }

    Outcome::RanOut
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::at;
    use crate::geom::{IVec2, Rect};
    use crate::level::room::Room;

    const NONE: Input = Input {
        move_x: 0,
        move_y: 0,
        jump: false,
        jump2: false,
        dash: false,
        grab: false,
        demodash: false,
    };
    const RIGHT: Input = Input { move_x: 1, ..NONE };

    // 88 px wide, with a floor whose top is y = 176
    fn floor_room_at(x: i32) -> Room {
        let tiles = format!("{}11111111111", "00000000000\n".repeat(22));
        Room::from_tiles(IVec2 { x, y: 0 }, &tiles)
    }

    fn two_rooms() -> Chapter {
        Chapter {
            rooms: vec![floor_room_at(0), floor_room_at(88)],
        }
    }

    // she stands on the floor, 8 px from the first room's right edge
    fn near_the_exit(chapter: &Chapter) -> Sim {
        Sim::new(chapter, 0, at(80, 176))
    }

    #[test]
    fn running_out_of_the_room_reports_the_frame_she_left() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        assert_eq!(
            play(&chapter, &start, &[RIGHT; 60]),
            Outcome::LeftRoom { frames: 6 }
        );
    }

    #[test]
    fn it_stops_at_the_exit_even_with_inputs_left_over() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        assert_eq!(
            play(&chapter, &start, &[RIGHT; 60]),
            play(&chapter, &start, &[RIGHT; 600])
        );
    }

    #[test]
    fn too_few_inputs_to_get_out_runs_out() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        assert_eq!(play(&chapter, &start, &[RIGHT; 3]), Outcome::RanOut);
        assert_eq!(play(&chapter, &start, &[NONE; 60]), Outcome::RanOut);
        assert_eq!(play(&chapter, &start, &[]), Outcome::RanOut);
    }

    #[test]
    fn falling_out_of_the_bottom_reports_the_frame_she_died() {
        // a single room with no floor and nothing under it
        let chapter = Chapter {
            rooms: vec![Room::from_tiles(
                IVec2 { x: 0, y: 0 },
                &"00000000000\n".repeat(23),
            )],
        };
        let start = Sim::new(&chapter, 0, at(40, 170));
        assert_eq!(
            play(&chapter, &start, &[NONE; 60]),
            Outcome::Died { frames: 15 }
        );
    }

    #[test]
    fn touching_the_finish_trigger_reports_the_frame_she_finished() {
        let mut chapter = two_rooms();
        chapter.rooms[0].finish_trigger = Some(Rect::new(40, 160, 16, 16));
        let start = Sim::new(&chapter, 0, at(48, 176));
        assert_eq!(
            play(&chapter, &start, &[NONE; 60]),
            Outcome::Finished { frames: 1 }
        );
    }

    #[test]
    fn the_starting_state_is_left_as_it_was() {
        let chapter = two_rooms();
        let start = near_the_exit(&chapter);
        let before = start.clone();

        let first = play(&chapter, &start, &[RIGHT; 60]);
        assert_eq!(start, before);
        assert_eq!(play(&chapter, &start, &[RIGHT; 60]), first);
    }
}
