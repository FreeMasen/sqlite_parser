use sqlite_parser::header::parse_header;
use std::io::{BufReader, Read, Cursor, Seek, SeekFrom};

fn main() {
    let file = std::fs::File::open("database.sqlite").unwrap();
    let mut reader = BufReader::new(file);
    // let cursor = Cursor::new(file);
    
    loop {
        let _new_offset = reader.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0u8;100];
        let ct = reader.read(&mut buf).unwrap();
        if ct != 100 {
            panic!("Unable to read the first 100 bytes of database.sqlite");
        }
        let header = parse_header(&buf).unwrap();
        println!("{:#?}", header);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
