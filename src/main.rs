use sqlite_parser::{parse_header, error::Error};

fn main() -> Result<(), Error> {
    let contents = std::fs::read("data.sqlite").expect("Failed to read data.sqlite");
    let db_header = parse_header(&contents[0..100])?;
    // Using the format placeholder {:#?} gives us the
    // debug print but pretty printed.
    println!("{:#?}", db_header);
    Ok(())
}
