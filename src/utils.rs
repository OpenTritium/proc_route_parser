use core::slice::SlicePattern;
use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("The length of string( {0:?}) is not an even number")]
    OddStringLength(String),
    #[error("Failed to convert the slice into u8 array")]
    SliceToBytes(#[from] std::array::TryFromSliceError),
    #[error("Invalid u8: {0},just ensure the ascii code is within 0..=(F/f)")]
    OutOfHexRange(u8),
}

pub(crate) fn hex_char_to_u8(hex: u8) -> Result<u8, ConvertError> {
    match hex {
        b'0'..=b'9' => Ok(hex - b'0'),
        b'a'..=b'f' => Ok(hex - b'a' + 10),
        b'A'..=b'F' => Ok(hex - b'A' + 10),
        a => Err(ConvertError::OutOfHexRange(a)),
    }
}

/// ensure mask `[0x00FF;0x00FF]` bits form
pub(crate) fn hex_char_pair_to_byte(double_char: [u8; 2]) -> Result<u8, ConvertError> {
    let [high, low] = double_char;
    Ok(hex_char_to_u8(high)? << 4 | hex_char_to_u8(low)?)
}

pub(crate) fn hex_str_to_bytes(text: &str) -> Result<Box<[u8]>, ConvertError> {
    if text.len() % 2 != 0 {
        return Err(ConvertError::OddStringLength(text.to_string()));
    }
    let mut buf = Vec::with_capacity(text.len() / 2);
    for hex_char_pair in text.as_bytes().chunks_exact(2) {
        let hex_byte_pair = [hex_char_pair[0], hex_char_pair[1]];
        buf.push(hex_char_pair_to_byte(hex_byte_pair)?);
    }
    Ok(buf.into_boxed_slice())
}

pub(crate) fn hex_str_to_ipv6(text: &str) -> Result<Ipv6Addr, ConvertError> {
    Ok(Ipv6Addr::from_octets(
        hex_str_to_bytes(text)?.as_slice().try_into()?,
    ))
}

pub(crate) fn hex_str_to_ipv4(text: &str) -> Result<Ipv4Addr, ConvertError> {
    Ok(Ipv4Addr::from_octets(
        hex_str_to_bytes(text)?
            .into_iter()
            .rev()
            .collect::<Vec<u8>>()
            .as_slice()
            .try_into()?,
    ))
}
