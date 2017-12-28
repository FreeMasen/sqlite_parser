use std::io::*;
use std::fs::{File};
use std::path::{PathBuf};
// use std::Wrapping;

fn main() {
    //create a path buffer for our file
    let path = PathBuf::from("db.sqlite");
    //open said file
    let file = File::open(path).expect("Unable to open db file");
    //create a reader than has an internal buffer of 100 bytes
    //and point it to our file
    let mut reader = BufReader::with_capacity(100, file);
    //fill the reader's buffer with the first 100 bytes
    //of our file
    let buf = reader.fill_buf().expect("Unable to fill buffer");
    //convert the buffer to a Vec<u8> this will make it a little
    //easier to work with
    let mut buf_vec: Vec<u8> = buf.to_vec();
    //slice the first 16 bytes off of the vector and
    //collect them into a new vector
    let first_16 = buf_vec.drain(0..16).collect();
    let magic_string = String::from_utf8(first_16).expect("Unable to convert from utf8 to magic string");
    //slice off the next two bytes, since we removed
    //the magic string bytes from this we don't have
    //keep track of where we want to start any longer
    let next_two: Vec<u8> = buf_vec.drain(0..2).collect();
    //Convert the bytes (big endian style) into a u16
    let page_size = parse_u16(next_two[0], next_two[1]);
    let single_bytes: Vec<u8> = buf_vec.drain(0..6).collect();
    let write_mode = single_bytes[0];
    let read_mode = single_bytes[1];
    let reserved = single_bytes[2];
    let max_fraction = single_bytes[3];
    let min_fraction = single_bytes[4];
    let min_leaf = single_bytes[5];
    //drain the 4 byte properties out of the vec original vec
    let four_bytes: Vec<u8> = buf_vec.drain(0..(4 * 12)).collect();
    //break this into chunks of 4 bytes
    let mut four_iter = four_bytes.chunks(4);
    //call next for each of the 4 byte properties we are looking for
    let chage_counter = parse_u32(four_iter.next().unwrap());
    let size = parse_u32(four_iter.next().unwrap());
    let first_free_truck = parse_u32(four_iter.next().unwrap());
    let free_count = parse_u32(four_iter.next().unwrap());
    let schema_cookie = parse_u32(four_iter.next().unwrap());
    let schema_format = parse_u32(four_iter.next().unwrap());
    let cache_size = parse_u32(four_iter.next().unwrap());
    let largest_b_root = parse_u32(four_iter.next().unwrap());
    let db_text_encoding = parse_u32(four_iter.next().unwrap());
    let user_version = parse_u32(four_iter.next().unwrap());
    let incremental_vac = parse_u32(four_iter.next().unwrap());
    let app_id = parse_u32(four_iter.next().unwrap());
    //Drain out the 20 bytes of reserved space
    let sql_reserved: Vec<u8> = buf_vec.drain(0..20).collect();
    let last_bytes: Vec<u8> = buf_vec.drain(0..8).collect();
    //break the last 8 bytes into two chunks
    let mut last_two = last_bytes.chunks(4);
    let version_valid_for = parse_u32(last_two.next().unwrap());
    let sqllite_version = parse_u32(last_two.next().unwrap());
    println!("Magic String: {:?}", magic_string);
    println!("Page size: {:?}", page_size);
    println!("Write mode: {:?}", write_mode);
    println!("Read mode: {:?}", read_mode);
    println!("Reserved per page: {:?}", reserved);
    println!("Maximum embedded payload fraction: {:?}", max_fraction);
    println!("Minimum embedded payload fraction: {:?}", min_fraction);
    println!("Leaf payload fraction: {:?}",min_leaf);
    println!("Change counter: {:?}",chage_counter);
    println!("Size: {:?}",size);
    println!("First Freelist truck page: {:?}",first_free_truck);
    println!("Freelist Page count: {:?}",free_count);
    println!("Schema cookie: {:?}",schema_cookie);
    println!("Schema Format: {:?}",schema_format);
    println!("Cache Page Size: {:?}",cache_size);
    println!("Largest B-Tree root page: {:?}",largest_b_root);
    println!("Text Encoding: {:?}",db_text_encoding);
    println!("User version: {:?}",user_version);
    println!("Incremental vacuum: {:?}",incremental_vac);
    println!("App ID: {:?}",app_id);
    println!("Reserved for expansion: {:?}",sql_reserved);
    println!("Version Valid for: {:?}",version_valid_for);
    println!("Version {:?}", sqllite_version);
}

pub fn parse_u16(big_end: u8, little_end: u8) -> u16 {
    //shift the big bits 8 places to the left
    (big_end    as u16) << 8 |
    //create a new number from the shifted big end bits and the original
    //little end bits
    little_end as u16
}

fn parse_u32(bytes: &[u8]) -> u32 {
    //shift our largest bytes over 24 bits
    //8+24 = 32 so this would be our left most 8
    (bytes[0] as u32) << 24 | 
    //shift the next value 16
    (bytes[1] as u32) << 16 |
    //shift the next value 8
    (bytes[2] as u32) << 8  |
    (bytes[3] as u32)
    //or them all together
}