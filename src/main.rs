mod chunks;
mod image;
mod tests;

use crate::chunks::read_png;
use crate::image::png_to_rgba;
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
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone())?;
    let mut file = BufReader::new(file);
    let png = read_png(&mut file)?;
    let rgba = png_to_rgba(&png);

    let mut out_file = File::create(args[2].clone()).unwrap();
    out_file.write_all(&rgba)?;

    Ok(())
}
