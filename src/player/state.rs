use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Climb,
    DashStart,
    Dashing { timer: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovingSolid {
    ZipMover,
    FallingBlock,
}

// the moving block she is hopping onto, and where it was when she last looked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HopSolid {
    pub kind: MovingSolid,
    pub index: usize,
    pub position: IVec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerState {
    pub position: IVec2,
    // sub pixel
    pub remainder: Vec2,
    pub speed: Vec2,
    pub jump_grace_timer: f32,
    pub var_jump_timer: f32,
    pub var_jump_speed: f32,
    pub jump_held: bool,
    pub jump2_held: bool,
    // -1 = left, 1 = right
    pub facing: i8,
    pub wall_slide_timer: f32,
    pub wall_slide_dir: i8,
    // the side she is holding on to while sliding down a wall with grab held
    pub climb_trigger_dir: i8,
    // she can't move along the wall for a moment after grabbing it
    pub climb_no_move_timer: f32,
    // -1 = climbing up, 1 = down, 0 = holding still
    pub last_climb_move: i8,
    // after a climb hop she waits to clear the ledge, then gets this sideways speed
    pub hop_wait_x: i8,
    pub hop_wait_x_speed: f32,
    pub climb_hop_solid: Option<HopSolid>,
    // her current top fall speed, between MAX_FALL and FAST_MAX_FALL
    pub max_fall: f32,
    pub stamina: f32,
    pub ducking: bool,
    // last frame's horizontal input
    pub move_x: i8,
    // after a neutral climb jump, pressing this way in time turns it into a wall jump
    pub wall_boost_dir: i8,
    pub wall_boost_timer: f32,
    pub dash_attack_timer: f32,
    // from a block that moved her this frame, and the last one that did, kept for a short while
    pub lift_speed: Vec2,
    pub last_lift_speed: Vec2,
    pub lift_speed_timer: f32,
    pub was_on_ground: bool,
    // after a wall jump, her left and right input is replaced by this for a while
    pub force_move_x: i8,
    pub force_move_x_timer: f32,
    // x speed she had when she hit a wall, given back if the wall is gone before the timer ends
    pub wall_speed_retained: f32,
    pub wall_speed_retention_timer: f32,
    pub mode: Mode,
    pub dashes: u8,
    pub dash_cooldown_timer: f32,
    pub dash_refill_cooldown_timer: f32,
    pub dash_dir: Vec2,
    pub before_dash_speed: Vec2,
    pub dash_started_on_ground: bool,
    pub auto_jump: bool,
    pub dash_held: bool,
    pub crouch_dash_held: bool,
    pub freeze_timer: f32,
    pub dead: bool,
}

impl PlayerState {
    pub fn new(position: IVec2, remainder: Vec2, speed: Vec2) -> Self {
        PlayerState {
            position,
            remainder,
            speed,
            jump_grace_timer: 0.0,
            var_jump_timer: 0.0,
            var_jump_speed: 0.0,
            jump_held: false,
            jump2_held: false,
            facing: 1,
            wall_slide_timer: WALL_SLIDE_TIME,
            wall_slide_dir: 0,
            climb_trigger_dir: 0,
            climb_no_move_timer: 0.0,
            last_climb_move: 0,
            hop_wait_x: 0,
            hop_wait_x_speed: 0.0,
            climb_hop_solid: None,
            max_fall: MAX_FALL,
            stamina: CLIMB_MAX_STAMINA,
            ducking: false,
            move_x: 0,
            wall_boost_dir: 0,
            wall_boost_timer: 0.0,
            dash_attack_timer: 0.0,
            lift_speed: Vec2 { x: 0.0, y: 0.0 },
            last_lift_speed: Vec2 { x: 0.0, y: 0.0 },
            lift_speed_timer: 0.0,
            was_on_ground: false,
            force_move_x: 0,
            force_move_x_timer: 0.0,
            wall_speed_retained: 0.0,
            wall_speed_retention_timer: 0.0,
            mode: Mode::Normal,
            dashes: MAX_DASHES,
            dash_cooldown_timer: 0.0,
            dash_refill_cooldown_timer: 0.0,
            dash_dir: Vec2 { x: 0.0, y: 0.0 },
            dash_started_on_ground: false,
            before_dash_speed: Vec2 { x: 0.0, y: 0.0 },
            auto_jump: false,
            dash_held: false,
            crouch_dash_held: false,
            freeze_timer: 0.0,
            dead: false,
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
    pub crouch_dash: bool,
}

impl Input {
    pub fn jump_down(self) -> bool {
        self.jump || self.jump2
    }
}

pub fn jump_pressed(state: &PlayerState, input: Input) -> bool {
    (!state.jump_held && input.jump) || (!state.jump2_held && input.jump2)
}

// coming from another state resets her top fall speed
pub fn set_normal(state: &mut PlayerState) {
    if state.mode != Mode::Normal {
        state.max_fall = MAX_FALL;
    }
    state.mode = Mode::Normal;
}

pub fn track_buttons(state: &mut PlayerState, input: Input) {
    state.jump_held = input.jump;
    state.jump2_held = input.jump2;
    state.dash_held = input.dash;
    state.crouch_dash_held = input.crouch_dash;
}
