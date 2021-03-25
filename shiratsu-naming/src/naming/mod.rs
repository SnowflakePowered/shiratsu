// Re-export common at top level

pub use common::NamingConvention;
pub use common::tokens::FlagType;
pub(crate) use common::parsers;

#[macro_use]
mod common;

/// Parsers for the No-Intro naming convention.
pub mod nointro;

/// Parsers for the TOSEC naming convention.
pub mod tosec;

/// Parsers for the GoodTools naming convention.
pub mod goodtools;


