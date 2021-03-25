use shiratsu_parse::error::ParseError;

pub enum ShiratsuError {
    StoneError(shiratsu_stone::StoneError),
    ParseError(ParseError),
    IOError(std::io::Error)
}

impl std::error::Error for ShiratsuError {}

impl std::fmt::Debug for ShiratsuError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiratsuError::StoneError(err) => write!(f, "{}", err),
            ShiratsuError::ParseError(err) => write!(f, "{}", err),
            ShiratsuError::IOError(err) => write!(f, "{}", err),
        }
    }
}

impl std::fmt::Display for ShiratsuError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseError> for ShiratsuError {
    fn from(err: ParseError) -> Self {
        ShiratsuError::ParseError(err)
    }
}

impl From<shiratsu_stone::StoneError> for ShiratsuError {
    fn from(err: shiratsu_stone::StoneError) -> Self {
        ShiratsuError::StoneError(err)
    }
}

impl From<std::io::Error> for ShiratsuError {
    fn from(err: std::io::Error) -> Self {
        ShiratsuError::IOError(err)
    }
}
