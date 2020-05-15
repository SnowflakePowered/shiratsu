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
	<game name="Advance Wars (2001)(Nintendo)(EU)(M4)[!]">
		<description>Advance Wars (2001)(Nintendo)(EU)(M4)[!]</description>
		<rom name="Advance Wars (2001)(Nintendo)(EU)(M4)[!].bin" size="8388608" crc="66fb29e9" sha1="d5f06a82c3e5f963ef169763edc2d691fed8124e" md5="f4c2b2fda444dcec1274844b9a764d64"/>
	</game>
	<game name="Altered Beast - Guardian of the Realms (2002)(Infogrames - Sega)(EU)(M5)[!]">
		<description>Altered Beast - Guardian of the Realms (2002)(Infogrames - Sega)(EU)(M5)[!]</description>
		<rom name="Altered Beast - Guardian of the Realms (2002)(Infogrames - Sega)(EU)(M5)[!].bin" size="8388608" crc="654f7916" sha1="4178bc2c89187dce127ab64dac2becf99fed0679" md5="c7c56aaed390488e4112ef64b51bb2ad"/>
	</game>
    </datafile>"#;
    let vecs = GameEntry::try_unchecked_from_tosec(xml)?;
    let demo = "Legend of TOSEC, The v20000101 (demo) (2019-04-02)(publisher)(US-EU)(Disc 3 of 3)(proto)";
    let demo_no = "Legend of TOSEC, The Rev 2 (2019-04-02)(publisher)(US)(File)(proto)(File)";
    for game in vecs.iter() {
        println!("{}", game)
    }
    // for platform in get_platforms() {
    //     println!("{:?}", platform.1)
    // }
    Ok(())
}
