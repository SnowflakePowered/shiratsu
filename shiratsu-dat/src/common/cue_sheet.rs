/// The exact contents of a CUE sheet belonging to a game entry.
///
/// CUE sheets are stored as bytes because source files are not guaranteed to
/// be UTF-8 and their original representation is needed for hash verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CueSheet {
    file_name: String,
    contents: Vec<u8>,
}

impl CueSheet {
    /// Creates a CUE sheet from its canonical filename and original bytes.
    pub fn new<S, B>(file_name: S, contents: B) -> Self
    where
        S: Into<String>,
        B: Into<Vec<u8>>,
    {
        Self {
            file_name: file_name.into(),
            contents: contents.into(),
        }
    }

    /// The canonical filename assigned to the CUE sheet by its source.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// The original, unmodified bytes of the CUE sheet.
    pub fn contents(&self) -> &[u8] {
        &self.contents
    }
}
