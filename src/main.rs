use std::io::*;
use std::fs::{File};
use std::path::{PathBuf};
fn main() {
    let path = PathBuf::from("db.sqlite");
    let file = File::open(path).expect("Unable to open db file");
    let mut reader = BufReader::with_capacity(100, file);
    let buf = reader.fill_buf().expect("Unable to fill buffer");
    let first_16 = buf.get(0..16).expect("Unable to slice off first 16 bytes");
    let magic_string = String::from_utf8(first_16.to_vec()).expect("Unable to convert from utf8 to magic string");
    println!("did it work?\n{:?}", magic_string);
}