// header.rs
use crate::error::Error;
use std::{
    convert::{TryFrom, TryInto},
    num::NonZeroU32,
};

/// The magic string at the start of all Sqlite3 database files is
/// `Sqlite format 3\{0}`, we keep this as a static slice of bytes since it
/// shouldn't ever change and the file we are reading is already bytes so
/// converting it to a string is unnecessary
static HEADER_STRING: &[u8] = &[
    /*
    S   q   l    i    t    e  ` `   f    o    r    m   a    t  ` `  3  \u{0}
    */
    83, 81, 76, 105, 116, 101, 32, 102, 111, 114, 109, 97, 116, 32, 51, 0,
];

#[derive(Debug)]
pub struct PageSize(u32);

#[derive(Debug)]
pub struct DatabaseHeader {
    pub page_size: PageSize,
    pub write_version: FormatVersion,
    pub read_version: FormatVersion,
    pub reserved_bytes: u8,
    pub change_counter: u32,
    pub database_size: Option<NonZeroU32>,
    pub free_page_list_info: Option<FreePageListInfo>,
    pub schema_cookie: u32,
    pub schema_version: SchemaVersion,
    pub cache_size: u32,
    pub vacuum_setting: Option<VacuumSetting>,
    pub text_encoding: TextEncoding,
    pub user_version: i32,
    pub application_id: u32,
    pub version_valid_for: u32,
    pub library_write_version: u32,
}

pub fn parse_header(bytes: &[u8]) -> Result<DatabaseHeader, Error> {
    validate_header_string(&bytes[0..16])?;
    let page_size = parse_page_size(&bytes[16..18])?;
    let write_version = FormatVersion::from(bytes[18]);
    let read_version = FormatVersion::from(bytes[19]);
    let reserved_bytes = bytes[20];
    validate_fraction(bytes[21], 64, "Maximum payload fraction")?;
    validate_fraction(bytes[22], 32, "Minimum payload fraction")?;
    validate_fraction(bytes[23], 32, "Leaf fraction")?;
    let change_counter = crate::try_parse_u32(&bytes[24..28], "change counter")?;
    let database_size = crate::try_parse_u32(&bytes[28..32], "")
        .map(NonZeroU32::new)
        .ok()
        .flatten();
    let first_free_page = crate::try_parse_u32(&bytes[32..36], "first free page")?;
    let free_page_len = crate::try_parse_u32(&bytes[36..40], "free page list length")?;
    let free_page_list_info = FreePageListInfo::new(first_free_page, free_page_len);
    let schema_cookie = crate::try_parse_u32(&bytes[40..44], "schema cookie")?;
    let raw_schema_version = crate::try_parse_u32(&bytes[44..48], "schema format version")?;
    let schema_version = SchemaVersion::try_from(raw_schema_version)?;
    let cache_size = crate::try_parse_u32(&bytes[48..52], "cache size")?;
    let raw_vacuum = crate::try_parse_u32(&bytes[52..56], "auto vacuum")?;
    let vacuum_setting = VacuumSetting::full(raw_vacuum);
    let raw_text_enc = crate::try_parse_u32(&bytes[56..60], "text encoding")?;
    let text_encoding = TextEncoding::try_from(raw_text_enc)?;
    let user_version = crate::try_parse_i32(&bytes[60..64], "user version")?;
    let application_id = crate::try_parse_u32(&bytes[64..68], "application id")?;
    validate_reserved_zeros(&bytes[68..92])
        .map_err(|e| eprintln!("{}", e))
        .ok();
    // new!
    let version_valid_for = crate::try_parse_u32(&bytes[92..96], "version valid for")?;
    let library_write_version = crate::try_parse_u32(&bytes[96..100], "library write version")?;
    Ok(DatabaseHeader {
        page_size,
        write_version,
        read_version,
        change_counter,
        reserved_bytes,
        database_size,
        free_page_list_info,
        schema_cookie,
        schema_version,
        cache_size,
        vacuum_setting,
        text_encoding,
        user_version,
        application_id,
        version_valid_for,
        library_write_version,
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

        return Err(Error::HeaderString(
            String::from_utf8_lossy(bytes).to_string(),
        ));
    }
    Ok(())
}

/// Parse the page size bytes the header into a `PageSize`
pub fn parse_page_size(bytes: &[u8]) -> Result<PageSize, Error> {
    // first we try and covert the slice into an array. This returns a `Result`
    // so we can use the `map_err` method on that to convert a possible error here
    // into _our_ error. Doing it this way allows us to use the `?` operator at the
    // end which will return early if this fails.
    let page_size_bytes: [u8; 2] = bytes.try_into().map_err(|_| {
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

fn validate_reserved_zeros(bytes: &[u8]) -> Result<(), Error> {
    for (i, &byte) in bytes.iter().enumerate() {
        if byte != 0 {
            return Err(Error::UnexpectedNonZero(format!(
                "Reserved space byte: {}",
                i
            )));
        }
    }
    Ok(())
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

/// The in header representation
/// of the Free Page List
#[derive(Debug)]
pub struct FreePageListInfo {
    /// The page number of the first
    /// free page
    pub start_page: NonZeroU32,
    /// The total count of free pages
    pub length: u32,
}

impl FreePageListInfo {
    // Remember a 0 would mean there are no free
    // pages so we can setup our constructor to
    // return None if the start_page is 0
    fn new(start_page: u32, length: u32) -> Option<Self> {
        // This will return None early if passed 0
        let start_page = NonZeroU32::new(start_page)?;
        Some(Self { start_page, length })
    }
}
#[derive(Debug)]
pub enum SchemaVersion {
    /// Baseline usable by all sqlite versions
    One,
    /// Usable from sqlite version 3.1.3 and above
    Two,
    /// Usable from sqlite version 3.1.4 and above
    Three,
    /// Usable from sqlite version 3.3.0 and above
    Four,
    /// Version > 4
    Unknown(NonZeroU32),
}

impl TryFrom<u32> for SchemaVersion {
    // Set the associated type to our error enum
    type Error = Error;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        Ok(match v {
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => {
                let value = NonZeroU32::new(v)
                    // ok_or_else will convert our Option to a Result
                    .ok_or_else(|| Error::UnexpectedZero("Schema Version".to_string()))?;
                Self::Unknown(value)
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VacuumSetting {
    /// Incremental vacuum is set to full
    Full(NonZeroU32),
}

impl VacuumSetting {
    /// A constructor that returns an optional
    /// VacuumSetting
    pub fn full(v: u32) -> Option<Self> {
        let non_zero = NonZeroU32::new(v)?;
        Some(VacuumSetting::Full(non_zero))
    }
}

#[derive(Debug)]
pub enum TextEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    Unknown(u32),
}

impl TryFrom<u32> for TextEncoding {
    type Error = Error;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self::Utf8),
            2 => Ok(Self::Utf16Le),
            3 => Ok(Self::Utf16Be),
            _ => Ok(Self::Unknown(v)),
        }
    }
}
