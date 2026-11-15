use celeste_tas_solver::map::load_room;
use celeste_tas_solver::physics::IVec2;

mod common;

#[test]
fn first_room_matches_the_game() {
    let Some(room) = common::first_room() else {
        eprintln!("skipped. Celeste install could not be found, try setting CELESTE_DIR");
        return;
    };
    assert_eq!(room.origin, IVec2 { x: 0, y: 0 });
    assert_eq!((room.width, room.height), (40, 23));
    assert!(room.is_solid(4, 18));
    assert!(!room.is_solid(5, 18));
    assert!(room.is_solid(10, 16));
    assert!(!room.is_solid(10, 15));
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
