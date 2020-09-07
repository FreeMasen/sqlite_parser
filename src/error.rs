

#[derive(Debug)]
pub enum Error {
    /// An error with the magic string
    /// at index 0 of all SQLite 3 files
    MagicString,
    InvalidPageSize(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MagicString => write!(f, 
                "Unexpected bytes at start of file, expected the magic string 'SQLite format 3\u{0}'"),
            Self::InvalidPageSize(msg) => write!(f, "Invalid page size, {}", msg),
        }
    }
}