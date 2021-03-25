#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// The type of flag.
pub enum FlagType
{
    /// The flag is parenthesized.
    Parenthesized,

    /// The flag is bracketed with square brackets.
    Bracketed,
}
