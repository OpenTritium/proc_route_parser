use anyhow::Result;
use proc_route_parser::*;

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
