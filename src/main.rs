mod chunks;
mod image;

use crate::chunks::read_png;
use crate::image::png_to_raw;
use std::fs::File;
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
    let file = File::open("tests/indexed_opaque.png")?;
    let mut file = BufReader::new(file);
    let png = read_png(&mut file)?;
    let mut out_file = File::create("./out.data").unwrap();
    png_to_raw(&png, &mut out_file);
    return Ok(());
}
