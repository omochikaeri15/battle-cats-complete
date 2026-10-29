const KEY: usize = 0xcc;

pub fn xor_row51_get(obj: &[u8], index: usize) -> Option<u32> {
    let value = obj.get(index.checked_mul(4)?..)?.get(..4)?;
    let key = obj.get(KEY..KEY + 4)?;

    let low = (key.first().copied()? ^ value.first().copied()?) as u32;
    let second = ((key.get(1).copied()? ^ value.get(1).copied()?) as u32) << 8;
    let third = ((key.get(2).copied()? ^ value.get(2).copied()?) as u32) << 16;
    let high = ((key.get(3).copied()? ^ value.get(3).copied()?) as u32) << 24;

    Some(low | second | third | high)
}
