// header.rs
use std::{num::NonZeroU32, convert::{TryFrom, TryInto}};
use crate::error::Error;

/// The magic string at the start of all Sqlite3 database files is
/// `Sqlite format 3\{0}`, we keep this as a static slice of bytes since it 
/// shouldn't ever change and the file we are reading is already bytes so 
/// converting it to a string is unnecessary
static HEADER_STRING: &[u8] = &[
 //  S   q   l    i    t    e  ` `   f    o    r    m   a    t  ` `  3  \u{0}
    83, 81, 76, 105, 116, 101, 32, 102, 111, 114, 109, 97, 116, 32, 51, 0,
];

#[derive(Debug)]
pub struct PageSize(u32);

#[derive(Debug)]
pub struct DatabaseHeader {
    pub page_size: PageSize,
    pub write_version: FormatVersion,
    pub read_version: FormatVersion,
    pub change_counter: u32,
    pub reserved_bytes: u8,
    /// If populated, the number of pages
    /// in this database
    pub database_size: Option<NonZeroU32>,
}

pub fn parse_header(bytes: &[u8]) -> Result<DatabaseHeader, Error> {
    validate_header_string(&bytes)?;
    let page_size = parse_page_size(&bytes)?;
    let write_version = FormatVersion::from(bytes[18]);
    let read_version = FormatVersion::from(bytes[19]);
    let reserved_bytes = bytes[20];
    validate_fraction(bytes[21], 64, "Maximum payload fraction")?;
    validate_fraction(bytes[21], 32, "Minimum payload fraction")?;
    validate_fraction(bytes[21], 32, "Leaf fraction")?;
    let change_counter =
        crate::try_parse_u32(&bytes[24..28]).map_err(|msg| Error::InvalidChangeCounter(msg))?;
    // since parsing a u32 would indicate a much larger error
    // than this value just being invalid, we will still
    // return early if this fails
    let database_size = crate::try_parse_u32(&bytes[28..32])
        // Passing NonZeroU32::new to map will get
        // us a Result<Option<NonZeroU32>>
        .map(NonZeroU32::new)
        // Ok will convert the outer Result to an
        // Option<Option<NonZeroU32>>
        .ok()
        // Lastly flatten, will automatically reduce this to
        // Option<NonZeroU32> So if we encounter Some(None)
        // we would get None while Some(Some(NonZeroU32))
        // would become Some(NonZeroU32) which is exactly
        // what we want!
        .flatten();
    Ok(DatabaseHeader {
        page_size,
        write_version,
        read_version,
        reserved_bytes,
        change_counter,
        database_size,
    })
}

/// Validate that the bytes provided match the special string
/// at the start of Sqlite3 files
pub fn validate_header_string(bytes: &[u8]) -> Result<(), Error> {
    let buf = &bytes[0..16];
    // if the provided bytes don't match the static HEADER_STRING,
    // we return early
    if buf != HEADER_STRING {
        // since we only head this way on the error case, we convert the provided
        // value into a string. We don't want to error in our error path if it isn't valid utf8
        // so we again use `from_utf8_lossy` and then convert that into a string. 
        return Err(Error::HeaderString(String::from_utf8_lossy(buf).to_string()))
    }
    Ok(())
}

/// Parse the page size bytes the header into a `PageSize`
pub fn parse_page_size(bytes: &[u8]) -> Result<PageSize, Error> {
    // first we try and covert the slice into an array. This returns a `Result`
    // so we can use the `map_err` method on that to convert a possible error here
    // into _our_ error. Doing it this way allows us to use the `?` operator at the 
    // end which will return early if this fails.
    let page_size_bytes: [u8;2] = bytes[16..18].try_into().map_err(|_| {
        Error::InvalidPageSize(format!("expected a 2 byte slice, found: {:?}", bytes))
    })?;
    // Now we can convert the value into a `u16`
    let raw_page_size = u16::from_be_bytes(page_size_bytes);
    // lastly we are going to use the `try_into` method defined below
    // to finish the job
    raw_page_size.try_into()
}

// Another trait implementation, similar to `Display`
// This one though, takes a generic argument that says
// what the input should be. 
impl TryFrom<u16> for PageSize {
    // We also have to add an "associated type" here that will
    // define the error we will return from the one method we 
    // have to define
    type Error = Error;
    // This is the single requirement for conforming to `TryFrom`
    fn try_from(v: u16) -> Result<PageSize, Self::Error> {
        // This looks a little different than what we had before. Instead
        // of having a series of `if`s, we instead use a single `match` statement
        match v {
            // if 1, we have a special case, we can return the `Ok`
            // value with the maximum page size
            1 => Ok(PageSize(65_536u32)),
            // If we find 0 or 2-511, we found and invalid page size
            // we use the `format!` macro to include the provided value in the
            // error message
            0 | 2..=511 => Err(Error::InvalidPageSize(format!(
                "value must be >= 512, found: {}",
                v
            ))),
            // This will catch all values >= 512
            _ => {
                // Since we know it is large enough, we check if it is a power of 2
                if v.is_power_of_two() {
                    // success, we can cast the provided value to a `u32` and be done
                    Ok(PageSize(v as u32))
                } else {
                    // failed, return an error with an additional explanation
                    Err(Error::InvalidPageSize(format!(
                        "value must be a power of 2 found: {}",
                        v
                    )))
                }
            }
        }
    }
}

/// Validate one of the payload/leaf fractions. If byte doesn't match
/// target will create an error with the provided name.
fn validate_fraction(byte: u8, target: u8, name: &str) -> Result<(), Error> {
    if byte != target {
        Err(Error::InvalidFraction(format!(
            "{} must be {}, found: {}",
            name, target, byte
        )))
    } else {
        Ok(())
    }
}

/// A value stored as a Write Format Version or
/// Read Format Version
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatVersion {
    /// Represents the rollback journal mode
    Legacy,
    /// Represents the Write Ahead Log mode
    WriteAheadLog,
    /// Represents any mode not 1 or 2, the value
    /// will be provided
    Unknown(u8),
}

impl From<u8> for FormatVersion {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Legacy,
            2 => Self::WriteAheadLog,
            _ => Self::Unknown(v),
        }
    }
}
