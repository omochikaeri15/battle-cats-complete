pub fn map_type_of_map_id(map_id: i32) -> i32 {
    if (map_id.wrapping_add(0x3e7) as u32) < 0x7cf {
        return 0;
    }

    if (map_id.wrapping_sub(0x3e8) as u32) < 0x3e8 {
        return 1;
    }

    if (map_id.wrapping_sub(0x7d0) as u32) < 0x3e8 {
        return 2;
    }

    if (map_id.wrapping_sub(0xfa0) as u32) < 0x3e8 {
        return -8;
    }

    if (map_id.wrapping_sub(0x1388) as u32) < 0x3e8 {
        return -5;
    }

    if (map_id.wrapping_sub(0x1770) as u32) < 0x3e8 {
        return 3;
    }

    if (map_id.wrapping_sub(0x1b58) as u32) < 0x3e8 {
        return -4;
    }

    if (map_id.wrapping_sub(0x2af8) as u32) < 0x3e8 {
        return 4;
    }

    if (map_id.wrapping_sub(0x2ee0) as u32) < 0x3e8 {
        return -6;
    }

    if (map_id.wrapping_sub(0xbb8) as u32) < 3 {
        return -2;
    }

    if (map_id.wrapping_sub(0xbbb) as u32) < 3 {
        return -3;
    }

    if (map_id.wrapping_sub(0xbbe) as u32) < 3 {
        return -7;
    }

    if (map_id.wrapping_sub(0x32c8) as u32) < 0x3e8 {
        return -9;
    }

    if (map_id.wrapping_sub(0x36b0) as u32) < 0x3e8 {
        return -10;
    }

    if (map_id.wrapping_sub(0x3e80) as u32) < 0x3e8 {
        return -11;
    }

    if (map_id.wrapping_sub(0x4e20) as u32) < 0x3e8 {
        return -12;
    }

    if (map_id.wrapping_sub(0x5208) as u32) < 0x3e8 {
        return -13;
    }

    if (map_id.wrapping_sub(0x55f0) as u32) < 0x3e8 {
        return -14;
    }

    if (map_id.wrapping_sub(0x59d8) as u32) < 0x3e8 {
        return -15;
    }

    if (map_id.wrapping_sub(0x5dc0) as u32) < 0x3e8 {
        return -16;
    }

    if (map_id.wrapping_sub(0x61a8) as u32) < 0x3e8 {
        return -17;
    }

    if (map_id.wrapping_sub(0x6978) as u32) < 0x3e8 {
        return -18;
    }

    if (map_id.wrapping_sub(0x7530) as u32) < 0x3e8 {
        return -19;
    }

    if (map_id.wrapping_sub(0x7918) as u32) < 0x3e8 {
        return -20;
    }

    if (map_id.wrapping_sub(0x80e8) as u32) < 0x3e8 {
        return -21;
    }

    if (map_id.wrapping_sub(0x84d0) as u32) < 0x3e8 {
        return -22;
    }

    if (map_id.wrapping_sub(0x8ca0) as u32) < 0x3e8 {
        return -23;
    }

    if (map_id.wrapping_sub(0x9088) as u32) < 0x3e8 {
        return -24;
    }

    if (map_id.wrapping_sub(0x9470) as u32) < 0x3e8 {
        return -25;
    }

    if (map_id.wrapping_sub(0x9858) as u32) < 0x3e8 {
        -26
    } else {
        -1
    }
}
