use crate::region::Region;
use crate::naming::FlagType;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsToken<'a>
{
    Title(&'a str),
    Region(Vec<&'a str>, Vec<Region>),
    Year(&'a str),
    Multilanguage(&'a str), // (M#)
    Translation(TranslationStatus, &'a str), // [T(+/-)...]
    Version(&'a str, &'a str, Option<&'a str>), // (REV/V/V /V_ ...)
    Volume(&'a str), // (Vol #)
    NInOne(Vec<&'a str>, Option<&'a str>), // list, sep (either + or ,)
    DumpCode(&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>), // (code, number, type, sep, argnum, args)
    GameHack(Option<&'a str>), // (... Hack)
    Media(&'a str, &'a str, Option<&'a str>),
    Flag(FlagType, &'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TranslationStatus
{
    Recent,
    Outdated,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GoodTool
{
    Generic,
    Good2600,
    Good5200,
    Good7800,
    GoodChaF,
    GoodCoCo,
    GoodCol,
    GoodCPC,
    GoodGB64,
    GoodGBA,
    GoodGBx,
    GoodGCOM,
    GoodGen,
    GoodGG,
    GoodINTV,
    GoodJag,
    GoodLynx,
    GoodMO5,
    GoodMSX1,
    GoodMSX2,
    GoodMTX,
    GoodN64,
    GoodNES,
    GoodNGPx,
    GoodOric,
    GoodPCE,
    GoodPico,
    GoodPSID,
    GoodSAMC,
    GoodSMS,
    GoodSNES,
    GoodSPC,
    GoodSV,
    GoodVBoy,
    GoodVect,
    GoodWSx
}

impl Default for GoodTool {
    fn default() -> Self {
        GoodTool::Generic
    }
}

impl <T> From<T> for GoodTool
    where T: AsRef<str>
{
    fn from(tool: T) -> Self {
        let s = tool.as_ref().strip_prefix("Good")
            .unwrap_or(tool.as_ref());

        match s {
            "2600" => GoodTool::Good2600,
            "5200" => GoodTool::Good5200,
            "7800" => GoodTool::Good7800,
            "ChaF" => GoodTool::GoodChaF,
            "CoCo" => GoodTool::GoodCoCo,
            "Col" => GoodTool::GoodCol,
            "CPC" => GoodTool::GoodCPC,
            "GB64" => GoodTool::GoodGB64,
            "GBA" => GoodTool::GoodGBA,
            "GBx" => GoodTool::GoodGBx,
            "GCOM" => GoodTool::GoodGCOM,
            "Gen" => GoodTool::GoodGen,
            "GG" => GoodTool::GoodGG,
            "INTV" => GoodTool::GoodINTV,
            "Jag" => GoodTool::GoodJag,
            "Lynx" => GoodTool::GoodLynx,
            "MO5" => GoodTool::GoodMO5,
            "MSX1" => GoodTool::GoodMSX1,
            "MSX2" => GoodTool::GoodMSX2,
            "MTX" => GoodTool::GoodMTX,
            "N64" => GoodTool::GoodN64,
            "NES" => GoodTool::GoodNES,
            "NGPx" => GoodTool::GoodNGPx,
            "Oric" => GoodTool::GoodOric,
            "PCE" => GoodTool::GoodPCE,
            "Pico" => GoodTool::GoodPico,
            "PSID" => GoodTool::GoodPSID,
            "SAMC" => GoodTool::GoodSAMC,
            "SMS" => GoodTool::GoodSMS,
            "SNES" => GoodTool::GoodSNES,
            "SPC" => GoodTool::GoodSPC,
            "SV" => GoodTool::GoodSV,
            "VBoy" => GoodTool::GoodVBoy,
            "Vect" => GoodTool::GoodVect,
            "WSx" => GoodTool::GoodWSx,
            _ => GoodTool::Generic
        }
    }
}