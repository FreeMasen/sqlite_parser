// lib.rs
pub mod error;
pub mod header;

use std::io::{Error as IoError, Read};

pub use error::Error;
pub use header::parse_header;

fn read_u32(reader: &mut impl Read, name: &'static str) -> Result<u32, Error> {
    read(reader).map_err(|e| Error::IoError(e, name))
}
fn read_i32(reader: &mut impl Read, name: &'static str) -> Result<i32, Error> {
    read(reader).map_err(|e| Error::IoError(e, name))
}

fn read_u16(reader: &mut impl Read, name: &'static str) -> Result<u16, Error> {
    read(reader).map_err(|e| Error::IoError(e, name))
}

fn read_u8(reader: &mut impl Read, name: &'static str) -> Result<u8, Error> {
    read(reader).map_err(|e| Error::IoError(e, name))
}

/// Read the number of bytes needed to construct T with the `FromBigEndian`
/// implementation for T
fn read<T, const N: usize>(reader: &mut impl Read) -> Result<T, IoError>
where
    T: FromBigEndian<N>,
{
    let bytes = read_bytes(reader)?;
    Ok(T::from_be_bytes(bytes))
}

/// Read `N` bytes from the provided reader into a new array
fn read_bytes<const N: usize>(reader: &mut impl Read) -> Result<[u8; N], IoError> {
    let mut ret = [0u8; N];
    reader.read_exact(&mut ret)?;
    Ok(ret)
}

/// A trait to unify the behavior or the primitive number types
/// which all provide a constructor named `from_be_bytes` which
/// that an array of `u8`s of the appropriate size.
///
/// This trait leverages the const generic N to define the size of the
/// array needed to construct that type
pub trait FromBigEndian<const N: usize>: Sized {
    fn from_be_bytes(bytes: [u8; N]) -> Self;
}

macro_rules! impl_from_big_endian {
    ($t:ty, $n:expr) => {
        impl FromBigEndian<$n> for $t {
            fn from_be_bytes(bytes: [u8; $n]) -> Self {
                <$t>::from_be_bytes(bytes)
            }
        }
    };
    ($t:ty) => {
        impl_from_big_endian!($t, { std::mem::size_of::<$t>() });
    };
}

impl_from_big_endian!(u32);
impl_from_big_endian!(i32);
impl_from_big_endian!(u16);
impl_from_big_endian!(i16);
impl_from_big_endian!(u8);
