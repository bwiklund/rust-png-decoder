use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// https://en.wikipedia.org/wiki/Portable_Network_Graphics
fn main() {
    let file = File::open("selene.png").unwrap();
    let mut file = BufReader::new(file);

    // HEADER
    let mut header = vec![];
    file.by_ref()
        .take(8)
        .read_to_end(&mut header)
        .expect("PNG header too short");

    let expect_header: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    for b in 0..header.len() {
        if header[b] != expect_header[b] {
            panic!("Invalid PNG header");
        }
    }

    while !file.buffer().is_empty() {
        let chunk = read_chunk(&mut file);
        println!("Chunk:");
        println!("- len: {} bytes", chunk.len);
        println!("- type: {}", std::str::from_utf8(&chunk.ty).unwrap());
        println!("- crc: {}", chunk.crc);
        println!();
    }
}

struct Chunk {
    ty: [u8; 4],
    len: u32,
    crc: u32,
    data: Vec<u8>,
}

fn read_chunk(file: &mut BufReader<File>) -> Chunk {
    // CHUNKS
    // Chunk length
    let len_vec = read_bytes(file, 4, String::from("Chunk length"));
    let mut len_bytes = [0u8; 4]; // is this the best way to convince rustc that this is 4 bytes???
    len_bytes.copy_from_slice(&len_vec[0..4]);
    let len = u32::from_be_bytes(len_bytes);

    // Chunk type
    let ty_vec = read_bytes(file, 4, String::from("Chunk type"));
    let mut ty = [0u8; 4];
    ty.copy_from_slice(&ty_vec[0..4]);

    // Chunk data
    let data = read_bytes(file, len as u64, String::from("Chunk data"));

    // Chunk CRC
    let crc_vec = read_bytes(file, 4, String::from("Chunk CRC"));
    let mut crc_bytes = [0u8; 4];
    crc_bytes.copy_from_slice(&crc_vec[0..4]);
    let crc = u32::from_be_bytes(crc_bytes);

    // TODO actually check CRC

    return Chunk {
        ty: ty,
        len: len,
        crc: crc,
        data: data,
    };
}

fn read_bytes(buf_reader: &mut BufReader<File>, len: u64, error_msg: String) -> Vec<u8> {
    let mut bytes = vec![];
    buf_reader
        .by_ref()
        .take(len)
        .read_to_end(&mut bytes)
        .expect(&format!("Error reading {}", error_msg));
    bytes
}
