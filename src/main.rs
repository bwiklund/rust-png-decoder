mod chunks;
mod image;
mod tests;

use crate::chunks::read_png;
use crate::image::png_to_rgba;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone())?;
    let png = read_png(&mut BufReader::new(file))?;
    let rgba = png_to_rgba(&png).unwrap();

    let mut out_file = File::create(args[2].clone())?;
    out_file.write_all(&rgba)?;

    Ok(())
}
