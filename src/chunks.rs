use crc::crc32;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub struct ChunkRaw {
  pub ty: [u8; 4],
  pub len: u32,
  pub crc: u32,
  pub data: Vec<u8>,
}

pub fn read_chunk(file: &mut BufReader<File>) -> ChunkRaw {
  // CHUNKS
  // Chunk length
  let len_vec = read_bytes(file, 4, String::from("Chunk length"));
  let len = bytes_to_u32(&len_vec[0..4]);

  // Chunk type
  let ty_vec = read_bytes(file, 4, String::from("Chunk type"));
  let mut ty = [0u8; 4];
  ty.copy_from_slice(&ty_vec[0..4]);

  // Chunk data
  let data = read_bytes(file, len as u64, String::from("Chunk data"));

  // Chunk CRC
  let crc_vec = read_bytes(file, 4, String::from("Chunk CRC"));
  let crc = bytes_to_u32(&crc_vec[0..4]);

  assert_eq!(
    crc32::checksum_ieee(&[&ty_vec[..], &data[..]].concat()),
    crc,
    "Chunk checksum failed"
  );

  return ChunkRaw {
    ty: ty,
    len: len,
    crc: crc,
    data: data,
  };
}

pub fn read_bytes(buf_reader: &mut BufReader<File>, len: u64, error_msg: String) -> Vec<u8> {
  let mut bytes = vec![];
  buf_reader
    .take(len)
    .read_to_end(&mut bytes)
    .expect(&format!("Error reading {}", error_msg));
  bytes
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

pub fn parse_ihdr_chunk(bytes: &[u8]) -> ChunkIHDR {
  if bytes.len() != 13 {
    panic!("IHDR header expects 13 bytes, found {}", bytes.len());
  }
  return ChunkIHDR {
    width: bytes_to_u32(&bytes[0..4]),
    height: bytes_to_u32(&bytes[4..8]),
    depth: bytes[8],
    color: bytes[9],
    compression: bytes[10],
    filter: bytes[11],
    interlace: bytes[12],
  };
}

#[derive(Debug)]
pub struct SRGB {
  pub rendering_intent: u8,
}

pub fn parse_srgb_chunk(bytes: &[u8]) -> SRGB {
  if bytes.len() != 1 {
    panic!("sRGB header expects 1 bytes, found {}", bytes.len());
  }
  return SRGB {
    rendering_intent: bytes[0],
  };
}
