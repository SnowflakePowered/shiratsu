
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// The development status of a release.
pub enum DevelopmentStatus {
    /// A commercially released, or feature complete product, whether distributed gratis or not, 
    /// released in an official capacity by the publisher or developer.
    ///
    /// This is equivalent to a lack of a status tag in both NoIntro and TOSEC standards.
    /// If the (Sample) tag is present, the development status should be `DevelopmentStatus::Release`,
    /// with `NameInfo::is_demo()` returning true.
    Release,
    /// An unfinished, but mostly feature complete product, that may or may not have been intentionally released.
    ///
    /// In No-Intro, this is equivalent to the (Beta) flag. In TOSEC, (alpha), (beta), (preview), and (pre-release) all
    /// fall under this status.
    Prerelease,
    /// An unreleased, unfinished product that was not released.
    ///
    /// In No-Intro, this is equivalent to the (Proto) flag. In TOSEC, this is equivalent to the (proto) flag.
    Prototype,
}

impl From<&DevelopmentStatus> for &str {
    fn from(status: &DevelopmentStatus) -> Self {
        match status {
            DevelopmentStatus::Release => "release",
            DevelopmentStatus::Prerelease => "prerelease",
            DevelopmentStatus::Prototype => "prototype"
        }
    }
}

impl AsRef<str> for DevelopmentStatus {
    fn as_ref(&self) -> &str {
        self.into()
    } 
}