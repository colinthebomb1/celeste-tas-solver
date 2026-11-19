use crate::geom::Vec2;

pub(crate) const MAX_RUN: f32 = 90.0;
pub(crate) const RUN_ACCEL: f32 = 1000.0;
pub(crate) const RUN_REDUCE: f32 = 400.0;
pub(crate) const DELTA_TIME: f32 = 0.0166667;
pub(crate) const GROUND_MULT: f32 = 1.0;
pub(crate) const AIR_MULT: f32 = 0.65;
pub(crate) const GRAVITY: f32 = 900.0;
pub(crate) const MAX_FALL: f32 = 160.0;
pub(crate) const FASTFALL_SPEED: f32 = 240.0;
pub(crate) const FASTFALL_ACCEL: f32 = 300.0;
pub(crate) const HALF_GRAV_THRESHOLD: f32 = 40.0;
pub(crate) const JUMP_SPEED: f32 = -105.0;
pub(crate) const JUMP_H_BOOST: f32 = 40.0;
pub(crate) const JUMP_HOLD_TIME: f32 = 0.2;
pub(crate) const COYOTE_TIME: f32 = 0.1;
pub(crate) const WALL_SLIDE_START_MAX: f32 = 20.0;
pub(crate) const WALL_SLIDE_TIME: f32 = 1.2;
pub(crate) const DASH_SPEED: f32 = 240.0;
pub(crate) const END_DASH_SPEED: f32 = 160.0;
pub(crate) const END_DASH_UP_MULT: f32 = 0.75;
pub(crate) const DASH_TIME: f32 = 0.15;
pub(crate) const DASH_COOLDOWN: f32 = 0.2;
pub(crate) const DASH_REFILL_COOLDOWN: f32 = 0.1;
pub(crate) const DASH_FREEZE_TIME: f32 = 0.05;
pub(crate) const MAX_DASHES: u8 = 1;
pub(crate) const TWO_DASHES: u8 = 2;
// a crystal also works below this much stamina
pub(crate) const CLIMB_TIRED_THRESHOLD: f32 = 20.0;
pub(crate) const SUPER_SPEED: f32 = 260.0;
pub(crate) const CLIMB_MAX_STAMINA: f32 = 110.0;
pub(crate) const CLIMB_JUMP_COST: f32 = 110.0 / 4.0;
pub(crate) const WALL_KICK_REACH: i32 = 3;
pub(crate) const RETENTION_TIME: f32 = 0.06;
pub(crate) const WALL_KICK_SPEED: f32 = MAX_RUN + JUMP_H_BOOST;
pub(crate) const WALLBOOST_TIME: f32 = 0.2;
// how far in front of her a wall can be grabbed
pub(crate) const CLIMB_CHECK_DIST: i32 = 2;
// grabbing reaches this far up for a wall that starts above her
pub(crate) const CLIMB_UP_CHECK_DIST: i32 = 2;
pub(crate) const CLIMB_GRAB_Y_MULT: f32 = 0.2;
pub(crate) const CLIMB_NO_MOVE_TIME: f32 = 0.1;
pub(crate) const CLIMB_UP_SPEED: f32 = -45.0;
pub(crate) const CLIMB_DOWN_SPEED: f32 = 80.0;
pub(crate) const CLIMB_SLIP_SPEED: f32 = 30.0;
pub(crate) const CLIMB_ACCEL: f32 = 900.0;
pub(crate) const CLIMB_UP_COST: f32 = 45.454544;
pub(crate) const CLIMB_STILL_COST: f32 = 10.0;
// a ceiling this close stops her climbing over a ledge
pub(crate) const CLIMBHOP_BLOCK_DIST: i32 = 6;
pub(crate) const CLIMBHOP_Y: f32 = -120.0;
pub(crate) const CLIMBHOP_X: f32 = 100.0;
pub(crate) const CLIMBHOP_FORCE_TIME: f32 = 0.2;
pub(crate) const UPWARD_CORNER_CORRECTION: i32 = 4;
// a dash with no sideways speed reaches a pixel further
pub(crate) const DASH_UPWARD_CORNER_CORRECTION: i32 = 5;
// sideways speed below this counts as none
pub(crate) const CORNER_CORRECTION_SPEED_SLACK: f32 = 0.01;
pub(crate) const DASH_CORNER_CORRECTION: i32 = 4;
pub(crate) const DASH_ATTACK_TIME: f32 = 0.3;
pub(crate) const WALLBOUNCE_SPEED_X: f32 = MAX_RUN + JUMP_H_BOOST * 2.0;
pub(crate) const WALLBOUNCE_SPEED_Y: f32 = -160.0;
pub(crate) const WALLBOUNCE_JUMP_TIME: f32 = 0.25;
pub(crate) const WALLBOUNCE_REACH: i32 = 5;
pub(crate) const WALL_KICK_FORCE_TIME: f32 = 0.16;
pub(crate) const CEILING_JUMP_GRACE: f32 = 0.05;
// normal hitbox size in pixels for collisions
pub(crate) const HITBOX_WIDTH: i32 = 8;
pub(crate) const HITBOX_HEIGHT: i32 = 11;
// hazards use the hurtbox
pub(crate) const HURTBOX_HEIGHT: i32 = 9;
// both boxes are shorter when crouching
pub(crate) const CROUCH_HITBOX_HEIGHT: i32 = 6;
pub(crate) const CROUCH_HURTBOX_HEIGHT: i32 = 4;
pub(crate) const CROUCH_FRICTION: f32 = 500.0;
pub(crate) const JUMP_THRU_ASSIST_SPEED: f32 = -40.0;
pub(crate) const DASH_V_FLOOR_SNAP_DIST: i32 = 3;
// lift speed from moving blocks
pub(crate) const LIFT_X_CAP: f32 = 250.0;
pub(crate) const LIFT_Y_CAP: f32 = -130.0;
pub(crate) const LIFT_SPEED_GRACE_TIME: f32 = 0.16;
pub(crate) const SPRING_SPEED: f32 = -185.0;
pub(crate) const SPRING_JUMP_TIME: f32 = 0.2;
pub(crate) const NO_LIFT: Vec2 = Vec2 { x: 0.0, y: 0.0 };
pub(crate) const DASH_H_JUMP_THRU_NUDGE: i32 = 6;
pub(crate) const ULTRA_SPEED_MULT: f32 = 1.2;
pub(crate) const HYPER_X_MULT: f32 = 1.25;
pub(crate) const HYPER_Y_MULT: f32 = 0.5;
