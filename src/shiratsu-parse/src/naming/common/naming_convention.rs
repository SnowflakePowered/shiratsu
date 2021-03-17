

/// Naming convention commonly used by DAT producers.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NamingConvention {
    /// Not a known naming convention
    Unknown,
    /// The naming convention used by The Old School Emulation Center
    /// 
    /// Compatible with TNC v3 or later.
    ///
    /// Defined at https://www.tosecdev.org/tosec-naming-convention
    TOSEC,
    /// The naming convention used by No-Intro, redump.org, and others.
    ///
    /// Defined at https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf
    NoIntro,
    /// The naming convention used by GoodTools.
    ///
    /// Defined loosely by GoodCodes.txt 1.0.0 by Psych0phobiA / q^-o|o-^p
    ///
    /// Also uses information from
    /// https://emulation.gametechwiki.com/index.php/GoodTools
    ///
    /// Empirically tested using OpenGood DAT files from
    /// https://github.com/SnowflakePowered/opengood
    GoodTools,
    /// The third revision of the TOSEC naming convention, supporting
    /// ZZZ-UNK- names.
    ///
    /// Referenced from
    /// https://web.archive.org/web/20141228081706/https://www.tosecdev.org/tosec-naming-convention
    TOSECV0,
}

impl From<&NamingConvention> for &str {
    fn from(naming: &NamingConvention) -> Self {
        match naming {
            NamingConvention::Unknown => "Unknown",
            NamingConvention::TOSEC => "TOSEC",
            NamingConvention::NoIntro => "No-Intro",
            NamingConvention::GoodTools => "GoodTools",
            NamingConvention::TOSECV0 => "TOSEC"
        }
    }
}

impl AsRef<str> for NamingConvention {
    fn as_ref(&self) -> &str {
        self.into()
    } 
}