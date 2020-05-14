extern crate shiratsu_lib;
use shiratsu_lib::dats::Result;

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
        <game name="Resident Evil 4, An - A, Die Neue, An Helping Hand These (USA) (Disc 2)">
		<category>Games</category>
		<description>Resident Evil 4 (USA) (Disc 2)</description>
		<rom name="Resident Evil 4 (USA) (Disc 2).iso" size="1459978240" crc="6c83a5ff" md5="2381acd2199d6e7566932df86901903d" sha1="c75f7936814636ffe03277f363fc3427c98602ee"/>
	</game>
    </datafile>"#;
    let vecs = shiratsu_lib::dats::parse_redump(xml)?;
    for game in vecs.iter() {
        println!("{:?}", game)
    }
    Ok(())
}
