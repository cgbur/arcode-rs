//! This crate provides the an efficient implementation of
//! an [arithmetic encoder/decoder](https://en.wikipedia.org/wiki/Arithmetic_coding). This crate is based off the paper
//! that describes arithmetic coding found [here](https://web.stanford.edu/class/ee398a/handouts/papers/WittenACM87ArithmCoding.pdf).
//! This implementation features many readability and performance improvements, especially on the decoding side.
//!
//! The goal of this project is not to provide an out-of-the-box compression solution.
//! Arithmetic coding ([entropy encoding](https://en.wikipedia.org/wiki/Entropy_encoding)) is the backbone of almost every
//! modern day compression scheme. This crate is meant to be included in future projects that rely on an efficient entropy
//! coder e.g. [PPM](https://en.wikipedia.org/wiki/Prediction_by_partial_matching), [LZ77/LZ78](https://en.wikipedia.org/wiki/LZ77_and_LZ78),
//! [h265/HEVC](https://en.wikipedia.org/wiki/High_Efficiency_Video_Coding).
//!
//! # Core components
//! There are a lot of structs available for use but for the average user there are only a few that will be used.
//! - [SourceModel](util/source_model/struct.SourceModel.html) models of the probability of symbols. Counts can be adjusted
//! as encoding is done to improve compression.
//! - [Encoder](encode/encoder/struct.ArithmeticEncoder.html) encodes symbols given a source model and a symbol.
//! - [Decoder](decode/decoder/struct.ArithmeticDecoder.html) decodes symbols given a source model and a bitstream.
//!
//! # Examples
//!
//! ## Input and output bitstreams
//! In order for arithmetic coding to work streams need to be read a bit at a time (for decoding and for the encoders output).
//! Because of this, [BitBit](https://docs.rs/bitbit) is required. Wrapping whatever your input is in a buffered reader/writer
//! should greatly improve performance.
//!
//! Using bitbit to create an input stream from a file that will be passed to encoder/decoder.
//! ```rust
//! use std::fs::File;
//! use std::io::{BufReader, Cursor};
//! use bitbit::{BitReader, MSB};
//! let r = Cursor::new(vec!['a' as u8, 'b' as u8, 'c' as u8]);
//! // let input_file = File::open("some file").unwrap();
//! let mut buffer_input = BufReader::new(r);
//! let mut input: BitReader<&mut BufReader<_>, MSB> = BitReader::new(&mut buffer_input);
//! ```
//! Using bitbit to create an output stream.
//! ```rust
//! use std::fs::File;
//! use std::io::{BufWriter, Write, Cursor};
//! use bitbit::BitWriter;
//! let r = Cursor::new(vec!['a' as u8, 'b' as u8, 'c' as u8]);
//! //let mut output_file = File::create("./compressed.any")?;
//! let mut buffered_output = BufWriter::new(r);
//! let mut out_writer = BitWriter::new(&mut buffered_output);
//! //once you are done encoding/decoding...
//! out_writer.pad_to_byte();
//! buffered_output.flush();
//! ```
//!
//!


pub mod util;
pub mod encode;
pub mod decode;

pub extern crate bitbit;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn another() {}
}