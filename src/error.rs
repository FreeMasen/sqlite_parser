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
    /// An invalid u32 was found
    InvalidU32(String),
    /// An invalid i32 was found
    InvalidI32(String),
    /// Encountered a 0 when NonZero was expected
    UnexpectedZero(String),
    /// Encountered a non-zero when zero was expected
    UnexpectedNonZero(String),
    /// The operating system encountered an error when
    /// attempting to read the sqlite file
    IoError(std::io::Error, &'static str),
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
            Self::InvalidFraction(msg) => write!(f, "{}", msg),
            Self::InvalidU32(msg) => write!(f, "Invalid u32: {}", msg),
            Self::InvalidI32(msg) => write!(f, "Invalid i32: {}", msg),
            Self::UnexpectedZero(what) => write!(f, "Expected non-zero value for {}", what),
            Self::UnexpectedNonZero(what) => write!(f, "Expected zero value for {}", what),
            Self::IoError(inner, what) => write!(f, "Io Error parsing {}: {}", what, inner),
        }
    }
}
