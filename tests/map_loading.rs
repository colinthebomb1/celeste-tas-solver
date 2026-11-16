use celeste_tas_solver::map::load_room;
use celeste_tas_solver::physics::IVec2;
use celeste_tas_solver::room::{Direction, Rect, Spikes};

mod common;

#[test]
fn first_room_matches_the_game() {
    let Some(room) = common::room("lvl_1") else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    assert_eq!(room.origin, IVec2 { x: 0, y: 0 });
    assert_eq!((room.width, room.height), (40, 23));
    assert!(room.is_solid(4, 18));
    assert!(!room.is_solid(5, 18));
    assert!(room.is_solid(10, 16));
    assert!(!room.is_solid(10, 15));
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
