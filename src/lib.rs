#![feature(ip_from)]
#![feature(slice_pattern)]
mod ipv4;
mod ipv6;
mod utils;

pub use ipv4::Ipv4RouteFlags;
pub use ipv6::Ipv6RouteFlags;

pub fn get_ipv4_route_table() -> ipv4::Ipv4RouteTable {
    ipv4::Ipv4RouteTable::default()
}

pub fn get_ipv6_route_table() -> ipv6::Ipv6RouteTable {
    ipv6::Ipv6RouteTable::default()
}


