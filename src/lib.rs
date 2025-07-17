#![feature(ip_from)]
#![feature(slice_pattern)]
#![feature(file_buffered)]
mod ipv4;
mod ipv6;
mod utils;

use crate::utils::ConvertError;
pub use ipv4::{Ipv4RouteEntry, Ipv4RouteFlags, Ipv4RouteTable};
pub use ipv6::{Ipv6RouteEntry, Ipv6RouteFlags, Ipv6RouteTable};
use std::io::Result as IoResult;
use thiserror::Error;

#[cfg(target_os = "linux")]
/// Get IPv4 route table via `/proc/net/route`
pub fn get_ipv4_route_table() -> IoResult<ipv4::Ipv4RouteTable> {
    ipv4::Ipv4RouteTable::open("/proc/net/route")
}

#[cfg(target_os = "linux")]
/// Get IPv6 route table via `/proc/net/ipv6_route`
pub fn get_ipv6_route_table() -> IoResult<ipv6::Ipv6RouteTable> {
    ipv6::Ipv6RouteTable::open("/proc/net/ipv6_route")
}

#[derive(Debug, Error)]
pub enum RouteParseError {
    #[error("I/O error reading route file")]
    Io(#[from] std::io::Error),

    #[error("Failed to convert hex value")]
    Convert(#[from] ConvertError),

    #[error("Invalid route entry format: expected {expected} fields, found {found}")]
    InvalidFieldCount { expected: usize, found: usize },

    #[error("Missing a required field at index {0}")]
    MissingField(usize),

    #[error("Failed to convert the slice into u8 array")]
    SliceToBytes(#[from] std::array::TryFromSliceError),
}

#[cfg(not(target_os = "linux"))]
compile_error!("This crate can only be compiled on Linux systems.");
