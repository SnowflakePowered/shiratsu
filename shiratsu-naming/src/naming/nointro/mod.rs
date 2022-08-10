//! Parsers for the No-Intro naming convention.
//!
//! Defined by the [No-Intro Naming Convention (2007-10-30)](https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf)
//! with support for extensions used by Redump as determined empirically.
//!
//! Such extensions include media type parts, version parsing extensions, etc.
//!
//! As per the No-Intro Naming Convention, this parser requires that a valid region flag
//! occurs before any other flag.
//!
//! ## Usage
//! ```
//! use shiratsu_naming::naming::nointro::NoIntroName;
//! use shiratsu_naming::naming::{NameError, TokenizedName};
//!
//! fn parse() -> Result<(), NameError> {
//!     let name = NoIntroName::try_parse("FIFA 20 - Portuguese (Brazil) In-Game Commentary (World) (Pt-BR) (DLC) (eShop)")?;
//!     assert_eq!(Some("FIFA 20 - Portuguese (Brazil) In-Game Commentary"), name.title());
//!     Ok(())
//! }
//! ```
mod parsers;
mod tokens;

pub use tokens::*;
