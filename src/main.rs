use sqlite_parser::header::parse_header;
use std::io::{BufReader, Seek, SeekFrom};

fn main() {
    let file = std::fs::File::open("database.sqlite").unwrap();
    let mut reader = BufReader::new(file);

    loop {
        let new_offset = reader.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(new_offset, 0);
        let header = parse_header(&mut reader).unwrap();
        println!("{:#?}", header);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
