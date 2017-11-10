use std::io::*;
use std::fs::{File};
use std::path::{PathBuf};
use std::num::Wrapping;
fn main() {
    let path_string = "db.sqlite";
    let path = PathBuf::from(path_string);
    let db_file = File::open(path).expect("unable to open db_file");
    let mut reader = BufReader::with_capacity(512, db_file);
    let mut header_buf = [0; 100];
    reader.read_exact(&mut header_buf).expect("Unable to fill header buffer");
    let header_vec = header_buf.to_vec();
    let header = SqlLiteHeader::from(header_vec);
    println!("header: {:?}", header);
}

fn parse_16_bits(big_end: u8, little_end: u8) -> u16 {
    (big_end as u16) << 8 | little_end as u16
}

fn parse_u32(bytes: Vec<u8>) -> u32 {
    (Wrapping(bytes[0] as u32) << 32 | 
    Wrapping(bytes[1] as u32) << 16 | 
    Wrapping(bytes[2] as u32) << 8 | 
    Wrapping(bytes[3] as u32)).0 
    //0 0 0 0  0 0 0 0
    //0 0 1 0  1 1 1 0 
    //0 0 0 0  0 0 0 1 
    //0 1 0 1  1 0 1 0
}

#[derive(Debug)]
struct SqlLiteHeader {
    magic_string: String,
    page_size: u16,
    write_ver: SqlIOVersion,
    read_ver: SqlIOVersion,
    reserved: u8,
    maximum_embedded_payload_fraction: u8,
    minimum_embeded_payload_fraction: u8,
    leaf_payload_fraction: u8,
    file_change_counter: u32,
    number_of_pages: u32,
    first_free_page: u32,
    number_of_free_pages: u32,
    schema_cookie: u32,
    schema_format: SchemaFormat,
    default_page_cache: u32,
    largest_b_root_page: u32,
    text_encoding: TextEncoding,
    user_version: u32,
    incremental_vacuum_mode: bool,
    app_id: u32,
    version_valid_for: u32,
    sqlite_version: u32,
}

impl SqlLiteHeader {
    fn from(mut bytes: Vec<u8>) -> SqlLiteHeader {
        let magic_string_bytes = bytes.drain(0..16).collect();
        println!("{:?}, {:?}", magic_string_bytes, bytes.len());
        let magic_string = String::from_utf8(magic_string_bytes).expect("magic string error");
        let page_size_bytes: Vec<u8> = bytes.drain(0..2).collect();
        let page_size = parse_16_bits(page_size_bytes[0], page_size_bytes[1]);
        let single_bytes: Vec<u8> = bytes.drain(0..6).collect();
        let write_format_b = single_bytes[0];
        let read_format = single_bytes[1];
        let reserved = single_bytes[2];
        let maximum_embedded_payload_fraction = single_bytes[3];
        let minimum_embeded_payload_fraction = single_bytes[4]; 
        let leaf_payload_fraction = single_bytes[5];
        let change_bytes: Vec<u8> = bytes.drain(0..4).collect();
        let file_change_counter = parse_u32(change_bytes);
        let size_b: Vec<u8> = bytes.drain(0..4).collect();
        let number_of_pages = parse_u32(size_b);
        let free_b1: Vec<u8> = bytes.drain(0..4).collect();
        let first_free_page = parse_u32(free_b1);
        let frees_b: Vec<u8> = bytes.drain(0..4).collect();
        let number_of_free_pages = parse_u32(frees_b);
        let schema_b: Vec<u8> = bytes.drain(0..4).collect();
        let schema_cookie = parse_u32(schema_b);
        let schema_f_b: Vec<u8> = bytes.drain(0..4).collect();
        let schema_format_num = parse_u32(schema_f_b);
        let schema_format = SchemaFormat::from(schema_format_num);
        let default_b: Vec<u8> = bytes.drain(0..4).collect();
        let default_page_cache = parse_u32(default_b);
        let big_b: Vec<u8> = bytes.drain(0..4).collect();
        let largest_b_root_page = parse_u32(big_b);
        let encoding_b: Vec<u8> = bytes.drain(0..4).collect();
        let encoding_num = parse_u32(encoding_b);
        let text_encoding = TextEncoding::from(encoding_num);
        let user_b: Vec<u8> = bytes.drain(0..4).collect();
        let user_version = parse_u32(user_b);
        let vacuum_b: Vec<u8> = bytes.drain(0..4).collect();
        let vac = parse_u32(vacuum_b);
        let incremental_vacuum_mode = match vac {
            1 => true,
            _ => false
        };
        let app_b: Vec<u8> = bytes.drain(0..4).collect();
        let app_id = parse_u32(app_b);
        let version_b: Vec<u8> = bytes.drain(0..4).collect();
        let version_valid_for = parse_u32(version_b);
        let _future: Vec<u8> = bytes.drain(0..20).collect();
        let sql_b: Vec<u8> = bytes.drain(0..4).collect();
        println!("{:?}", sql_b);
        let sqlite_version = parse_u32(sql_b);
        SqlLiteHeader {
            magic_string,
            page_size,
            write_ver: SqlIOVersion::from(write_format_b),
            read_ver: SqlIOVersion::from(read_format),
            reserved,
            maximum_embedded_payload_fraction,
            minimum_embeded_payload_fraction,
            leaf_payload_fraction,
            file_change_counter,
            number_of_pages,
            first_free_page,
            number_of_free_pages,
            schema_cookie,
            schema_format,
            default_page_cache,
            largest_b_root_page,
            text_encoding,
            user_version,
            incremental_vacuum_mode,
            app_id,
            version_valid_for,
            sqlite_version,
        }
    }
}
#[derive(Debug)]
enum SqlIOVersion {
    Legacy,
    WritAhead
}

impl SqlIOVersion {
    fn from(num: u8) -> SqlIOVersion {
       match num {
           1 => SqlIOVersion::Legacy,
           2 => SqlIOVersion::WritAhead,
           _ => panic!("Unknown version number")
       }
    }
}
#[derive(Debug)]
enum SchemaFormat {
    Original,
    AddColumn,
    NullColumn,
    Desc
}

impl SchemaFormat {
    fn from(num: u32) -> SchemaFormat {
        match num {
            1 => SchemaFormat::Original,
            2 => SchemaFormat::AddColumn,
            3 => SchemaFormat::NullColumn,
            4 => SchemaFormat::Desc,
            _ => panic!("Unknown Schema Format")
        }
    }
}
#[derive(Debug)]
enum TextEncoding {
    UTF8,
    UTF16le,
    UTF16be
}

impl TextEncoding {
    fn from(num: u32) -> TextEncoding {
        match num {
            1 => TextEncoding::UTF8,
            2 => TextEncoding::UTF16le,
            3 => TextEncoding::UTF16be,
            _ => panic!("Unknown TextEncoding")
        }
    }
}