use crate::{
    RouteParseError,
    utils::{hex_char_to_u8, hex_str_to_bytes, hex_str_to_ipv4},
};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    iter::Skip,
    net::Ipv4Addr,
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone)]
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
    #[derive(Debug,Clone)]
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
    lines: Skip<Lines<BufReader<File>>>,
}

impl Ipv4RouteTable {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let reader = File::open_buffered(path)?;
        let lines = reader.lines().skip(1);
        Ok(Self { lines })
    }
}

impl Iterator for Ipv4RouteTable {
    type Item = Result<Ipv4RouteEntry, RouteParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line_result| {
            let line = line_result?;
            line.parse::<Ipv4RouteEntry>()
        })
    }
}

impl FromStr for Ipv4RouteEntry {
    type Err = RouteParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        const IPV4_ROUTE_FILED_COUNT: usize = 11;
        if fields.len() < IPV4_ROUTE_FILED_COUNT {
            return Err(RouteParseError::InvalidFieldCount {
                expected: IPV4_ROUTE_FILED_COUNT,
                found: fields.len(),
            });
        }
        let get_field = |i: usize| {
            fields
                .get(i)
                .cloned()
                .ok_or(RouteParseError::MissingField(i))
        };
        Ok(Ipv4RouteEntry {
            name: get_field(0)?.to_string(),
            dest: hex_str_to_ipv4(get_field(1)?)?,
            gateway: hex_str_to_ipv4(get_field(2)?)?,
            flags: Ipv4RouteFlags::from_bits_retain(u16::from_be_bytes(
                (*hex_str_to_bytes(get_field(3)?)?).try_into()?,
            )),
            ref_count: hex_char_to_u8(get_field(4)?.as_bytes()[0])?,
            use_count: hex_char_to_u8(get_field(5)?.as_bytes()[0])?,
            metric: hex_char_to_u8(get_field(6)?.as_bytes()[0])?,
            mask: hex_str_to_ipv4(get_field(7)?)?,
            mtu: hex_char_to_u8(get_field(8)?.as_bytes()[0])?,
            window: hex_char_to_u8(get_field(9)?.as_bytes()[0])?,
            irtt: hex_char_to_u8(get_field(10)?.as_bytes()[0])?,
        })
    }
}
