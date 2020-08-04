use crate::chunks::ChunkHashMap;
use crate::chunks::Png;
use crate::chunks::{parse_ihdr_chunk, ChunkRaw};

// TODO only store the last line for filters
// TODO take a buffer writer instead, or something
pub fn png_to_rgba(png: &Png) -> Result<Vec<u8>, String> {
  let ihdr = chunk_or_err(&png.chunks, b"IHDR")?;
  let ihdr = parse_ihdr_chunk(&ihdr.data)?;

  let has_alpha = (1 << 2) & ihdr.color > 0;
  let has_color = (1 << 1) & ihdr.color > 0;
  let has_palette = (1 << 0) & ihdr.color > 0;

  if has_palette && !has_color {}

  let mut raw_channels = 0;
  if has_palette {
    if has_color {
      raw_channels += 1;
    } else {
      return Err("Cannot set palette flag without color flag".to_string());
    }
  } else {
    if has_color {
      raw_channels += 3;
    }
    if has_alpha {
      raw_channels += 1;
    }
  }

  let idat = chunk_or_err(&png.chunks, b"IDAT")?;

  // regardless of grayscale / truecolor / indexed, the channels are all encoded the same way
  let channels = idat_to_channels(&idat.data, raw_channels, ihdr.width, ihdr.height)?;

  // now apply the palette if we're in indexed mode
  let rgba;
  if has_palette {
    let plte = chunk_or_err(&png.chunks, b"PLTE")?;
    rgba = apply_palette(&channels, &plte.data)?;
  } else {
    rgba = channels;
  }

  Ok(rgba)
}

fn chunk_or_err<'a>(chunks: &'a ChunkHashMap, name: &[u8; 4]) -> Result<&'a ChunkRaw, String> {
  Ok(
    chunks
      .get(name)
      .ok_or_else(|| format!("{} chunk missing", std::str::from_utf8(name).unwrap()))?,
  )
}

fn idat_to_channels(
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
        return Err(format!("Invalid line filter type: {:x}", filter));
      }
    }
  }

  Ok(out)
}

// pixels that would be offscreen up or left are all treated as zeros, this helps with that
#[inline]
fn lookup(v: &[u8], bpp: i32, width: u32, x: i32, y: i32, component: i32) -> u8 {
  if x < 0 || y < 0 {
    0
  } else {
    v[((x + y * width as i32) * bpp + component) as usize]
  }
}

// Paeth, A, B, or C, whichever is closest to p = A + B − C
fn paeth_predictor(a: i32, b: i32, c: i32) -> i32 {
  let p = a + b - c;
  let pa = i32::abs(p - a);
  let pb = i32::abs(p - b);
  let pc = i32::abs(p - c);

  if pa <= pb && pa <= pc {
    a
  } else if pb <= pc {
    b
  } else {
    c
  }
}

pub fn apply_palette(indexes: &[u8], palette_bytes: &[u8]) -> Result<Vec<u8>, String> {
  if palette_bytes.len() % 3 != 0 {
    return Err("Palette data size not multiple of three".to_string());
  }

  let mut out = vec![];

  for b in indexes {
    let palette_index = (b * 3) as usize;
    if palette_index + 3 > palette_bytes.len() {
      return Err(format!("Invalid palette index: {}", b));
    }
    // palettes are ALWAYS rgb, 1 byte each
    out.push(palette_bytes[palette_index]);
    out.push(palette_bytes[palette_index + 1]);
    out.push(palette_bytes[palette_index + 2]);
    out.push(255); // hang on... these can have alpha, no?
  }

  Ok(out)
}
