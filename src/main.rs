use sqlite_parser::{
    error::Error,
    header::{parse_magic_string,
    parse_page_size,},
};

fn main() -> Result<(), Error> {
    // first, read in all the bytes of our file
    // using unwrap to just panic if this fails
    let contents = std::fs::read("data.sqlite").unwrap();
    // capture our 16 and 17 bytes in a slice
    parse_magic_string(&contents[0..16])?;
    let page_size = parse_page_size(&contents[16..18])?;
    // print that slice to the screen
    println!("{:?}", page_size);
    Ok(())
}
