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
//! In the git repository there is an [example.rs](https://github.com/Dakati/arithmetic-rs/blob/master/example/example.rs)
//! file that is a complete
//! encode and decode with some benchmarks. It is hard to construct examples that
//! run in the markdown because I don't have access to actual files.
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
//! //using a cursor because the example cant compile without an actual file
//! let r = Cursor::new(vec!['a' as u8, 'b' as u8, 'c' as u8]);
//! // let input_file = File::open("some file").unwrap();
//! let mut buffer_input = BufReader::new(r);
//! let mut input: BitReader<_, MSB> = BitReader::new(&mut buffer_input);
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
//! ## Source Model(s)
//! Depending on your application you could have one or hundreds/thousands of source models.
//! The source model is heavily relied on by the encoder and the decoder. If the decoder ever becomes
//! out of phase with the encoder you will be decoding nonsense.
//!
//! ```rust
//! use arcode::util::source_model::SourceModel;
//! // create a new model that has symbols 0-256
//! // 8 bit values + one EOF marker
//! let mut model_with_eof = SourceModel::new_eof(257, 256);
//! // model for 8 bit 0 - 255, if we arent using
//! // the EOF flag set it to anything outside the range.
//! let model_without_eof = SourceModel::new_eof(256, 9999);
//!
//! // update the probability of symbol 4.
//! model_with_eof.update_symbol(4);
//!```
//! ## Encode
//! Encoding some simple input
//! ```rust
//! use arcode::encode::encoder::ArithmeticEncoder;
//! use arcode::util::source_model::SourceModel;
//! use std::io::Cursor;
//! use bitbit::BitWriter;
//!
//! let mut encoder = ArithmeticEncoder::new(30);
//! let mut source_model = SourceModel::new_eof(10, 9);
//! let mut output = Cursor::new(vec![]);
//! let mut out_writer = BitWriter::new(&mut output);
//! let to_encode: [u32; 5] = [7, 2, 2, 2, 7];
//!
//! for x in to_encode.iter() {
//!     encoder.encode(*x, &mut source_model, &mut out_writer).unwrap();
//!     source_model.update_symbol(*x);
//! }
//!
//! encoder.encode(source_model.get_eof(), &source_model, &mut out_writer).unwrap();
//! encoder.finish_encode(&mut out_writer).unwrap();
//! out_writer.pad_to_byte().unwrap();
//!
//! assert_eq!(output.get_ref(), &[184, 96, 208]);
//! ```
//! ## Decode
//! ```rust
//! use std::io::Cursor;
//! use arcode::util::source_model::SourceModel;
//! use bitbit::{BitReader, MSB};
//! use arcode::decode::decoder::ArithmeticDecoder;
//!
//! let input = Cursor::new(vec![184, 96, 208]);
//! let mut source_model = SourceModel::new_eof(10, 9);
//! let mut output = Vec::new();
//! let mut in_reader: BitReader<_, MSB> = BitReader::new(input);
//! let mut decoder = ArithmeticDecoder::new(30);
//!
//! while !decoder.is_finished() {
//!     let sym = decoder.decode(&source_model, &mut in_reader).unwrap();
//!     source_model.update_symbol(sym);
//!     if sym != source_model.get_eof() { output.push(sym)};
//! }
//!
//! assert_eq!(output, &[7, 2, 2, 2, 7]);
//! ```


pub mod util;
pub mod encode;
pub mod decode;

pub extern crate bitbit;
