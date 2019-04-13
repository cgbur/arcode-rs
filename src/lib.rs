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
//! In the git repository there is an example.rs file that is a complete
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
//! use arithmetic_coder::util::source_model::SourceModel;
//! // create a new model that has symbols 0-256
//! // 8 bit values + one EOF marker
//! let mut model_with_eof = SourceModel::new(257, 256);
//! // model for 8 bit 0 - 255, if we arent using
//! // the EOF flag set it to anything outside the range.
//! let model_without_eof = SourceModel::new(256, 9999);
//!
//! // update the probability of symbol 4.
//! model_with_eof.update_symbol(4);
//!```
//!
//! ## Encode
//! An example of using multiple models to context adaptive encode
//! ```rust
//! // make 256 models (one for every value a byte can represent).
//! use arithmetic_coder::util::source_model::SourceModel;
//! use std::io::{Cursor, BufReader, BufWriter};
//! use std::fs::File;
//! use bitbit::{BitReader, BitWriter, MSB};
//! use std::fs;
//! use arithmetic_coder::encode::encoder::ArithmeticEncoder;
//!
//! let mut encoder = ArithmeticEncoder::new(42);
//! let mut models = Vec::with_capacity(257);
//!  for i in 0..257 {
//!      models.push(SourceModel::new(257, 256));
//!  }
//! // setup your input
//! //let input_file = File::open(input_path)?;
//! let input = Cursor::new(vec![0,0,0]);
//! let output = Cursor::new(vec![]);
//! let mut buffer_input = BufReader::new(input);
//! let mut buffered_output = BufWriter::new(output);
//! let mut input: BitReader<_, MSB> = BitReader::new(&mut buffer_input);
//! let mut out_writer = BitWriter::new(&mut buffered_output);
//!
//!  //let num_bytes = fs::metadata(input_path).unwrap().len();
//!   let num_bytes = 3;
//!   let mut current_model = &mut models[0];
//!   for x in 0..num_bytes {
//!       let symbol: u32 = input.read_byte().unwrap() as u32;
//!       encoder.encode(symbol, current_model, &mut out_writer).unwrap();
//!       current_model.update_symbol(symbol);
//!       current_model = &mut models[symbol as usize];
//!   }
//!   encoder.encode(current_model.get_eof(), current_model, &mut out_writer);
//!   encoder.finish_encode( &mut out_writer).unwrap();
//!   out_writer.pad_to_byte().unwrap();
//! ```
//! ## Decode
//! Now to decode that same adaptive encode.
//! ```rust
//! use arithmetic_coder::decode::decoder::ArithmeticDecoder;
//! use arithmetic_coder::util::source_model::SourceModel;
//! use std::io::{Cursor, BufReader, BufWriter, Write};
//! use bitbit::{BitReader, MSB, BitWriter};
//! let mut decoder = ArithmeticDecoder::new(42);
//! let mut models = Vec::with_capacity(257);
//!  for i in 0..257 {
//!      models.push(SourceModel::new(257, 256));
//!  }
//! // setup your input
//! //let input_file = File::open(input_path)?;
//! let input = Cursor::new(vec![0; 500]); //just an example so random length array
//! let output = Cursor::new(vec![]);
//! let mut buffer_input = BufReader::new(input);
//! let mut buffered_output = BufWriter::new(output);
//! let mut input: BitReader<_, MSB> = BitReader::new(&mut buffer_input);
//! let mut out_writer = BitWriter::new(&mut buffered_output);
//!
//! let mut current_model = &mut models[0];
//!
//! // without a properly encoded file this wont work, please see the github for the entire
//! // example that you can throw into a new rust project and run.
//! //while !decoder.is_finished() {
//! while false {
//!     let sym = decoder.decode(current_model, &mut input).unwrap();
//!     if sym != current_model.get_eof() { out_writer.write_byte(sym as u8).unwrap(); }
//!     current_model.update_symbol(sym);
//!     current_model = &mut models[sym as usize];
//! }
//! buffered_output.flush().unwrap();
//! ```
//!
//!


pub mod util;
pub mod encode;
pub mod decode;

pub extern crate bitbit;
