

/// Naming convention commonly used by DAT producers.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NamingConvention {
    /// Not a known naming convention
    Unknown,
    /// The naming convention used by The Old School Emulation Center
    ///
    /// Defined by the [TOSEC Naming Convention (2015-03-23)](https://www.tosecdev.org/tosec-naming-convention),
    /// with support for violations present in [TOSEC 2021-02-14](https://www.tosecdev.org/news/releases/167-tosec-release-2021-02-14).
    ///
    /// For more information, see [`naming::tosec`](tosec/index.html).
    TOSEC,
    /// The naming convention used by No-Intro, Redump, and others.
    ///
    /// Defined by the [No-Intro Naming Convention (2007-10-30)](https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf)
    /// with support for extensions used by Redump as determined empirically.
    ///
    /// For more information, see [`naming::nointro`](nointro/index.html).
    NoIntro,
    /// The naming convention used by GoodTools.
    ///
    /// Defined loosely by [GoodCodes.txt](https://raw.githubusercontent.com/SnowflakePowered/shiratsu/f5b668c44d9087204d0ec94b3002c643a5d82109/shiratsu-naming/src/naming/goodtools/GoodCodes.txt).
    ///
    /// Also uses information from [Emulation GameTech Wiki](https://emulation.gametechwiki.com/index.php/GoodTools)
    ///
    /// Specifically guarantees support for the 2016-04-03 GoodTools release,
    /// using DAT files from [OpenGood](https://github.com/SnowflakePowered/opengood)
    ///
    /// For more information, see [`naming::goodtools`](goodtools/index.html).
    GoodTools,
}

impl From<&NamingConvention> for &str {
    fn from(naming: &NamingConvention) -> Self {
        match naming {
            NamingConvention::Unknown => "Unknown",
            NamingConvention::TOSEC => "TOSEC",
            NamingConvention::NoIntro => "No-Intro",
            NamingConvention::GoodTools => "GoodTools",
        }
    }
}

impl AsRef<str> for NamingConvention {
    fn as_ref(&self) -> &str {
        self.into()
    } 
}