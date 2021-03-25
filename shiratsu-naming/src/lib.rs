//! A library for parsing ROM entry file names from the most popular
//! naming conventions used in ROM catalogue .dat files.

/// Parsers and validators for region strings from various naming conventions.
pub mod region;
/// Parsers for various ROM naming conventions.
pub mod naming;

/// Error types and handling.
mod error;
pub use error::*;