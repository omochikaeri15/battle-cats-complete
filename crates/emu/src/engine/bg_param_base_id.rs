pub fn bg_param_base_id(name: &[u8]) -> i32 {
    match name.len() {
        0 => return 0,
        8 if name == b"worldTop" => return 4,
        0xb if name == b"worldBottom" => return 5,
        7 if name == b"bgImage" => return 1,
        9 if name == b"worldLeft" => return 2,
        0xa if name == b"worldRight" => return 3,
        _ => {}
    }

    match name {
        b"screenLeft" => 6,
        b"screenRight" => 7,
        b"screenTop" => 8,
        b"screenBottom" => 9,
        b"screen" => 0xa,
        b"frontChara" => 0xb,
        b"backChara" => 0xc,
        b"secondToFrame" => 0xd,
        b"percentToFloat" => 0xe,
        b"percentToAlpha" => 0xf,
        b"animeInterval" => 0x11,
        b"animeLength" => 0x10,
        _ => 0,
    }
}
