//! Parsers for various ROM naming conventions.
//!
//! ## Zero-copy guarantee
//! All parsers are zero-copy but may allocate. Tokens returned are slices of
//! the input string.
//!
//! ## Order significance
//! The order of returned tokens in a name is significant in order of appearance
//! in the input file name, and tokens are not guaranteed to have consistent semantics
//! outside of a parsed name.
//!
//! ## Runtime analysis
//! Because of ambiguities in conventions of each parser type, lookaheads are unavoidable
//! and thus each parser has a quadratic worse-case runtime, in the length of the string.
//! This is acceptable for small inputs such as the file names of catalogued ROM files.
//!
//! The zero-copy guarantee should ensure that parsing is sufficiently fast for large numbers
//! of inputs.
// Re-export common at top level

pub use common::NamingConvention;
pub use common::tokens::TokenizedName;
pub use common::tokens::FlagType;
pub use common::error::NameError;

pub(crate) use common::parsers;

#[macro_use]
mod common;

pub mod nointro;
pub mod tosec;
pub mod goodtools;


