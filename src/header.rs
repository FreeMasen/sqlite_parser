use crate::error::Error;

/// The magic string at the start of all Sqlite3 database files is
/// `Sqlite format 3\{0}`
static HEADER_STRING: &[u8] = &[
    83, 81, 76, 105, 116, 101, 32, 102, 111, 114, 109, 97, 116, 32, 51, 0,
];

pub fn validate_header_string(buf: &[u8]) -> Result<(), Error> {
    if buf != HEADER_STRING {
        Err(Error::HeaderString(String::from_utf8_lossy(buf).to_string()))
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PageSize(pub u32);

impl std::convert::TryFrom<u16> for PageSize {
    type Error = Error;
    fn try_from(v: u16) -> Result<PageSize, Error> {
        match v {
            1 => Ok(PageSize(65_536u32)),
            0 | 2..=511 => Err(Error::InvalidPageSize(format!(
                "value must be >= 512, found: {}",
                v
            ))),
            _ => {
                if v.is_power_of_two() {
                    Ok(PageSize(v as u32))
                } else {
                    Err(Error::InvalidPageSize(format!(
                        "value must be a power of 2 found: {}",
                        v
                    )))
                }
            }
        }
    }
}

pub fn parse_page_size(bytes: &[u8]) -> Result<PageSize, Error> {
    use std::convert::TryInto;
    if bytes.len() != 2 {
        return Err(Error::InvalidPageSize(format!("invalid byte length found: {:?}", bytes)));
    }
    // convert it into a [u8;2] or return early on any issues
    let fixed: [u8; 2] = bytes
        .try_into()
        .map_err(|_| Error::InvalidPageSize(format!("invalid byte length found: {:?}", bytes)))?;
    // now we can create our `u16`
    let raw = u16::from_be_bytes(fixed);
    // Lastly we use our `TryInto` implementation
    raw.try_into()
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn page_size_max() {
        let page_size = parse_page_size(&1u16.to_be_bytes()).unwrap();
        assert_eq!(page_size.0, 65_536u32)
    }

    #[test]
    #[should_panic = "value must be a power of 2 found"]
    fn page_size_not_pow2() {
        let bytes = 4099u16.to_be_bytes();
        parse_page_size(&bytes).unwrap();
    }

    #[test]
    #[should_panic = "value must be >= 512, found"]
    fn page_too_small() {
        let bytes = 500u16.to_be_bytes();
        parse_page_size(&bytes).unwrap();
    }

    #[test]
    #[should_panic = "invalid byte length found"]
    fn page_byte_length_too_small() {
        let bytes = 200u8.to_be_bytes();
        parse_page_size(&bytes).unwrap();
    }

    #[test]
    fn all_valid_page_sizes() {
        for i in 9u32..16 {
            let val = 2u16.pow(i);
            let bytes = val.to_be_bytes();
            let page_size = parse_page_size(&bytes).unwrap();
            assert_eq!(page_size.0, val as u32);
        }
    }
}
