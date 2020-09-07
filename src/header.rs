use crate::error::Error;

static MAGIC_STRING: &[u8] = &[
    83, 81, 76, 105, 116, 101, 32, 102, 111, 114, 109, 97, 116, 32, 51, 0,
];

pub fn parse_magic_string(buf: &[u8]) -> Result<(), Error> {
    if buf != MAGIC_STRING {
        Err(Error::MagicString)
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
            0 | 2..=511 => Err(Error::InvalidPageSize(format!("value must be >= 512, found: {}", v))),
            _ => {
                if is_pow_2(v) {
                    Ok(PageSize(v as u32))
                } else {
                    Err(Error::InvalidPageSize(format!("value must be a power of 2 found: {}", v)))
                }
            }
        }
    }
}

pub fn parse_page_size(bytes: &[u8]) -> Result<PageSize, Error> {
    use std::convert::TryInto;
    // ensure that the slice is exactly 2 bytes by splitting on 2
    let (two_bytes, _) = bytes.split_at(std::mem::size_of::<u16>());
    // convert it into a [u8;2] or return early on any issues
    let fixed: [u8; 2] = two_bytes.try_into().map_err(|_| {
        Error::InvalidPageSize(format!("invalid byte length found: {:?}", bytes))
    })?;
    // now we can create our `u16`
    let raw = u16::from_be_bytes(fixed);
    // Lastly we use our `TryInto` implementation
    raw.try_into()
}

/// Check if a value is a power of 2
fn is_pow_2(v: u16) -> bool {
    // First we convert the value to a float
    let flt = f32::from(v);
    // Now we check that `log2` has no remainder
    flt.log2() % 1.0 == 0.0
}