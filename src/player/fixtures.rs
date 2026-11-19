// rooms, inputs and starting positions shared by the player tests
use super::constants::*;
use super::dash::{aim_vector, dash_begin, dash_launch};
use super::movement::set_lift_speed;
use super::state::{Frame, Input, Player, State};
use super::step::step;
use crate::entities::Spikes;
use crate::geom::{Direction, IVec2, Vec2};
use crate::level::room::Room;

pub(crate) const FLOOR_Y: i32 = 144;
pub(crate) const LEDGE_X: i32 = 43;
pub(crate) const WALL_X: i32 = 76;

// a frame with these buttons held and nothing newly pressed
pub(crate) fn frame(input: Input, ground: bool) -> Frame {
    Frame {
        input,
        ground,
        ..Frame::default()
    }
}

pub(crate) fn room() -> Room {
    let text = "00000000001\n".repeat(18) + &"11111000001\n".repeat(5);
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

pub(crate) fn at_rest() -> Player {
    Player::new(
        IVec2 { x: 0, y: 64 },
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 0.0, y: 0.0 },
    )
}

pub(crate) fn standing() -> Player {
    Player::new(
        IVec2 { x: 0, y: FLOOR_Y },
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 0.0, y: 0.0 },
    )
}

pub(crate) fn falling_against_wall() -> Player {
    let mut s = at_rest();
    s.position.x = WALL_X;
    s.speed.y = 100.0;
    s
}

pub(crate) fn aim(move_x: i8, move_y: i8, facing: i8) -> Vec2 {
    aim_vector(
        Input {
            move_x,
            move_y,
            ..Default::default()
        },
        facing,
    )
}

pub(crate) fn mid_dash(dir: Vec2) -> Player {
    let mut s = standing();
    let room = room();
    dash_begin(&mut s, &room, &room.initial, frame(Input::default(), true));
    dash_launch(&mut s, dir, true);
    s
}

pub(crate) fn player_at(x: i32, y: i32, speed_x: f32, speed_y: f32) -> Player {
    let mut s = at_rest();
    s.position = IVec2 { x, y };
    s.speed = Vec2 {
        x: speed_x,
        y: speed_y,
    };
    s
}

// spikes pointing up, with their base along y
pub(crate) fn up_spikes(x: i32, y: i32, width: i32) -> Spikes {
    Spikes::new(Direction::Up, x, y, width)
}

pub(crate) fn spiked_floor() -> Room {
    let mut room = room();
    room.spikes.push(up_spikes(0, FLOOR_Y, 40));
    room
}

pub(crate) fn spiked_platform_room() -> Room {
    let mut room = platform_room();
    room.spikes.push(up_spikes(40, PLATFORM_Y, 40));
    room
}

pub(crate) fn dashing_right() -> Player {
    let mut s = standing();
    s.state = State::Dash { timer: DASH_TIME };
    s.dash.dir = Vec2 { x: 1.0, y: 0.0 };
    s.speed = Vec2 {
        x: DASH_SPEED,
        y: 0.0,
    };
    s.coyote_timer = COYOTE_TIME;
    s
}

pub(crate) fn dash_for(start: Player, frames: usize, input: Input) -> Player {
    let room = room();
    let mut s = start;
    for _ in 0..frames {
        s = step(s, input, &room, &room.initial);
    }
    s
}

pub(crate) const DASH_RIGHT: Input = Input {
    move_x: 1,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: true,
    grab: false,
    demodash: false,
};
pub(crate) const JUMP_RIGHT: Input = Input {
    move_x: 1,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: false,
    demodash: false,
};

pub(crate) const JUMP_1: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: false,
    demodash: false,
};
pub(crate) const JUMP_2: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: true,
    dash: false,
    grab: false,
    demodash: false,
};
pub(crate) const JUMP_BOTH: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: true,
    dash: false,
    grab: false,
    demodash: false,
};

pub(crate) fn left_wall_room() -> Room {
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &"10000000000\n".repeat(23))
}

// flush against the left wall
pub(crate) const LEFT_WALL_X: i32 = 12;

pub(crate) const JUMP_GRAB: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: true,
    demodash: false,
};

pub(crate) fn retaining(x: i32, speed_x: f32, retained: f32) -> Player {
    let mut s = player_at(x, 100, speed_x, 0.0);
    s.wall.retained_speed = retained;
    s.wall.retention_timer = 0.03;
    s
}

// an 8 px high tunnel from x = 24 rightwards: she fits crouching, not standing
pub(crate) fn tunnel_room() -> Room {
    let text =
        "00000000000\n".repeat(16) + "00011111111\n" + "00000000000\n" + &"11111111111\n".repeat(5);
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

pub(crate) fn crouching_at(x: i32, y: i32) -> Player {
    let mut s = player_at(x, y, 0.0, 0.0);
    s.crouching = true;
    s
}

pub(crate) const DOWN: Input = Input {
    move_x: 0,
    move_y: 1,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    demodash: false,
};

pub(crate) fn launching() -> Player {
    let mut s = standing();
    s.state = State::DashStart;
    s
}

pub(crate) fn wallboosting() -> Player {
    let mut s = player_at(40, 100, 0.0, -100.0);
    s.wall.boost_dir = -1;
    s.wall.boost_timer = 0.1;
    s.stamina = 50.0;
    s
}

pub(crate) const LEFT: Input = Input {
    move_x: -1,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    demodash: false,
};

// a ceiling block spanning x 16..48, y 40..48
pub(crate) fn ceiling_room() -> Room {
    let text = "00000000000\n".repeat(5) + "00111100000\n" + &"00000000000\n".repeat(17);
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}

// her head touches the ceiling block's underside at this y
pub(crate) const UNDER_CEILING_Y: i32 = 59;

pub(crate) fn rising_at(x: i32, speed_x: f32) -> Player {
    player_at(x, UNDER_CEILING_Y, speed_x, -240.0)
}

pub(crate) fn dashing_at(x: i32, y: i32, speed_x: f32, speed_y: f32) -> Player {
    let mut s = player_at(x, y, speed_x, speed_y);
    s.state = State::Dash { timer: 0.1 };
    s
}

// flush against the right face of the floor block, in the pit
pub(crate) const PIT_X: i32 = 44;

pub(crate) fn air_dashing_down_at(x: i32, speed_x: f32) -> Player {
    dashing_at(x, FLOOR_Y - 1, speed_x, 240.0)
}

pub(crate) const JUMP: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: true,
    jump2: false,
    dash: false,
    grab: false,
    demodash: false,
};

// just after a dash straight up
pub(crate) fn bouncing_at(x: i32) -> Player {
    let mut s = player_at(x, 100, 0.0, -120.0);
    s.dash.dir = Vec2 { x: 0.0, y: -1.0 };
    s.dash.attack_timer = 0.1;
    s
}

// the usual room with a platform over the pit, spanning x 40..80 with its top at y 120
pub(crate) const PLATFORM_Y: i32 = 120;

pub(crate) fn platform_room() -> Room {
    let mut room = room();
    room.jump_thrus
        .push(crate::entities::JumpThru::new(40, PLATFORM_Y, 40));
    room
}

pub(crate) fn exact_y(s: &Player) -> f32 {
    s.position.y as f32 + s.subpixel.y
}

pub(crate) const DEMODASH: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: false,
    demodash: true,
};

pub(crate) fn lifted(mut s: Player, x: f32, y: f32) -> Player {
    set_lift_speed(&mut s, Vec2 { x, y });
    s
}

// the usual room with a spring on the floor, its base centred at x = 20
pub(crate) fn spring_room() -> Room {
    let mut room = room();
    room.springs.push(crate::entities::Spring::new(20, FLOOR_Y));
    room
}

pub(crate) fn falling(speed_y: f32) -> Player {
    player_at(60, 20, 0.0, speed_y)
}

pub(crate) const GRAB: Input = Input {
    move_x: 0,
    move_y: 0,
    jump: false,
    jump2: false,
    dash: false,
    grab: true,
    demodash: false,
};
pub(crate) const GRAB_UP: Input = Input { move_y: -1, ..GRAB };
pub(crate) const GRAB_DOWN: Input = Input { move_y: 1, ..GRAB };

// holding the wall at WALL_X, free to move along it
pub(crate) fn climbing(y: i32) -> Player {
    let mut s = player_at(WALL_X, y, 0.0, 0.0);
    s.facing = 1;
    s.state = State::Climb;
    s
}

// a wall on the right, solid only in these rows. tiles are 8 px
pub(crate) fn partial_wall(solid_rows: std::ops::Range<usize>) -> Room {
    let text: String = (0..23)
        .map(|row| {
            if solid_rows.contains(&row) {
                "00000000001\n"
            } else {
                "00000000000\n"
            }
        })
        .collect();
    Room::from_tiles(IVec2 { x: 0, y: 0 }, &text)
}
