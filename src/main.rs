use std::io::*;
use std::fs::{File};
use std::path::{PathBuf};
use std::Wrapping;

fn main() {
    let path = PathBuf::from("db.sqlite");
    let file = File::open(path).expect("Unable to open db file");
    let mut reader = BufReader::with_capacity(100, file);
    let buf = reader.fill_buf().expect("Unable to fill buffer");
    let first_16 = buf.get(0..16).expect("Unable to slice off first 16 bytes");
    let magic_string = String::from_utf8(first_16.to_vec()).expect("Unable to convert from utf8 to magic string");
    let next_two = buf.get(16..18).expect("Unable to slice of next two");
    let page_size = parse_u16(next_two[0], next_two[1]);

}

pub fn parse_u16(big_end: u8, little_end: u8) -> u16 {
    (
    Wrapping(big_end    as u16) << 8 |
    Wrapping(little_end as u16)
    ).0
}