use super::constants::*;
use crate::geom::{IVec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Normal,
    Climb,
    // the frame a dash picks its direction, after the freeze
    DashStart,
    Dash { timer: f32 },
}

// the moving block she is hopping onto, and where it was when she last looked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HopSolid {
    // its index in the room's solids
    pub id: usize,
    pub position: IVec2,
}

// which buttons were down last frame, to tell a press from a hold
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Held {
    pub jump: bool,
    pub jump2: bool,
    pub dash: bool,
    pub demodash: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Wall {
    pub slide_timer: f32,
    pub slide_dir: i8,
    // wallboost: after a neutral climb jump, pressing this way in time turns it into a wall kick
    pub boost_dir: i8,
    pub boost_timer: f32,
    // x speed she had when she hit a wall, given back if the wall is gone before the timer ends
    pub retained_speed: f32,
    pub retention_timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Climb {
    // the side she is holding on to while sliding down a wall with grab held
    pub trigger_dir: i8,
    // she can't move along the wall for a moment after grabbing it
    pub no_move_timer: f32,
    // -1 = climbing up, 1 = down, 0 = holding still
    pub last_move: i8,
    // after a climbhop she waits to clear the ledge, then gets this sideways speed
    pub hop_wait: i8,
    pub hop_speed: f32,
    pub hop_solid: Option<HopSolid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dash {
    pub cooldown: f32,
    // the ground gives her dash back only after this runs out
    pub refill_cooldown: f32,
    pub dir: Vec2,
    pub speed_before: Vec2,
    pub started_on_ground: bool,
    // runs a little longer than the dash itself
    pub attack_timer: f32,
}

// speed from a block that moved her this frame, and the last one that did, kept for a short while
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Lift {
    pub speed: Vec2,
    pub last_speed: Vec2,
    pub timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub position: IVec2,
    pub subpixel: Vec2,
    pub speed: Vec2,
    // -1 = left, 1 = right
    pub facing: i8,
    pub state: State,
    pub crouching: bool,
    pub dead: bool,
    pub freeze_timer: f32,

    pub dashes: u8,
    pub stamina: f32,
    // her current top fall speed, between MAX_FALL and FASTFALL_SPEED
    pub max_fall: f32,
    pub coyote_timer: f32,
    // while this runs, holding jump keeps her rising at jump_speed
    pub jump_timer: f32,
    pub jump_speed: f32,
    pub auto_jump: bool,

    // last frame's horizontal input
    pub move_x: i8,
    pub was_on_ground: bool,
    // after a wall kick or climbhop, her left and right input is replaced by this for a while
    pub forced_move: i8,
    pub forced_move_timer: f32,

    pub held: Held,
    pub wall: Wall,
    pub climb: Climb,
    pub dash: Dash,
    pub lift: Lift,
}

impl Player {
    pub fn new(position: IVec2, subpixel: Vec2, speed: Vec2) -> Self {
        Player {
            position,
            subpixel,
            speed,
            facing: 1,
            state: State::Normal,
            crouching: false,
            dead: false,
            freeze_timer: 0.0,
            dashes: MAX_DASHES,
            stamina: CLIMB_MAX_STAMINA,
            max_fall: MAX_FALL,
            coyote_timer: 0.0,
            jump_timer: 0.0,
            jump_speed: 0.0,
            auto_jump: false,
            move_x: 0,
            was_on_ground: false,
            forced_move: 0,
            forced_move_timer: 0.0,
            held: Held::default(),
            wall: Wall {
                slide_timer: WALL_SLIDE_TIME,
                ..Wall::default()
            },
            climb: Climb::default(),
            dash: Dash::default(),
            lift: Lift::default(),
        }
    }
}

// one frame of input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Input {
    // -1 = left, 0 = neither, 1 = right
    pub move_x: i8,
    // -1 = up, 0 = neither, 1 = down
    pub move_y: i8,
    pub jump: bool,
    // second jump button
    pub jump2: bool,
    pub dash: bool,
    pub grab: bool,
    pub demodash: bool,
}

impl Input {
    pub fn jump_down(self) -> bool {
        self.jump || self.jump2
    }
}

// what holds for the frame, the buttons, and whether she started it on the ground
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Frame {
    pub input: Input,
    // where a dash would go
    pub aim: Vec2,
    pub ground: bool,
    pub jump_pressed: bool,
    pub dash_pressed: bool,
    pub demodash_pressed: bool,
}

pub(crate) fn jump_pressed(player: &Player, input: Input) -> bool {
    (!player.held.jump && input.jump) || (!player.held.jump2 && input.jump2)
}

// coming from another state resets her top fall speed
pub(crate) fn set_normal(player: &mut Player) {
    if player.state != State::Normal {
        player.max_fall = MAX_FALL;
    }
    player.state = State::Normal;
}

pub fn track_buttons(player: &mut Player, input: Input) {
    player.held.jump = input.jump;
    player.held.jump2 = input.jump2;
    player.held.dash = input.dash;
    player.held.demodash = input.demodash;
}
