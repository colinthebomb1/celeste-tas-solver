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

// Temporary until room collision: the floor of 1A's first room.
pub const FLOOR_Y: i32 = 144;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
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
        }
    }
}

// one frame of input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Input {
    // -1 = left, 0 = neither, 1 = right
    pub move_x: i8,
    pub jump: bool,
}

pub fn on_ground(state: &PlayerState) -> bool {
    let speed_y = state.speed.y;
    let position_y = state.position.y;

    if speed_y >= 0.0 && position_y  == FLOOR_Y {
        return true;
    }
    
    false
}

pub fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value > target {
        (value - max_move).max(target)
    } else {
        (value + max_move).min(target)
    }
}

pub fn move_h(state: &mut PlayerState, amount: f32) {
    let new_remainder = state.remainder.x + amount;
    // C# Math.Round
    let movement_x = new_remainder.round_ties_even() as i32;

    state.position.x += movement_x;
    state.remainder.x = new_remainder - (movement_x as f32);
}

pub fn move_v(state: &mut PlayerState, amount: f32) {
    let new_remainder = state.remainder.y + amount;
    // C# Math.Round
    let movement_y = new_remainder.round_ties_even() as i32;

    let target_y = state.position.y + movement_y;

    if target_y > FLOOR_Y {
        state.position.y = FLOOR_Y;
        state.remainder.y = 0.0;
        state.speed.y = 0.0;
        return;
    }

    state.position.y = target_y;
    state.remainder.y = new_remainder - (movement_y as f32);
}

pub fn jump(state: &mut PlayerState, input: Input) {
    state.jump_grace_timer = 0.0;
    state.var_jump_timer = VAR_JUMP_TIME;
    state.speed.x += JUMP_H_BOOST * input.move_x as f32;
    state.speed.y = JUMP_SPEED;
    state.var_jump_speed = state.speed.y
}

pub fn step(mut state: PlayerState, input: Input) -> PlayerState {
    let ground = on_ground(&state);

    if ground {
        state.jump_grace_timer = JUMP_GRACE_TIME;
    } else if state.jump_grace_timer > 0.0 {
        state.jump_grace_timer -= DELTA_TIME;
    }

    if state.var_jump_timer > 0.0 {
        state.var_jump_timer -= DELTA_TIME;
    }
    
    let jump_pressed = input.jump && !state.jump_held;
    state.jump_held = input.jump;

    let mult = if ground {
        GROUND_MULT
    } else {
        AIR_MULT
    };
    let move_x = input.move_x as f32;
    let target = MAX_RUN * move_x;

    let max_move_x = if state.speed.x.abs() > MAX_RUN && (state.speed.x.signum() as i8) == input.move_x {
        RUN_REDUCE * mult * DELTA_TIME 
    } else {
        RUN_ACCEL * mult * DELTA_TIME 
    };
    state.speed.x = approach(state.speed.x, target, max_move_x);

    if !ground {
        let grav_mult = if state.speed.y.abs() < HALF_GRAV_THRESHOLD && input.jump {
            0.5
        } else {
            1.0
        };
        state.speed.y = approach(state.speed.y, MAX_FALL, GRAVITY * grav_mult * DELTA_TIME);
    }

    if state.var_jump_timer > 0.0 {
        if input.jump {
            state.speed.y = state.speed.y.min(state.var_jump_speed);
        } else {
            state.var_jump_timer = 0.0;
        }
    }

    if jump_pressed && state.jump_grace_timer > 0.0 {
        jump(&mut state, input);
    }
    
    let (speed_x, speed_y) = (state.speed.x, state.speed.y);
    move_h(&mut state, speed_x * DELTA_TIME);
    move_v(&mut state, speed_y * DELTA_TIME);

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_rest() -> PlayerState {
        PlayerState::new(IVec2 { x: 0, y: 0 }, Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 0.0, y: 0.0 })
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
        move_h(&mut s, 0.25);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.remainder.x, 0.25);
    }

    #[test]
    fn move_h_rounds_to_whole_pixels() {
        let mut s = at_rest();
        move_h(&mut s, 0.75);
        assert_eq!(s.position.x, 1);
        assert_eq!(s.remainder.x, -0.25);

        move_h(&mut s, -2.0);
        assert_eq!(s.position.x, -1);
        assert_eq!(s.remainder.x, -0.25);
    }

    #[test]
    fn move_h_ties_round_to_even_like_csharp() {
        let mut s = at_rest();
        move_h(&mut s, 0.5);
        assert_eq!(s.position.x, 0);
        assert_eq!(s.remainder.x, 0.5);

        move_h(&mut s, 1.0);
        assert_eq!(s.position.x, 2);
        assert_eq!(s.remainder.x, -0.5);
    }

    #[test]
    fn on_ground_when_standing_on_floor() {
        assert!(on_ground(&standing()));
    }

    #[test]
    fn not_on_ground_above_floor() {
        let mut s = standing();
        s.position.y -= 1;
        assert!(!on_ground(&s));
    }

    #[test]
    fn not_on_ground_while_moving_up() {
        let mut s = standing();
        s.speed.y = -10.0;
        assert!(!on_ground(&s));
    }

    #[test]
    fn move_v_moves_up_freely() {
        let mut s = standing();
        move_v(&mut s, -1.75);
        assert_eq!(s.position.y, FLOOR_Y - 2);
        assert_eq!(s.remainder.y, 0.25);
    }

    #[test]
    fn move_v_stops_at_floor() {
        let mut s = standing();
        s.position.y = FLOOR_Y - 2;
        s.remainder.y = 0.3;
        s.speed.y = 160.0;
        move_v(&mut s, 2.6);
        assert_eq!(s.position.y, FLOOR_Y);
        assert_eq!(s.remainder.y, 0.0);
        assert_eq!(s.speed.y, 0.0);
    }

    #[test]
    fn jump_sets_speed_and_timers() {
        let mut s = standing();
        s.jump_grace_timer = JUMP_GRACE_TIME;
        jump(&mut s, Input { move_x: 0, jump: true });
        assert_eq!(s.speed.y, JUMP_SPEED);
        assert_eq!(s.var_jump_speed, JUMP_SPEED);
        assert_eq!(s.var_jump_timer, VAR_JUMP_TIME);
        assert_eq!(s.jump_grace_timer, 0.0);
    }

    #[test]
    fn jump_adds_horizontal_boost() {
        let mut s = standing();
        s.speed.x = 90.0;
        jump(&mut s, Input { move_x: 1, jump: true });
        assert_eq!(s.speed.x, 90.0 + JUMP_H_BOOST);
    }
}
