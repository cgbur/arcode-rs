# Arcode
An arithmetic coder for Rust.


[![Crates.io](https://img.shields.io/crates/v/arcode?color=blueviolet)](https://crates.io/crates/arcode)
[![Crates.io](https://img.shields.io/crates/l/arcode)](https://crates.io/crates/arcode)
[![GitHub top language](https://img.shields.io/github/languages/top/cgburgess/arcode-rs?color=orange)](https://crates.io/crates/arcode)

## About

This crate provides the an efficient implementation of
an [arithmetic encoder/decoder](https://en.wikipedia.org/wiki/Arithmetic_coding). This crate is based off the paper
that describes arithmetic coding found [here](https://web.stanford.edu/class/ee398a/handouts/papers/WittenACM87ArithmCoding.pdf).
This implementation features many readability and performance improvements, especially on the decoding side.

The goal of this project is not to provide an out-of-the-box compression solution.
Arithmetic coding ([entropy encoding](https://en.wikipedia.org/wiki/Entropy_encoding)) is the backbone of almost every
modern day compression scheme. This crate is meant to be included in future projects that rely on an efficient entropy
coder e.g. [PPM](https://en.wikipedia.org/wiki/Prediction_by_partial_matching), [LZ77/LZ78](https://en.wikipedia.org/wiki/LZ77_and_LZ78),
[h265/HEVC](https://en.wikipedia.org/wiki/High_Efficiency_Video_Coding).

## Core components

There are a lot of structs available for use but for the average user there are only a few that will be used.
- [SourceModel](util/source_model/struct.SourceModel.html) models of the probability of symbols. Counts can be adjusted
as encoding is done to improve compression.
- [Encoder](encode/encoder/struct.ArithmeticEncoder.html) encodes symbols given a source model and a symbol.
- [Decoder](decode/decoder/struct.ArithmeticDecoder.html) decodes symbols given a source model and a bitstream.

# Examples
In the git repository there is an [old_complex.rs](https://github.com/cgburgess/arcode-rs/blob/master/example/example.rs)
file that does context switching on a per character basis. A simpler example can be found at [new_simple.rs](https://github.com/cgburgess/arcode-rs/blob/master/tests/integration_test.rs)

## Input and output bitstreams
In order for arithmetic coding to work streams need to be read a bit at a time (for decoding and for the encoders output).
Because of this, [BitBit](https://docs.rs/bitbit) is required. **Wrapping whatever your input is in a buffered reader/writer
should greatly improve performance.**

Using bitbit to create an input stream.
```rust
use arcode::bitbit::{BitReader, MSB, BitWriter};
use std::io::Cursor;

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
```

### Source Model(s)
Depending on your application you could have one or hundreds/thousands of source models.
The source model is heavily relied on by the encoder and the decoder. If the decoder ever becomes
out of phase with the encoder you will be decoding nonsense.

```rust
use arcode::util::source_model_builder::{SourceModelBuilder, EOFKind};
// create a new model that has symbols 0-256
// 8 bit values + one EOF marker
let mut model_with_eof = SourceModelBuilder::new()
          .num_symbols(256).eof(EOFKind::EndAddOne).build();
// model for 8 bit 0 - 255, if we arent using
// the EOF flag set it to anything outside the range.
// Both of the below are equivalent
let model_without_eof = SourceModelBuilder::new()
          .num_symbols(256).eof(EOFKind::None).build();
let model_without_eof = SourceModelBuilder::new()
         .num_symbols(256).build();

// update the probability of symbol 4.
model_with_eof.update_symbol(4);
``
## Encode
Encoding some simple input
```rust
use arcode::encode::encoder::ArithmeticEncoder;
use std::io::Cursor;
use bitbit::BitWriter;
use arcode::util::source_model_builder::{SourceModelBuilder, EOFKind};

let mut encoder = ArithmeticEncoder::new(30);
let mut source_model = SourceModelBuilder::new()
      .num_symbols(10).eof(EOFKind::End).build();
let mut output = Cursor::new(vec![]);
let mut out_writer = BitWriter::new(&mut output);
let to_encode: [u32; 5] = [7, 2, 2, 2, 7];

for x in to_encode.iter() {
    encoder.encode(*x, &mut source_model, &mut out_writer).unwrap();
    source_model.update_symbol(*x);
}

encoder.encode(source_model.eof(), &source_model, &mut out_writer).unwrap();
encoder.finish_encode(&mut out_writer).unwrap();
out_writer.pad_to_byte().unwrap();

assert_eq!(output.get_ref(), &[184, 96, 208]);
```
### Decode
```rust
use std::io::Cursor;
use bitbit::{BitReader, MSB};
use arcode::decode::decoder::ArithmeticDecoder;
use arcode::util::source_model_builder::{SourceModelBuilder, EOFKind};

let input = Cursor::new(vec![184, 96, 208]);
let mut source_model = SourceModelBuilder::new()
      .num_symbols(10).eof(EOFKind::End).build();
let mut output = Vec::new();
let mut in_reader: BitReader<_, MSB> = BitReader::new(input);
let mut decoder = ArithmeticDecoder::new(30);

while !decoder.finished() {
    let sym = decoder.decode(&source_model, &mut in_reader).unwrap();
    source_model.update_symbol(sym);
    if sym != source_model.eof() { output.push(sym)};
}

assert_eq!(output, &[7, 2, 2, 2, 7]);
```
