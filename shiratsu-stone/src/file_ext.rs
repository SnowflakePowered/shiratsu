use serde::{Deserialize, Deserializer};
use std::hash::Hash;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct FileExt(String);

impl AsRef<str> for FileExt
{
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl <'de> Deserialize<'de> for FileExt
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let v: String = Deserialize::deserialize(deserializer)?;
        Ok(FileExt::from(v))
    }
}

impl From<String> for FileExt
{
    fn from(s: String) -> Self {
        if s.starts_with(".") {
            FileExt(s[1..].to_lowercase())
        } else {
            FileExt(s.to_lowercase())
        }
    }
}

impl From<&str> for FileExt
{
    fn from(s: &str) -> Self {
        if s.starts_with(".") {
            FileExt(s[1..].to_lowercase())
        } else {
            FileExt(s.to_lowercase())
        }
    }
}