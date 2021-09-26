// lib.rs
pub mod error;
mod header;

pub use header::parse_header;

// A little strange but since this might end up being
// used in a large number of places, we can use a
// String in the error position of our result. This
// will allow the caller to insert their own error
// with the more context.
fn try_parse_u32(bytes: &[u8]) -> Result<u32, String> {
    use std::convert::TryInto;
    // Just like with our u16, we are going to need to convert
    // a slice into an array of 4 bytes. Using the `try_into`
    // method on a slice, we will fail if the slice isn't exactly
    // 4 bytes. We can use `map_err` to build our string only if
    // it fails
    let arr: [u8; 4] = bytes.try_into().map_err(|_| {
        format!(
            "expected a 4 byte slice, found a {} byte slice",
            bytes.len()
        )
    })?;
    // Finally we use the `from_be_bytes` constructor for a u32
    Ok(u32::from_be_bytes(arr))
}
