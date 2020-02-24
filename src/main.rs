

fn main() {
    // first, read in all the bytes of our file
    // using unwrap to just panic if this fails
    let contents = std::fs::read("data.sqlite").unwrap();
    // capture our 16 and 17 bytes in a slice
    let page_size = &contents[16..18];
    // print that slice to the screen
    println!("{:?}", page_size);
}
