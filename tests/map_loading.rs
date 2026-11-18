use celeste_tas_solver::level::map::load_room;
use celeste_tas_solver::player::IVec2;
use celeste_tas_solver::entities::{DashBlock, JumpThru, Spikes, Spring};
use celeste_tas_solver::level::room::{Direction, Rect};

mod common;

#[test]
fn first_room_matches_the_game() {
    let Some(room) = common::room("lvl_1") else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    assert_eq!(room.origin, IVec2 { x: 0, y: 0 });
    assert_eq!((room.width, room.height), (40, 23));
    assert!(room.tile_solid(4, 18));
    assert!(!room.tile_solid(5, 18));
    assert!(room.tile_solid(10, 16));
    assert!(!room.tile_solid(10, 15));
    assert_eq!(room.name, "lvl_1");
    assert_eq!(room.bounds, Rect { left: 0, top: 0, width: 320, height: 180 });
    assert_eq!(room.bottom(), 180);
    assert_eq!(room.spikes.len(), 5);
    assert!(room.spikes.contains(&Spikes::new(Direction::Up, 40, 168, 40)));
    assert!(room.spikes.contains(&Spikes::new(Direction::Left, 272, 80, 16)));
}

#[test]
fn every_room_in_1a_loads() {
    let Some(map) = common::map_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let levels = &map.child("levels").unwrap().children;
    assert_eq!(levels.len(), 38);
    for level in levels {
        let name = level.text("name").unwrap();
        load_room(&map, name).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    }
}

#[test]
fn rooms_load_their_jump_thrus() {
    let Some(chapter) = common::chapter_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let room = |name: &str| &chapter.rooms[chapter.room_named(name).unwrap()];

    assert!(room("lvl_1").jump_thrus.is_empty());
    assert_eq!(room("lvl_3").jump_thrus, vec![JumpThru::new(512, -200, 32)]);
    assert_eq!(chapter.rooms.iter().map(|room| room.jump_thrus.len()).sum::<usize>(), 77);
}

#[test]
fn rooms_load_their_dash_blocks() {
    let Some(chapter) = common::chapter_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let with_dash_blocks: Vec<&str> = chapter
        .rooms
        .iter()
        .filter(|room| !room.dash_blocks.is_empty())
        .map(|room| room.name.as_str())
        .collect();

    assert_eq!(with_dash_blocks, ["lvl_5", "lvl_6", "lvl_6c", "lvl_7z"]);
    let lvl_6 = &chapter.rooms[chapter.room_named("lvl_6").unwrap()];
    assert_eq!(lvl_6.dash_blocks, vec![DashBlock::new(1544, -1096, 16, 24)]);
}

#[test]
fn rooms_load_their_springs() {
    let Some(chapter) = common::chapter_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let lvl_5 = &chapter.rooms[chapter.room_named("lvl_5").unwrap()];

    assert_eq!(lvl_5.springs, vec![Spring::new(1320, -896)]);
    assert_eq!(chapter.rooms.iter().map(|room| room.springs.len()).sum::<usize>(), 13);
}

#[test]
fn rooms_load_their_zip_movers() {
    let Some(chapter) = common::chapter_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    let lvl_5 = &chapter.rooms[chapter.room_named("lvl_5").unwrap()];

    assert_eq!(lvl_5.zip_movers.len(), 1);
    let zip = &lvl_5.zip_movers[0];
    assert_eq!((zip.start, zip.target), (IVec2 { x: 1128, y: -760 }, IVec2 { x: 1128, y: -840 }));
    assert_eq!((zip.width, zip.height), (32, 16));
}

#[test]
fn chapter_1a_rooms_tile_the_route() {
    let Some(chapter) = common::chapter_1a() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    assert_eq!(chapter.rooms.len(), 38);

    let lvl_6a = &chapter.rooms[chapter.room_named("lvl_6a").unwrap()];
    assert_eq!(lvl_6a.bounds, Rect { left: 1616, top: -1216, width: 392, height: 192 });

    // just above the first room is the second, just right of lvl_6 is lvl_6a
    assert_eq!(chapter.room_at(254.0, -10.0), chapter.room_named("lvl_2"));
    assert_eq!(chapter.room_at(1620.0, -1176.0), chapter.room_named("lvl_6a"));
    assert_eq!(chapter.room_at(-50.0, 50.0), None);
}
