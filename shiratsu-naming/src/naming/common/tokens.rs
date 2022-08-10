use crate::naming::{NameError, NamingConvention};
use std::fmt::Display;
use std::slice::Iter;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// The type of generic flag that appears in a catalogued file name.
///
/// This type is mainly used for generic, non-structured, or non-defined flags.
///
/// Known or defined flags in a naming convention have their type defined implicitly,
/// and may change depending on the position the token in the name.
pub enum FlagType {
    /// The flag is parenthesized.
    Parenthesized,

    /// The flag is bracketed with square brackets.
    Bracketed,
}

/// A name that is tokenized according to some naming convention.
pub trait TokenizedName<'a, T>
where
    Self: Display + From<Vec<T>> + PartialEq + Eq,
{
    /// Get the title of the tokenized name.
    fn title(&self) -> Option<&'a str>;

    /// Returns an iterator over the tokens of this name.
    fn iter(&self) -> Iter<'_, T>;

    /// Tries to parse the input string using the naming convention of this
    /// tokenized name.
    ///
    /// Returns `NameError::ParseError` if parsing fails.
    fn try_parse<S: AsRef<str> + ?Sized>(input: &'a S) -> Result<Self, NameError>;

    /// The naming convention of this tokenized name.
    fn naming_convention() -> NamingConvention;
}
