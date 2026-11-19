use crate::geom::Vec2;

// one frame, in seconds
pub(crate) const DELTA_TIME: f32 = 0.0166667;

// ---- running
pub(crate) const MAX_RUN: f32 = 90.0;
pub(crate) const RUN_ACCEL: f32 = 1000.0;
pub(crate) const RUN_DECEL: f32 = 400.0;
pub(crate) const GROUND_ACCEL_MULT: f32 = 1.0;
pub(crate) const AIR_ACCEL_MULT: f32 = 0.65;
pub(crate) const CROUCH_FRICTION: f32 = 500.0;

// ---- falling
pub(crate) const GRAVITY: f32 = 900.0;
pub(crate) const MAX_FALL: f32 = 160.0;
pub(crate) const FASTFALL_SPEED: f32 = 240.0;
pub(crate) const FASTFALL_ACCEL: f32 = 300.0;
pub(crate) const HALF_GRAVITY_THRESHOLD: f32 = 40.0;

// ---- jumping
pub(crate) const JUMP_SPEED: f32 = -105.0;
pub(crate) const JUMP_X_BOOST: f32 = 40.0;
pub(crate) const JUMP_HOLD_TIME: f32 = 0.2;
pub(crate) const COYOTE_TIME: f32 = 0.1;
pub(crate) const CEILING_JUMP_GRACE: f32 = 0.05;

// ---- wall slides, wall kicks and wallbounces
pub(crate) const WALL_SLIDE_START_MAX: f32 = 20.0;
pub(crate) const WALL_SLIDE_TIME: f32 = 1.2;
pub(crate) const RETENTION_TIME: f32 = 0.06;
pub(crate) const WALL_KICK_SPEED: f32 = MAX_RUN + JUMP_X_BOOST;
pub(crate) const WALL_KICK_REACH: i32 = 3;
pub(crate) const WALL_KICK_FORCE_TIME: f32 = 0.16;
pub(crate) const WALLBOOST_TIME: f32 = 0.2;
pub(crate) const WALLBOUNCE_SPEED_X: f32 = MAX_RUN + JUMP_X_BOOST * 2.0;
pub(crate) const WALLBOUNCE_SPEED_Y: f32 = -160.0;
pub(crate) const WALLBOUNCE_JUMP_TIME: f32 = 0.25;
pub(crate) const WALLBOUNCE_REACH: i32 = 5;

// ---- dashing
// what a refill gives her. a two-dash crystal gives one more
pub(crate) const NORMAL_DASHES: u8 = 1;
pub(crate) const TWO_DASHES: u8 = 2;
pub(crate) const DASH_SPEED: f32 = 240.0;
pub(crate) const DASH_TIME: f32 = 0.15;
pub(crate) const DASH_END_SPEED: f32 = 160.0;
pub(crate) const DASH_END_UP_MULT: f32 = 0.75;
pub(crate) const DASH_COOLDOWN: f32 = 0.2;
pub(crate) const DASH_REFILL_COOLDOWN: f32 = 0.1;
pub(crate) const DASH_FREEZE_TIME: f32 = 0.05;
pub(crate) const DASH_ATTACK_TIME: f32 = 0.3;
// a sideways dash this close above the floor snaps down onto it
pub(crate) const DASH_FLOOR_SNAP: i32 = 3;
pub(crate) const DASH_JUMP_THRU_NUDGE: i32 = 6;

// ---- supers, hypers and ultras
pub(crate) const SUPER_SPEED: f32 = 260.0;
pub(crate) const HYPER_X_MULT: f32 = 1.25;
pub(crate) const HYPER_Y_MULT: f32 = 0.5;
pub(crate) const ULTRA_SPEED_MULT: f32 = 1.2;

// ---- corner correction
pub(crate) const UPWARD_CORNER_CORRECTION: i32 = 4;
// a dash with no sideways speed reaches a pixel further
pub(crate) const DASH_UPWARD_CORNER_CORRECTION: i32 = 5;
// sideways speed below this counts as none
pub(crate) const CORNER_CORRECTION_SPEED_SLACK: f32 = 0.01;
pub(crate) const DASH_CORNER_CORRECTION: i32 = 4;

// ---- grabbing and climbing
pub(crate) const CLIMB_MAX_STAMINA: f32 = 110.0;
// a crystal also works below this much stamina
pub(crate) const CLIMB_TIRED_THRESHOLD: f32 = 20.0;
pub(crate) const CLIMB_JUMP_COST: f32 = 110.0 / 4.0;
pub(crate) const CLIMB_UP_COST: f32 = 45.454544;
pub(crate) const CLIMB_STILL_COST: f32 = 10.0;
// how far in front of her a wall can be grabbed
pub(crate) const GRAB_REACH: i32 = 2;
// grabbing reaches this far up for a wall that starts above her
pub(crate) const GRAB_REACH_UP: i32 = 2;
pub(crate) const GRAB_Y_MULT: f32 = 0.2;
// she can't climb up or down for this long after grabbing
pub(crate) const GRAB_PAUSE_TIME: f32 = 0.1;
pub(crate) const CLIMB_UP_SPEED: f32 = -45.0;
pub(crate) const CLIMB_DOWN_SPEED: f32 = 80.0;
pub(crate) const CLIMB_SLIP_SPEED: f32 = 30.0;
pub(crate) const CLIMB_ACCEL: f32 = 900.0;
// a ceiling this close stops her climbing over a ledge
pub(crate) const CLIMBHOP_BLOCK_DIST: i32 = 6;
// so do spikes this far ahead of her
pub(crate) const CLIMBHOP_SPIKE_REACH: i32 = 8;
pub(crate) const CLIMBHOP_Y: f32 = -120.0;
pub(crate) const CLIMBHOP_X: f32 = 100.0;
pub(crate) const CLIMBHOP_FORCE_TIME: f32 = 0.2;

// ---- her boxes, in pixels
// collisions use the hitbox
pub(crate) const HITBOX_WIDTH: i32 = 8;
pub(crate) const HITBOX_HEIGHT: i32 = 11;
// hazards use the hurtbox
pub(crate) const HURTBOX_HEIGHT: i32 = 9;
// both boxes are shorter when crouching
pub(crate) const CROUCH_HITBOX_HEIGHT: i32 = 6;
pub(crate) const CROUCH_HURTBOX_HEIGHT: i32 = 4;

// ---- liftboost from moving blocks
// lift speed from moving blocks
pub(crate) const LIFT_X_CAP: f32 = 250.0;
pub(crate) const LIFT_Y_CAP: f32 = -130.0;
pub(crate) const LIFT_SPEED_GRACE_TIME: f32 = 0.16;
pub(crate) const NO_LIFT: Vec2 = Vec2 { x: 0.0, y: 0.0 };

// ---- springs and jump-throughs
pub(crate) const SPRING_SPEED: f32 = -185.0;
pub(crate) const SPRING_JUMP_TIME: f32 = 0.2;
pub(crate) const JUMP_THRU_ASSIST_SPEED: f32 = -40.0;
// spikes this close above her stop the lift through a jump-through
pub(crate) const JUMP_THRU_ASSIST_SPIKE_REACH: i32 = 2;
