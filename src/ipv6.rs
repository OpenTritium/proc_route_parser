use crate::{
    RouteParseError,
    ipv4::Ipv4RouteFlags,
    utils::{hex_char_pair_to_byte, hex_str_to_bytes, hex_str_to_ipv6},
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Result as IoResult},
    net::Ipv6Addr,
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct Ipv6RouteEntry {
    pub dest: Ipv6Addr,
    pub dest_prefix: u8,
    pub src: Ipv6Addr,
    pub src_prefix: u8,
    pub next_hop: Ipv6Addr,
    pub metric: u32,
    pub ref_count: u32,
    pub use_count: u32,
    pub flags: Ipv6RouteFlags,
    pub name: String,
}

bitflags::bitflags! {
    #[derive(Debug,Clone)]
    pub struct Ipv6RouteFlags:u32 {
        /// Route is active and available (RTF_UP)
        /// Indicates the route is valid and operational
        const UP = Ipv4RouteFlags::UP.bits() as u32;
        /// Route uses a gateway (RTF_GATEWAY)
        /// When set, the nexthop field contains a valid gateway address
        const GATEWAY = Ipv4RouteFlags::GATEWAY.bits() as u32;
        /// Host route (specific to single host) (RTF_HOST)
        /// Indicates the destination is a complete host address
        const HOST = Ipv4RouteFlags::HOST.bits() as u32;
        /// Reinstate route for dynamic routing (RTF_REINSTATE)
        /// Used by routing daemons to restore routes after link recovery
        const REINSTATE = Ipv4RouteFlags::REINSTATE.bits() as u32;
        /// Dynamically installed route (RTF_DYNAMIC)
        /// Created by routing daemon or redirect, not static configuration
        const DYNAMIC = Ipv4RouteFlags::DYNAMIC.bits() as u32;
        /// Modified route (RTF_MODIFIED)
        /// Altered by ICMP redirect or other dynamic update
        const MODIFIED = Ipv4RouteFlags::MODIFIED.bits() as u32;
        /// MTU field is valid (RTF_MTU)
        /// Specifies Path MTU Discovery information for this route
        const MTU = Ipv4RouteFlags::MTU.bits() as u32;
        /// Window field is valid (RTF_WINDOW)
        /// Contains TCP window clamp value for this route
        const WINDOW = Ipv4RouteFlags::WINDOW.bits() as u32;
        /// Initial RTT estimate (RTF_IRTT)
        /// Contains TCP initial round trip time estimate (in milliseconds)
        const IRTT = Ipv4RouteFlags::IRTT.bits() as u32;
        /// Reject route (RTF_REJECT)
        /// Packets will be dropped with ICMP unreachable error
        const REJECT = Ipv4RouteFlags::REJECT.bits() as u32;
        /// Default route learned via Neighbor Discovery (ND) protocol.
        /// Corresponds to `RTF_DEFAULT` (0x00010000).
        const DEFAULT = 0x00010000;
        /// (Deprecated) All gateways assumed to be on the same physical link.
        /// Corresponds to `RTF_ALLONLINK` (0x00020000).
        /// Note: This flag is deprecated and may be removed in future kernel versions.
        const ALL_ON_LINK = 0x00020000;
        /// Route created through address auto-configuration (SLAAC/RA).
        /// Corresponds to `RTF_ADDRCONF` (0x00040000).
        const ADDR_CONF = 0x00040000;
        /// Prefix-only route from Router Advertisement (RA) messages.
        /// Corresponds to `RTF_PREFIX_RT` (0x00080000).
        const PREFIX_ONLY = 0x00080000;
        /// Route to an anycast address (shared by multiple nodes).
        /// Corresponds to `RTF_ANYCAST` (0x00100000).
        const ANYCAST = 0x00100000;
        /// Route without explicit next-hop address (requires route lookup).
        /// Corresponds to `RTF_NONEXTHOP` (0x00200000).
        const NO_NEXT_HOP = 0x00200000;
        /// Temporary route with expiration time (automatically removed).
        /// Corresponds to `RTF_EXPIRES` (0x00400000).
        const EXPIRES = 0x00400000;
        /// Route created from RA Route Information Option (RFC 4191).
        /// Corresponds to `RTF_ROUTEINFO` (0x00800000).
        const ROUTE_INFO = 0x00800000;
        /// Read-only cached route entry (managed by kernel, not user).
        /// Corresponds to `RTF_CACHE` (0x01000000).
        const CACHE	= 0x01000000;
        /// Flow-specific route (uncommon, relates to IPv6 Flow Label field).
        /// Corresponds to `RTF_FLOW` (0x02000000).
        const FLOW	= 0x02000000;
        /// Policy-based routing entry (non-standard path selection).
        /// Corresponds to `RTF_POLICY` (0x04000000).
        const POLICY = 0x04000000;
        /// Reserved preference value (should not be used).
        /// Part of RTF_PREF mask (0x18000000).
        const PREF_RESERVED = 0;
        /// High priority route preference (value 1 << 27).
        /// Part of RTF_PREF mask (0x18000000).
        const PREF_HIGH = 1 << 27;
        /// Medium priority route preference (value 2 << 27).
        /// Part of RTF_PREF mask (0x18000000).
        const PREF_MEDIUM = 2 << 27;
        /// Low priority route preference (value 3 << 27).
        /// Part of RTF_PREF mask (0x18000000).
        const PREF_LOW = 3 << 27;
        /// Per-CPU route cache entry (kernel-managed optimization).
        /// Corresponds to `RTF_PCPU` (0x40000000).
        /// Note: Read-only flag, cannot be set by userspace.
        const PER_CPU = 0x40000000;
        /// Local interface route (loopback or interface address).
        /// Corresponds to `RTF_LOCAL` (0x80000000).
        const LOCAL = 0x80000000;
    }
}

pub struct Ipv6RouteTable {
    line_iter: Lines<BufReader<File>>,
}

impl Ipv6RouteTable {
    pub fn open(file_path: impl AsRef<Path>) -> IoResult<Self> {
        let reader = File::open_buffered(file_path)?;
        let line_iter = reader.lines();
        Ok(Self { line_iter })
    }
}

impl FromStr for Ipv6RouteEntry {
    type Err = RouteParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            return Err(RouteParseError::InvalidFieldCount {
                expected: 10,
                found: fields.len(),
            });
        }
        let get_field = |i: usize| {
            fields
                .get(i)
                .cloned()
                .ok_or(RouteParseError::MissingField(i))
        };

        Ok(Ipv6RouteEntry {
            dest: hex_str_to_ipv6(get_field(0)?)?,
            dest_prefix: hex_char_pair_to_byte(get_field(1)?.as_bytes().try_into()?)?,
            src: hex_str_to_ipv6(get_field(2)?)?,
            src_prefix: hex_char_pair_to_byte(get_field(3)?.as_bytes().try_into()?)?,
            next_hop: hex_str_to_ipv6(get_field(4)?)?,
            metric: u32::from_be_bytes((*hex_str_to_bytes(get_field(5)?)?).try_into()?),
            ref_count: u32::from_be_bytes((*hex_str_to_bytes(get_field(6)?)?).try_into()?),
            use_count: u32::from_be_bytes((*hex_str_to_bytes(get_field(7)?)?).try_into()?),
            flags: Ipv6RouteFlags::from_bits_retain(u32::from_be_bytes(
                (*hex_str_to_bytes(get_field(8)?)?).try_into()?,
            )),
            name: get_field(9)?.to_string(),
        })
    }
}

impl Iterator for Ipv6RouteTable {
    type Item = Result<Ipv6RouteEntry, RouteParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_iter.next().map(|line_result| {
            let line = line_result?;
            line.parse::<Ipv6RouteEntry>()
        })
    }
}
