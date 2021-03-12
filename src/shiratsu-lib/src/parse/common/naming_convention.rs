

/// Naming convention commonly used by DAT producers.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NamingConvention {
    /// Not a known naming convention
    Unknown,
    /// The naming convention used by The Old School Emulation Center
    ///
    /// Defined at https://www.tosecdev.org/tosec-naming-convention
    TOSEC,
    /// The naming convention used by No-Intro, redump.org, and others.
    ///
    /// Defined at https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf
    NoIntro,
}

impl From<&NamingConvention> for &str {
    fn from(naming: &NamingConvention) -> Self {
        match naming {
            NamingConvention::Unknown => "Unknown",
            NamingConvention::TOSEC => "TOSEC",
            NamingConvention::NoIntro => "No-Intro",
        }
    }
}

impl AsRef<str> for NamingConvention {
    fn as_ref(&self) -> &str {
        self.into()
    } 
}