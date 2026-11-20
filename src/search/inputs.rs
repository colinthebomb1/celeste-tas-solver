use std::collections::HashMap;

use crate::geom::Rect;
use crate::level::room::Chapter;
use crate::player::collision::on_ground;
use crate::player::constants::*;
use crate::player::{Input, State};
use crate::search::candidate_inputs;
use crate::sim::Sim;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Normal,
    Climb,
    DashStart,
    Dash,
}

// everything the rules below read from a state. states that match here get the same inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Situation {
    frozen: bool,
    kind: Kind,
    ground: bool,
    // something solid is close enough to grab, kick off or wallbounce from
    near_wall: bool,
    held_jump: bool,
    held_jump2: bool,
    held_dash: bool,
    held_demodash: bool,
    // a dash press could start a dash this frame, and is sure to
    dash_possible: bool,
    dash_certain: bool,
    crouching: bool,
    falling_fast: bool,
    // holding jump changes nothing this frame
    hold_does_nothing: bool,
    // a jump press has no kind of jump to start
    no_jump_to_start: bool,
    forced_move: bool,
    facing_left: bool,
}

fn after_a_tick(timer: f32) -> f32 {
    if timer > 0.0 {
        timer - DELTA_TIME
    } else {
        timer
    }
}

fn situation(chapter: &Chapter, sim: &Sim) -> Situation {
    let player = &sim.player;
    let room = &chapter.rooms[sim.room];
    let ground = on_ground(player, room, &sim.entities);
    let kind = match player.state {
        State::Normal => Kind::Normal,
        State::Climb => Kind::Climb,
        State::DashStart => Kind::DashStart,
        State::Dash { .. } => Kind::Dash,
    };

    let can_start_dash =
        matches!(kind, Kind::Normal | Kind::Climb) && after_a_tick(player.dash.cooldown) <= 0.0;
    // the ground gives her dash back once the refill cooldown is over
    let refills = ground && player.dash.refill_cooldown <= 0.0;

    // the furthest any wall check reaches from where her standing hitbox could be this frame
    let up = GRAB_REACH_UP + DASH_JUMP_THRU_NUDGE;
    let reach = Rect::new(
        player.position.x - HITBOX_WIDTH / 2 - WALLBOUNCE_REACH,
        player.position.y - HITBOX_HEIGHT - up,
        HITBOX_WIDTH + 2 * WALLBOUNCE_REACH,
        HITBOX_HEIGHT + up,
    );
    let near_wall = room.is_solid(&sim.entities, &reach);

    let coyote = if ground {
        COYOTE_TIME
    } else {
        after_a_tick(player.coyote_timer)
    };
    let jump_hold_over = after_a_tick(player.jump_timer) <= 0.0;
    let no_lift = player.lift.speed == NO_LIFT && player.lift.last_speed == NO_LIFT;
    let past_half_gravity = ground || player.speed.y.abs() >= HALF_GRAVITY_THRESHOLD;

    Situation {
        frozen: player.freeze_timer > 0.0,
        kind,
        ground,
        near_wall,
        held_jump: player.held.jump,
        held_jump2: player.held.jump2,
        held_dash: player.held.dash,
        held_demodash: player.held.demodash,
        dash_possible: can_start_dash && (player.dashes > 0 || refills),
        dash_certain: can_start_dash && player.dashes > 0,
        crouching: player.crouching,
        falling_fast: player.speed.y >= MAX_FALL,
        hold_does_nothing: no_lift && (player.auto_jump || (jump_hold_over && past_half_gravity)),
        no_jump_to_start: coyote <= 0.0 && !near_wall,
        forced_move: player.forced_move_timer > 0.0,
        facing_left: player.facing == -1,
    }
}

// whether no earlier candidate does the same. of two that match, the later in the list is skipped
fn worth_trying(input: Input, s: &Situation) -> bool {
    let normal = s.kind == Kind::Normal;
    let pressed_jump = (input.jump && !s.held_jump) || (input.jump2 && !s.held_jump2);
    let only_holding_jump = input.jump_down() && !pressed_jump;
    let dash_button = input.dash || input.demodash;
    let dash_starts =
        s.dash_possible && ((input.dash && !s.held_dash) || (input.demodash && !s.held_demodash));

    // the second jump button is only for pressing again while the first is held
    if input.jump2 && !s.held_jump && !s.held_jump2 {
        return false;
    }
    // holding jump only does anything in the normal state
    if only_holding_jump && !normal {
        return false;
    }

    if dash_button && !dash_starts {
        return false;
    }
    // a demodash is a dash that crouches her, which crouching or holding down already does
    if input.demodash && !s.held_dash && (s.crouching || input.move_y == 1) {
        return false;
    }

    // up and down are not read during a dash
    if s.kind == Kind::Dash && input.move_y != -1 {
        return false;
    }
    // in the normal state up does nothing, and down only matters in these cases
    let down_matters = s.ground || s.falling_fast || s.near_wall || dash_starts;
    if normal && (input.move_y == 0 || (input.move_y == 1 && !down_matters)) {
        return false;
    }

    if input.grab && !(s.kind == Kind::Climb || s.near_wall) {
        return false;
    }

    // no direction leaves her facing the way she already does
    let repeats_facing = input.move_x == if s.facing_left { 0 } else { 1 };
    // a forced move ignores left and right
    let ignored_direction = s.forced_move && input.move_x != -1;

    if normal && dash_starts && s.dash_certain && !input.grab {
        // the frame a dash begins reads only her facing and whether she crouches
        if input.jump_down() || repeats_facing || ignored_direction {
            return false;
        }
        if input.dash && input.move_y == 1 && (s.crouching || !s.held_demodash) {
            return false;
        }
    } else if normal && !dash_button {
        if only_holding_jump && s.hold_does_nothing {
            return false;
        }
        if pressed_jump && s.hold_does_nothing && !s.ground && s.no_jump_to_start {
            return false;
        }
        if ignored_direction {
            return false;
        }
    } else if matches!(s.kind, Kind::Dash | Kind::DashStart) {
        if pressed_jump && s.no_jump_to_start {
            return false;
        }
        // with no jump out of the dash, left and right only set her facing or her aim
        if !pressed_jump {
            if s.kind == Kind::Dash && (repeats_facing || ignored_direction) {
                return false;
            }
            if s.kind == Kind::DashStart && input.move_y == 0 && repeats_facing {
                return false;
            }
        }
    }

    true
}

// the inputs worth trying from each kind of situation, worked out the first time it is met
pub(crate) struct InputLists {
    all: Vec<Input>,
    by_situation: HashMap<Situation, Vec<Input>>,
}

impl InputLists {
    pub(crate) fn new() -> InputLists {
        InputLists {
            all: candidate_inputs(),
            by_situation: HashMap::new(),
        }
    }

    // every candidate, for checking the rules against
    pub(crate) fn all(&self) -> &[Input] {
        &self.all
    }

    // the candidates that could lead somewhere new from this state
    pub(crate) fn worth_trying(&mut self, chapter: &Chapter, sim: &Sim) -> &[Input] {
        let situation = situation(chapter, sim);
        let all = &self.all;

        self.by_situation.entry(situation).or_insert_with(|| {
            // nothing is read during a freeze but which buttons are down
            if situation.frozen {
                return vec![all[0]];
            }
            all.iter()
                .copied()
                .filter(|&input| worth_trying(input, &situation))
                .collect()
        })
    }
}
