use crc::crc32;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/*

Questions:
1. is `copy_from_slice` the best way to convince rust that i have an array of fixed size for `from_be_bytes`
2. read_chunk should probably return a Result instead of panicking, yeah?
3. is this `std::str::from_utf8(&chunk.ty).unwrap() == "IHDR"` too verbose? how can it be shorter
*/

// https://en.wikipedia.org/wiki/Portable_Network_Graphics
fn main() {
    let file = File::open("selene.png").unwrap();
    let mut file = BufReader::new(file);

    // HEADER
    let expect_header: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    let header = read_bytes(
        &mut file,
        expect_header.len() as u64,
        String::from("PNG header"),
    );

    for b in 0..header.len() {
        if header[b] != expect_header[b] {
            panic!("Invalid PNG header");
        }
    }

    while !file.buffer().is_empty() {
        let chunk = read_chunk(&mut file);
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
                let rgba = decompress_png_to_raw(&chunk.data, 52); // FIXME actually use width from IHDR chunk
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
}

struct ChunkRaw {
    ty: [u8; 4],
    len: u32,
    crc: u32,
    data: Vec<u8>,
}

fn read_chunk(file: &mut BufReader<File>) -> ChunkRaw {
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

fn read_bytes(buf_reader: &mut BufReader<File>, len: u64, error_msg: String) -> Vec<u8> {
    let mut bytes = vec![];
    buf_reader
        .take(len)
        .read_to_end(&mut bytes)
        .expect(&format!("Error reading {}", error_msg));
    bytes
}

fn bytes_to_u32(v: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(v);
    return u32::from_be_bytes(bytes);
}

#[derive(Debug)]
struct ChunkIHDR {
    width: u32,
    height: u32,
    depth: u8,
    color: u8,
    compression: u8,
    filter: u8,
    interlace: u8,
}

fn parse_ihdr_chunk(bytes: &[u8]) -> ChunkIHDR {
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
struct SRGB {
    rendering_intent: u8,
}

fn parse_srgb_chunk(bytes: &[u8]) -> SRGB {
    if bytes.len() != 1 {
        panic!("sRGB header expects 1 bytes, found {}", bytes.len());
    }
    return SRGB {
        rendering_intent: bytes[0],
    };
}

fn decompress_png_to_raw(compressed: &[u8], width: u32) -> Vec<u8> {
    let bytes = inflate::inflate_bytes_zlib(compressed).unwrap();
    println!("{} -> {}", compressed.len(), bytes.len());

    let mut rgba = vec![];

    let mut idx = 0;
    loop {
        let line_filter = bytes[idx];
        idx += 1;

        for _i in 0..width {
            let r = bytes[idx + 0];
            let g = bytes[idx + 1];
            let b = bytes[idx + 2];
            let a = bytes[idx + 3];
            idx += 4;

            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(a);
        }

        if idx >= bytes.len() {
            break;
        }
    }

    return rgba;
}
