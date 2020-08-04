use crc::crc32;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, ErrorKind};

pub struct Png {
  pub chunks: HashMap<String, ChunkRaw>,
}

pub struct ChunkRaw {
  pub ty: [u8; 4],
  pub len: u32,
  pub crc: u32,
  pub data: Vec<u8>,
}

pub fn read_png(file: &mut BufReader<File>) -> std::io::Result<Png> {
  // HEADER
  let expect_header: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  let mut header = vec![0; expect_header.len()];
  file.read_exact(&mut header)?;

  for b in 0..header.len() {
    if header[b] != expect_header[b] {
      panic!("Invalid PNG header");
    }
  }

  let mut chunks: HashMap<String, ChunkRaw> = HashMap::new();

  while !file.buffer().is_empty() {
    let chunk = read_chunk(file)?;
    let ty_str = String::from(std::str::from_utf8(&chunk.ty).unwrap()); // TODO

    chunks.insert(ty_str, chunk);
  }

  // println!(
  //   "{}, {} bytes, crc {}",
  //   std::str::from_utf8(&chunk.ty).unwrap(),
  //   chunk.len,
  //   chunk.crc
  // );

  return Ok(Png { chunks });
}

pub fn read_chunk(file: &mut BufReader<File>) -> std::io::Result<ChunkRaw> {
  // CHUNKS
  // Chunk length
  let mut len = [0; 4];
  file.read_exact(&mut len)?;
  let len = bytes_to_u32(&len);

  // Chunk type
  let mut ty = [0; 4];
  file.read_exact(&mut ty)?;

  // Chunk data
  let mut data = vec![];
  file.take(len as u64).read_to_end(&mut data)?;

  // Chunk CRC
  let mut crc = [0; 4];
  file.read_exact(&mut crc)?;
  let crc = bytes_to_u32(&crc);

  let is_crc_valid = crc32::checksum_ieee(&[&ty[..], &data[..]].concat()) == crc;

  if !is_crc_valid {
    // TODO is this idiomatic?
    return Err(Error::new(ErrorKind::Other, "Chunk CRC validation failed"));
  }

  return Ok(ChunkRaw { ty, len, crc, data });
}

pub fn bytes_to_u32(v: &[u8]) -> u32 {
  let mut bytes = [0u8; 4];
  bytes.copy_from_slice(v);
  return u32::from_be_bytes(bytes);
}

#[derive(Debug)]
pub struct ChunkIHDR {
  pub width: u32,
  pub height: u32,
  pub depth: u8,
  pub color: u8,
  pub compression: u8,
  pub filter: u8,
  pub interlace: u8,
}

pub fn parse_ihdr_chunk(bytes: &[u8]) -> Result<ChunkIHDR, String> {
  if bytes.len() != 13 {
    return Err(format!(
      "IHDR header expects 13 bytes, found {}",
      bytes.len()
    ));
  }
  return Ok(ChunkIHDR {
    width: bytes_to_u32(&bytes[0..4]),
    height: bytes_to_u32(&bytes[4..8]),
    depth: bytes[8],
    color: bytes[9],
    compression: bytes[10],
    filter: bytes[11],
    interlace: bytes[12],
  });
}

#[derive(Debug)]
pub struct IDAT {
  pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct SRGB {
  pub rendering_intent: u8,
}

#[allow(dead_code)]
pub fn parse_srgb_chunk(bytes: &[u8]) -> Result<SRGB, String> {
  if bytes.len() != 1 {
    return Err(format!(
      "sRGB header expects 1 bytes, found {}",
      bytes.len()
    ));
  }
  return Ok(SRGB {
    rendering_intent: bytes[0],
  });
}
