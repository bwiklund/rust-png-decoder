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
    let mut file = BufReader::new(file);
    let png = read_png(&mut file)?;
    let rgba = png_to_rgba(&png);

    let mut out_file = File::create(args[2].clone()).unwrap();
    out_file.write_all(&rgba)?;

    Ok(())
}
