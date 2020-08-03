mod chunks;
mod image;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::chunks::{parse_ihdr_chunk, parse_srgb_chunk, read_chunk};
use crate::image::decompress_png_to_raw;

/*

Questions:
1. is `copy_from_slice` the best way to convince rust that i have an array of fixed size for `from_be_bytes`
2. read_chunk should probably return a Result instead of panicking, yeah?
3. is this `std::str::from_utf8(&chunk.ty).unwrap() == "IHDR"` too verbose? how can it be shorter
*/

// https://en.wikipedia.org/wiki/Portable_Network_Graphics
fn main() -> std::io::Result<()> {
    let file = File::open("selene_truecolor_alpha.png")?;
    let mut file = BufReader::new(file);

    // HEADER
    let expect_header: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    let mut header = vec![0; expect_header.len()];
    file.read_exact(&mut header)?;

    for b in 0..header.len() {
        if header[b] != expect_header[b] {
            panic!("Invalid PNG header");
        }
    }

    while !file.buffer().is_empty() {
        let chunk = read_chunk(&mut file)?;
        println!(
            "{}, {} bytes, crc {}",
            std::str::from_utf8(&chunk.ty).unwrap(),
            chunk.len,
            chunk.crc
        );

        let ty_str = std::str::from_utf8(&chunk.ty).unwrap();

        // TODO are these case sensitive in the spec?
        match ty_str {
            "IHDR" => {
                println!("{:#?}", parse_ihdr_chunk(&chunk.data));
            }
            "sRGB" => {
                println!("{:#?}", parse_srgb_chunk(&chunk.data));
            }
            "IDAT" => {
                println!("IDAT chunk (bytes omitted)");
                let rgba = decompress_png_to_raw(&chunk.data, 4, 52, 52); // FIXME actually use width from IHDR chunk
                let mut out_file = File::create("./out.data").unwrap();
                out_file.write(&rgba).unwrap();
            }
            "IEND" => {
                println!("IEND chunk");
            }
            _ => println!("unknown chunk type: {}", ty_str),
        }

        println!();
    }

    return Ok(());
}
