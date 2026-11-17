use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::physics::IVec2;
use crate::entities::jump_thru::load_jump_thrus;
use crate::entities::spikes::load_spikes;
use crate::entities::zip_mover::load_zip_movers;
use crate::room::{Chapter, Room};

#[derive(Debug, Clone, PartialEq)]
pub enum MapError {
    // ran out of bytes mid read
    UnexpectedEnd,
    // file doesn't start with "CELESTE MAP"
    BadHeader,
    // string isn't valid UTF-8
    BadText,
    // index outside the string table
    BadLookup(i16),
    BadValueType(u8),
    RoomNotFound(String),
    MissingField(&'static str),
    Io(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Float(f32),
    Text(String),
}

// node of the map file's tree, like an XML tag: <name attributes...>children</name>
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub children: Vec<Element>,
}

impl Element {
    // The first child with this name.
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }

    // The attribute's value if it exists and is a Value::Int.
    pub fn int(&self, key: &str) -> Option<i32> {
        match self.attributes.get(key) {
            Some(Value::Int(value)) => Some(*value),
            _ => None,
        }
    }

    // The attribute's value if it exists and is a Value::Text.
    pub fn text(&self, key: &str) -> Option<&str> {
        match self.attributes.get(key) {
            Some(Value::Text(value)) => Some(value),
            _ => None,
        }
    }
}

pub struct MapReader<'file> {
    bytes: &'file [u8],
    pos: usize,
}

impl<'file> MapReader<'file> {
    pub fn new(bytes: &'file [u8]) -> MapReader<'file> {
        MapReader { bytes, pos: 0 }
    }

    pub fn take(&mut self, count: usize) -> Result<&'file [u8], MapError> {
        let end = self.pos + count;

        if end > self.bytes.len() {
            return Err(MapError::UnexpectedEnd);
        }
        
        let taken_bytes = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(taken_bytes)
    }

    pub fn read_u8(&mut self) -> Result<u8, MapError> {
        let bytes = self.take(1)?;
        
        Ok(bytes[0])
    }

    // multi-byte numbers are little-endian
    pub fn read_i16(&mut self) -> Result<i16, MapError> {
        let bytes = self.take(2)?;
        let byte_array = [bytes[0], bytes[1]];

        Ok(i16::from_le_bytes(byte_array))
    }

    pub fn read_i32(&mut self) -> Result<i32, MapError> {
        let bytes = self.take(4)?;
        let byte_array = [bytes[0], bytes[1], bytes[2], bytes[3]];

        Ok(i32::from_le_bytes(byte_array))
    }

    pub fn read_f32(&mut self) -> Result<f32, MapError> {
        let bytes = self.take(4)?;
        let byte_array = [bytes[0], bytes[1], bytes[2], bytes[3]];

        Ok(f32::from_le_bytes(byte_array))
    }

    // C# BinaryWriter string: the length in bytes, then that many bytes of UTF-8.
    // The length is a varint, 7 bits per byte, lowest bits first, and the top bit
    // of each byte is a flag for if there is another length byte
    pub fn read_string(&mut self) -> Result<String, MapError> {
        let mut len = 0;
        let mut shift = 0;

        loop {
            let b = self.read_u8()?;

            len |= ((b & 0x7F) as usize) << shift;
            shift += 7;

            if b & 0x80 == 0 {
                break;
            }
        }

        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| MapError::BadText)
    }
}

// run length encoded text, pairs of (repeat count, character).
pub fn decode_rle(bytes: &[u8]) -> String {
    let mut decoded_string = String::new();

    for pair in bytes.chunks(2) {
        let count = pair[0];
        let character = pair[1];

        for _ in 0..count {
            decoded_string.push(character as char);
        }
    }

    decoded_string
}

pub fn read_value(reader: &mut MapReader, lookup: &[String], kind: u8) -> Result<Value, MapError> {
    match kind {
        0 => Ok(Value::Bool(reader.read_u8()? != 0)),
        1 => Ok(Value::Int(reader.read_u8()? as i32)),
        2 => Ok(Value::Int(reader.read_i16()? as i32)),
        3 => Ok(Value::Int(reader.read_i32()?)),
        4 => Ok(Value::Float(reader.read_f32()?)),
        5 => Ok(Value::Text(read_lookup(reader, lookup)?)),
        6 => Ok(Value::Text(reader.read_string()?)),
        7 => Ok(Value::Text(decode_rle_string(reader)?)),
        other => Err(MapError::BadValueType(other))
    }
}

fn decode_rle_string(reader: &mut MapReader) -> Result<String, MapError> {
        let count = reader.read_i16()? as usize;
        let bytes = reader.take(count)?;
        let decoded_string = decode_rle(bytes);

        Ok(decoded_string)
}

fn read_lookup(reader: &mut MapReader, lookup: &[String]) -> Result<String, MapError> {
    let i = reader.read_i16()?;
    let word = lookup.get(i as usize).ok_or(MapError::BadLookup(i))?;
    
    Ok(word.clone())
}

// one element and its children recursively
pub fn read_element(reader: &mut MapReader, lookup: &[String]) -> Result<Element, MapError> {
    let name = read_lookup(reader, lookup)?;
    let mut attributes = HashMap::new();
    let mut children = Vec::new();

    let attribute_count = reader.read_u8()?;
    for _ in 0..attribute_count {
        let key = read_lookup(reader, lookup)?;

        let kind = reader.read_u8()?;
        let value = read_value(reader, lookup, kind)?;

        attributes.insert(key, value);
    }

    let child_count = reader.read_i16()?;
    for _ in 0..child_count {
        children.push(read_element(reader, lookup)?);
    }

    Ok(Element { name, attributes, children })
}

// read the root element of the map
pub fn parse_map(bytes: &[u8]) -> Result<Element, MapError> {
    let mut reader = MapReader::new(bytes);
    let mut lookup = Vec::new();
    
    if reader.read_string()? != "CELESTE MAP" {
        return Err(MapError::BadHeader);
    }

    // ignore package name
    reader.read_string()?;

    let lookup_count = reader.read_i16()?;
    for _ in 0..lookup_count {
        lookup.push(reader.read_string()?);
    }

    read_element(&mut reader, &lookup)
}

// find a room by name and build its tile grid
pub fn load_room(map: &Element, name: &str) -> Result<Room, MapError> {
    let levels = map.child("levels").ok_or(MapError::MissingField("levels"))?;
    let level = levels
        .children
        .iter()
        .find(|level| level.text("name") == Some(name))
        .ok_or(MapError::RoomNotFound(name.to_string()))?;

    room_from_level(level)
}

pub fn load_chapter(map: &Element) -> Result<Chapter, MapError> {
    let levels = map.child("levels").ok_or(MapError::MissingField("levels"))?;
    let rooms = levels.children.iter().map(room_from_level).collect::<Result<_, _>>()?;

    Ok(Chapter { rooms })
}

fn room_from_level(level: &Element) -> Result<Room, MapError> {
    let x = level.int("x").ok_or(MapError::MissingField("x"))?;
    let y = level.int("y").ok_or(MapError::MissingField("y"))?;
    let origin = IVec2 { x, y };

    let solids = level.child("solids").ok_or(MapError::MissingField("solids"))?;
    let tiles = solids.text("innerText").ok_or(MapError::MissingField("innerText"))?;

    let mut room = Room::from_tiles(origin, tiles);
    room.name = level.text("name").unwrap_or_default().to_string();
    if let (Some(width), Some(height)) = (level.int("width"), level.int("height")) {
        room.set_size(width, height);
    }
    room.spikes = load_spikes(level, origin)?;
    room.jump_thrus = load_jump_thrus(level, origin)?;
    room.zip_movers = load_zip_movers(level, origin)?;
    
    Ok(room)
}

pub fn load_map(path: &Path) -> Result<Element, MapError> {
    let bytes = std::fs::read(path).map_err(|e| MapError::Io(e.to_string()))?;
    parse_map(&bytes)
}

pub fn find_celeste_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CELESTE_DIR") {
        return Some(PathBuf::from(dir));
    }
    let steam_path = PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Celeste");
    steam_path.is_dir().then_some(steam_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{jump_thru, Spikes, ZipMover};
    use crate::room::Direction;

    const LOOKUP: [&str; 25] = [
        "Map", "levels", "level", "name", "x", "y", "solids", "innerText", "room_a", "room_b",
        "flag", "big", "ratio", "note", "entities", "spikesUp", "spikesDown", "spikesLeft",
        "spikesRight", "width", "height", "strawberry", "jumpThru", "zipMover", "node",
    ];

    fn string(s: &str) -> Vec<u8> {
        assert!(s.len() < 128);
        let mut out = vec![s.len() as u8];
        out.extend(s.as_bytes());
        out
    }

    fn index(word: &str) -> Vec<u8> {
        let i = LOOKUP.iter().position(|w| *w == word).unwrap() as i16;
        i.to_le_bytes().to_vec()
    }

    fn v_bool(v: bool) -> Vec<u8> {
        vec![0, v as u8]
    }

    fn v_u8(v: u8) -> Vec<u8> {
        vec![1, v]
    }

    fn v_i16(v: i16) -> Vec<u8> {
        [vec![2], v.to_le_bytes().to_vec()].concat()
    }

    fn v_i32(v: i32) -> Vec<u8> {
        [vec![3], v.to_le_bytes().to_vec()].concat()
    }

    fn v_f32(v: f32) -> Vec<u8> {
        [vec![4], v.to_le_bytes().to_vec()].concat()
    }

    fn v_lookup(word: &str) -> Vec<u8> {
        [vec![5], index(word)].concat()
    }

    fn v_string(s: &str) -> Vec<u8> {
        [vec![6], string(s)].concat()
    }

    fn v_rle(pairs: &[(u8, char)]) -> Vec<u8> {
        let body: Vec<u8> = pairs.iter().flat_map(|&(count, c)| [count, c as u8]).collect();
        [vec![7], (body.len() as i16).to_le_bytes().to_vec(), body].concat()
    }

    fn element(name: &str, attributes: &[(&str, Vec<u8>)], children: &[Vec<u8>]) -> Vec<u8> {
        let mut out = index(name);
        out.push(attributes.len() as u8);
        for (key, value) in attributes {
            out.extend(index(key));
            out.extend(value);
        }
        out.extend((children.len() as i16).to_le_bytes());
        for child in children {
            out.extend(child);
        }
        out
    }

    fn file(root: Vec<u8>) -> Vec<u8> {
        let mut out = string("CELESTE MAP");
        out.extend(string("test"));
        out.extend((LOOKUP.len() as i16).to_le_bytes());
        for word in LOOKUP {
            out.extend(string(word));
        }
        out.extend(root);
        out
    }

    fn level(name: &str, x: i16, y: i16, tiles: &[(u8, char)]) -> Vec<u8> {
        element(
            "level",
            &[("name", v_lookup(name)), ("x", v_i16(x)), ("y", v_i16(y))],
            &[element("solids", &[("innerText", v_rle(tiles))], &[])],
        )
    }

    // room_a is "001\n111" at (0, 0); room_b is "1" at (320, -184)
    fn tiny_map() -> Vec<u8> {
        let room_a = level("room_a", 0, 0, &[(2, '0'), (1, '1'), (1, '\n'), (3, '1')]);
        let room_b = level("room_b", 320, -184, &[(1, '1')]);
        file(element("Map", &[], &[element("levels", &[], &[room_a, room_b])]))
    }

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    #[test]
    fn take_returns_bytes_in_order() {
        let mut r = MapReader::new(&[1, 2, 3, 4, 5]);
        assert_eq!(r.take(2), Ok(&[1, 2][..]));
        assert_eq!(r.take(3), Ok(&[3, 4, 5][..]));
        assert_eq!(r.take(0), Ok(&[][..]));
    }

    #[test]
    fn take_past_the_end_is_an_error() {
        let mut r = MapReader::new(&[1, 2]);
        assert_eq!(r.take(3), Err(MapError::UnexpectedEnd));
    }

    #[test]
    fn read_u8_advances() {
        let mut r = MapReader::new(&[7, 200]);
        assert_eq!(r.read_u8(), Ok(7));
        assert_eq!(r.read_u8(), Ok(200));
        assert_eq!(r.read_u8(), Err(MapError::UnexpectedEnd));
    }

    #[test]
    fn read_i16_is_little_endian() {
        let mut r = MapReader::new(&[0x34, 0x12, 0xFE, 0xFF, 0x01]);
        assert_eq!(r.read_i16(), Ok(0x1234));
        assert_eq!(r.read_i16(), Ok(-2));
        assert_eq!(r.read_i16(), Err(MapError::UnexpectedEnd));
    }

    #[test]
    fn read_i32_and_f32() {
        let bytes = [100_000i32.to_le_bytes(), 1.5f32.to_le_bytes()].concat();
        let mut r = MapReader::new(&bytes);
        assert_eq!(r.read_i32(), Ok(100_000));
        assert_eq!(r.read_f32(), Ok(1.5));
    }

    #[test]
    fn read_string_reads_length_then_text() {
        let bytes = [string("hello"), string(""), string("lvl_1")].concat();
        let mut r = MapReader::new(&bytes);
        assert_eq!(r.read_string(), Ok("hello".to_string()));
        assert_eq!(r.read_string(), Ok("".to_string()));
        assert_eq!(r.read_string(), Ok("lvl_1".to_string()));
    }

    #[test]
    fn read_string_with_two_byte_length() {
        let mut bytes = vec![0xAC, 0x02];
        bytes.extend(std::iter::repeat_n(b'a', 300));
        bytes.push(42);
        let mut r = MapReader::new(&bytes);
        assert_eq!(r.read_string(), Ok("a".repeat(300)));
        assert_eq!(r.read_u8(), Ok(42));
    }

    #[test]
    fn read_string_errors() {
        assert_eq!(MapReader::new(&[5, b'h', b'i']).read_string(), Err(MapError::UnexpectedEnd));
        assert_eq!(MapReader::new(&[1, 0xFF]).read_string(), Err(MapError::BadText));
    }

    #[test]
    fn decode_rle_repeats_characters() {
        assert_eq!(decode_rle(&[3, b'1', 2, b'0']), "11100");
        assert_eq!(decode_rle(&[2, b'0', 1, b'\n', 1, b'1']), "00\n1");
        assert_eq!(decode_rle(&[]), "");
    }

    #[test]
    fn parse_map_reads_the_tree() {
        let map = parse_map(&tiny_map()).unwrap();
        assert_eq!(map.name, "Map");
        assert_eq!(map.children.len(), 1);

        let levels = &map.children[0];
        assert_eq!(levels.name, "levels");
        assert_eq!(levels.children.len(), 2);

        let room_b = &levels.children[1];
        assert_eq!(room_b.name, "level");
        assert_eq!(room_b.attributes["name"], text("room_b"));
        assert_eq!(room_b.attributes["x"], Value::Int(320));
        assert_eq!(room_b.attributes["y"], Value::Int(-184));

        let solids = &levels.children[0].children[0];
        assert_eq!(solids.name, "solids");
        assert_eq!(solids.attributes["innerText"], text("001\n111"));
    }

    #[test]
    fn parse_map_reads_every_value_type() {
        let attributes = [
            ("flag", v_bool(true)),
            ("x", v_u8(200)),
            ("y", v_i16(-184)),
            ("big", v_i32(100_000)),
            ("ratio", v_f32(1.5)),
            ("name", v_lookup("room_a")),
            ("note", v_string("hi")),
            ("innerText", v_rle(&[(2, 'a')])),
        ];
        let map = parse_map(&file(element("Map", &attributes, &[]))).unwrap();
        assert_eq!(map.attributes.len(), 8);
        assert_eq!(map.attributes["flag"], Value::Bool(true));
        assert_eq!(map.attributes["x"], Value::Int(200));
        assert_eq!(map.attributes["y"], Value::Int(-184));
        assert_eq!(map.attributes["big"], Value::Int(100_000));
        assert_eq!(map.attributes["ratio"], Value::Float(1.5));
        assert_eq!(map.attributes["name"], text("room_a"));
        assert_eq!(map.attributes["note"], text("hi"));
        assert_eq!(map.attributes["innerText"], text("aa"));
    }

    #[test]
    fn parse_map_rejects_other_files() {
        let mut bytes = string("NOT A MAP");
        bytes.extend(string("test"));
        assert_eq!(parse_map(&bytes), Err(MapError::BadHeader));
    }

    #[test]
    fn parse_map_truncated_file() {
        let bytes = tiny_map();
        assert_eq!(parse_map(&bytes[..bytes.len() - 3]), Err(MapError::UnexpectedEnd));
    }

    #[test]
    fn parse_map_bad_lookup_index() {
        let root = [99i16.to_le_bytes().to_vec(), vec![0, 0, 0]].concat();
        assert_eq!(parse_map(&file(root)), Err(MapError::BadLookup(99)));
    }

    #[test]
    fn parse_map_unknown_value_type() {
        let root = element("Map", &[("x", vec![9, 0])], &[]);
        assert_eq!(parse_map(&file(root)), Err(MapError::BadValueType(9)));
    }

    #[test]
    fn element_child_finds_by_name() {
        let map = parse_map(&tiny_map()).unwrap();
        assert_eq!(map.child("levels").unwrap().children.len(), 2);
        assert!(map.child("nope").is_none());
    }

    #[test]
    fn element_int_and_text_check_the_type() {
        let map = parse_map(&tiny_map()).unwrap();
        let room_b = &map.child("levels").unwrap().children[1];
        assert_eq!(room_b.int("x"), Some(320));
        assert_eq!(room_b.text("name"), Some("room_b"));
        assert_eq!(room_b.int("name"), None);
        assert_eq!(room_b.text("x"), None);
        assert_eq!(room_b.int("nope"), None);
    }

    #[test]
    fn load_room_builds_the_grid() {
        let map = parse_map(&tiny_map()).unwrap();

        let a = load_room(&map, "room_a").unwrap();
        assert_eq!(a.origin, IVec2 { x: 0, y: 0 });
        assert_eq!((a.width, a.height), (3, 2));
        assert!(!a.is_solid(0, 0));
        assert!(a.is_solid(2, 0));
        assert!(a.is_solid(0, 1));

        let b = load_room(&map, "room_b").unwrap();
        assert_eq!(b.origin, IVec2 { x: 320, y: -184 });
        assert!(b.is_solid(0, 0));
    }

    #[test]
    fn load_room_names_the_room() {
        let map = parse_map(&tiny_map()).unwrap();
        assert_eq!(load_room(&map, "room_b").unwrap().name, "room_b");
    }

    #[test]
    fn load_chapter_loads_every_room_in_order() {
        let map = parse_map(&tiny_map()).unwrap();
        let chapter = load_chapter(&map).unwrap();
        let names: Vec<&str> = chapter.rooms.iter().map(|room| room.name.as_str()).collect();
        assert_eq!(names, ["room_a", "room_b"]);
        assert_eq!(chapter.rooms[1], load_room(&map, "room_b").unwrap());
    }

    #[test]
    fn load_room_unknown_name() {
        let map = parse_map(&tiny_map()).unwrap();
        assert_eq!(load_room(&map, "nope"), Err(MapError::RoomNotFound("nope".to_string())));
    }

    #[test]
    fn load_room_without_solids() {
        let bare = element("level", &[("name", v_lookup("room_a")), ("x", v_i16(0)), ("y", v_i16(0))], &[]);
        let bytes = file(element("Map", &[], &[element("levels", &[], &[bare])]));
        let map = parse_map(&bytes).unwrap();
        assert_eq!(load_room(&map, "room_a"), Err(MapError::MissingField("solids")));
    }
    fn entity(name: &str, x: i16, y: i16, size: Option<(&str, u8)>) -> Vec<u8> {
        let mut attributes = vec![("x", v_i16(x)), ("y", v_i16(y))];
        if let Some((key, length)) = size {
            attributes.push((key, v_u8(length)));
        }
        element(name, &attributes, &[])
    }

    fn map_with_entities(entities: &[Vec<u8>]) -> Element {
        let room = element(
            "level",
            &[("name", v_lookup("room_b")), ("x", v_i16(320)), ("y", v_i16(-184))],
            &[
                element("solids", &[("innerText", v_rle(&[(1, '1')]))], &[]),
                element("entities", &[], entities),
            ],
        );
        parse_map(&file(element("Map", &[], &[element("levels", &[], &[room])]))).unwrap()
    }

    #[test]
    fn load_room_reads_spikes_in_world_coordinates() {
        let map = map_with_entities(&[
            entity("spikesUp", 8, 16, Some(("width", 24))),
            entity("strawberry", 1, 1, None),
            entity("spikesLeft", 40, 0, Some(("height", 16))),
            entity("spikesDown", 0, 8, Some(("width", 8))),
            entity("spikesRight", 16, 32, Some(("height", 40))),
        ]);
        let room = load_room(&map, "room_b").unwrap();
        assert_eq!(
            room.spikes,
            vec![
                Spikes::new(Direction::Up, 328, -168, 24),
                Spikes::new(Direction::Left, 360, -184, 16),
                Spikes::new(Direction::Down, 320, -176, 8),
                Spikes::new(Direction::Right, 336, -152, 40),
            ]
        );
    }

    #[test]
    fn load_room_reads_jump_thrus_in_world_coordinates() {
        let map = map_with_entities(&[
            entity("jumpThru", 8, 160, Some(("width", 40))),
            entity("strawberry", 1, 1, None),
            entity("jumpThru", 0, 16, Some(("width", 8))),
        ]);
        let room = load_room(&map, "room_b").unwrap();
        assert_eq!(room.jump_thrus, vec![jump_thru(328, -24, 40), jump_thru(320, -168, 8)]);
        assert!(room.spikes.is_empty());
    }

    #[test]
    fn load_room_jump_thru_missing_its_width() {
        let map = map_with_entities(&[entity("jumpThru", 8, 160, None)]);
        assert_eq!(load_room(&map, "room_b"), Err(MapError::MissingField("width")));
    }

    fn zip_mover_entity(node: Option<(i16, i16)>) -> Vec<u8> {
        let attributes = [("x", v_i16(104)), ("y", v_i16(264)), ("width", v_u8(32)), ("height", v_u8(16))];
        let nodes: Vec<Vec<u8>> = node
            .map(|(x, y)| element("node", &[("x", v_i16(x)), ("y", v_i16(y))], &[]))
            .into_iter()
            .collect();
        element("zipMover", &attributes, &nodes)
    }

    #[test]
    fn load_room_reads_zip_movers_in_world_coordinates() {
        let map = map_with_entities(&[zip_mover_entity(Some((104, 184)))]);
        let room = load_room(&map, "room_b").unwrap();
        assert_eq!(room.zip_movers, vec![ZipMover::new(424, 80, 32, 16, 424, 0)]);
    }

    #[test]
    fn load_room_zip_mover_missing_its_target() {
        let map = map_with_entities(&[zip_mover_entity(None)]);
        assert_eq!(load_room(&map, "room_b"), Err(MapError::MissingField("node")));
    }

    #[test]
    fn load_room_without_entities_has_no_spikes() {
        let map = parse_map(&tiny_map()).unwrap();
        assert!(load_room(&map, "room_a").unwrap().spikes.is_empty());
    }

    #[test]
    fn load_room_spikes_missing_their_length() {
        let map = map_with_entities(&[entity("spikesUp", 8, 16, None)]);
        assert_eq!(load_room(&map, "room_b"), Err(MapError::MissingField("width")));

        let map = map_with_entities(&[entity("spikesRight", 8, 16, Some(("width", 8)))]);
        assert_eq!(load_room(&map, "room_b"), Err(MapError::MissingField("height")));
    }
}
