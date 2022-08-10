//! Parsers for the TOSEC naming convention.
//!
//! Defined by the [TOSEC Naming Convention (2015-03-23)](https://www.tosecdev.org/tosec-naming-convention),
//! with support for violations present in [TOSEC 2021-02-14](https://www.tosecdev.org/news/releases/167-tosec-release-2021-02-14).
//!
//! The TOSEC parser will parse invalid file names beginning with `ZZZ-UNK-`, while emitting
//! warnings that indicate they are violations.
//!
//! The TOSEC parser API also includes methods to help repair those names to conform with the
//! TOSEC naming convention.
//!
//! ## Usage
//! ```
//! use shiratsu_naming::naming::tosec::TOSECName;
//! use shiratsu_naming::naming::{NameError, TokenizedName};
//!
//! fn parse() -> Result<(), NameError> {
//!     let name = TOSECName::try_parse("Escape from the Mindmaster (1982)(Starpath)(PAL)(Part 3 of 4)[Supercharger Cassette]")?;
//!     assert_eq!(Some("Escape from the Mindmaster"), name.title());
//!     Ok(())
//! }
//!```
//!
//! TOSEC names implement a `into_strict` API that will try to make non-conforming names
//! into TNC conforming names.
//!```
//! use shiratsu_naming::naming::tosec::TOSECName;
//! use shiratsu_naming::naming::{NameError, TokenizedName};
//! fn repair() -> Result<(), NameError>
//! {
//!     let name = TOSECName::try_parse("ZZZ-UNK-Clicks! (test) by Domin, Matthias (2001) (PD)")?;
//!     assert_eq!(Some("Clicks! (test)"), name.title());
//!     let repaired = name.into_strict();
//!     assert_eq!("Clicks! (test) (Domin, Mathias)(2001)(PD)", &repaired.to_string());
//!     Ok(())
//! }
//! ```
mod parsers;
mod tokens;

pub use tokens::*;
