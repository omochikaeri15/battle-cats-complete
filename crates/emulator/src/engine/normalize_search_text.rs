use crate::Fault;

pub fn normalize_search_text(text: &[u8]) -> Result<Vec<u8>, Fault> {
    let mut points: Vec<u32> = Vec::new();
    let mut at = 0usize;

    while at < text.len() {
        let lead = text[at];
        let (width, mut point) = if lead < 0x80 {
            (1usize, u32::from(lead))
        } else if lead & 0xe0 == 0xc0 {
            (2, u32::from(lead & 0x1f))
        } else if lead & 0xf0 == 0xe0 {
            (3, u32::from(lead & 0x0f))
        } else if lead & 0xf8 == 0xf0 {
            (4, u32::from(lead & 0x07))
        } else {
            return Err(Fault::invalid_argument());
        };

        if at + width > text.len() {
            return Err(Fault::invalid_argument());
        }

        let mut extra = 1usize;

        while extra < width {
            let byte = text[at + extra];

            if byte & 0xc0 != 0x80 {
                return Err(Fault::invalid_argument());
            }

            point = point << 6 | u32::from(byte & 0x3f);
            extra += 1;
        }

        if point > 0x10ffff {
            return Err(Fault::invalid_argument());
        }

        points.push(point);
        at += width;
    }

    let mut out: Vec<u8> = Vec::new();

    for point in &points {
        let mut point = *point;

        if point.wrapping_sub(0xff10) <= 9 || point.wrapping_sub(0xff21) <= 0x19 {
            point = point.wrapping_add(0xffff0120);
        } else if point.wrapping_sub(0xff41) < 0x1a {
            point = point.wrapping_sub(0xfee0);
        }

        if point.wrapping_sub(0x41) <= 0x19 {
            point |= 0x20;
        } else if point.wrapping_sub(0x3041) < 0x56 {
            point = point.wrapping_add(0x60);
        }

        if point < 0x80 {
            out.push(point as u8);
        } else if point < 0x800 {
            out.push(0xc0 | (point >> 6) as u8);
            out.push(0x80 | (point & 0x3f) as u8);
        } else if point < 0x10000 {
            out.push(0xe0 | (point >> 12) as u8);
            out.push(0x80 | (point >> 6 & 0x3f) as u8);
            out.push(0x80 | (point & 0x3f) as u8);
        } else {
            out.push(0xf0 | (point >> 18) as u8);
            out.push(0x80 | (point >> 12 & 0x3f) as u8);
            out.push(0x80 | (point >> 6 & 0x3f) as u8);
            out.push(0x80 | (point & 0x3f) as u8);
        }
    }

    Ok(out)
}
