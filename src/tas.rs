use crate::player::Input;

// the keys held on one frame, in the order TAS files list them
fn keys(input: Input) -> Vec<&'static str> {
    let held = [
        (input.move_x == -1, "L"),
        (input.move_x == 1, "R"),
        (input.move_y == -1, "U"),
        (input.move_y == 1, "D"),
        (input.jump, "J"),
        (input.jump2, "K"),
        (input.dash, "X"),
        (input.demodash, "Z"),
        (input.grab, "G"),
    ];

    held.into_iter()
        .filter(|(down, _)| *down)
        .map(|(_, key)| key)
        .collect()
}

// the lines of a TAS file for these inputs: a frame count, then the keys held that long
pub fn tas_lines(inputs: &[Input]) -> String {
    let mut lines = String::new();

    for same in inputs.chunk_by(|a, b| a == b) {
        let mut parts = vec![format!("{:4}", same.len())];
        parts.extend(keys(same[0]).into_iter().map(String::from));

        lines.push_str(&parts.join(","));
        lines.push('\n');
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Input = Input {
        move_x: 0,
        move_y: 0,
        jump: false,
        jump2: false,
        dash: false,
        grab: false,
        demodash: false,
    };
    const RIGHT: Input = Input { move_x: 1, ..NONE };

    #[test]
    fn frames_with_the_same_keys_share_a_line() {
        let dash = Input {
            dash: true,
            ..RIGHT
        };
        let jump = Input {
            jump: true,
            ..RIGHT
        };
        let inputs = [vec![dash; 5], vec![jump; 6], vec![RIGHT; 5]].concat();

        assert_eq!(tas_lines(&inputs), "   5,R,X\n   6,R,J\n   5,R\n");
    }

    #[test]
    fn frames_with_no_keys_are_just_a_count() {
        assert_eq!(tas_lines(&[NONE; 40]), "  40\n");
    }

    #[test]
    fn no_inputs_is_no_lines() {
        assert_eq!(tas_lines(&[]), "");
    }

    #[test]
    fn every_key_has_its_letter() {
        let left_up = Input {
            move_x: -1,
            move_y: -1,
            jump2: true,
            demodash: true,
            ..NONE
        };
        let right_down = Input {
            move_y: 1,
            jump: true,
            dash: true,
            grab: true,
            ..RIGHT
        };

        assert_eq!(tas_lines(&[left_up]), "   1,L,U,K,Z\n");
        assert_eq!(tas_lines(&[right_down]), "   1,R,D,J,X,G\n");
    }

    #[test]
    fn the_same_keys_apart_get_separate_lines() {
        let inputs = [RIGHT, NONE, RIGHT];
        assert_eq!(tas_lines(&inputs), "   1,R\n   1\n   1,R\n");
    }
}
