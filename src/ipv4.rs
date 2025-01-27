use std::{
    fs::File,
    io::{BufRead, BufReader},
    net::Ipv4Addr,
};

#[derive(Debug)]
pub struct Ipv4RouteEntry {
    pub name: String,
    pub dest: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub flags: Ipv4RouteFlags,
    pub ref_count: u8,
    pub use_count: u8,
    pub metric: u8,
    pub mask: Ipv4Addr,
    pub mtu: u8,
    pub window: u8,
    pub irtt: u8,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct Ipv4RouteFlags : u16 {
        /// Route is active and available (RTF_UP)
        /// Indicates the route is valid and operational
        const UP = 0x0001;
        /// Route uses a gateway (RTF_GATEWAY)
        /// When set, the nexthop field contains a valid gateway address
        const GATEWAY = 0x0002;
        /// Host route (specific to single host) (RTF_HOST)
        /// Indicates the destination is a complete host address
        const HOST = 0x0004;
        /// Reinstate route for dynamic routing (RTF_REINSTATE)
        /// Used by routing daemons to restore routes after link recovery
        const REINSTATE = 0x0008;
        /// Dynamically installed route (RTF_DYNAMIC)
        /// Created by routing daemon or redirect, not static configuration
        const DYNAMIC = 0x0010;
        /// Modified route (RTF_MODIFIED)
        /// Altered by ICMP redirect or other dynamic update
        const MODIFIED = 0x0020;
        /// MTU field is valid (RTF_MTU)
        /// Specifies Path MTU Discovery information for this route
        const MTU = 0x0040;
        /// Window field is valid (RTF_WINDOW)
        /// Contains TCP window clamp value for this route
        const WINDOW = 0x0080;
        /// Initial RTT estimate (RTF_IRTT)
        /// Contains TCP initial round trip time estimate (in milliseconds)
        const IRTT = 0x0100;
        /// Reject route (RTF_REJECT)
        /// Packets will be dropped with ICMP unreachable error
        const REJECT = 0x200;
    }
}

pub struct Ipv4RouteTable {
    pub line_iter: std::iter::Skip<std::io::Lines<BufReader<File>>>,
}

#[cfg(target_os = "linux")]
impl Default for Ipv4RouteTable {
    fn default() -> Self {
        Self {
            line_iter: BufReader::new(File::open("/proc/net/route").unwrap())
                .lines()
                .skip(1),
        }
    }
}

impl Iterator for Ipv4RouteTable {
    type Item = Ipv4RouteEntry;

    fn next(&mut self) -> Option<Self::Item> {
        use super::utils::*;
        let Some(Ok(line)) = self.line_iter.next() else {
            return None;
        };
        let fields: Vec<&str> = line.split_whitespace().collect();
        Some(Ipv4RouteEntry {
            name: fields[0].to_string(),
            dest: hex_str_to_ipv4(fields[1]).unwrap(),
            gateway: hex_str_to_ipv4(fields[2]).unwrap(),
            flags: Ipv4RouteFlags::from_bits_retain(u16::from_be_bytes(
                (*hex_str_to_bytes(fields[3]).unwrap()).try_into().unwrap(),
            )),
            ref_count: hex_char_to_u8(fields[4].as_bytes()[0]).unwrap(),
            use_count: hex_char_to_u8(fields[5].as_bytes()[0]).unwrap(),
            metric: hex_char_to_u8(fields[6].as_bytes()[0]).unwrap(),
            mask: hex_str_to_ipv4(fields[7]).unwrap(),
            mtu: hex_char_to_u8(fields[8].as_bytes()[0]).unwrap(),
            window: hex_char_to_u8(fields[9].as_bytes()[0]).unwrap(),
            irtt: hex_char_to_u8(fields[10].as_bytes()[0]).unwrap(),
        })
    }
}
