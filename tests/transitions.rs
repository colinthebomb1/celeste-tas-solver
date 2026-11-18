use celeste_tas_solver::geom::IVec2;
use celeste_tas_solver::geom::Vec2;
use celeste_tas_solver::level::transition::{
    TRANSITION_FRAMES, before_transition, find_transition, finish_transition,
};
use celeste_tas_solver::player::Player;

mod common;

struct Row {
    x: f64,
    y: f64,
    speed_x: f64,
    speed_y: f64,
    // "lvl_3b" for a trace row in room [3b]
    room: String,
    no_control: bool,
}

fn pair(text: &str) -> (f64, f64) {
    let (a, b) = text.split_once(", ").expect("bad pair");
    (a.parse().unwrap(), b.parse().unwrap())
}

fn parse(line: &str) -> Row {
    let fields: Vec<&str> = line.split('\t').collect();
    let (x, y) = pair(fields[4]);
    let (speed_x, speed_y) = pair(fields[5]);
    let room = fields
        .iter()
        .find(|field| field.starts_with('[') && field.ends_with(']'))
        .expect("no room column");
    Row {
        x,
        y,
        speed_x,
        speed_y,
        room: format!("lvl_{}", &room[1..room.len() - 1]),
        no_control: fields[7].contains("NoControl"),
    }
}

fn player(row: &Row) -> Player {
    let position = IVec2 {
        x: row.x.round_ties_even() as i32,
        y: row.y.round_ties_even() as i32,
    };
    let subpixel = Vec2 {
        x: (row.x - position.x as f64) as f32,
        y: (row.y - position.y as f64) as f32,
    };
    Player::new(
        position,
        subpixel,
        Vec2 {
            x: row.speed_x as f32,
            y: row.speed_y as f32,
        },
    )
}

// every room change in the community TAS: it starts on the frame the game starts it, goes to the
// room the game goes to, and leaves her where the game does. no physics is simulated here
#[test]
fn transitions_match_the_trace() {
    let Some(chapter) = common::chapter_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let text = include_str!("traces/community/1A_full.txt");
    // the last rows are the level exit and have no position
    let rows: Vec<Row> = text
        .lines()
        .skip(1)
        .take_while(|line| line.split('\t').nth(8).is_some())
        .map(parse)
        .collect();

    let mut transitions = 0;
    for i in 1..rows.len() {
        let row = &rows[i];
        if row.no_control && rows[i - 1].no_control {
            continue;
        }

        let room = chapter
            .room_named(&row.room)
            .expect("unknown room in the trace");
        let mut player = player(row);
        let found = find_transition(&player, &chapter, room);

        if !row.no_control {
            // the trace doesn't say whether she is crouching, and a crouching player leaves later
            let mut crouching = player;
            crouching.crouching = true;
            assert!(
                found.is_none() || find_transition(&crouching, &chapter, room).is_none(),
                "row {i}: a transition out of {} that the game didn't make",
                row.room
            );
            continue;
        }

        let end = &rows[i + TRANSITION_FRAMES];
        let transition =
            found.unwrap_or_else(|| panic!("row {i}: no transition out of {}", row.room));
        assert_eq!(
            chapter.rooms[transition.to].name, end.room,
            "row {i}: wrong room"
        );

        before_transition(&mut player, transition.direction);
        finish_transition(&mut player, &chapter, transition);

        assert_eq!(
            (player.position.x as f64, player.position.y as f64),
            (end.x, end.y),
            "row {i}: wrong position"
        );
        assert_eq!(
            (player.speed.x as f64, player.speed.y as f64),
            (end.speed_x, end.speed_y),
            "row {i}: wrong speed"
        );
        assert_eq!(player.subpixel, Vec2 { x: 0.0, y: 0.0 });
        transitions += 1;
    }

    assert_eq!(transitions, 19);
}
