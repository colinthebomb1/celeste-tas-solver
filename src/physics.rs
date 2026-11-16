use crate::room::{Direction, Rect, Room, Spikes};
use crate::Vec2;

pub const MAX_RUN: f32 = 90.0;
pub const RUN_ACCEL: f32 = 1000.0;
pub const RUN_REDUCE: f32 = 400.0;
pub const DELTA_TIME: f32 = 0.0166667;
pub const GROUND_MULT: f32 = 1.0;
pub const AIR_MULT: f32 = 0.65;
pub const GRAVITY: f32 = 900.0;
pub const MAX_FALL: f32 = 160.0;
pub const HALF_GRAV_THRESHOLD: f32 = 40.0;
pub const JUMP_SPEED: f32 = -105.0;
pub const JUMP_H_BOOST: f32 = 40.0;
pub const VAR_JUMP_TIME: f32 = 0.2;
pub const JUMP_GRACE_TIME: f32 = 0.1;
pub const WALL_SLIDE_START_MAX: f32 = 20.0;
pub const WALL_SLIDE_TIME: f32 = 1.2;
pub const DASH_SPEED: f32 = 240.0;
pub const END_DASH_SPEED: f32 = 160.0;
pub const END_DASH_UP_MULT: f32 = 0.75;
pub const DASH_TIME: f32 = 0.15;
pub const DASH_COOLDOWN: f32 = 0.2;
pub const DASH_REFILL_COOLDOWN: f32 = 0.1;
pub const DASH_FREEZE_TIME: f32 = 0.05;
pub const MAX_DASHES: u8 = 1;
pub const SUPER_JUMP_H: f32 = 260.0;
pub const CLIMB_MAX_STAMINA: f32 = 110.0;
pub const CLIMB_JUMP_COST: f32 = 110.0 / 4.0;
pub const WALL_JUMP_CHECK_DIST: i32 = 3;
pub const WALL_SPEED_RETENTION_TIME: f32 = 0.06;
pub const WALL_JUMP_H_SPEED: f32 = MAX_RUN + JUMP_H_BOOST;
pub const CLIMB_JUMP_BOOST_TIME: f32 = 0.2;
pub const UPWARD_CORNER_CORRECTION: i32 = 4;
pub const DASH_CORNER_CORRECTION: i32 = 4;
pub const DASH_ATTACK_TIME: f32 = 0.3;
pub const SUPER_WALL_JUMP_H: f32 = MAX_RUN + JUMP_H_BOOST * 2.0;
pub const SUPER_WALL_JUMP_SPEED: f32 = -160.0;
pub const SUPER_WALL_JUMP_VAR_TIME: f32 = 0.25;
pub const SUPER_WALL_JUMP_CHECK_DIST: i32 = 5;
pub const CEILING_VAR_JUMP_GRACE: f32 = 0.05;
// normal hitbox size in pixels for collisions
pub const HITBOX_WIDTH: i32 = 8;
pub const HITBOX_HEIGHT: i32 = 11;
// hazards use the hurtbox
pub const HURTBOX_HEIGHT: i32 = 9;
// both boxes are shorter when ducking
pub const DUCK_HITBOX_HEIGHT: i32 = 6;
pub const DUCK_HURTBOX_HEIGHT: i32 = 4;
pub const DUCK_FRICTION: f32 = 500.0;
pub const DODGE_SLIDE_SPEED_MULT: f32 = 1.2;
pub const DUCK_SUPER_JUMP_X_MULT: f32 = 1.25;
pub const DUCK_SUPER_JUMP_Y_MULT: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    DashStart,
    Dashing { timer: f32 },
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
    pub stamina: f32,
    pub ducking: bool,
    // last frame's horizontal input
    pub move_x: i8,
    // after a neutral climb jump, pressing this way in time turns it into a wall jump
    pub wall_boost_dir: i8,
    pub wall_boost_timer: f32,
    pub dash_attack_timer: f32,
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
            stamina: CLIMB_MAX_STAMINA,
            ducking: false,
            move_x: 0,
            wall_boost_dir: 0,
            wall_boost_timer: 0.0,
            dash_attack_timer: 0.0,
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
}

impl Input {

    pub fn jump_down(self) -> bool {
        self.jump || self.jump2
    }
}

pub fn jump_pressed(state: &PlayerState, input: Input) -> bool {
    (!state.jump_held && input.jump) || (!state.jump2_held && input.jump2)
}

pub fn collide_at(room: &Room, x: i32, y: i32) -> bool {
    room.collide_rect(x - HITBOX_WIDTH / 2, y - HITBOX_HEIGHT, HITBOX_WIDTH, HITBOX_HEIGHT)
}

pub fn hitbox_height(state: &PlayerState) -> i32 {
    if state.ducking { DUCK_HITBOX_HEIGHT } else { HITBOX_HEIGHT }
}

pub fn collides(state: &PlayerState, room: &Room, dx: i32, dy: i32) -> bool {
    let height = hitbox_height(state);

    room.collide_rect(
        state.position.x + dx - HITBOX_WIDTH / 2,
        state.position.y + dy - height,
        HITBOX_WIDTH,
        height,
    )
}

pub fn can_unduck(state: &PlayerState, room: &Room) -> bool {
    !state.ducking || !collide_at(room, state.position.x, state.position.y)
}

pub fn hurtbox(state: &PlayerState) -> Rect {
    let (top, height) = if state.ducking {
        (state.position.y - DUCK_HITBOX_HEIGHT, DUCK_HURTBOX_HEIGHT)
    } else {
        (state.position.y - HITBOX_HEIGHT, HURTBOX_HEIGHT)
    };

    Rect {
        left: state.position.x - HITBOX_WIDTH / 2,
        top,
        width: HITBOX_WIDTH,
        height,
    }
}

pub fn spikes_kill(state: &PlayerState, spikes: &Spikes) -> bool {
    let madeline_hurtbox = &hurtbox(state);

    if !spikes.hitbox.overlaps(madeline_hurtbox) {
        return false;
    }

    match spikes.direction {
        Direction::Up => state.speed.y >= 0.0 && madeline_hurtbox.top + madeline_hurtbox.height <= spikes.hitbox.top + spikes.hitbox.height,
        Direction::Down => state.speed.y <= 0.0,
        Direction::Left => state.speed.x >= 0.0,
        Direction::Right => state.speed.x <= 0.0
    }
}

// top of hitbox is more than 4 px below the room's bottom edge
pub fn fell_out(state: &PlayerState, room: &Room) -> bool {
    state.position.y - HITBOX_HEIGHT > room.bottom() + 4
}

pub fn on_ground(state: &PlayerState, room: &Room) -> bool {
    state.speed.y >= 0.0 && collides(state, room, 0, 1)
}

pub fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value > target {
        (value - max_move).max(target)
    } else {
        (value + max_move).min(target)
    }
}

pub fn duck_free_at(state: &PlayerState, room: &Room, dx: i32) -> bool {
    !room.collide_rect(
        state.position.x + dx - HITBOX_WIDTH / 2,
        state.position.y - DUCK_HITBOX_HEIGHT,
        HITBOX_WIDTH,
        DUCK_HITBOX_HEIGHT,
    )
}

fn dashing(state: &PlayerState) -> bool {
    matches!(state.mode, Mode::DashStart | Mode::Dashing { .. })
}

pub fn dash_corner_correction_h(state: &mut PlayerState, room: &Room, ground: bool) -> bool {
    if !dashing(state) {
        return false;
    }

    let dir = sign(state.speed.x) as i32;

    if ground && duck_free_at(state, room, dir) {
        state.ducking = true;
        return true;
    }

    if state.speed.y == 0.0 && state.speed.x != 0.0 {
        for i in 1..=DASH_CORNER_CORRECTION {
            for dy in [i, -i] {
                if !collides(state, room, dir, dy) {
                    for _ in 0..i {
                        if collides(state, room, 0, dy.signum()) {
                            break;
                        }
                        state.position.y += dy.signum();
                    }
                    if !collides(state, room, dir, 0) {
                        state.position.x += dir;
                    }
                    return true;
                }
            }
        }
    }

    false
}

// ground is whether she was on the ground at the start of the frame
pub fn move_h(state: &mut PlayerState, room: &Room, amount: f32, ground: bool) {
    let new_remainder = state.remainder.x + amount;
    // C# Math.Round
    let movement_x = new_remainder.round_ties_even() as i32;
    state.remainder.x = new_remainder - (movement_x as f32);

    let step = movement_x.signum();
    for _ in 0..movement_x.abs() {
        if collides(state, room, step, 0) {
            if dash_corner_correction_h(state, room, ground) {
                state.remainder.x = 0.0;
                return;
            }

            if state.wall_speed_retention_timer <= 0.0 {
                state.wall_speed_retained = state.speed.x;
                state.wall_speed_retention_timer = WALL_SPEED_RETENTION_TIME;
            }

            state.remainder.x = 0.0;
            state.speed.x = 0.0;
            state.dash_attack_timer = 0.0;
            return;
        }
        state.position.x += step;
    }
}

pub fn move_v(state: &mut PlayerState, room: &Room, amount: f32) {
    let new_remainder = state.remainder.y + amount;
    // C# Math.Round
    let movement_y = new_remainder.round_ties_even() as i32;
    state.remainder.y = new_remainder - (movement_y as f32);

    let step = movement_y.signum();
    for _ in 0..movement_y.abs() {
        if collides(state, room, 0, step) {
            state.remainder.y = 0.0;

            if state.speed.y < 0.0 {
                if upward_corner_correction(state, room) {
                    return;
                }

                if state.var_jump_timer < VAR_JUMP_TIME - CEILING_VAR_JUMP_GRACE {
                    state.var_jump_timer = 0.0;
                }
            } else if state.speed.y > 0.0 {
                if dash_corner_correction_down(state, room) {
                    return;
                }

                // dash slide
                if state.dash_dir.x != 0.0 && state.dash_dir.y > 0.0 {
                    state.dash_dir = Vec2 { x: sign(state.dash_dir.x), y: 0.0 };
                    state.speed.x *= DODGE_SLIDE_SPEED_MULT;
                    state.ducking = true;
                }
            }

            state.speed.y = 0.0;
            state.dash_attack_timer = 0.0;
            return;
        }
        state.position.y += step;
    }
}

pub fn dash_corner_correction_down(state: &mut PlayerState, room: &Room) -> bool {
    if !dashing(state) || state.dash_started_on_ground {
        return false;
    }

    let mut dirs = Vec::new();
    if state.speed.x <= 0.0 {
        dirs.push(-1);
    }
    if state.speed.x >= 0.0 {
        dirs.push(1);
    }

    for dir in dirs {
        for i in 1..=DASH_CORNER_CORRECTION {
            if !collides(state, room, dir * i, 1) {
                for _ in 0..i {
                    if collides(state, room, dir, 0) {
                        break;
                    }
                    state.position.x += dir;
                }
                if !collides(state, room, 0, 1) {
                    state.position.y += 1;
                }
                return true;
            }
        }
    }

    false
}

pub fn upward_corner_correction(state: &mut PlayerState, room: &Room) -> bool {
    if state.speed.x <= 0.0 {
        for i in 1..=UPWARD_CORNER_CORRECTION {
            if !collides(state, room, -i, -1) {
                state.position.x -= i;
                state.position.y -= 1;
                return true;
            }
        }
    }

    if state.speed.x >= 0.0 {
        for i in 1..=UPWARD_CORNER_CORRECTION {
            if !collides(state, room, i, -1) {
                state.position.x += i;
                state.position.y -= 1;
                return true;
            }
        }
    }

    false
}

pub fn wall_at(state: &PlayerState, room: &Room, dir: i8) -> bool {
    collides(state, room, dir as i32, 0)
}

pub fn lerp(value1: f32, value2: f32, amount: f32) -> f32 {
    value1 * (1.0 - amount) + value2 * amount
}


pub fn wall_slide_max_fall(state: &mut PlayerState, room: &Room, input: Input) -> f32 {
    let mut max = MAX_FALL;
    
    if input.move_x == state.facing {
        if state.speed.y >= 0.0 && state.wall_slide_timer > 0.0 && wall_at(state, room, state.facing) {
            state.wall_slide_dir = state.facing;
        }

        if state.wall_slide_dir != 0 {
            max = lerp(MAX_FALL, WALL_SLIDE_START_MAX, state.wall_slide_timer / WALL_SLIDE_TIME)
        }
    }

    max
}

pub fn jump(state: &mut PlayerState, input: Input) {
    state.jump_grace_timer = 0.0;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.var_jump_timer = VAR_JUMP_TIME;
    state.speed.x += JUMP_H_BOOST * input.move_x as f32;
    state.speed.y = JUMP_SPEED;
    state.var_jump_speed = state.speed.y;
    state.auto_jump = false;
    state.wall_boost_timer = 0.0;
    state.dash_attack_timer = 0.0;
}

pub fn aim_vector(input: Input, facing: i8) -> Vec2 {
    if input.move_x == 0 && input.move_y == 0 {
        Vec2 { x: facing as f32, y: 0.0 }
    } else if input.move_x != 0 && input.move_y != 0 { // diagonal
        let d = 1.0 / 2f32.sqrt();
        Vec2 { x: input.move_x as f32 * d, y: input.move_y as f32 * d }
    } else {
        Vec2 { x: input.move_x as f32, y: input.move_y as f32 }
    }
}

pub fn can_super_jump(state: &PlayerState, room: &Room, jump_pressed: bool) -> bool {
    jump_pressed && state.jump_grace_timer > 0.0 && state.dash_dir.y == 0.0 && can_unduck(state, room)
}

pub fn super_jump(state: &mut PlayerState) {
    state.jump_grace_timer = 0.0;
    state.var_jump_timer = VAR_JUMP_TIME;
    state.auto_jump = false;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.wall_boost_timer = 0.0;
    state.dash_attack_timer = 0.0;
    state.speed.x = SUPER_JUMP_H * state.facing as f32;
    state.speed.y = JUMP_SPEED;

    // hyper
    if state.ducking {
        state.ducking = false;
        state.speed.x *= DUCK_SUPER_JUMP_X_MULT;
        state.speed.y *= DUCK_SUPER_JUMP_Y_MULT;
    }

    state.var_jump_speed = state.speed.y;
    state.mode = Mode::Normal;
}

pub fn wall_bounce_ready(state: &PlayerState) -> bool {
    state.dash_attack_timer > 0.0 && state.dash_dir.x == 0.0 && state.dash_dir.y == -1.0
}

pub fn wall_jump_check(state: &PlayerState, room: &Room, dir: i8) -> bool {
    let dir = dir as i32;
    let mut dist = WALL_JUMP_CHECK_DIST;

    if wall_bounce_ready(state) {
        let facing_her = if dir > 0 { Direction::Left } else { Direction::Right };
        let reach = Rect {
            left: state.position.x + dir * SUPER_WALL_JUMP_CHECK_DIST - HITBOX_WIDTH / 2,
            top: state.position.y - hitbox_height(state),
            width: HITBOX_WIDTH,
            height: hitbox_height(state),
        };
        let spiked = room.spikes.iter().any(|s| s.direction == facing_her && s.hitbox.overlaps(&reach));

        if !spiked {
            dist = SUPER_WALL_JUMP_CHECK_DIST;
        }
    }

    collides(state, room, dir * dist, 0)
}

pub fn super_wall_jump(state: &mut PlayerState, dir: i8) {
    state.ducking = false;
    state.jump_grace_timer = 0.0;
    state.var_jump_timer = SUPER_WALL_JUMP_VAR_TIME;
    state.auto_jump = false;
    state.dash_attack_timer = 0.0;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.wall_boost_timer = 0.0;

    state.speed.x = SUPER_WALL_JUMP_H * dir as f32;
    state.speed.y = SUPER_WALL_JUMP_SPEED;
    state.var_jump_speed = state.speed.y;
    state.mode = Mode::Normal;
}

// a jump press with no coyote time, next to a wall (the right one is checked first).
// true if she jumped. plain wall jumps aren't in yet
pub fn jump_off_wall(state: &mut PlayerState, room: &Room, input: Input, ground: bool) -> bool {
    if !can_unduck(state, room) {
        return false;
    }

    let wall = if wall_jump_check(state, room, 1) {
        1
    } else if wall_jump_check(state, room, -1) {
        -1
    } else {
        return false;
    };

    // during a dash straight up, a jump is always a wall bounce
    let dashing_up = dashing(state) && state.dash_dir.x == 0.0 && state.dash_dir.y == -1.0;

    if dashing_up {
        super_wall_jump(state, -wall);
    } else if can_climb_jump(state, room, input) {
        climb_jump(state, input, ground);
    } else if wall_bounce_ready(state) {
        super_wall_jump(state, -wall);
    } else {
        return false;
    }

    true
}

pub fn can_climb_jump(state: &PlayerState, room: &Room, input: Input) -> bool {
    let able = input.grab && state.stamina > 0.0;

    if wall_jump_check(state, room, 1) {
        state.facing == 1 && able
    } else if wall_jump_check(state, room, -1) {
        state.facing == -1 && able
    } else {
        false
    }
}

pub fn climb_jump(state: &mut PlayerState, input: Input, ground: bool) {
    if !ground {
        state.stamina -= CLIMB_JUMP_COST;
    }

    jump(state, input);

    if input.move_x == 0 {
        state.wall_boost_dir = -state.facing;
        state.wall_boost_timer = CLIMB_JUMP_BOOST_TIME;
    }

    state.mode = Mode::Normal;
}

pub fn update_wall_speed_retention(state: &mut PlayerState, room: &Room) {
    if state.wall_speed_retention_timer <= 0.0 {
        return;
    }

    let dir = sign(state.wall_speed_retained);

    if sign(state.speed.x) == -dir {
        state.wall_speed_retention_timer = 0.0;
    } else if !collides(state, room, dir as i32, 0) {
        state.speed.x = state.wall_speed_retained;
        state.wall_speed_retention_timer = 0.0;
    } else {
        state.wall_speed_retention_timer -= DELTA_TIME;
    }
}

pub fn can_dash(state: &PlayerState, dash_pressed: bool) -> bool {
    dash_pressed && state.dash_cooldown_timer <= 0.0 && state.dashes > 0
}

pub fn dash_begin(state: &mut PlayerState, room: &Room, input: Input, ground: bool) {
    state.dashes -= 1;
    state.freeze_timer = DASH_FREEZE_TIME;
    state.dash_cooldown_timer = DASH_COOLDOWN;
    state.dash_refill_cooldown_timer = DASH_REFILL_COOLDOWN;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.before_dash_speed = state.speed;
    state.dash_attack_timer = DASH_ATTACK_TIME;
    state.dash_started_on_ground = ground;
    state.speed = Vec2 { x: 0.0, y: 0.0 };
    state.dash_dir = Vec2 { x: 0.0, y: 0.0 };

    // holding down at the start of a dash ducks
    if !ground && state.ducking && can_unduck(state, room) {
        state.ducking = false;
    } else if !state.ducking && input.move_y == 1 {
        state.ducking = true;
    }

    state.mode = Mode::DashStart
}

// C# Math.Sign
fn sign(v: f32) -> f32 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

pub fn dash_launch(state: &mut PlayerState, aim: Vec2, ground: bool) {
    let mut new_speed_x = aim.x * DASH_SPEED;
    let new_speed_y = aim.y * DASH_SPEED;

    if sign(state.before_dash_speed.x) == sign(new_speed_x) && state.before_dash_speed.x.abs() > new_speed_x.abs() {
        new_speed_x = state.before_dash_speed.x;
    }

    state.speed = Vec2 { x: new_speed_x, y: new_speed_y };
    state.dash_dir = aim;

    if aim.x != 0.0 {
        state.facing = sign(aim.x) as i8;
    }

    // dash slide - a down diagonal dash on the ground becomes a ducked horizontal one
    if ground && state.dash_dir.x != 0.0 && state.dash_dir.y > 0.0 && state.speed.y > 0.0 {
        state.dash_dir = Vec2 { x: sign(state.dash_dir.x), y: 0.0 };
        state.speed.y = 0.0;
        state.speed.x *= DODGE_SLIDE_SPEED_MULT;
        state.ducking = true;
    }

    state.mode = Mode::Dashing { timer: DASH_TIME };
}

pub fn dash_end(state: &mut PlayerState) {
    state.auto_jump = true;
    
    if state.dash_dir.y <= 0.0 {
        state.speed.x = state.dash_dir.x * END_DASH_SPEED;
        state.speed.y = state.dash_dir.y * END_DASH_SPEED;
    }

    if state.speed.y < 0.0 {
        state.speed.y *= END_DASH_UP_MULT;
    }

    state.mode = Mode::Normal;
}

pub fn step(mut state: PlayerState, input: Input, room: &Room) -> PlayerState {
    if state.dead {
        return state;
    }

    let jump_pressed = jump_pressed(&state, input);
    state.jump_held = input.jump;
    state.jump2_held = input.jump2;

    let dash_pressed = input.dash && !state.dash_held;
    state.dash_held = input.dash;

    if state.freeze_timer > 0.0 {
        state.freeze_timer = (state.freeze_timer - DELTA_TIME).max(0.0);
        return state;
    }

    let ground = on_ground(&state, room);

    if state.wall_slide_dir != 0 {
        state.wall_slide_timer = (state.wall_slide_timer - DELTA_TIME).max(0.0);
        state.wall_slide_dir = 0;
    }

    // wall boost
    if state.wall_boost_timer > 0.0 {
        state.wall_boost_timer -= DELTA_TIME;

        if state.move_x == state.wall_boost_dir {
            state.speed.x = WALL_JUMP_H_SPEED * state.move_x as f32;
            state.stamina += CLIMB_JUMP_COST;
            state.wall_boost_timer = 0.0;
        }
    }

    if ground {
        state.jump_grace_timer = JUMP_GRACE_TIME;
        state.wall_slide_timer = WALL_SLIDE_TIME;
        state.stamina = CLIMB_MAX_STAMINA;
        state.auto_jump = false;
    } else if state.jump_grace_timer > 0.0 {
        state.jump_grace_timer -= DELTA_TIME;
    }

    if state.dash_attack_timer > 0.0 {
        state.dash_attack_timer -= DELTA_TIME;
    }

    if state.dash_cooldown_timer > 0.0 {
        state.dash_cooldown_timer -= DELTA_TIME;
    }

    if state.dash_refill_cooldown_timer > 0.0 {
        state.dash_refill_cooldown_timer -= DELTA_TIME;
    } else if ground {
        state.dashes = MAX_DASHES;
    }

    state.move_x = input.move_x;

    if input.move_x != 0 {
        state.facing = input.move_x;
    }
    
    let aim = aim_vector(input, state.facing);

    update_wall_speed_retention(&mut state, room);

    if state.var_jump_timer > 0.0 {
        state.var_jump_timer -= DELTA_TIME;
    }

    match state.mode {
        Mode::Normal => {
            if can_dash(&state, dash_pressed) {
                dash_begin(&mut state, room, input, ground);
            } else {
                normal_update(&mut state, room, input, ground, jump_pressed);
            }
        }

        Mode::DashStart => {
            if can_super_jump(&state, room, jump_pressed) {
                super_jump(&mut state);
            } else if jump_pressed && jump_off_wall(&mut state, room, input, ground) {
                // jumped out of the dash
            } else {
                dash_launch(&mut state, aim, ground);
            }
        }
        
        Mode::Dashing { timer } => {
            if can_super_jump(&state, room, jump_pressed) {
                super_jump(&mut state);
            } else if jump_pressed && jump_off_wall(&mut state, room, input, ground) {
                // jumped out of the dash
            } else if timer > 0.0 {
                state.mode = Mode::Dashing { timer: timer - DELTA_TIME };
            } else {
                dash_end(&mut state);
            }
        }
    }

    if state.speed.y > 0.0 && !ground && can_unduck(&state, room) {
        state.ducking = false;
    }

    let (speed_x, speed_y) = (state.speed.x, state.speed.y);
    move_h(&mut state, room, speed_x * DELTA_TIME, ground);
    move_v(&mut state, room, speed_y * DELTA_TIME);

    if room.spikes.iter().any(|s| spikes_kill(&state, s)) || fell_out(&state, room) {
        state.speed = Vec2 { x: 0.0, y: 0.0 };
        state.dead = true;
    }

    state
}

fn normal_update(state: &mut PlayerState, room: &Room, input: Input, ground: bool, jump_pressed: bool) {
    if state.ducking {
        if ground && input.move_y != 1 && can_unduck(state, room) {
            state.ducking = false;
        }
    } else if ground && input.move_y == 1 && state.speed.y >= 0.0 {
        state.ducking = true;
    }

    let mult = if ground {
        GROUND_MULT
    } else {
        AIR_MULT
    };
    let move_x = input.move_x as f32;
    let target = MAX_RUN * move_x;

    let max_move_x = if state.speed.x.abs() > MAX_RUN && (state.speed.x.signum() as i8) == input.move_x {
        RUN_REDUCE * DELTA_TIME * mult
    } else {
        RUN_ACCEL * DELTA_TIME * mult
    };
    if state.ducking && ground {
        state.speed.x = approach(state.speed.x, 0.0, DUCK_FRICTION * DELTA_TIME);
    } else {
        state.speed.x = approach(state.speed.x, target, max_move_x);
    }

    if !ground {
        let max_fall = wall_slide_max_fall(state, room, input);
        let grav_mult = if state.speed.y.abs() < HALF_GRAV_THRESHOLD && (input.jump_down() || state.auto_jump) {
            0.5
        } else {
            1.0
        };
        state.speed.y = approach(state.speed.y, max_fall, GRAVITY * grav_mult * DELTA_TIME);
    }

    if state.var_jump_timer > 0.0 {
        if input.jump_down() || state.auto_jump {
            state.speed.y = state.speed.y.min(state.var_jump_speed);
        } else {
            state.var_jump_timer = 0.0;
        }
    }

    if jump_pressed {
        if state.jump_grace_timer > 0.0 {
            jump(state, input);
        } else {
            jump_off_wall(state, room, input, ground);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Direction;

    const FLOOR_Y: i32 = 144;
    const LEDGE_X: i32 = 43;
    const WALL_X: i32 = 76;

    fn room() -> Room {
        let text = "00000000001\n".repeat(18) + &"11111000001\n".repeat(5);
        Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
    }

    fn at_rest() -> PlayerState {
        PlayerState::new(IVec2 { x: 0, y: 64 }, Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 0.0, y: 0.0 })
    }

    fn standing() -> PlayerState {
        PlayerState::new(IVec2 { x: 0, y: FLOOR_Y }, Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 0.0, y: 0.0 })
    }

    #[test]
    fn approach_moves_toward_target() {
        assert_eq!(approach(0.0, 10.0, 3.0), 3.0);
        assert_eq!(approach(10.0, 0.0, 3.0), 7.0);
    }

    #[test]
    fn approach_does_not_overshoot() {
        assert_eq!(approach(9.0, 10.0, 3.0), 10.0);
        assert_eq!(approach(1.0, 0.0, 3.0), 0.0);
        assert_eq!(approach(-1.0, 0.0, 3.0), 0.0);
    }

    #[test]
    fn approach_at_target_stays() {
        assert_eq!(approach(5.0, 5.0, 3.0), 5.0);
    }

    #[test]
    fn move_h_small_amount_only_changes_remainder() {
        let mut s = at_rest();
        move_h(&mut s, &room(), 0.25, false);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.remainder.x, 0.25);
    }

    #[test]
    fn move_h_rounds_to_whole_pixels() {
        let mut s = at_rest();
        move_h(&mut s, &room(), 0.75, false);
        assert_eq!(s.position.x, 1);
        assert_eq!(s.remainder.x, -0.25);

        move_h(&mut s, &room(), -2.0, false);
        assert_eq!(s.position.x, -1);
        assert_eq!(s.remainder.x, -0.25);
    }

    #[test]
    fn move_h_ties_round_to_even_like_csharp() {
        let mut s = at_rest();
        move_h(&mut s, &room(), 0.5, false);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.remainder.x, 0.5);

        move_h(&mut s, &room(), 1.0, false);
        assert_eq!(s.position.x, 2);
        assert_eq!(s.remainder.x, -0.5);
    }

    #[test]
    fn collide_at_uses_her_hitbox() {
        let room = room();
        assert!(!collide_at(&room, 0, FLOOR_Y));
        assert!(collide_at(&room, 0, FLOOR_Y + 1));
        assert!(collide_at(&room, LEDGE_X, FLOOR_Y + 1));
        assert!(!collide_at(&room, LEDGE_X + 1, FLOOR_Y + 1));
        assert!(!collide_at(&room, WALL_X, 64));
        assert!(collide_at(&room, WALL_X + 1, 64));
        assert!(!collide_at(&room, 60, FLOOR_Y + 12));
    }

    #[test]
    fn on_ground_when_standing_on_floor() {
        assert!(on_ground(&standing(), &room()));
    }

    #[test]
    fn not_on_ground_above_floor() {
        let mut s = standing();
        s.position.y -= 1;
        assert!(!on_ground(&s, &room()));
    }

    #[test]
    fn not_on_ground_while_moving_up() {
        let mut s = standing();
        s.speed.y = -10.0;
        assert!(!on_ground(&s, &room()));
    }

    #[test]
    fn move_v_moves_up_freely() {
        let mut s = standing();
        move_v(&mut s, &room(), -1.75);
        assert_eq!(s.position.y, FLOOR_Y - 2);
        assert_eq!(s.remainder.y, 0.25);
    }

    #[test]
    fn move_v_stops_at_floor() {
        let mut s = standing();
        s.position.y = FLOOR_Y - 2;
        s.remainder.y = 0.3;
        s.speed.y = 160.0;
        move_v(&mut s, &room(), 2.6);
        assert_eq!(s.position.y, FLOOR_Y);
        assert_eq!(s.remainder.y, 0.0);
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn jump_sets_speed_and_timers() {
        let mut s = standing();
        s.jump_grace_timer = JUMP_GRACE_TIME;
        jump(&mut s, Input { move_x: 0, jump: true, ..Default::default() });
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.var_jump_speed, JUMP_SPEED);
        assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
        assert_eq!(s.jump_grace_timer, 0.0);
    }

    #[test]
    fn jump_resets_wall_slide_timer() {
        let mut s = standing();
        s.wall_slide_timer = 0.3;
        jump(&mut s, Input { move_x: 0, jump: true, ..Default::default() });
        assert_eq!(s.wall_slide_timer, WALL_SLIDE_TIME);
    }

    #[test]
    fn move_h_stops_at_wall() {
        let mut s = at_rest();
        s.position.x = WALL_X - 1;
        s.remainder.x = 0.3;
        s.speed.x = 90.0;
        move_h(&mut s, &room(), 1.5, false);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(s.remainder.x, 0.0);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn move_h_reaching_wall_exactly_is_not_a_collision() {
        let mut s = at_rest();
        s.position.x = WALL_X - 1;
        s.speed.x = 90.0;
        move_h(&mut s, &room(), 1.25, false);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(s.remainder.x, 0.25);
        assert_eq!(s.speed.x, 90.0);
    }

    #[test]
    fn wall_at_detects_wall_to_the_right_only() {
        let mut s = at_rest();
        s.position.x = WALL_X;
        assert!(wall_at(&s, &room(), 1));
        assert!(!wall_at(&s, &room(), -1));
        s.position.x = WALL_X - 1;
        assert!(!wall_at(&s, &room(), 1));
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        assert_eq!(lerp(160.0, 20.0, 0.0), 160.0);
        assert_eq!(lerp(160.0, 20.0, 1.0), 20.0);
        assert_eq!(lerp(160.0, 20.0, 0.5), 90.0);
    }

    fn falling_against_wall() -> PlayerState {
        let mut s = at_rest();
        s.position.x = WALL_X;
        s.speed.y = 100.0;
        s
    }

    #[test]
    fn wall_slide_starts_at_full_timer() {
        let mut s = falling_against_wall();
        let max = wall_slide_max_fall(&mut s, &room(), Input { move_x: 1, jump: false, ..Default::default() });
        assert_eq!(max, WALL_SLIDE_START_MAX);
        assert_eq!(s.wall_slide_dir, 1);
    }

    #[test]
    fn wall_slide_max_rises_as_timer_runs_out() {
        let mut s = falling_against_wall();
        s.wall_slide_timer = WALL_SLIDE_TIME * 0.5;
        let max = wall_slide_max_fall(&mut s, &room(), Input { move_x: 1, jump: false, ..Default::default() });
        assert_eq!(max, lerp(MAX_FALL, WALL_SLIDE_START_MAX, 0.5));
    }

    #[test]
    fn no_wall_slide_without_holding_toward_wall() {
        let mut s = falling_against_wall();
        assert_eq!(wall_slide_max_fall(&mut s, &room(), Input { move_x: 0, jump: false, ..Default::default() }), MAX_FALL);
        assert_eq!(s.wall_slide_dir, 0);
    }

    #[test]
    fn no_wall_slide_while_moving_up() {
        let mut s = falling_against_wall();
        s.speed.y = -50.0;
        assert_eq!(wall_slide_max_fall(&mut s, &room(), Input { move_x: 1, jump: false, ..Default::default() }), MAX_FALL);
        assert_eq!(s.wall_slide_dir, 0);
    }

    #[test]
    fn no_wall_slide_away_from_wall() {
        let mut s = falling_against_wall();
        s.position.x = WALL_X - 5;
        assert_eq!(wall_slide_max_fall(&mut s, &room(), Input { move_x: 1, jump: false, ..Default::default() }), MAX_FALL);
        assert_eq!(s.wall_slide_dir, 0);
    }

    #[test]
    fn jump_adds_horizontal_boost() {
        let mut s = standing();
        s.speed.x = 90.0;
        jump(&mut s, Input { move_x: 1, jump: true, ..Default::default() });
        assert_eq!(s.speed.x, 90.0 + JUMP_H_BOOST);
    }

    #[test]
    fn not_on_ground_past_ledge() {
        let mut s = standing();
        s.position.x = LEDGE_X;
        assert!(on_ground(&s, &room()));
        s.position.x = LEDGE_X + 1;
        assert!(!on_ground(&s, &room()));
    }

    #[test]
    fn move_v_falls_through_floor_level_past_ledge() {
        let mut s = standing();
        s.position.x = LEDGE_X + 1;
        s.speed.y = 60.0;
        move_v(&mut s, &room(), 1.0);
        assert_eq!(s.position.y, FLOOR_Y + 1);
        assert_eq!(s.speed.y, 60.0);
    }

    #[test]
    fn move_h_blocked_by_side_of_ledge() {
        let mut s = at_rest();
        s.position = IVec2 { x: LEDGE_X + 1, y: FLOOR_Y + 5 };
        s.speed.x = -90.0;
        move_h(&mut s, &room(), -3.0, false);
        assert_eq!(s.position.x, LEDGE_X + 1);
        assert_eq!(s.speed.x, 0.0);
    }

    fn aim(move_x: i8, move_y: i8, facing: i8) -> Vec2 {
        aim_vector(Input { move_x, move_y, ..Default::default() }, facing)
    }

    #[test]
    fn aim_defaults_to_facing() {
        assert_eq!(aim(0, 0, 1), Vec2 { x: 1.0, y: 0.0 });
        assert_eq!(aim(0, 0, -1), Vec2 { x: -1.0, y: 0.0 });
    }

    #[test]
    fn aim_cardinal_ignores_facing() {
        assert_eq!(aim(0, -1, -1), Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(aim(0, 1, 1), Vec2 { x: 0.0, y: 1.0 });
        assert_eq!(aim(-1, 0, 1), Vec2 { x: -1.0, y: 0.0 });
    }

    #[test]
    fn aim_diagonal_is_normalized() {
        let d = 1.0 / 2f32.sqrt();
        assert_eq!(aim(1, -1, 1), Vec2 { x: d, y: -d });
        assert_eq!(aim(-1, 1, 1), Vec2 { x: -d, y: d });
    }

    #[test]
    fn can_dash_needs_press_dash_and_no_cooldown() {
        let mut s = standing();
        assert!(can_dash(&s, true));
        assert!(!can_dash(&s, false));
        s.dash_cooldown_timer = 0.1;
        assert!(!can_dash(&s, true));
        s.dash_cooldown_timer = 0.0;
        s.dashes = 0;
        assert!(!can_dash(&s, true));
    }

    #[test]
    fn dash_begin_freezes_and_zeroes_speed() {
        let mut s = standing();
        s.speed = Vec2 { x: 90.0, y: 0.0 };
        s.wall_slide_timer = 0.3;
        dash_begin(&mut s, &room(), Input::default(), true);
        assert_eq!(s.mode, Mode::DashStart);
        assert_eq!(s.dashes, MAX_DASHES - 1);
        assert_eq!(s.freeze_timer, DASH_FREEZE_TIME);
        assert_eq!(s.dash_cooldown_timer, DASH_COOLDOWN);
        assert_eq!(s.dash_refill_cooldown_timer, DASH_REFILL_COOLDOWN);
        assert_eq!(s.wall_slide_timer, WALL_SLIDE_TIME);
        assert_eq!(s.before_dash_speed, Vec2 { x: 90.0, y: 0.0 });
        assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
        assert_eq!(s.dash_dir, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn dash_launch_sets_speed_and_starts_timer() {
        let mut s = standing();
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: -DASH_SPEED });
        assert_eq!(s.dash_dir, Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(s.mode, Mode::Dashing { timer: DASH_TIME });
    }

    #[test]
    fn dash_launch_keeps_faster_speed_in_same_direction() {
        let mut s = standing();
        s.speed.x = 300.0;
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
        assert_eq!(s.speed.x, 300.0);

        let mut s = standing();
        s.speed.x = -300.0;
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
        assert_eq!(s.speed.x, DASH_SPEED);
    }

    #[test]
    fn dash_launch_up_drops_horizontal_speed() {
        let mut s = standing();
        s.speed.x = 300.0;
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_launch_turns_to_face_horizontal_dashes() {
        let mut s = standing();
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, Vec2 { x: -1.0, y: 0.0 }, true);
        assert_eq!(s.facing, -1);

        let mut s = standing();
        s.facing = -1;
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 }, true);
        assert_eq!(s.facing, -1);
    }

    fn dashing(dir: Vec2) -> PlayerState {
        let mut s = standing();
        dash_begin(&mut s, &room(), Input::default(), true);
        dash_launch(&mut s, dir, true);
        s
    }

    #[test]
    fn dash_end_horizontal() {
        let mut s = dashing(Vec2 { x: 1.0, y: 0.0 });
        dash_end(&mut s);
        assert_eq!(s.mode, Mode::Normal);
        assert!(s.auto_jump);
        assert_eq!(s.speed, Vec2 { x: END_DASH_SPEED, y: 0.0 });
    }

    #[test]
    fn dash_end_up_is_slower() {
        let mut s = dashing(Vec2 { x: 0.0, y: -1.0 });
        dash_end(&mut s);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: -END_DASH_SPEED * END_DASH_UP_MULT });
    }

    #[test]
    fn dash_end_down_keeps_speed() {
        let mut s = dashing(Vec2 { x: 0.0, y: 1.0 });
        dash_end(&mut s);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: DASH_SPEED });
    }

    #[test]
    fn dash_refills_on_ground_after_refill_cooldown() {
        let mut s = standing();
        s.dashes = 0;
        s = step(s, Input::default(), &room());
        assert_eq!(s.dashes, MAX_DASHES);
    }

    #[test]
    fn dash_does_not_refill_during_refill_cooldown() {
        let mut s = standing();
        s.dashes = 0;
        s.dash_refill_cooldown_timer = DASH_REFILL_COOLDOWN;
        s = step(s, Input::default(), &room());
        assert_eq!(s.dashes, 0);
    }

    #[test]
    fn dash_does_not_refill_in_air() {
        let mut s = at_rest();
        s.dashes = 0;
        s = step(s, Input::default(), &room());
        assert_eq!(s.dashes, 0);
    }

    #[test]
    fn frozen_frames_do_nothing() {
        let mut s = standing();
        s.speed.x = 90.0;
        s.freeze_timer = DASH_FREEZE_TIME;
        let before = s;
        s = step(s, Input { move_x: -1, ..Default::default() }, &room());
        assert_eq!(s.position, before.position);
        assert_eq!(s.speed, before.speed);
        assert!(s.freeze_timer < DASH_FREEZE_TIME);
    }

    #[test]
    fn auto_jump_cleared_on_ground() {
        let mut s = standing();
        s.auto_jump = true;
        s = step(s, Input::default(), &room());
        assert!(!s.auto_jump);
    }
    #[test]
    fn hurtbox_is_two_pixels_shorter_than_the_hitbox() {
        let mut s = at_rest();
        s.position = IVec2 { x: 100, y: 50 };
        assert_eq!(hurtbox(&s), Rect { left: 96, top: 39, width: 8, height: 9 });
    }

    fn player_at(x: i32, y: i32, speed_x: f32, speed_y: f32) -> PlayerState {
        let mut s = at_rest();
        s.position = IVec2 { x, y };
        s.speed = Vec2 { x: speed_x, y: speed_y };
        s
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

    fn spiked_floor() -> Room {
        let mut room = room();
        room.spikes.push(Spikes::new(Direction::Up, 0, FLOOR_Y, 40));
        room
    }

    #[test]
    fn step_kills_on_spikes() {
        let s = step(standing(), Input::default(), &spiked_floor());
        assert!(s.dead);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn step_kills_on_the_frame_she_lands_on_spikes() {
        let room = spiked_floor();
        let mut s = player_at(0, FLOOR_Y - 5, 0.0, 160.0);
        s = step(s, Input::default(), &room);
        assert!(!s.dead);
        s = step(s, Input::default(), &room);
        assert!(s.dead);
        assert_eq!(s.position.y, FLOOR_Y);
    }

    #[test]
    fn no_spikes_no_death() {
        let s = step(standing(), Input::default(), &room());
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
    fn step_kills_when_she_falls_out() {
        let room = room();
        let mut s = player_at(60, 190, 0.0, 160.0);
        s = step(s, Input::default(), &room);
        assert!(!s.dead);
        s = step(s, Input::default(), &room);
        s = step(s, Input::default(), &room);
        assert!(s.dead);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: 0.0 });
    }

    #[test]
    fn falling_inside_the_room_is_not_death() {
        let s = step(player_at(60, 150, 0.0, 160.0), Input::default(), &room());
        assert!(!s.dead);
    }

    fn dashing_right() -> PlayerState {
        let mut s = standing();
        s.mode = Mode::Dashing { timer: DASH_TIME };
        s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
        s.speed = Vec2 { x: DASH_SPEED, y: 0.0 };
        s.jump_grace_timer = JUMP_GRACE_TIME;
        s
    }

    #[test]
    fn can_super_jump_during_a_horizontal_dash() {
        assert!(can_super_jump(&dashing_right(), &room(), true));
    }

    #[test]
    fn can_super_jump_needs_a_jump_press() {
        assert!(!can_super_jump(&dashing_right(), &room(), false));
    }

    #[test]
    fn can_super_jump_needs_coyote_time() {
        let mut s = dashing_right();
        s.jump_grace_timer = 0.0;
        assert!(!can_super_jump(&s, &room(), true));
    }

    #[test]
    fn can_super_jump_not_in_a_vertical_or_diagonal_dash() {
        let d = 1.0 / 2f32.sqrt();
        let mut s = dashing_right();
        s.dash_dir = Vec2 { x: 0.0, y: -1.0 };
        assert!(!can_super_jump(&s, &room(), true));
        s.dash_dir = Vec2 { x: d, y: -d };
        assert!(!can_super_jump(&s, &room(), true));
        s.dash_dir = Vec2 { x: 0.0, y: 1.0 };
        assert!(!can_super_jump(&s, &room(), true));
    }

    #[test]
    fn super_jump_launches_the_way_she_faces() {
        let mut s = dashing_right();
        super_jump(&mut s);
        assert_eq!(s.speed, Vec2 { x: SUPER_JUMP_H, y: JUMP_SPEED });

        let mut s = dashing_right();
        s.facing = -1;
        super_jump(&mut s);
        assert_eq!(s.speed, Vec2 { x: -SUPER_JUMP_H, y: JUMP_SPEED });
    }

    #[test]
    fn super_jump_ends_the_dash_and_starts_a_variable_jump() {
        let mut s = dashing_right();
        s.auto_jump = true;
        s.wall_slide_timer = 0.5;
        super_jump(&mut s);
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.jump_grace_timer, 0.0);
        assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
        assert_eq!(s.var_jump_speed, JUMP_SPEED);
        assert_eq!(s.wall_slide_timer, WALL_SLIDE_TIME);
        assert!(!s.auto_jump);
    }

    fn dash_for(start: PlayerState, frames: usize, input: Input) -> PlayerState {
        let room = room();
        let mut s = start;
        for _ in 0..frames {
            s = step(s, input, &room);
        }
        s
    }

    const DASH_RIGHT: Input = Input { move_x: 1, move_y: 0, jump: false, jump2: false, dash: true, grab: false };
    const JUMP_RIGHT: Input = Input { move_x: 1, move_y: 0, jump: true, jump2: false, dash: false, grab: false };

    #[test]
    fn step_super_jumps_out_of_a_dash() {
        let s = step(dash_for(standing(), 5, DASH_RIGHT), JUMP_RIGHT, &room());
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.speed, Vec2 { x: SUPER_JUMP_H, y: JUMP_SPEED });
    }

    #[test]
    fn step_super_jumps_on_the_launch_frame() {
        let s = step(dash_for(standing(), 4, DASH_RIGHT), JUMP_RIGHT, &room());
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.speed, Vec2 { x: SUPER_JUMP_H, y: JUMP_SPEED });
    }

    #[test]
    fn step_super_jumps_on_the_last_dash_frame() {
        let s = step(dash_for(standing(), 14, DASH_RIGHT), JUMP_RIGHT, &room());
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.speed, Vec2 { x: SUPER_JUMP_H, y: JUMP_SPEED });
    }

    #[test]
    fn step_no_super_jump_in_the_air() {
        let s = step(dash_for(at_rest(), 5, DASH_RIGHT), JUMP_RIGHT, &room());
        assert!(matches!(s.mode, Mode::Dashing { .. }));
        assert_eq!(s.speed, Vec2 { x: DASH_SPEED, y: 0.0 });
    }

    #[test]
    fn step_no_super_jump_out_of_an_up_dash() {
        let up_dash = Input { move_x: 0, move_y: -1, jump: false, jump2: false, dash: true, grab: false };
        let s = step(dash_for(standing(), 5, up_dash), JUMP_RIGHT, &room());
        assert!(matches!(s.mode, Mode::Dashing { .. }));
        assert_eq!(s.speed, Vec2 { x: 0.0, y: -DASH_SPEED });
    }

    const JUMP_1: Input = Input { move_x: 0, move_y: 0, jump: true, jump2: false, dash: false, grab: false };
    const JUMP_2: Input = Input { move_x: 0, move_y: 0, jump: false, jump2: true, dash: false, grab: false };
    const JUMP_BOTH: Input = Input { move_x: 0, move_y: 0, jump: true, jump2: true, dash: false, grab: false };

    #[test]
    fn jump_down_with_either_button() {
        assert!(JUMP_1.jump_down());
        assert!(JUMP_2.jump_down());
        assert!(JUMP_BOTH.jump_down());
        assert!(!Input::default().jump_down());
    }

    #[test]
    fn jump_pressed_when_a_button_goes_down() {
        assert!(jump_pressed(&standing(), JUMP_1));
        assert!(jump_pressed(&standing(), JUMP_2));
        assert!(!jump_pressed(&standing(), Input::default()));
    }

    #[test]
    fn jump_pressed_not_while_the_same_button_stays_down() {
        let mut s = standing();
        s.jump_held = true;
        assert!(!jump_pressed(&s, JUMP_1));
        s.jump_held = false;
        s.jump2_held = true;
        assert!(!jump_pressed(&s, JUMP_2));
    }

    #[test]
    fn jump_pressed_by_one_button_while_the_other_is_held() {
        let mut s = standing();
        s.jump_held = true;
        assert!(jump_pressed(&s, JUMP_BOTH));
        assert!(jump_pressed(&s, JUMP_2));
    }

    #[test]
    fn step_tracks_each_button() {
        let s = step(standing(), JUMP_2, &room());
        assert!(s.jump2_held);
        assert!(!s.jump_held);
        let s = step(s, JUMP_1, &room());
        assert!(s.jump_held);
        assert!(!s.jump2_held);
    }

    #[test]
    fn step_second_button_jumps_while_the_first_is_held() {
        let room = room();
        let mut s = standing();
        s.jump_held = true;
        s = step(s, JUMP_1, &room);
        assert_eq!(s.speed.y, 0.0);
        s = step(s, JUMP_2, &room);
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn step_second_button_holds_a_variable_jump() {
        let room = room();
        let mut s = step(standing(), JUMP_1, &room);
        for _ in 0..5 {
            s = step(s, JUMP_2, &room);
        }
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    fn left_wall_room() -> Room {
        Room::from_tiles(IVec2 { x: 0, y: 0 }, &"10000000000\n".repeat(23))
    }

    // flush against the left wall
    const LEFT_WALL_X: i32 = 12;

    const JUMP_GRAB: Input = Input { move_x: 0, move_y: 0, jump: true, jump2: false, dash: false, grab: true };

    #[test]
    fn wall_jump_check_reaches_two_pixels_of_gap() {
        let room = room();
        assert!(wall_jump_check(&player_at(WALL_X, 100, 0.0, 0.0), &room, 1));
        assert!(wall_jump_check(&player_at(WALL_X - 2, 100, 0.0, 0.0), &room, 1));
        assert!(!wall_jump_check(&player_at(WALL_X - 3, 100, 0.0, 0.0), &room, 1));
    }

    #[test]
    fn wall_jump_check_only_on_the_side_asked() {
        assert!(!wall_jump_check(&player_at(WALL_X, 100, 0.0, 0.0), &room(), -1));

        let room = left_wall_room();
        assert!(wall_jump_check(&player_at(LEFT_WALL_X, 100, 0.0, 0.0), &room, -1));
        assert!(wall_jump_check(&player_at(LEFT_WALL_X + 2, 100, 0.0, 0.0), &room, -1));
        assert!(!wall_jump_check(&player_at(LEFT_WALL_X + 3, 100, 0.0, 0.0), &room, -1));
        assert!(!wall_jump_check(&player_at(LEFT_WALL_X, 100, 0.0, 0.0), &room, 1));
    }

    #[test]
    fn can_climb_jump_facing_a_wall_with_grab_held() {
        assert!(can_climb_jump(&player_at(WALL_X, 100, 0.0, 0.0), &room(), JUMP_GRAB));

        let mut s = player_at(LEFT_WALL_X, 100, 0.0, 0.0);
        s.facing = -1;
        assert!(can_climb_jump(&s, &left_wall_room(), JUMP_GRAB));
    }

    #[test]
    fn can_climb_jump_needs_grab() {
        let no_grab = Input { grab: false, ..JUMP_GRAB };
        assert!(!can_climb_jump(&player_at(WALL_X, 100, 0.0, 0.0), &room(), no_grab));
    }

    #[test]
    fn can_climb_jump_needs_stamina() {
        let mut s = player_at(WALL_X, 100, 0.0, 0.0);
        s.stamina = 0.0;
        assert!(!can_climb_jump(&s, &room(), JUMP_GRAB));
    }

    #[test]
    fn can_climb_jump_needs_a_wall() {
        assert!(!can_climb_jump(&player_at(WALL_X - 3, 100, 0.0, 0.0), &room(), JUMP_GRAB));
    }

    #[test]
    fn can_climb_jump_not_facing_away_from_the_wall() {
        let mut s = player_at(WALL_X, 100, 0.0, 0.0);
        s.facing = -1;
        assert!(!can_climb_jump(&s, &room(), JUMP_GRAB));

        let s = player_at(LEFT_WALL_X, 100, 0.0, 0.0);
        assert!(!can_climb_jump(&s, &left_wall_room(), JUMP_GRAB));
    }

    #[test]
    fn climb_jump_in_the_air_costs_stamina() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.mode = Mode::Dashing { timer: 0.1 };
        climb_jump(&mut s, Input { move_x: 1, ..JUMP_GRAB }, false);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
        assert_eq!(s.speed, Vec2 { x: JUMP_H_BOOST, y: JUMP_SPEED });
        assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
        assert_eq!(s.mode, Mode::Normal);
    }

    #[test]
    fn climb_jump_on_the_ground_is_free() {
        let mut s = standing();
        climb_jump(&mut s, JUMP_GRAB, true);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert_eq!(s.speed.y, JUMP_SPEED);
    }

    #[test]
    fn step_refills_stamina_on_the_ground() {
        let mut s = standing();
        s.stamina = 10.0;
        s = step(s, Input::default(), &room());
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_does_not_refill_stamina_in_the_air() {
        let mut s = at_rest();
        s.stamina = 10.0;
        s = step(s, Input::default(), &room());
        assert_eq!(s.stamina, 10.0);
    }

    #[test]
    fn step_climb_jumps_next_to_a_wall() {
        let s = step(player_at(WALL_X, 100, 0.0, 30.0), JUMP_GRAB, &room());
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
    }

    #[test]
    fn step_no_climb_jump_without_grab() {
        let no_grab = Input { grab: false, ..JUMP_GRAB };
        let s = step(player_at(WALL_X, 100, 0.0, 30.0), no_grab, &room());
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_ground_jump_wins_over_climb_jump() {
        let mut s = player_at(36, FLOOR_Y, 0.0, 0.0);
        s.stamina = 50.0;
        s = step(s, Input { move_x: 1, ..JUMP_GRAB }, &room());
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_climb_jumps_out_of_a_dash() {
        let mut s = player_at(WALL_X - 2, 100, DASH_SPEED, 0.0);
        s.mode = Mode::Dashing { timer: 0.1 };
        s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
        s = step(s, Input { move_x: 1, ..JUMP_GRAB }, &room());
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA - CLIMB_JUMP_COST);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: JUMP_SPEED });
    }

    #[test]
    fn move_h_into_a_wall_remembers_her_speed() {
        let mut s = player_at(WALL_X - 1, 100, 120.0, 0.0);
        move_h(&mut s, &room(), 3.0, false);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall_speed_retained, 120.0);
        assert_eq!(s.wall_speed_retention_timer, WALL_SPEED_RETENTION_TIME);
    }

    #[test]
    fn move_h_keeps_the_first_speed_while_the_timer_runs() {
        let mut s = player_at(WALL_X - 1, 100, 50.0, 0.0);
        s.wall_speed_retained = 200.0;
        s.wall_speed_retention_timer = 0.03;
        move_h(&mut s, &room(), 3.0, false);
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall_speed_retained, 200.0);
        assert_eq!(s.wall_speed_retention_timer, 0.03);
    }

    #[test]
    fn move_h_without_a_collision_retains_nothing() {
        let mut s = player_at(0, 100, 120.0, 0.0);
        move_h(&mut s, &room(), 3.0, false);
        assert_eq!(s.wall_speed_retention_timer, 0.0);
    }

    fn retaining(x: i32, speed_x: f32, retained: f32) -> PlayerState {
        let mut s = player_at(x, 100, speed_x, 0.0);
        s.wall_speed_retained = retained;
        s.wall_speed_retention_timer = 0.03;
        s
    }

    #[test]
    fn retention_gives_the_speed_back_once_the_wall_is_gone() {
        let mut s = retaining(WALL_X - 1, 0.0, 120.0);
        update_wall_speed_retention(&mut s, &room());
        assert_eq!(s.speed.x, 120.0);
        assert_eq!(s.wall_speed_retention_timer, 0.0);
    }

    #[test]
    fn retention_waits_while_the_wall_is_still_there() {
        let mut s = retaining(WALL_X, 0.0, 120.0);
        update_wall_speed_retention(&mut s, &room());
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall_speed_retention_timer, 0.03 - DELTA_TIME);
    }

    #[test]
    fn retention_is_cancelled_by_moving_the_other_way() {
        let mut s = retaining(WALL_X - 1, -10.0, 120.0);
        update_wall_speed_retention(&mut s, &room());
        assert_eq!(s.speed.x, -10.0);
        assert_eq!(s.wall_speed_retention_timer, 0.0);
    }

    #[test]
    fn retention_works_to_the_left() {
        let room = left_wall_room();
        let mut s = retaining(LEFT_WALL_X, 0.0, -120.0);
        update_wall_speed_retention(&mut s, &room);
        assert_eq!(s.speed.x, 0.0);

        let mut s = retaining(LEFT_WALL_X + 1, 0.0, -120.0);
        update_wall_speed_retention(&mut s, &room);
        assert_eq!(s.speed.x, -120.0);
    }

    #[test]
    fn retention_does_nothing_once_the_timer_is_out() {
        let mut s = retaining(WALL_X - 1, 0.0, 120.0);
        s.wall_speed_retention_timer = 0.0;
        update_wall_speed_retention(&mut s, &room());
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn step_gives_retained_speed_back_before_moving() {
        let s = step(retaining(WALL_X - 20, 0.0, 120.0), Input { move_x: 1, ..Default::default() }, &room());
        assert!(s.speed.x > 100.0);
        assert!(s.position.x > WALL_X - 20);
        assert_eq!(s.wall_speed_retention_timer, 0.0);
    }

    #[test]
    fn step_running_into_a_wall_starts_retention() {
        let s = step(player_at(WALL_X, 100, 90.0, 0.0), Input { move_x: 1, ..Default::default() }, &room());
        assert_eq!(s.speed.x, 0.0);
        assert_eq!(s.wall_speed_retained, 90.0);
        assert_eq!(s.wall_speed_retention_timer, WALL_SPEED_RETENTION_TIME);
    }

    // an 8 px high tunnel from x = 24 rightwards: she fits ducking, not standing
    fn tunnel_room() -> Room {
        let text = "00000000000\n".repeat(16) + "00011111111\n" + "00000000000\n" + &"11111111111\n".repeat(5);
        Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
    }

    fn ducking_at(x: i32, y: i32) -> PlayerState {
        let mut s = player_at(x, y, 0.0, 0.0);
        s.ducking = true;
        s
    }

    const DOWN: Input = Input { move_x: 0, move_y: 1, jump: false, jump2: false, dash: false, grab: false };

    #[test]
    fn collides_uses_the_shorter_hitbox_when_ducking() {
        let room = tunnel_room();
        assert!(!collides(&ducking_at(40, FLOOR_Y), &room, 0, 0));
        assert!(collides(&player_at(40, FLOOR_Y, 0.0, 0.0), &room, 0, 0));
        assert!(collides(&ducking_at(40, FLOOR_Y), &room, 0, 1));
        assert!(collides(&ducking_at(40, FLOOR_Y), &room, 0, -3));
    }

    #[test]
    fn can_unduck_when_not_ducking() {
        assert!(can_unduck(&player_at(40, FLOOR_Y, 0.0, 0.0), &tunnel_room()));
    }

    #[test]
    fn can_unduck_with_room_overhead() {
        assert!(can_unduck(&ducking_at(8, FLOOR_Y), &tunnel_room()));
    }

    #[test]
    fn can_unduck_not_under_a_low_ceiling() {
        assert!(!can_unduck(&ducking_at(40, FLOOR_Y), &tunnel_room()));
    }

    #[test]
    fn hurtbox_is_shorter_when_ducking() {
        assert_eq!(hurtbox(&ducking_at(100, 50)), Rect { left: 96, top: 44, width: 8, height: 4 });
    }

    #[test]
    fn dash_begin_ducks_when_down_is_held() {
        let mut s = standing();
        dash_begin(&mut s, &room(), DOWN, true);
        assert!(s.ducking);
    }

    #[test]
    fn dash_begin_stays_standing_without_down() {
        let mut s = standing();
        dash_begin(&mut s, &room(), Input::default(), true);
        assert!(!s.ducking);
    }

    #[test]
    fn dash_begin_stays_ducked_on_the_ground() {
        let mut s = ducking_at(0, FLOOR_Y);
        dash_begin(&mut s, &room(), Input::default(), true);
        assert!(s.ducking);
    }

    #[test]
    fn dash_begin_stands_up_in_the_air() {
        let mut s = ducking_at(0, 64);
        dash_begin(&mut s, &room(), Input::default(), false);
        assert!(!s.ducking);
    }

    #[test]
    fn super_jump_while_ducking_is_a_hyper() {
        let mut s = dashing_right();
        s.ducking = true;
        super_jump(&mut s);
        assert_eq!(s.speed, Vec2 { x: SUPER_JUMP_H * DUCK_SUPER_JUMP_X_MULT, y: JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT });
        assert_eq!(s.var_jump_speed, JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT);
        assert!(!s.ducking);
    }

    #[test]
    fn can_super_jump_needs_room_to_stand_up() {
        let mut s = ducking_at(40, FLOOR_Y);
        s.mode = Mode::Dashing { timer: DASH_TIME };
        s.dash_dir = Vec2 { x: 1.0, y: 0.0 };
        s.jump_grace_timer = JUMP_GRACE_TIME;
        assert!(!can_super_jump(&s, &tunnel_room(), true));
        s.position.x = 8;
        assert!(can_super_jump(&s, &tunnel_room(), true));
    }

    fn launching() -> PlayerState {
        let mut s = standing();
        s.mode = Mode::DashStart;
        s
    }

    #[test]
    fn dash_launch_down_diagonal_on_the_ground_slides() {
        let d = 1.0 / 2f32.sqrt();
        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: -d, y: d }, true);
        assert_eq!(s.speed, Vec2 { x: -d * DASH_SPEED * DODGE_SLIDE_SPEED_MULT, y: 0.0 });
        assert_eq!(s.dash_dir, Vec2 { x: -1.0, y: 0.0 });
        assert!(s.ducking);
    }

    #[test]
    fn dash_launch_down_diagonal_in_the_air_does_not_slide() {
        let d = 1.0 / 2f32.sqrt();
        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: d, y: d }, false);
        assert_eq!(s.speed, Vec2 { x: d * DASH_SPEED, y: d * DASH_SPEED });
        assert_eq!(s.dash_dir, Vec2 { x: d, y: d });
        assert!(!s.ducking);
    }

    #[test]
    fn dash_launch_straight_down_or_sideways_does_not_slide() {
        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: 0.0, y: 1.0 }, true);
        assert_eq!(s.speed, Vec2 { x: 0.0, y: DASH_SPEED });
        assert!(!s.ducking);

        let mut s = launching();
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 }, true);
        assert_eq!(s.speed, Vec2 { x: DASH_SPEED, y: 0.0 });
        assert!(!s.ducking);
    }

    #[test]
    fn step_ducks_when_down_is_held_on_the_ground() {
        assert!(step(standing(), DOWN, &room()).ducking);
    }

    #[test]
    fn step_does_not_duck_in_the_air() {
        assert!(!step(at_rest(), DOWN, &room()).ducking);
    }

    #[test]
    fn step_stands_up_when_down_is_released() {
        assert!(!step(ducking_at(0, FLOOR_Y), Input::default(), &room()).ducking);
    }

    #[test]
    fn step_stays_ducked_while_down_is_held() {
        assert!(step(ducking_at(0, FLOOR_Y), DOWN, &room()).ducking);
    }

    #[test]
    fn step_stays_ducked_under_a_low_ceiling() {
        assert!(step(ducking_at(40, FLOOR_Y), Input::default(), &tunnel_room()).ducking);
    }

    #[test]
    fn step_ducking_on_the_ground_uses_duck_friction() {
        let mut s = ducking_at(0, FLOOR_Y);
        s.speed.x = 90.0;
        s = step(s, Input { move_x: 1, ..DOWN }, &room());
        assert_eq!(s.speed.x, approach(90.0, 0.0, DUCK_FRICTION * DELTA_TIME));
    }

    #[test]
    fn step_stands_up_when_falling() {
        let mut s = ducking_at(0, 64);
        s.speed.y = 50.0;
        assert!(!step(s, DOWN, &room()).ducking);
    }

    #[test]
    fn step_stays_ducked_while_rising() {
        let mut s = ducking_at(0, 64);
        s.speed.y = -100.0;
        assert!(step(s, Input::default(), &room()).ducking);
    }

    #[test]
    fn step_hyper_from_a_down_dash() {
        let down_dash = Input { dash: true, ..DOWN };
        let s = step(dash_for(standing(), 4, down_dash), JUMP_RIGHT, &room());
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.speed, Vec2 { x: SUPER_JUMP_H * DUCK_SUPER_JUMP_X_MULT, y: JUMP_SPEED * DUCK_SUPER_JUMP_Y_MULT });
        assert!(!s.ducking);
    }

    #[test]
    fn climb_jump_without_a_direction_arms_the_wall_boost() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        climb_jump(&mut s, JUMP_GRAB, false);
        assert_eq!(s.wall_boost_dir, -1);
        assert_eq!(s.wall_boost_timer, CLIMB_JUMP_BOOST_TIME);

        let mut s = player_at(LEFT_WALL_X, 100, 0.0, 30.0);
        s.facing = -1;
        climb_jump(&mut s, JUMP_GRAB, false);
        assert_eq!(s.wall_boost_dir, 1);
    }

    #[test]
    fn climb_jump_with_a_direction_does_not_arm_the_wall_boost() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        climb_jump(&mut s, Input { move_x: 1, ..JUMP_GRAB }, false);
        assert_eq!(s.wall_boost_timer, 0.0);
    }

    #[test]
    fn jumps_cancel_the_wall_boost() {
        let mut s = standing();
        s.wall_boost_timer = 0.1;
        jump(&mut s, Input::default());
        assert_eq!(s.wall_boost_timer, 0.0);

        let mut s = dashing_right();
        s.wall_boost_timer = 0.1;
        super_jump(&mut s);
        assert_eq!(s.wall_boost_timer, 0.0);
    }

    fn wall_boosting() -> PlayerState {
        let mut s = player_at(40, 100, 0.0, -100.0);
        s.wall_boost_dir = -1;
        s.wall_boost_timer = 0.1;
        s.stamina = 50.0;
        s
    }

    const LEFT: Input = Input { move_x: -1, move_y: 0, jump: false, jump2: false, dash: false, grab: false };

    #[test]
    fn step_remembers_the_direction_held() {
        assert_eq!(step(at_rest(), LEFT, &room()).move_x, -1);
        assert_eq!(step(at_rest(), Input::default(), &room()).move_x, 0);
    }

    #[test]
    fn step_wall_boost_happens_the_frame_after_the_press() {
        let room = room();
        let mut s = step(wall_boosting(), LEFT, &room);
        assert_eq!(s.stamina, 50.0);
        assert_eq!(s.speed.x, approach(0.0, -MAX_RUN, RUN_ACCEL * DELTA_TIME * AIR_MULT));

        s = step(s, Input::default(), &room);
        assert_eq!(s.stamina, 50.0 + CLIMB_JUMP_COST);
        assert_eq!(s.wall_boost_timer, 0.0);
        assert_eq!(s.speed.x, approach(-WALL_JUMP_H_SPEED, 0.0, RUN_ACCEL * DELTA_TIME * AIR_MULT));
    }

    #[test]
    fn step_wall_boost_ignores_the_other_direction() {
        let room = room();
        let mut s = step(wall_boosting(), Input { move_x: 1, ..LEFT }, &room);
        s = step(s, Input::default(), &room);
        assert_eq!(s.stamina, 50.0);
        assert!(s.wall_boost_timer > 0.0);
    }

    #[test]
    fn step_wall_boost_runs_out() {
        let room = room();
        let mut s = wall_boosting();
        s.wall_boost_timer = 0.01;
        s = step(s, LEFT, &room);
        s = step(s, LEFT, &room);
        assert_eq!(s.stamina, 50.0);
    }

    // a ceiling block spanning x 16..48, y 40..48
    fn ceiling_room() -> Room {
        let text = "00000000000\n".repeat(5) + "00111100000\n" + &"00000000000\n".repeat(17);
        Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
    }

    // her head touches the ceiling block's underside at this y
    const UNDER_CEILING_Y: i32 = 59;

    fn rising_at(x: i32, speed_x: f32) -> PlayerState {
        player_at(x, UNDER_CEILING_Y, speed_x, -240.0)
    }

    #[test]
    fn move_v_up_into_a_corner_slides_around_it() {
        let mut s = rising_at(50, 0.0);
        s.remainder.y = 0.25;
        move_v(&mut s, &ceiling_room(), -4.0);
        assert_eq!(s.position, IVec2 { x: 52, y: UNDER_CEILING_Y - 1 });
        assert_eq!(s.speed.y, -240.0);
        assert_eq!(s.remainder.y, 0.0);
    }

    #[test]
    fn corner_correction_reaches_four_pixels() {
        let room = ceiling_room();
        let mut s = rising_at(48, 0.0);
        move_v(&mut s, &room, -1.0);
        assert_eq!(s.position, IVec2 { x: 52, y: UNDER_CEILING_Y - 1 });

        let mut s = rising_at(47, 0.0);
        move_v(&mut s, &room, -1.0);
        assert_eq!(s.position, IVec2 { x: 47, y: UNDER_CEILING_Y });
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn corner_correction_slides_left_too() {
        let mut s = rising_at(14, 0.0);
        move_v(&mut s, &ceiling_room(), -1.0);
        assert_eq!(s.position, IVec2 { x: 12, y: UNDER_CEILING_Y - 1 });
    }

    #[test]
    fn corner_correction_only_slides_the_way_she_is_moving() {
        let room = ceiling_room();
        let mut s = rising_at(50, -10.0);
        move_v(&mut s, &room, -1.0);
        assert_eq!(s.position, IVec2 { x: 50, y: UNDER_CEILING_Y });
        assert_eq!(s.speed.y, 0.0);

        let mut s = rising_at(14, 10.0);
        move_v(&mut s, &room, -1.0);
        assert_eq!(s.position, IVec2 { x: 14, y: UNDER_CEILING_Y });

        let mut s = rising_at(50, 10.0);
        move_v(&mut s, &room, -1.0);
        assert_eq!(s.position, IVec2 { x: 52, y: UNDER_CEILING_Y - 1 });
    }

    #[test]
    fn ceiling_bonk_ends_the_variable_jump() {
        let mut s = rising_at(32, 0.0);
        s.var_jump_timer = 0.1;
        move_v(&mut s, &ceiling_room(), -1.0);
        assert_eq!(s.speed.y, 0.0);
        assert_eq!(s.var_jump_timer, 0.0);
    }

    #[test]
    fn ceiling_bonk_at_the_start_of_a_jump_keeps_the_variable_jump() {
        let mut s = rising_at(32, 0.0);
        s.var_jump_timer = 0.18;
        move_v(&mut s, &ceiling_room(), -1.0);
        assert_eq!(s.speed.y, 0.0);
        assert_eq!(s.var_jump_timer, 0.18);
    }

    #[test]
    fn corner_correction_keeps_the_variable_jump() {
        let mut s = rising_at(50, 0.0);
        s.var_jump_timer = 0.1;
        move_v(&mut s, &ceiling_room(), -1.0);
        assert_eq!(s.var_jump_timer, 0.1);
    }

    #[test]
    fn landing_does_not_corner_correct() {
        let mut s = player_at(LEDGE_X, FLOOR_Y, 0.0, 100.0);
        move_v(&mut s, &room(), 2.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X, y: FLOOR_Y });
        assert_eq!(s.speed.y, 0.0);
    }

    fn dashing_at(x: i32, y: i32, speed_x: f32, speed_y: f32) -> PlayerState {
        let mut s = player_at(x, y, speed_x, speed_y);
        s.mode = Mode::Dashing { timer: 0.1 };
        s
    }

    // flush against the right face of the floor block, in the pit
    const PIT_X: i32 = 44;

    #[test]
    fn dash_begin_remembers_whether_it_started_on_the_ground() {
        let mut s = standing();
        dash_begin(&mut s, &room(), Input::default(), true);
        assert!(s.dash_started_on_ground);

        let mut s = at_rest();
        s.dash_started_on_ground = true;
        dash_begin(&mut s, &room(), Input::default(), false);
        assert!(!s.dash_started_on_ground);
    }

    #[test]
    fn dash_into_a_ledge_just_below_its_top_pops_up_onto_it() {
        let mut s = dashing_at(PIT_X, FLOOR_Y + 2, -240.0, 0.0);
        s.remainder.x = 0.25;
        move_h(&mut s, &room(), -3.0, false);
        assert_eq!(s.position, IVec2 { x: PIT_X - 1, y: FLOOR_Y });
        assert_eq!(s.speed.x, -240.0);
        assert_eq!(s.remainder.x, 0.0);
        assert_eq!(s.wall_speed_retention_timer, 0.0);
    }

    #[test]
    fn dash_corner_correction_reaches_four_pixels() {
        let mut s = dashing_at(PIT_X, FLOOR_Y + 4, -240.0, 0.0);
        move_h(&mut s, &room(), -3.0, false);
        assert_eq!(s.position, IVec2 { x: PIT_X - 1, y: FLOOR_Y });

        let mut s = dashing_at(PIT_X, FLOOR_Y + 5, -240.0, 0.0);
        move_h(&mut s, &room(), -3.0, false);
        assert_eq!(s.position, IVec2 { x: PIT_X, y: FLOOR_Y + 5 });
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_corner_correction_needs_a_dash() {
        let mut s = player_at(PIT_X, FLOOR_Y + 2, -240.0, 0.0);
        move_h(&mut s, &room(), -3.0, false);
        assert_eq!(s.position, IVec2 { x: PIT_X, y: FLOOR_Y + 2 });
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_corner_correction_needs_a_level_dash() {
        let mut s = dashing_at(PIT_X, FLOOR_Y + 2, -170.0, -170.0);
        move_h(&mut s, &room(), -3.0, false);
        assert_eq!(s.position, IVec2 { x: PIT_X, y: FLOOR_Y + 2 });
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn duck_free_at_checks_the_ducking_hitbox() {
        let room = tunnel_room();
        let s = player_at(20, FLOOR_Y, 0.0, 0.0);
        assert!(duck_free_at(&s, &room, 1));
        assert!(collides(&s, &room, 1, 0));
        assert!(!duck_free_at(&player_at(20, FLOOR_Y - 4, 0.0, 0.0), &room, 1));
    }

    #[test]
    fn ground_dash_into_a_low_gap_ducks() {
        let room = tunnel_room();
        let mut s = dashing_at(20, FLOOR_Y, 240.0, 0.0);
        move_h(&mut s, &room, 4.0, true);
        assert!(s.ducking);
        assert_eq!(s.position.x, 20);
        assert_eq!(s.speed.x, 240.0);

        move_h(&mut s, &room, 4.0, true);
        assert_eq!(s.position.x, 24);
    }

    #[test]
    fn air_dash_into_a_low_gap_does_not_duck() {
        let mut s = dashing_at(20, FLOOR_Y, 240.0, 0.0);
        move_h(&mut s, &tunnel_room(), 4.0, false);
        assert!(!s.ducking);
        assert_eq!(s.speed.x, 0.0);
    }

    fn air_dashing_down_at(x: i32, speed_x: f32) -> PlayerState {
        dashing_at(x, FLOOR_Y - 1, speed_x, 240.0)
    }

    #[test]
    fn air_dash_down_onto_a_ledge_edge_slips_off() {
        let mut s = air_dashing_down_at(LEDGE_X - 1, 0.0);
        s.remainder.y = 0.25;
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X + 1, y: FLOOR_Y + 1 });
        assert_eq!(s.speed.y, 240.0);
        assert_eq!(s.remainder.y, 0.0);
    }

    #[test]
    fn downward_corner_correction_reaches_four_pixels() {
        let mut s = air_dashing_down_at(LEDGE_X - 3, 0.0);
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X + 1, y: FLOOR_Y + 1 });

        let mut s = air_dashing_down_at(LEDGE_X - 4, 0.0);
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X - 4, y: FLOOR_Y });
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn downward_corner_correction_only_slips_the_way_she_is_moving() {
        let mut s = air_dashing_down_at(LEDGE_X - 1, -10.0);
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X - 1, y: FLOOR_Y });

        let mut s = air_dashing_down_at(LEDGE_X - 1, 10.0);
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X + 1, y: FLOOR_Y + 1 });
    }

    #[test]
    fn downward_corner_correction_not_for_ground_dashes_or_falls() {
        let mut s = air_dashing_down_at(LEDGE_X - 1, 0.0);
        s.dash_started_on_ground = true;
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X - 1, y: FLOOR_Y });

        let mut s = player_at(LEDGE_X - 1, FLOOR_Y - 1, 0.0, 240.0);
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position, IVec2 { x: LEDGE_X - 1, y: FLOOR_Y });
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn landing_after_a_down_diagonal_dash_slides() {
        let d = 1.0 / 2f32.sqrt();
        let mut s = player_at(0, FLOOR_Y - 1, 100.0, 200.0);
        s.dash_dir = Vec2 { x: d, y: d };
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.position.y, FLOOR_Y);
        assert_eq!(s.speed, Vec2 { x: 100.0 * DODGE_SLIDE_SPEED_MULT, y: 0.0 });
        assert_eq!(s.dash_dir, Vec2 { x: 1.0, y: 0.0 });
        assert!(s.ducking);
    }

    #[test]
    fn landing_after_a_straight_down_dash_does_not_slide() {
        let mut s = player_at(0, FLOOR_Y - 1, 100.0, 200.0);
        s.dash_dir = Vec2 { x: 0.0, y: 1.0 };
        move_v(&mut s, &room(), 4.0);
        assert_eq!(s.speed, Vec2 { x: 100.0, y: 0.0 });
        assert!(!s.ducking);
    }

    const JUMP: Input = Input { move_x: 0, move_y: 0, jump: true, jump2: false, dash: false, grab: false };

    // just after a dash straight up
    fn bouncing_at(x: i32) -> PlayerState {
        let mut s = player_at(x, 100, 0.0, -120.0);
        s.dash_dir = Vec2 { x: 0.0, y: -1.0 };
        s.dash_attack_timer = 0.1;
        s
    }

    #[test]
    fn super_wall_jump_is_faster_and_higher_than_a_jump() {
        let mut s = bouncing_at(LEFT_WALL_X);
        s.ducking = true;
        s.auto_jump = true;
        super_wall_jump(&mut s, 1);
        assert_eq!(s.speed, Vec2 { x: SUPER_WALL_JUMP_H, y: SUPER_WALL_JUMP_SPEED });
        assert_eq!(s.var_jump_speed, SUPER_WALL_JUMP_SPEED);
        assert_eq!(s.var_jump_timer, SUPER_WALL_JUMP_VAR_TIME);
        assert_eq!(s.dash_attack_timer, 0.0);
        assert_eq!(s.mode, Mode::Normal);
        assert!(!s.ducking);
        assert!(!s.auto_jump);
    }

    #[test]
    fn wall_jump_check_reaches_further_for_a_wall_bounce() {
        let room = room();
        assert!(wall_jump_check(&bouncing_at(WALL_X - 4), &room, 1));
        assert!(!wall_jump_check(&bouncing_at(WALL_X - 5), &room, 1));
        assert!(!wall_jump_check(&player_at(WALL_X - 4, 100, 0.0, 0.0), &room, 1));
    }

    #[test]
    fn wall_jump_check_is_short_again_when_the_wall_has_spikes() {
        let mut room = room();
        room.spikes.push(Spikes::new(Direction::Left, WALL_X + 4, 88, 16));
        assert!(!wall_jump_check(&bouncing_at(WALL_X - 4), &room, 1));
        assert!(wall_jump_check(&bouncing_at(WALL_X - 2), &room, 1));
    }

    #[test]
    fn dash_attack_runs_from_the_start_of_a_dash() {
        let mut s = standing();
        dash_begin(&mut s, &room(), Input::default(), true);
        assert_eq!(s.dash_attack_timer, DASH_ATTACK_TIME);

        let s = dash_for(standing(), 6, DASH_RIGHT);
        assert!(s.dash_attack_timer > 0.0 && s.dash_attack_timer < DASH_ATTACK_TIME);
    }

    #[test]
    fn jumps_and_collisions_end_the_dash_attack() {
        let mut s = standing();
        s.dash_attack_timer = 0.1;
        jump(&mut s, Input::default());
        assert_eq!(s.dash_attack_timer, 0.0);

        let mut s = player_at(WALL_X, 100, 90.0, 0.0);
        s.dash_attack_timer = 0.1;
        move_h(&mut s, &room(), 3.0, false);
        assert_eq!(s.dash_attack_timer, 0.0);

        let mut s = player_at(0, FLOOR_Y, 0.0, 100.0);
        s.dash_attack_timer = 0.1;
        move_v(&mut s, &room(), 3.0);
        assert_eq!(s.dash_attack_timer, 0.0);
    }

    #[test]
    fn step_wall_bounce_after_an_upward_dash() {
        let s = step(bouncing_at(WALL_X - 4), JUMP, &room());
        assert_eq!(s.speed, Vec2 { x: -SUPER_WALL_JUMP_H, y: SUPER_WALL_JUMP_SPEED });

        let s = step(bouncing_at(LEFT_WALL_X + 4), JUMP, &left_wall_room());
        assert_eq!(s.speed, Vec2 { x: SUPER_WALL_JUMP_H, y: SUPER_WALL_JUMP_SPEED });
    }

    #[test]
    fn step_wall_bounce_during_an_upward_dash_ignores_grab() {
        let mut s = bouncing_at(WALL_X);
        s.speed.y = -DASH_SPEED;
        s.mode = Mode::Dashing { timer: 0.1 };
        s = step(s, JUMP_GRAB, &room());
        assert_eq!(s.mode, Mode::Normal);
        assert_eq!(s.speed, Vec2 { x: -SUPER_WALL_JUMP_H, y: SUPER_WALL_JUMP_SPEED });
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
    }

    #[test]
    fn step_no_wall_bounce_once_the_dash_attack_is_over() {
        let mut s = bouncing_at(WALL_X - 4);
        s.dash_attack_timer = 0.0;
        s = step(s, JUMP, &room());
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn step_holding_grab_without_pressing_jump_does_not_climb_jump() {
        let mut s = player_at(WALL_X, 100, 0.0, 30.0);
        s.jump_held = true;
        s = step(s, JUMP_GRAB, &room());
        assert_eq!(s.stamina, CLIMB_MAX_STAMINA);
        assert!(s.speed.y > 0.0);
    }

    #[test]
    fn dead_player_does_not_change() {
        let mut before = standing();
        before.dead = true;
        let after = step(before, Input { move_x: 1, jump: true, ..Default::default() }, &room());
        assert_eq!(after, before);
    }
}
