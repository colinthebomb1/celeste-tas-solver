use crate::geom::IVec2;
use crate::geom::Rect;
use crate::level::map::{Element, MapError, triggers_named};
use crate::level::room::Room;

// the map events that end a chapter. only chapter 1's so far
const FINISH_EVENTS: [&str; 1] = ["end_city"];

pub(crate) fn load_finish_trigger(
    level: &Element,
    origin: IVec2,
    room: &mut Room,
) -> Result<(), MapError> {
    for trigger in triggers_named(level, "eventTrigger") {
        let ends_chapter = trigger
            .text("event")
            .is_some_and(|event| FINISH_EVENTS.contains(&event));

        if ends_chapter {
            room.finish_trigger = Some(Rect::new(
                origin.x + trigger.require("x")?,
                origin.y + trigger.require("y")?,
                trigger.require("width")?,
                trigger.require("height")?,
            ));
        }
    }

    Ok(())
}
