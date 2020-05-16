extern crate shiratsu_lib;
use std::fs::File;
use std::io::BufReader;


use shiratsu_lib::parse::*;
use shiratsu_lib::stone::PlatformId;
use shiratsu_lib::parse::{
    nointro::*,
    tosec::*,
    redump::*,
};

use std::convert::TryInto;
use shiratsu_lib::error::Result;
use shiratsu_lib::stone::*;
fn main() -> Result<()>{
    let file = File::open("redump.dat")?;
    let mut reader = BufReader::new(file);

    let vecs = GameEntry::try_unchecked_from_redump_buf(reader)?;
    let stone = StonePlatforms::get();
    // let demo = "Legend of TOSEC, The v20000101 (demo) (2019-04-02)(publisher)(US-EU)(Disc 3 of 3)(proto)";
    // let demo_no = "Legend of TOSEC, The Rev 2 (2019-04-02)(publisher)(US)(File)(proto)(File)";
    let platform_id: &PlatformId = "NINTENDO_GCN".try_into()?;
    let platform = stone.platform(platform_id)?;
    println!("{:?}", platform.get_mimetype_for_ext("nes"));
    // println!("Parsed {} games", vecs.iter().count());
    for game in vecs.iter() {
        for rom in game.rom_entries() {
            println!("{:?}", rom.find_mimetype(platform_id));
        }
    }
    let nes = String::from("NINTENDO_NES").try_into()?;
    println!("{:?}", stone.platform(nes)?);
    Ok(())
}
