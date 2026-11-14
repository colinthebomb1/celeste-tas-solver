use crate::room::Room;
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

// normal hitbox size in pixels
pub const HITBOX_WIDTH: i32 = 8;
pub const HITBOX_HEIGHT: i32 = 11;

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
    // -1 = left, 1 = right
    pub facing: i8,
    pub wall_slide_timer: f32,
    pub wall_slide_dir: i8,
    pub mode: Mode,
    pub dashes: u8,
    pub dash_cooldown_timer: f32,
    pub dash_refill_cooldown_timer: f32,
    pub dash_dir: Vec2,
    pub before_dash_speed: Vec2,
    pub auto_jump: bool,
    pub dash_held: bool,
    pub freeze_timer: f32,
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
            facing: 1,
            wall_slide_timer: WALL_SLIDE_TIME,
            wall_slide_dir: 0,
            mode: Mode::Normal,
            dashes: MAX_DASHES,
            dash_cooldown_timer: 0.0,
            dash_refill_cooldown_timer: 0.0,
            dash_dir: Vec2 { x: 0.0, y: 0.0 },
            before_dash_speed: Vec2 { x: 0.0, y: 0.0 },
            auto_jump: false,
            dash_held: false,
            freeze_timer: 0.0,
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
    pub dash: bool,
}

pub fn collide_at(room: &Room, x: i32, y: i32) -> bool {
    room.collide_rect(x - HITBOX_WIDTH / 2, y - HITBOX_HEIGHT, HITBOX_WIDTH, HITBOX_HEIGHT)
}

pub fn on_ground(state: &PlayerState, room: &Room) -> bool {
    state.speed.y >= 0.0 && collide_at(room, state.position.x, state.position.y + 1)
}

pub fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value > target {
        (value - max_move).max(target)
    } else {
        (value + max_move).min(target)
    }
}

pub fn move_h(state: &mut PlayerState, room: &Room, amount: f32) {
    let new_remainder = state.remainder.x + amount;
    // C# Math.Round
    let movement_x = new_remainder.round_ties_even() as i32;
    state.remainder.x = new_remainder - (movement_x as f32);

    let step = movement_x.signum();
    for _ in 0..movement_x.abs() {
        if collide_at(room, state.position.x + step, state.position.y) {
            state.remainder.x = 0.0;
            state.speed.x = 0.0;
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
        if collide_at(room, state.position.x, state.position.y + step) {
            state.remainder.y = 0.0;
            state.speed.y = 0.0;
            return;
        }
        state.position.y += step;
    }
}

pub fn wall_at(state: &PlayerState, room: &Room, dir: i8) -> bool {
    collide_at(room, state.position.x + dir as i32, state.position.y)
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

pub fn can_dash(state: &PlayerState, dash_pressed: bool) -> bool {
    dash_pressed && state.dash_cooldown_timer <= 0.0 && state.dashes > 0
}

pub fn dash_begin(state: &mut PlayerState) {
    state.dashes -= 1;
    state.freeze_timer = DASH_FREEZE_TIME;
    state.dash_cooldown_timer = DASH_COOLDOWN;
    state.dash_refill_cooldown_timer = DASH_REFILL_COOLDOWN;
    state.wall_slide_timer = WALL_SLIDE_TIME;
    state.before_dash_speed = state.speed;
    state.speed = Vec2 { x: 0.0, y: 0.0 };
    state.dash_dir = Vec2 { x: 0.0, y: 0.0 };
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

pub fn dash_launch(state: &mut PlayerState, aim: Vec2) {
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
    let jump_pressed = input.jump && !state.jump_held;
    state.jump_held = input.jump;
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

    if ground {
        state.jump_grace_timer = JUMP_GRACE_TIME;
        state.wall_slide_timer = WALL_SLIDE_TIME;
        state.auto_jump = false;
    } else if state.jump_grace_timer > 0.0 {
        state.jump_grace_timer -= DELTA_TIME;
    }

    if state.dash_cooldown_timer > 0.0 {
        state.dash_cooldown_timer -= DELTA_TIME;
    }

    if state.dash_refill_cooldown_timer > 0.0 {
        state.dash_refill_cooldown_timer -= DELTA_TIME;
    } else if ground {
        state.dashes = MAX_DASHES;
    }

    if input.move_x != 0 {
        state.facing = input.move_x;
    }
    
    let aim = aim_vector(input, state.facing);

    if state.var_jump_timer > 0.0 {
        state.var_jump_timer -= DELTA_TIME;
    }

    match state.mode {
        Mode::Normal => {
            if can_dash(&state, dash_pressed) {
                dash_begin(&mut state);
            } else {
                normal_update(&mut state, room, input, ground, jump_pressed);
            }
        }

        Mode::DashStart => { 
            dash_launch(&mut state, aim);
        }
        
        Mode::Dashing { timer } => {
            if timer > 0.0 {
                state.mode = Mode::Dashing { timer: timer - DELTA_TIME };
            } else {
                dash_end(&mut state);
            }
        }
    }

    let (speed_x, speed_y) = (state.speed.x, state.speed.y);
    move_h(&mut state, room, speed_x * DELTA_TIME);
    move_v(&mut state, room, speed_y * DELTA_TIME);

    state
}

fn normal_update(state: &mut PlayerState, room: &Room, input: Input, ground: bool, jump_pressed: bool) {
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
    state.speed.x = approach(state.speed.x, target, max_move_x);

    if !ground {
        let max_fall = wall_slide_max_fall(state, room, input);
        let grav_mult = if state.speed.y.abs() < HALF_GRAV_THRESHOLD && (input.jump || state.auto_jump) {
            0.5
        } else {
            1.0
        };
        state.speed.y = approach(state.speed.y, max_fall, GRAVITY * grav_mult * DELTA_TIME);
    }

    if state.var_jump_timer > 0.0 {
        if input.jump || state.auto_jump {
            state.speed.y = state.speed.y.min(state.var_jump_speed);
        } else {
            state.var_jump_timer = 0.0;
        }
    }

    if jump_pressed && state.jump_grace_timer > 0.0 {
        jump(state, input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        move_h(&mut s, &room(), 0.25);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.remainder.x, 0.25);
    }

    #[test]
    fn move_h_rounds_to_whole_pixels() {
        let mut s = at_rest();
        move_h(&mut s, &room(), 0.75);
        assert_eq!(s.position.x, 1);
        assert_eq!(s.remainder.x, -0.25);

        move_h(&mut s, &room(), -2.0);
        assert_eq!(s.position.x, -1);
        assert_eq!(s.remainder.x, -0.25);
    }

    #[test]
    fn move_h_ties_round_to_even_like_csharp() {
        let mut s = at_rest();
        move_h(&mut s, &room(), 0.5);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.remainder.x, 0.5);

        move_h(&mut s, &room(), 1.0);
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
        move_h(&mut s, &room(), 1.5);
        assert_eq!(s.position.x, WALL_X);
        assert_eq!(s.remainder.x, 0.0);
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn move_h_reaching_wall_exactly_is_not_a_collision() {
        let mut s = at_rest();
        s.position.x = WALL_X - 1;
        s.speed.x = 90.0;
        move_h(&mut s, &room(), 1.25);
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
        move_h(&mut s, &room(), -3.0);
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
        dash_begin(&mut s);
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
        dash_begin(&mut s);
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(s.speed, Vec2 { x: 0.0, y: -DASH_SPEED });
        assert_eq!(s.dash_dir, Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(s.mode, Mode::Dashing { timer: DASH_TIME });
    }

    #[test]
    fn dash_launch_keeps_faster_speed_in_same_direction() {
        let mut s = standing();
        s.speed.x = 300.0;
        dash_begin(&mut s);
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 });
        assert_eq!(s.speed.x, 300.0);

        let mut s = standing();
        s.speed.x = -300.0;
        dash_begin(&mut s);
        dash_launch(&mut s, Vec2 { x: 1.0, y: 0.0 });
        assert_eq!(s.speed.x, DASH_SPEED);
    }

    #[test]
    fn dash_launch_up_drops_horizontal_speed() {
        let mut s = standing();
        s.speed.x = 300.0;
        dash_begin(&mut s);
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(s.speed.x, 0.0);
    }

    #[test]
    fn dash_launch_turns_to_face_horizontal_dashes() {
        let mut s = standing();
        dash_begin(&mut s);
        dash_launch(&mut s, Vec2 { x: -1.0, y: 0.0 });
        assert_eq!(s.facing, -1);

        let mut s = standing();
        s.facing = -1;
        dash_begin(&mut s);
        dash_launch(&mut s, Vec2 { x: 0.0, y: -1.0 });
        assert_eq!(s.facing, -1);
    }

    fn dashing(dir: Vec2) -> PlayerState {
        let mut s = standing();
        dash_begin(&mut s);
        dash_launch(&mut s, dir);
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
}
