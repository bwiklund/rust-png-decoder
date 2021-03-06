use crc::crc32;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, ErrorKind};

pub type ChunkHashMap = HashMap<[u8; 4], ChunkRaw>;

pub struct Png {
  pub chunks: ChunkHashMap,
}

pub struct ChunkRaw {
  pub ty: [u8; 4],
  pub len: u32,
  pub crc: u32,
  pub data: Vec<u8>,
}

pub fn read_png(file: &mut BufReader<File>) -> std::io::Result<Png> {
  let expect_header: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  let mut header = vec![0; expect_header.len()];
  file.read_exact(&mut header)?;

  for b in 0..header.len() {
    if header[b] != expect_header[b] {
      return Err(Error::new(ErrorKind::Other, "PNG header invalid"));
    }
  }

  let mut chunks: HashMap<[u8; 4], ChunkRaw> = HashMap::new();

  while !file.buffer().is_empty() {
    let chunk = read_chunk(file)?;

    chunks.insert(chunk.ty, chunk);
  }

  Ok(Png { chunks })
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
    Err(Error::new(ErrorKind::Other, "Chunk CRC validation failed"))
  } else {
    Ok(ChunkRaw { ty, len, crc, data })
  }
}

pub fn bytes_to_u32(v: &[u8]) -> u32 {
  let mut bytes = [0u8; 4];
  bytes.copy_from_slice(v);
  u32::from_be_bytes(bytes)
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
    Err(format!(
      "IHDR header expects 13 bytes, found {}",
      bytes.len()
    ))
  } else {
    Ok(ChunkIHDR {
      width: bytes_to_u32(&bytes[0..4]),
      height: bytes_to_u32(&bytes[4..8]),
      depth: bytes[8],
      color: bytes[9],
      compression: bytes[10],
      filter: bytes[11],
      interlace: bytes[12],
    })
  }
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
    Err(format!(
      "sRGB header expects 1 bytes, found {}",
      bytes.len()
    ))
  } else {
    Ok(SRGB {
      rendering_intent: bytes[0],
    })
  }
}
