use emu::engine::{FormatArg, TextRenderer, Texture};

pub struct Formatter;

impl Formatter {
    fn expand(pattern: &[u8], args: &[FormatArg<'_>]) -> Vec<u8> {
        let mut out = Vec::with_capacity(pattern.len());
        let mut next = 0usize;
        let mut at = 0usize;

        while at < pattern.len() {
            if pattern[at] != b'%' {
                out.push(pattern[at]);
                at += 1;

                continue;
            }

            let mut scan = at + 1;

            if pattern.get(scan) == Some(&b'%') {
                out.push(b'%');
                at = scan + 1;

                continue;
            }

            let zero = pattern.get(scan) == Some(&b'0');

            if zero {
                scan += 1;
            }

            let mut width = 0usize;

            while pattern.get(scan).is_some_and(u8::is_ascii_digit) {
                width = width * 10 + (pattern[scan] - b'0') as usize;
                scan += 1;
            }

            let Some(kind) = pattern.get(scan).copied() else {
                out.push(pattern[at]);
                at += 1;

                continue;
            };

            match (kind, args.get(next)) {
                (b'd', Some(FormatArg::Int(value))) => {
                    let digits = value.unsigned_abs().to_string();
                    let sign = usize::from(*value < 0);
                    let pad = width.saturating_sub(digits.len() + sign);

                    if *value < 0 {
                        out.push(b'-');
                    }

                    out.extend(std::iter::repeat_n(if zero { b'0' } else { b' ' }, pad));
                    out.extend_from_slice(digits.as_bytes());
                    next += 1;
                }
                (b'd', Some(FormatArg::Text(text))) | (b's' | b'@', Some(FormatArg::Text(text))) => {
                    out.extend_from_slice(text);
                    next += 1;
                }
                (b's' | b'@', Some(FormatArg::Int(value))) => {
                    out.extend_from_slice(value.to_string().as_bytes());
                    next += 1;
                }
                _ => out.extend_from_slice(&pattern[at..=scan]),
            }

            at = scan + 1;
        }

        out
    }
}

impl TextRenderer for Formatter {
    fn text_texture(
        &mut self,
        _text: &[u8],
        _font: &[u8],
        _size: i32,
        _align: i32,
        _width: i32,
    ) -> Texture {
        Texture::default()
    }

    fn format(&mut self, pattern: &[u8], args: &[&[u8]]) -> Vec<u8> {
        let owned: Vec<FormatArg<'_>> = args.iter().map(|text| FormatArg::Text(text)).collect();

        Self::expand(pattern, &owned)
    }

    fn substitute(&mut self, text: &[u8], tokens: &[(&[u8], &[u8])]) -> Vec<u8> {
        let mut out = text.to_vec();

        for (name, value) in tokens {
            if name.is_empty() {
                continue;
            }

            let mut at = 0usize;

            while at + name.len() <= out.len() {
                if &out[at..at + name.len()] == *name {
                    out.splice(at..at + name.len(), value.iter().copied());
                    at += value.len();
                } else {
                    at += 1;
                }
            }
        }

        out
    }

    fn format_args(&mut self, pattern: &[u8], args: &[FormatArg<'_>]) -> Vec<u8> {
        Self::expand(pattern, args)
    }

    fn stage_name(&mut self, _map_type: i32, _map_index: i32, _stage: i32) -> Vec<u8> {
        Vec::new()
    }

    fn text_width(&mut self, text: &[u8], size: i32) -> i32 {
        (text.len() as i32).wrapping_mul(size) / 2
    }
}
