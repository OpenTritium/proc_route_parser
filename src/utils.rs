use core::slice::SlicePattern;
use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("The length of string( {0}) is not an even number")]
    OddStringLength(String),

    #[error("Failed to convert the slice into u8 array")]
    SliceToBytes(#[from] std::array::TryFromSliceError),

    #[error("Invalid u8: {0},just ensure the ascii code is within 0..=(F/f)")]
    OutOfHexRange(u8),
}

#[inline(always)]
pub(crate) fn hex_char_to_u8(hex: u8) -> Result<u8, ConvertError> {
    match hex {
        b'0'..=b'9' => Ok(hex - b'0'),
        b'a'..=b'f' => Ok(hex - b'a' + 10),
        b'A'..=b'F' => Ok(hex - b'A' + 10),
        a => Err(ConvertError::OutOfHexRange(a)),
    }
}

#[inline(always)]
pub(crate) fn hex_char_pair_to_byte(double_chars: [u8; 2]) -> Result<u8, ConvertError> {
    let [high, low] = double_chars;
    Ok(hex_char_to_u8(high)? << 4 | hex_char_to_u8(low)?)
}

#[inline(always)]
pub(crate) fn hex_str_to_bytes(text: &str) -> Result<Box<[u8]>, ConvertError> {
    if text.len() % 2 != 0 {
        return Err(ConvertError::OddStringLength(text.to_string()));
    }
    let mut buf = Vec::with_capacity(text.len() / 2);
    for hex_char_pair in text.as_bytes().chunks_exact(2) {
        let (hex_byte_pair, _) = hex_char_pair.as_chunks::<2>();
        buf.push(hex_char_pair_to_byte(hex_byte_pair[0])?);
    }
    Ok(buf.into_boxed_slice())
}

#[inline(always)]
pub(crate) fn hex_str_to_ipv6(text: &str) -> Result<Ipv6Addr, ConvertError> {
    Ok(Ipv6Addr::from_octets(
        hex_str_to_bytes(text)?.as_slice().try_into()?,
    ))
}

#[inline(always)]
pub(crate) fn hex_str_to_ipv4(text: &str) -> Result<Ipv4Addr, ConvertError> {
    if text.len() != 8 {
        return Err(ConvertError::OddStringLength(text.to_string()));
    }
    let mut bytes = [0u8; 4];
    for (i, chunk) in text.as_bytes().chunks_exact(2).enumerate() {
        bytes[i] = hex_char_pair_to_byte([chunk[0], chunk[1]])?;
    }
    let addr_u32 = u32::from_le_bytes(bytes);
    Ok(Ipv4Addr::from(addr_u32))
}
