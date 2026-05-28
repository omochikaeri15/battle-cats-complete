use crate::common::utils::csv;

#[derive(Clone, Debug)]
pub struct Keyframe {
    pub frame: i32,
    pub value: i32,
    pub ease_mode: i32,
    pub ease_power: i32,
}

#[derive(Clone, Debug)]
pub struct AnimModification {
    pub part_id: usize,
    pub modification_type: i32,
    pub loop_count: i32,
    pub keyframes: Vec<Keyframe>,
    pub min_frame: i32,
    pub max_frame: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Animation {
    pub curves: Vec<AnimModification>,
    pub max_frame: i32,
}

impl Animation {
    #[inline(always)]
    pub fn parse(bytes: impl AsRef<[u8]>) -> Option<Self> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Option<Self> {
        let content = csv::scrub(bytes);
        let delimiter = ',';

        let lines: Vec<&str> = content.lines().filter(|line_ref| !line_ref.trim().is_empty()).collect();

        if lines.is_empty() { return None; }

        fn parse_num<T: std::str::FromStr + Default>(input_string: &str) -> T {
            input_string.trim().parse().unwrap_or_default()
        }

        let mut curves = Vec::new();
        let mut current_line_idx = 0;

        if current_line_idx < lines.len() && lines[current_line_idx].trim().starts_with('[') {
            current_line_idx += 1;
        }
        if current_line_idx < lines.len() { current_line_idx += 1; }
        if current_line_idx < lines.len() { current_line_idx += 1; }

        while current_line_idx < lines.len() {
            let current_line = lines[current_line_idx];
            let parts: Vec<&str> = current_line.split(delimiter).collect();
            current_line_idx += 1;

            if parts.len() < 5 { continue; }

            let part_id: usize = parse_num(parts[0]);
            let mod_type: i32 = parse_num(parts[1]);
            let loop_behavior: i32 = parse_num(parts[2]);
            let min_frame: i32 = parse_num(parts[3]);
            let max_frame: i32 = parse_num(parts[4]);

            if current_line_idx >= lines.len() { break; }
            let count_line = lines[current_line_idx];
            current_line_idx += 1;

            let count_str = count_line.split(delimiter).next().unwrap_or("");
            let keyframe_count: usize = parse_num(count_str);

            let mut keyframes = Vec::new();

            for _ in 0..keyframe_count {
                if current_line_idx >= lines.len() { break; }
                let keyframe_line = lines[current_line_idx];
                current_line_idx += 1;
                let keyframe_parts: Vec<&str> = keyframe_line.split(delimiter).collect();

                if keyframe_parts.len() >= 2 {
                    let frame: i32 = parse_num(keyframe_parts[0]);
                    let value: i32 = parse_num(keyframe_parts[1]);

                    let ease_mode = keyframe_parts.get(2).map_or(0, |text_part| parse_num(text_part));
                    let ease_power = keyframe_parts.get(3).map_or(0, |text_part| parse_num(text_part));

                    keyframes.push(Keyframe { frame, value, ease_mode, ease_power });
                }
            }

            if !keyframes.is_empty() {
                curves.push(AnimModification {
                    part_id, modification_type: mod_type, loop_count: loop_behavior,
                    keyframes, min_frame, max_frame,
                });
            }
        }

        let mut max_len = 0;
        for curve in &curves {
            if let Some(last_keyframe) = curve.keyframes.last() {
                if last_keyframe.frame > max_len { max_len = last_keyframe.frame; }
            }
        }

        Some(Self { curves, max_frame: max_len })
    }
}