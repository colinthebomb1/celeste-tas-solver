use crate::geom::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraceFrame {
    pub position: Vec2,
    pub speed: Vec2,
}

pub fn parse_trace(text: &str) -> Vec<TraceFrame> {
    let mut frames: Vec<TraceFrame> = vec![];

    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();

        let position_fields: Vec<f32> = fields[4]
            .split(',')
            .filter_map(|s| s.trim().parse::<f32>().ok())
            .collect();
        let speed_fields: Vec<f32> = fields[5]
            .split(',')
            .filter_map(|s| s.trim().parse::<f32>().ok())
            .collect();

        let frame = TraceFrame {
            position: Vec2 {
                x: position_fields[0],
                y: position_fields[1],
            },
            speed: Vec2 {
                x: speed_fields[0],
                y: speed_fields[1],
            },
        };

        frames.push(frame);
    }

    frames
}
