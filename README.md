# PROCFS ROUTE PARSER

## Supported Platform

Linux Only.

## How dose it work

it will read `/proc/net/route` and `/proc/net/ipv6_route` then parse them.

## How to use

```rust
fn main() -> Result<()> {
    for entry_result in get_ipv6_route_table()? {
        if let Ok(entry) = entry_result {
            if entry.flags.contains(Ipv6RouteFlags::UP) {
                println!("{:?}", entry);
            }
        }
    }
    Ok(())
}
```
