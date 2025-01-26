#![feature(ip_from)]
#![feature(slice_pattern)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
    net::Ipv6Addr,
};

mod utils {
    use core::slice::SlicePattern;
    use std::net::Ipv6Addr;
    pub(crate) fn hex_char_to_u8(hex: u8) -> u8 {
        match hex {
            b'0'..=b'9' => hex - b'0',
            b'a'..=b'f' => hex - b'a' + 10,
            b'A'..=b'F' => hex - b'A' + 10,
            _ => panic!("parse error"),
        }
    }

    pub(crate) fn hex_char_pair_to_byte(double_char: [u8; 2]) -> u8 {
        let [high, low] = double_char;
        hex_char_to_u8(high) << 4 | hex_char_to_u8(low)
    }

    pub(crate) fn hex_str_to_bytes(text: &str) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(text.len() / 2);
        for hex_char_pair in text.as_bytes().chunks_exact(2) {
            let hex_byte_pair = [hex_char_pair[0], hex_char_pair[1]];
            buf.push(hex_char_pair_to_byte(hex_byte_pair));
        }
        buf.into_boxed_slice()
    }
    pub(crate) fn hex_str_to_ipv6(text: &str) -> Ipv6Addr {
        let bytes = hex_str_to_bytes(text);
        Ipv6Addr::from_octets(bytes.as_slice().try_into().unwrap())
    }
}

#[derive(Debug)]
struct Ipv6RouteEntry {
    dest: Ipv6Addr,
    dest_prefix: u8,
    src: Ipv6Addr,
    src_prefix: u8,
    next_hop: Ipv6Addr,
    metric: u32,
    ref_count: u32,
    use_count: u32,
    flags: u32,
    name: String,
}

impl Ipv6RouteEntry {
    const UNSPECIFIED:Ipv6Addr = Ipv6Addr::from_bits(0);
    fn is_inbound(&self)->bool{
        !self.is_outbound()
    }
    fn is_outbound(&self)->bool{
        self.dest_prefix == 0
    }
    fn is_loopback(&self) -> bool {
        self.dest.is_loopback() || self.src.is_loopback() || (self.dest == Self::UNSPECIFIED && self.src == Self::UNSPECIFIED)
    }
    fn is_default(&self) -> bool {
        self.dest ==  Self::UNSPECIFIED || self.src == Self::UNSPECIFIED 
    }
}

struct Ipv6RouteTable {
    line_iter: std::io::Lines<BufReader<File>>,
}

impl Default for Ipv6RouteTable {
    fn default() -> Self {
        Self {
            line_iter: BufReader::new(File::open("/proc/net/ipv6_route").unwrap()).lines(),
        }
    }
}

impl Iterator for Ipv6RouteTable {
    type Item = Ipv6RouteEntry;

    fn next(&mut self) -> Option<Self::Item> {
        use utils::*;
        let Some(line) = self.line_iter.next() else {
            return None;
        };
        let Ok(line) = line else {
            return None;
        };
        let fields: Vec<&str> = line.split_whitespace().collect();
        Some(Ipv6RouteEntry {
            dest: hex_str_to_ipv6(fields[0]),
            dest_prefix: hex_char_pair_to_byte(fields[1].as_bytes().try_into().unwrap()),
            src: hex_str_to_ipv6(fields[2]),
            src_prefix: hex_char_pair_to_byte(fields[3].as_bytes().try_into().unwrap()),
            next_hop: hex_str_to_ipv6(fields[4]),
            metric: u32::from_be_bytes((*hex_str_to_bytes(fields[5])).try_into().unwrap()),
            ref_count: u32::from_be_bytes((*hex_str_to_bytes(fields[6])).try_into().unwrap()),
            use_count: u32::from_be_bytes((*hex_str_to_bytes(fields[7])).try_into().unwrap()),
            flags: u32::from_be_bytes((*hex_str_to_bytes(fields[8])).try_into().unwrap()),
            name: fields[9].to_string(),
        })
    }
}
fn main() {
    let t = Ipv6RouteTable::default();
    for e in t {
        if e.is_loopback() {
            println!("{:?}", e);
        }
    }
}
