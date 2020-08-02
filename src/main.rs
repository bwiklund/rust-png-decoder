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

    // CHUNKS
    // Chunk length
    let mut chunk_len = vec![];
    file.by_ref()
        .take(4)
        .read_to_end(&mut chunk_len)
        .expect("EOF before chunk length");

    println!("Chunk:");
    // is this the best way to convince rustc that this is 4 bytes???
    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&chunk_len[0..4]);
    let len = u32::from_be_bytes(len_bytes);
    println!("- len: {} bytes", len);

    // Chunk type
    let mut chunk_type = vec![];
    file.by_ref()
        .take(4)
        .read_to_end(&mut chunk_type)
        .expect("EOF before chunk type");
    println!("- type: {}", std::str::from_utf8(&chunk_type).unwrap());

    // Chunk data
    let mut chunk_data = vec![];
    file.by_ref()
        .take(len as u64)
        .read_to_end(&mut chunk_data)
        .expect("EOF before chunk data length reached");

    let mut chunk_crc = vec![];
    file.by_ref()
        .take(4)
        .read_to_end(&mut chunk_crc)
        .expect("EOF before chunk CRC");
    let mut crc_bytes = [0u8; 4];
    crc_bytes.copy_from_slice(&chunk_crc[0..4]);
    let crc = u32::from_be_bytes(crc_bytes);

    println!("- crc: {}", crc); // TODO actually check this
}
