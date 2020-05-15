extern crate shiratsu_lib;
use shiratsu_lib::dats::*;
use shiratsu_lib::dats::{
    nointro::*,
    tosec::*,
    redump::*,
};
use shiratsu_lib::dats::Result;
use shiratsu_lib::stone::get_platforms;
fn main() -> Result<()>{
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE datafile PUBLIC "-//Logiqx//DTD ROM Management Datafile//EN" "http://www.logiqx.com/Dats/datafile.dtd">
    <datafile>
        <header>
            <name>Nintendo - Nintendo 3DS (Encrypted)</name>
            <description>Nintendo - Nintendo 3DS (Encrypted)</description>
            <version>20200422-094756</version>
            <author>aci68, ajax16384, ajshell1, b2071988, Bent, C. V. Reynolds, Connie, coraz, Datman, DeadSkullzJr, Densetsu, einstein95, Fonix, Gefflon, Hiccup, InternalLoss, Jack, jimmsu, Money_114, PPLToast, relax, Rifu, scorp256, SonGoku, Tauwasser, Xenirina, xuom2, zg</author>
            <homepage>No-Intro</homepage>
            <url>http://www.no-intro.org</url>
        </header>
        <game name="Advance Guardian Heroes (2004)(Treasure - Ubisoft)(US)[!]">
		<description>Advance Guardian Heroes (2004)(Treasure - Ubisoft)(US)[!]</description>
		<rom name="Advance Guardian Heroes (2004)(Treasure - Ubisoft)(US)[!].bin" size="8388608" crc="c501917f" sha1="d518d0a4818cc356ed79b29bc3c0e2264c0c2d07" md5="a26c440065d89f56275644ffa34140ef"/>
	</game>
    <game name="Phantasy Star Online Episode I &amp; II (Europe) (En,Ja,Fr,De,Es)">
    <category>Games</category>
    <serial>DL-DOL-GPOP-EUR, SLUS 10230</serial>
    <description>Phantasy Star Online Episode I &amp; II (Europe) (En,Ja,Fr,De,Es)</description>
    <rom name="Phantasy Star Online Episode I &amp; II (Europe) (En,Ja,Fr,De,Es).iso" size="1459978240" crc="c9c7fbfe" md5="ee4c772cc90ecb6537a331538e1bf4db" sha1="da74531ed5bd62af29af4afe8ac33d97b0d21d4c"/>
</game>
    </datafile>"#;
    let vecs = GameEntry::try_unchecked_from_redump(xml)?;
    let demo = "Legend of TOSEC, The v20000101 (demo) (2019-04-02)(publisher)(US-EU)(Disc 3 of 3)(proto)";
    let demo_no = "Legend of TOSEC, The Rev 2 (2019-04-02)(publisher)(US)(File)(proto)(File)";
    for game in vecs.iter() {
        println!("{}", game)
    }
    Ok(())
}
