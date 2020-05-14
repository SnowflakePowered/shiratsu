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
        <game name="[BIOS] Nintendo 3DS - boot11 (World)">
            <description>[BIOS] Nintendo 3DS - boot11 (World)</description>
            <game_id>################</game_id>
            <rom name="[BIOS] Nintendo 3DS - boot11 (World).bin" size="65536" crc="39ED2007" md5="43454041B0CDDF1A34C130593AFA8B9B" sha1="F81039BBC208A54D9C6DEC31BD5D30F6EC09D2BD" status="verified"/>
        </game>
        <game name="[BIOS] Nintendo 3DS - boot9 (World)">
            <description>[BIOS] Nintendo 3DS - boot9 (World)</description>
            <game_id>################</game_id>
            <rom name="[BIOS] Nintendo 3DS - boot9 (World).bin" size="65536" crc="E0989F6D" md5="D8675E80E5DD3A9AFAAF885D79B14E9C" sha1="5A3D3D6DF4743E6B50AFE0FC717FA8A12BC888E6" status="verified"/>
        </game>
        <game name="100% Pascal Sensei - Kanpeki Paint Bombers (Japan)">
            <description>100% Pascal Sensei - Kanpeki Paint Bombers (Japan)</description>
            <game_id>00040000001AE600</game_id>
            <rom name="100% Pascal Sensei - Kanpeki Paint Bombers (Japan).3ds" size="536870912" crc="7D831164" md5="C5C9A7F2737D9CB2C49D8273483C4EAC" sha1="611D812AE65A6C0B5C8965BB0F31BB41A34B769C" serial="CTR-P-BP4J"/>
            <rom name="100% Pascal Sensei - Kanpeki Paint Bombers (Japan).3ds" size="536870912" crc="7D831164" md5="C5C9A7F2737D9CB2C49D8273483C4EAC" sha1="611D812AE65A6C0B5C8965BB0F31BB41A34B769C" serial="CTR-P-BP4J"/>
            <rom name="100% Pascal Sensei - Kanpeki Paint Bombers (Japan).3ds" size="536870912" crc="7D831164" md5="C5C9A7F2737D9CB2C49D8273483C4EAC" sha1="611D812AE65A6C0B5C8965BB0F31BB41A34B769C" serial="CTR-P-BP4J"/>

            </game>
    </datafile>"#;
    let vecs = shiratsu_lib::dats::parse_nointro(xml)?;
    for game in vecs.iter() {
        println!("{:?}", game)
    }
    Ok(())
}
