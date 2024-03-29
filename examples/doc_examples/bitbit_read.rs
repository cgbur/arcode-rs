#![allow(dead_code, unused_variables, unused_mut)]
use std::io::Cursor;

use arcode::bitbit::{BitReader, BitWriter, MSB};

fn read_example() {
    // normally you would have a Read type with a BufReader
    let mut source = Cursor::new(vec![0u8; 4]);
    let input: BitReader<_, MSB> = BitReader::new(&mut source);
}

fn out_example() {
    // once again would be Write type with a BufWriter
    let compressed = Cursor::new(vec![]);
    let mut compressed_writer = BitWriter::new(compressed);
}
