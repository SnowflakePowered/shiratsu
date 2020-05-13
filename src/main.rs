extern crate shiratsu_lib;

use shiratsu_lib::stone;
use shiratsu_lib::region;
fn main() -> Result<(), region::RegionError>{
    println!("{:?}", stone::get_stone());
    println!("{:?}", region::parse_regions("JP-GR, EU").iter().map(|r| r.into())
        .collect::<Vec<&'static str>>());
    Ok(())
}
