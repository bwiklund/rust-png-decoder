// TODO only store the last line in ram, stream the rest of the image out immediately
pub fn decompress_png_to_raw(
  compressed: &[u8],
  bpp: i32,
  width: u32,
  height: u32,
) -> Result<Vec<u8>, String> {
  let bytes = inflate::inflate_bytes_zlib(compressed)?;
  println!("{} -> {}", compressed.len(), bytes.len());

  let mut out: Vec<u8> = vec![];

  let mut idx = 0; // TODO calculate this from x and y so its not dependent on idx+=1 occuring at the right time
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
