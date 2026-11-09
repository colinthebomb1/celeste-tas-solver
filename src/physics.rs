use crate::Vec2;

pub const MAX_RUN: f32 = 90.0;
pub const RUN_ACCEL: f32 = 1000.0;
pub const RUN_REDUCE: f32 = 400.0;
pub const DELTA_TIME: f32 = 0.0166667;

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
    // pixels per second
    pub speed: Vec2,
}

// on frame of input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Input {
    // -1 = left, 0 = neither, 1 = right
    pub move_x: i8,
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
    let movement = new_remainder.round_ties_even() as i32;

    if movement != 0 {
        state.position.x += movement;
    }

    state.remainder.x = new_remainder - (movement as f32);
}

pub fn step(mut state: PlayerState, input: Input) -> PlayerState {
    let move_x = input.move_x as f32;
    let target = MAX_RUN * move_x;

    let max_move = if state.speed.x.abs() > MAX_RUN && (state.speed.x.signum() as i8) == input.move_x {
        RUN_REDUCE * DELTA_TIME
    } else {
        RUN_ACCEL * DELTA_TIME
    };
    state.speed.x = approach(state.speed.x, target, max_move);
    let speed_x = state.speed.x;

    move_h(&mut state, speed_x * DELTA_TIME);
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_rest() -> PlayerState {
        PlayerState {
            position: IVec2 { x: 0, y: 0 },
            remainder: Vec2 { x: 0.0, y: 0.0 },
            speed: Vec2 { x: 0.0, y: 0.0 },
        }
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
}
