use std::fmt;
use crate::NameInfo;

impl fmt::Display for NameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "    (entry \"{}\")", self.entry_title())?;
        writeln!(f, "    (release \"{}\")", self.release_title())?;
        writeln!(
            f,
            "    (region {:?})",
            self.region()
                .iter()
                .map(|r| r.into())
                .collect::<Vec<&str>>()
        )?;
        writeln!(f, "    (part {})", self.part_number().map(|i| format!("{}", i)).as_deref().unwrap_or("None"))?;
        writeln!(f, "    (version \"{}\")", self.version().unwrap_or("None"))?;
        writeln!(f, "    (status {:?})", self.development_status())?;
        writeln!(f, "    (is-demo? {})", self.is_demo())?;
        writeln!(f, "    (is-unlicensed? {})", self.is_unlicensed())?;
        write!(f, "    (naming {:?})", self.naming_convention())
    }
}