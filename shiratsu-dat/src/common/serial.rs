use std::borrow::Cow;
use regex::Regex;
use lazy_static::lazy_static;
use shiratsu_stone::PlatformId;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Serial(String);

impl Serial {
    pub fn new(serial_str: String) -> Serial {
        Serial(serial_str)
    }

    pub fn as_normalized(&self, ruleset: &PlatformId) -> Cow<Serial>
    {
        match ruleset.as_ref() {
            "SONY_PSX" | "SONY_PS2" | "SONY_PS3" | "SONY_PS4" |"SONY_PSP" | "SONY_PSV" => rule_sony(self),
            "NINTENDO_GCN" => rule_nintendo_gcn(self),
            "NINTENDO_WII" => rule_nintendo_wii(self),
            "NINTENDO_WIIU" => rule_nintendo_wiiu(self),
            "NINTENDO_3DS" => rule_nintendo_3ds(self),
            "NINTENDO_NSW" => rule_nintendo_nsw(self),
            "SEGA_GEN" | "SEGA_CD" | "SEGA_DC" | "SEGA_GG" | "SEGA_SAT" | "SEGA_32X" | "SEGA_32X_CD" => rule_sega(self),
            "NEC_TGCD" => rule_nec_tgcd(self),
           _ => Cow::Borrowed(self)
        }
    }
}

fn rule_nec_tgcd<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^(?P<code>[\d\w]{4,5})[ -](?P<number>[\d\w]+)$").unwrap();
    }
    let serial_str = serial.as_ref();
    match REWRITE_RULE.replace_all(serial_str, "$code$number") {
        Cow::Borrowed(_) => {
            Cow::Borrowed(serial)
        },
        Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
    }
}

fn rule_nintendo<'a>(regex: &Regex, serial: &'a Serial) -> Cow<'a, Serial> {
    let serial_str = serial.as_ref();

    match regex.replace_all(serial_str, "$code") {
        Cow::Borrowed(_) => {
            Cow::Borrowed(serial)
        },
        Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
    }
}

fn rule_nintendo_gcn<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^DL-DOL-(?P<code>[\w]{4})-[-\w\(\)]+$").unwrap();
    }

    rule_nintendo(&REWRITE_RULE, serial)
}


fn rule_nintendo_wii<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^RVL-(?P<code>[\w]{4})-[-\w\(\)]+$").unwrap();
    }
    rule_nintendo(&REWRITE_RULE, serial)
}

fn rule_nintendo_wiiu<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^WUP-(P|M|N|T|U|B)-(?P<code>[\w]{4})-[-\w\(\)]+$").unwrap();
    }
    rule_nintendo(&REWRITE_RULE, serial)
}

fn rule_nintendo_3ds<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^CTR-(P|M|N|T|U|B)-(?P<code>[\w]{4})(-[-\w\(\)]+)*$").unwrap();
    }
    rule_nintendo(&REWRITE_RULE, serial)
}

fn rule_nintendo_nsw<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^LA-H-(?P<code>[\w]{5})(-[-\w\(\)]+)*$").unwrap();
    }
    rule_nintendo(&REWRITE_RULE, serial)
}

fn rule_sony<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^(?P<code>[a-zA-Z]+)[-_ ](?P<number>\d+)([#-_ /]*(\w?|$))*$").unwrap();
    }
    let serial_str = serial.as_ref();
    
    match REWRITE_RULE.replace_all(serial_str, "$code-$number") {
        Cow::Borrowed(_) => {
            Cow::Borrowed(serial)
        },
        Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
    }
}


// 
fn rule_sega<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref REWRITE_RULE: Regex = Regex::new(r"^(?P<pre>[\d\w]+)-(?P<code>[\d\w]+)(-[\w\d.]+)$").unwrap();
        static ref REWRITE_RULE_2: Regex = Regex::new(r"^(?P<pre>MK|T|GS)(?P<code>[\d\w]+)(-[\w\d.]+)?$").unwrap();
        static ref REWRITE_RULE_3: Regex = Regex::new(r"^(?P<pre>0{2,3})(?P<code>[\d]+)(-\d{2}\w?)?$").unwrap();
    }
    let serial_str = serial.as_ref();
    
    if REWRITE_RULE.is_match(serial_str) {
        return match REWRITE_RULE.replace_all(serial_str, "$pre-$code") {
            Cow::Borrowed(_) => {
                Cow::Borrowed(serial)
            },
            Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
        }
    }
    if REWRITE_RULE_2.is_match(serial_str) {
        return match REWRITE_RULE_2.replace_all(serial_str, "$pre-$code") {
            Cow::Borrowed(_) => {
                Cow::Borrowed(serial)
            },
            Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
        }
    }
    if REWRITE_RULE_3.is_match(serial_str) {
        return match REWRITE_RULE_3.replace_all(serial_str, "$pre$code") {
            Cow::Borrowed(_) => {
                Cow::Borrowed(serial)
            },
            Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
        }
    }
    
    Cow::Borrowed(serial)
}

#[test]
fn test_rule_sony() {
    assert_eq!("SLUS-20302", rule_sony(&Serial::new("SLUS 20302".to_string())).as_ref().as_ref());
    assert_eq!("SLUS-20216", rule_sony(&Serial::new("SLUS 20216GH".to_string())).as_ref().as_ref());
    assert_eq!("SLES-50330", rule_sony(&Serial::new("SLES-50330#2".to_string())).as_ref().as_ref());
    assert_eq!("SLES-50330", rule_sony(&Serial::new("SLES-50330#".to_string())).as_ref().as_ref());
    assert_eq!("BCUS-98114", rule_sony(&Serial::new("BCUS-98114".to_string())).as_ref().as_ref());
    assert_eq!("BCUS-98114", rule_sony(&Serial::new("BCUS-98114SA".to_string())).as_ref().as_ref());
    assert_eq!("SLES-50330", rule_sony(&Serial::new("SLES-50330/ANZ".to_string())).as_ref().as_ref());
}

#[test]
fn test_rule_nintendo() {
    assert_eq!("GC3E", rule_nintendo_gcn(&Serial::new("DL-DOL-GC3E-0-USA".to_string())).as_ref().as_ref());
    assert_eq!("SJRE", rule_nintendo_wii(&Serial::new("RVL-SJRE-USA-B0".to_string())).as_ref().as_ref());
    assert_eq!("AH9J", rule_nintendo_wiiu(&Serial::new("WUP-P-AH9J-JPN-0".to_string())).as_ref().as_ref());
    assert_eq!("JRBP", rule_nintendo_3ds(&Serial::new("CTR-N-JRBP".to_string())).as_ref().as_ref());
    assert_eq!("BABBD", rule_nintendo_nsw(&Serial::new("LA-H-BABBD".to_string())).as_ref().as_ref());
    assert_eq!("BABBD", rule_nintendo_nsw(&Serial::new("LA-H-BABBD-USA-0".to_string())).as_ref().as_ref());
}

#[test]
fn test_rule_sega() {
    assert_eq!("MK-81086", rule_sega(&Serial::new("MK-81086".to_string())).as_ref().as_ref());
    assert_eq!("MK-81086", rule_sega(&Serial::new("MK-81086-80".to_string())).as_ref().as_ref());
    assert_eq!("839-81086", rule_sega(&Serial::new("839-81086-50".to_string())).as_ref().as_ref());
    assert_eq!("MK-1034", rule_sega(&Serial::new("MK1034".to_string())).as_ref().as_ref());
    assert_eq!("MK-1034", rule_sega(&Serial::new("MK1034-50".to_string())).as_ref().as_ref());
    assert_eq!("T-81086", rule_sega(&Serial::new("T-81086-80".to_string())).as_ref().as_ref());
    assert_eq!("T-17704D", rule_sega(&Serial::new("T-17704D-09A".to_string())).as_ref().as_ref());
    assert_eq!("00001014", rule_sega(&Serial::new("00001014-00".to_string())).as_ref().as_ref());
}


impl AsRef<str> for Serial {
    fn as_ref(&self) -> &str {
        &self.0
    }
}