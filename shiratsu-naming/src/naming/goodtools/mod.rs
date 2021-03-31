//! Parsers for the GoodTools naming convention.
//!
//! Defined loosely by [GoodCodes.txt](https://raw.githubusercontent.com/SnowflakePowered/shiratsu/25f2c858dc3a9373e27de3df559cd00931d8e55f/shiratsu-naming/src/naming/goodtools/GoodCodes.txt).
//!
//! Also uses information from [Emulation GameTech Wiki](https://emulation.gametechwiki.com/index.php/GoodTools).
//!
//! Specifically guarantees support for the 2016-04-03 GoodTools release,
//! using DAT files from [OpenGood](https://github.com/SnowflakePowered/opengood).
//!
//! The region parser will parse regions conforming to the 2016-04-03 release of the tools.
//! This means that GoodGen v0.999.7 region codes are not officially supported, but GoodMSX region
//! codes of the form _are_.
//!
//! In practice, this means that numbered tags such as `(1)` and `(4)` are parsed as regions,
//! but `(F)` and `(B)` are always parsed as France and Brazil respectively.
//!
//! GoodGB64 support is also not guaranteed because of the idiosyncrasies in the GB64 naming
//! scheme. This parser should still work to retrieve the title however.
//!
//! ## Usage
//! ```
//! use shiratsu_naming::naming::goodtools::GoodToolsName;
//! use shiratsu_naming::naming::{NameError, TokenizedName};
//!
//! fn parse() -> Result<(), NameError> {
//!     let name = GoodToolsName::try_parse("Fire Emblem - Ankoku Ryuu to Hikari no Tsurugi (J) [hM04][b1]")?;
//!     assert_eq!(Some("Fire Emblem - Ankoku Ryuu to Hikari no Tsurugi"), name.title());
//!     Ok(())
//! }
//! ```
mod parsers;
mod tokens;

pub(crate) use parsers::parse_region as parse_goodtools_region;

pub use tokens::*;