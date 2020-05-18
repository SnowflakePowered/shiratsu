extern crate rusqlite;

use std::borrow::Cow;
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

mod database;

use database::ShiratsuDatabase;
fn main() -> Result<()>{
    let file = File::open("redump.dat")?;
    let mut reader = BufReader::new(file);

    let vecs = GameEntry::try_from_redump_buf(reader)?;
    let stone = StonePlatforms::get();
    // let demo = "Legend of TOSEC, The v20000101 (demo) (2019-04-02)(publisher)(US-EU)(Disc 3 of 3)(proto)";
    // let demo_no = "Legend of TOSEC, The Rev 2 (2019-04-02)(publisher)(US)(File)(proto)(File)";
    let platform_id: &PlatformId = "NINTENDO_GCN".try_into()?;
    // let platform = stone.platform(platform_id)?;
    // println!("{:?}", platform.get_mimetype_for_ext("nes"));
    let mut db = ShiratsuDatabase::new().unwrap();
    // println!("Parsed {} games", vecs.iter().count());
    // let nes = String::from("NINTENDO").try_into()?;

    let serial = Serial::new("LSP-020170".to_string());
    // match  {
        
    // }
    println!("{:?}", serial.as_normalized("SONY_PSX".try_into()?));
    for game in vecs.iter() {
        db.add_entry(game, platform_id).unwrap();
    }

    db.save("test.db",None).unwrap();
    // println!("{:?}", stone.platform(nes)?);
    println!("{:?}", StonePlatforms::version());
    Ok(())
}
