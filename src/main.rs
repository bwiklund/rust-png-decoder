use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// https://en.wikipedia.org/wiki/Portable_Network_Graphics
fn main() {
    let file = File::open("selene.png").unwrap();
    let file = BufReader::new(file);

    let mut buf = vec![];
    file.take(8)
        .read_to_end(&mut buf)
        .expect("PNG header too short");

    let expect_header: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    for b in 0..buf.len() {
        if buf[b] != expect_header[b] {
            panic!("Invalid PNG header");
        }
    }

    println!("so far so good");
}
