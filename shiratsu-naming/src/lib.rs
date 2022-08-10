#![allow(clippy::type_complexity)]
//! A library for parsing ROM entry file names from the most popular
//! naming conventions used in ROM catalogue .dat files.
//!
//! ## Supported Naming Conventions
//! `shiratsu_naming` supports the following naming conventions
//!
//! * No-Intro
//! * TOSEC
//! * GoodTools
//!
//! For more information, see the documentation for the [`naming`](naming/index.html) module.

/// Parsers and validators for region strings from various naming conventions.
pub mod region;

pub mod naming;
