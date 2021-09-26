// error.rs

/// Representation of our possible errors.
/// Each variant will contain a string for more
/// detailed description
#[derive(Debug)]
pub enum Error {
    /// An error related to the first 16 bytes in a Sqlite3 file
    HeaderString(String),
    /// An error parsing the PageSize of a Sqlite3
    InvalidPageSize(String),
    /// An error parsing the maximum/ payload fraction
    /// or leaf fraction
    InvalidFraction(String),
    /// The change counter failed to parse
    InvalidChangeCounter(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::HeaderString(v) => write!(
                f,
                "Unexpected bytes at start of file, \
                    expected the magic string 'SQLite format 3\u{0}',\
                    found {:?}",
                v
            ),
            Self::InvalidPageSize(msg) => write!(f, "Invalid page size, {}", msg),
            // For our new case, we are just
            // going to print the inner message
            Self::InvalidFraction(msg) => write!(f, "{}", msg),
            Self::InvalidChangeCounter(msg) => write!(f, "Invalid change counter: {}", msg),
        }
    }
}
