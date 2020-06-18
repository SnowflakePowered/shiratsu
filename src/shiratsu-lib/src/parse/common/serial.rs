use std::borrow::Cow;
use regex::Regex;
use lazy_static::lazy_static;

use crate::stone::PlatformId;
#[derive(Debug, Clone)]
pub struct Serial(String);

impl Serial {
    pub fn new(serial_str: String) -> Serial {
        Serial(serial_str)
    }

    pub fn as_normalized<'a>(&'a self, ruleset: &PlatformId) -> Cow<'a, Serial> {
        match ruleset.as_ref() {
            "SONY_PSX" | "SONY_PS2" | "SONY_PS3" | "SONY_PS4" |"SONY_PSP" | "SONY_PSV" => rule_sony(self),
            "NINTENDO_GCN" => rule_nintendo_gcn(self),
            "NINTENDO_WII" => rule_nintendo_wii(self),
            "NINTENDO_WIIU" => rule_nintendo_wiiu(self),
            "NINTENDO_3DS" => rule_nintendo_3ds(self),
           _ => Cow::Borrowed(self)
        }
    }
}

fn rule_nintendo<'a>(regex: &Regex, serial: &'a Serial) -> Cow<'a, Serial> {
    let serial_str = serial.as_ref();

    if regex.is_match(serial_str) {
        Cow::Owned(Serial::new(String::from(&serial_str[7..11])))
    } else {
        Cow::Borrowed(serial)
    }
}

fn rule_nintendo_gcn<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref VERIFY_RULE: Regex = Regex::new(r"^DL-DOL-([\w]{4})-[-\w\(\)]+$").unwrap();
    }

    rule_nintendo(&VERIFY_RULE, serial)
}


fn rule_nintendo_wii<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref VERIFY_RULE: Regex = Regex::new(r"^RVL-([\w]{4})-[-\w\(\)]+$").unwrap();
    }
    rule_nintendo(&VERIFY_RULE, serial)
}

fn rule_nintendo_wiiu<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref VERIFY_RULE: Regex = Regex::new(r"^WUP-P-([\w]{4})-[-\w\(\)]+$").unwrap();
    }
    rule_nintendo(&VERIFY_RULE, serial)
}

fn rule_nintendo_3ds<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref VERIFY_RULE: Regex = Regex::new(r"^CTR-P-([\w]{4})-[-\w\(\)]+$").unwrap();
    }
    rule_nintendo(&VERIFY_RULE, serial)
}

fn rule_sony<'a>(serial: &'a Serial) -> Cow<'a, Serial> {
    lazy_static! {
        static ref VERIFY_RULE: Regex = Regex::new(r"^[a-zA-Z]+[-]\d+(/\w+)?$").unwrap();
        static ref REWRITE_RULE: Regex = Regex::new(r"^(?P<code>[a-zA-Z]+)[-_ ](?P<number>\d+)([-_ /]\w+)*$").unwrap();
    }
    let serial_str = serial.as_ref();
    
    if VERIFY_RULE.is_match(serial_str) {
        return Cow::Borrowed(serial);
    }

    match REWRITE_RULE.replace_all(serial_str, "$code-$number") {
        Cow::Borrowed(_) => {
            Cow::Borrowed(serial)
        },
        Cow::Owned(new_string) => Cow::Owned(Serial::new(new_string))
    }
}

impl AsRef<str> for Serial {
    fn as_ref(&self) -> &str {
        &self.0
    }
}