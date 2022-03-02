//! # Arcode
//! An arithmetic coder for Rust.
//!
//!
//! [![Crates.io](https://img.shields.io/crates/v/arcode?color=blueviolet)](https://crates.io/crates/arcode)
//! [![Crates.io](https://img.shields.io/crates/l/arcode)](https://crates.io/crates/arcode)
//! [![GitHub top language](https://img.shields.io/github/languages/top/cgburgess/arcode-rs?color=orange)](https://crates.io/crates/arcode)
//!
//! ## About
//!
//! This crate provides the an efficient implementation of
//! an [arithmetic encoder/decoder](https://en.wikipedia.org/wiki/Arithmetic_coding). This crate is based off the paper
//! that describes arithmetic coding found [here](https://web.stanford.edu/class/ee398a/handouts/papers/WittenACM87ArithmCoding.pdf).
//! This implementation features many readability and performance improvements,
//! especially on the decoding side.
//!
//! The goal of this project is not to provide an out-of-the-box compression
//! solution. Arithmetic coding ([entropy encoding](https://en.wikipedia.org/wiki/Entropy_encoding)) is the backbone of almost every
//! modern day compression scheme. This crate is meant to be included in future
//! projects that rely on an efficient entropy coder e.g. [PPM](https://en.wikipedia.org/wiki/Prediction_by_partial_matching), [LZ77/LZ78](https://en.wikipedia.org/wiki/LZ77_and_LZ78),
//! [h265/HEVC](https://en.wikipedia.org/wiki/High_Efficiency_Video_Coding).
//!
//! ## Core components
//!
//! There are a lot of structs available for use but for the average user there
//! are only a few that will be used.
//! - [`Model`] models of the probability of symbols. Counts can be adjusted
//! as encoding is done to improve compression.
//! - [`Encoder`](ArithmeticEncoder) encodes symbols given a source model and a
//!   symbol.
//! - ['Decoder'](ArithmeticDecoder) decodes symbols given a source model and a
//!   bitstream.
//!
//! # Examples
//! In the git repository there is an [old_complex.rs](https://github.com/cgburgess/arcode-rs/blob/master/example/example.rs)
//! file that does context switching on a per character basis. A simpler example can be found at [new_simple.rs](https://github.com/cgburgess/arcode-rs/blob/master/tests/integration_test.rs)
//!
//! ## Input and output bitstreams
//! In order for arithmetic coding to work streams need to be read a bit at a
//! time (for decoding and for the encoders output). Because of this, [BitBit](https://docs.rs/bitbit) is required. **Wrapping whatever your input is in a buffered reader/writer
//! should greatly improve performance.**
//!
//! Using bitbit to create an input stream.
//! ```rust
//! use std::io::Cursor;
//!
//! use arcode::bitbit::{BitReader, BitWriter, MSB};
//!
//! fn read_example() {
//!     // normally you would have a Read type with a BufReader
//!     let mut source = Cursor::new(vec![0u8; 4]);
//!     let input: BitReader<_, MSB> = BitReader::new(&mut source);
//! }
//!
//! fn out_example() {
//!     // once again would be Write type with a BufWriter
//!     let compressed = Cursor::new(vec![]);
//!     let mut compressed_writer = BitWriter::new(compressed);
//! }
//! ```
//!
//! ### Source Model(s)
//! Depending on your application you could have one or many source models.
//! The source model is relied on by the encoder and the decoder. If the decoder
//! ever becomes out of phase with the encoder you will be decoding nonsense.
//!
//! #### Model Builder
//! In order to make a source model you need to use the [`model::Builder`]
//! struct.
//!
//! ```rust
//! use arcode::{EOFKind, Model};
//!
//! fn source_model_example() {
//!     // create a new model that has symbols 0-256
//!     // 8 bit values + one EOF marker
//!     let mut model_with_eof = Model::builder()
//!         .num_symbols(256)
//!         .eof(EOFKind::EndAddOne)
//!         .build();
//!
//!     // model for 8 bit 0 - 255, if we arent using
//!     // the EOF flag we can set it to NONE or let it default
//!     // to none as in the second example below.
//!     let model_without_eof = Model::builder().num_symbols(256).eof(EOFKind::None).build();
//!     let model_without_eof = Model::builder().num_symbols(256).build();
//!
//!     // we can also create a model for 0-255 using num_bits
//!     let model_8_bit = Model::builder().num_bits(8).build();
//!
//!     // update the probability of symbol 4.
//!     model_with_eof.update_symbol(4);
//! }
//! ```
//! ## Encode
//! Encoding some simple input
//! ```rust
//! use std::io::{Cursor, Result};
//!
//! use arcode::{bitbit::BitWriter, ArithmeticEncoder, EOFKind, Model};
//!
//! /// Encodes bytes and returns the compressed form
//! fn encode(data: &[u8]) -> Result<Vec<u8>> {
//!     let mut model = Model::builder().num_bits(8).eof(EOFKind::EndAddOne).build();
//!
//!     // make a stream to collect the compressed data
//!     let compressed = Cursor::new(vec![]);
//!     let mut compressed_writer = BitWriter::new(compressed);
//!
//!     let mut encoder = ArithmeticEncoder::new(48);
//!
//!     for &sym in data {
//!         encoder.encode(sym as u32, &model, &mut compressed_writer)?;
//!         model.update_symbol(sym as u32);
//!     }
//!
//!     encoder.encode(model.eof(), &model, &mut compressed_writer)?;
//!     encoder.finish_encode(&mut compressed_writer)?;
//!     compressed_writer.pad_to_byte()?;
//!
//!     // retrieves the bytes from the writer. This will
//!     // be cleaner when bitbit updates. Not necessary if
//!     // using files or a stream
//!     Ok(compressed_writer.get_ref().get_ref().clone())
//! }
//! ```
//! ### Decode
//! ```rust
//! use std::io::{Cursor, Result};
//!
//! use arcode::{
//!     bitbit::{BitReader, MSB},
//!     ArithmeticDecoder, EOFKind, Model,
//! };
//!
//! /// Decompresses the data
//! fn decode(data: &[u8]) -> Result<Vec<u8>> {
//!     let mut model = Model::builder().num_bits(8).eof(EOFKind::EndAddOne).build();
//!
//!     let mut input_reader = BitReader::<_, MSB>::new(data);
//!     let mut decoder = ArithmeticDecoder::new(48);
//!     let mut decompressed_data = vec![];
//!
//!     while !decoder.finished() {
//!         let sym = decoder.decode(&model, &mut input_reader)?;
//!         model.update_symbol(sym);
//!         decompressed_data.push(sym as u8);
//!     }
//!
//!     decompressed_data.pop(); // remove the EOF
//!
//!     Ok(decompressed_data)
//! }
//! ```

pub mod binary;
mod decode;
mod encode;
pub mod model;
mod range;

pub use bitbit;
pub use decode::ArithmeticDecoder;
pub use encode::ArithmeticEncoder;
pub use model::{EOFKind, Model};
pub use range::Range;
