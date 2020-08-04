mod chunks;
mod image;

use crate::chunks::read_png;
use crate::image::png_to_raw;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/*

Questions:
- is `copy_from_slice` the best way to convince rust that i have an array of fixed size for `from_be_bytes`
- read_chunk should probably return a Result instead of panicking, yeah?
- is it idiomatic to end with a return or an implicit return expression
- TODO fix all the unwraps
*/

// https://en.wikipedia.org/wiki/Portable_Network_Graphics
fn main() -> std::io::Result<()> {
    test_case("indexed_opaque")?;
    test_case("truecolor_rgba")?;
    return Ok(());
}

fn test_case(case_name: &str) -> std::io::Result<()> {
    let file = File::open(format!("tests/{}.png", case_name))?;
    let mut file = BufReader::new(file);
    let png = read_png(&mut file)?;
    let rgba = png_to_raw(&png);
    // let mut out_file = File::create("./out.data").unwrap();

    let mut expect_file = File::open(format!("tests/{}.data", case_name)).unwrap();
    let mut expect_bytes = vec![];
    expect_file.read_to_end(&mut expect_bytes)?;

    assert_eq!(rgba, expect_bytes);
    return Ok(());
}
