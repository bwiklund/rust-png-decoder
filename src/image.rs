// TODO only store the last line in ram, stream the rest of the image out immediately

use crate::chunks::parse_ihdr_chunk;
use crate::chunks::Png;
use std::fs::File;
use std::io::prelude::*;

// TODO take a buffer writer instead, or something
pub fn png_to_raw(png: &Png, out_file: &mut File) {
  let ihdr = png.chunks.get(&String::from("IHDR")).unwrap();
  let ihdr = parse_ihdr_chunk(&ihdr.data).unwrap();

  let has_alpha = 0b0100 & ihdr.color > 0;
  let has_color = 0b0010 & ihdr.color > 0;
  let has_palette = 0b0001 & ihdr.color > 0;

  if has_palette && !has_color {}

  let mut raw_channels = 0;
  if has_palette {
    if has_color {
      raw_channels += 1;
    } else {
      panic!(); // png spec says palette flag can only exist with color
    }
  } else {
    if has_color {
      raw_channels += 3;
    }
    if has_alpha {
      raw_channels += 1;
    }
  }

  let idat = png.chunks.get(&String::from("IDAT")).unwrap();

  // regardless of grayscale / truecolor / indexed, the channels are all encoded the same way
  let channels = idat_to_image(&idat.data, raw_channels, ihdr.width, ihdr.height).unwrap();

  // now apply the palette if we're in indexed mode
  let rgba;
  if has_palette {
    let plte = png.chunks.get(&String::from("PLTE")).unwrap();
    rgba = apply_palette(&channels, &plte.data);
  } else {
    rgba = channels;
  }

  out_file.write(&rgba).unwrap();
}

pub fn idat_to_image(
  compressed: &[u8],
  bpp: i32,
  width: u32,
  height: u32,
) -> Result<Vec<u8>, String> {
  let bytes = inflate::inflate_bytes_zlib(compressed)?;

  let mut out: Vec<u8> = vec![];

  let mut idx = 0;
  for y in 0..height as i32 {
    let filter = bytes[idx];
    idx += 1;

    match filter {
      0 => {
        // None
        for _x in 0..width {
          for _i in 0..bpp {
            out.push(bytes[idx]);
            idx += 1;
          }
        }
      }
      1 => {
        // Sub (left)
        for x in 0..width as i32 {
          for i in 0..bpp {
            let pred = lookup(&out, bpp, width, x - 1, y, i);
            let val = (pred as i32 + bytes[idx] as i32) % 256;
            out.push(val as u8);
            idx += 1;
          }
        }
      }
      2 => {
        // Up
        for x in 0..width as i32 {
          for i in 0..bpp {
            let pred = lookup(&out, bpp, width, x, y - 1, i);
            let val = (pred as i32 + bytes[idx] as i32) % 256;
            out.push(val as u8);
            idx += 1;
          }
        }
      }
      3 => {
        // Average (of left and top)
        for x in 0..width as i32 {
          for i in 0..bpp {
            let pred = (lookup(&out, bpp, width, x - 1, y, i) as i32
              + lookup(&out, bpp, width, x, y - 1, i) as i32)
              / 2;
            let val = (pred + bytes[idx] as i32) % 256;
            out.push(val as u8);
            idx += 1;
          }
        }
      }
      4 => {
        for x in 0..width as i32 {
          for i in 0..bpp {
            let pred = paeth_predictor(
              lookup(&out, bpp, width, x - 1, y, i) as i32,
              lookup(&out, bpp, width, x, y - 1, i) as i32,
              lookup(&out, bpp, width, x - 1, y - 1, i) as i32,
            );
            let val = (pred + bytes[idx] as i32) % 256;
            out.push(val as u8);
            idx += 1;
          }
        }
      }
      _ => {
        panic!(format!("Invalid line filter type: {:x}", filter));
      }
    }
  }

  return Ok(out);
}

// pixels that would be offscreen up or left are all treated as zeros, this helps with that
fn lookup(v: &[u8], bpp: i32, width: u32, x: i32, y: i32, component: i32) -> u8 {
  if x < 0 || y < 0 {
    return 0;
  } else {
    return v[((x + y * width as i32) * bpp + component) as usize];
  }
}

// Paeth, A, B, or C, whichever is closest to p = A + B − C
fn paeth_predictor(a: i32, b: i32, c: i32) -> i32 {
  let p = a + b - c;
  let pa = i32::abs(p - a);
  let pb = i32::abs(p - b);
  let pc = i32::abs(p - c);

  if pa <= pb && pa <= pc {
    return a;
  } else if pb <= pc {
    return b;
  } else {
    return c;
  }
}

pub fn apply_palette(indexes: &[u8], palette_bytes: &[u8]) -> Vec<u8> {
  if palette_bytes.len() % 3 != 0 {
    panic!();
  }

  let mut out = vec![];

  for i in 0..indexes.len() {
    let b = indexes[i];
    let palette_index = (b * 3) as usize;
    // TODO bounds check indices
    // palettes are ALWAYS rgb, 1 byte each
    out.push(palette_bytes[palette_index + 0]);
    out.push(palette_bytes[palette_index + 1]);
    out.push(palette_bytes[palette_index + 2]);
    out.push(255); // hang on... these can have alpha, no?
  }

  return out;
}
