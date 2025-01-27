# How to use

```rust
use proc_route_parser::{
    Ipv4RouteFlags, Ipv6RouteFlags, get_ipv4_route_table, get_ipv6_route_table,
};

fn main() {
    for e in get_ipv4_route_table() {
        if e.flags
            .contains(Ipv4RouteFlags::UP | Ipv4RouteFlags::REJECT)
        {
            println!("{:?}", e);
        }
    }
    for e in get_ipv6_route_table() {
        if e.flags.contains(Ipv6RouteFlags::UP) {
            println!("{:?}", e);
        }
    }
}
```