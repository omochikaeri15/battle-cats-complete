use std::convert::TryInto;

use crate::bcu::cryptology::Error;

pub fn take_bytes(data: &[u8], amount: usize) -> Result<(&[u8], &[u8]), Error> {
    if data.len() < amount {
        return Err(Error::InvalidLength);
    }

    Ok(data.split_at(amount))
}

pub fn take_array<const SIZE: usize>(data: &[u8]) -> Result<([u8; SIZE], &[u8]), Error> {
    if data.len() < SIZE {
        return Err(Error::InvalidLength);
    }

    let (head, tail) = data.split_at(SIZE);
    let array = head.try_into().map_err(|_error| Error::InvalidLength)?;

    Ok((array, tail))
}

pub fn take_u32(data: &[u8]) -> Result<(u32, &[u8]), Error> {
    let (bytes, tail) = take_array::<4>(data)?;
    let value = u32::from_le_bytes(bytes);

    Ok((value, tail))
}