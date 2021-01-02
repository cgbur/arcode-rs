use arcode::bitbit::{BitReader, BitWriter, MSB};
use arcode::decode::decoder::ArithmeticDecoder;
use arcode::encode::encoder::ArithmeticEncoder;
use arcode::util::source_model_builder::{EOFKind, SourceModelBuilder};
use sherlock::SHERLOCK;
use std::io::{Cursor, Result};

mod sherlock;

/// Encodes bytes and returns the compressed form
fn encode(data: &[u8]) -> Result<Vec<u8>> {
  let mut model = SourceModelBuilder::new()
    .num_bits(8)
    .eof(EOFKind::EndAddOne)
    .build();

  // make a stream to collect the compressed data
  let compressed = Cursor::new(vec![]);
  let mut compressed_writer = BitWriter::new(compressed);

  let mut encoder = ArithmeticEncoder::new(48);

  for &sym in data {
    encoder.encode(sym as u32, &model, &mut compressed_writer)?;
    model.update_symbol(sym as u32);
  }

  encoder.encode(model.eof(), &model, &mut compressed_writer)?;
  encoder.finish_encode(&mut compressed_writer)?;
  compressed_writer.pad_to_byte()?;

  // retrieves the bytes from the writer. This will
  // be cleaner when bitbit updates. Not necessary if
  // using files or a stream
  Ok(compressed_writer.get_ref().get_ref().clone())
}

/// Decompresses the data
fn decode(data: &[u8]) -> Result<Vec<u8>> {
  let mut model = SourceModelBuilder::new()
    .num_bits(8)
    .eof(EOFKind::EndAddOne)
    .build();

  let mut input_reader = BitReader::<_, MSB>::new(data);
  let mut decoder = ArithmeticDecoder::new(48);
  let mut decompressed_data = vec![];

  while !decoder.finished() {
    let sym = decoder.decode(&model, &mut input_reader)?;
    model.update_symbol(sym);
    decompressed_data.push(sym as u8);
  }

  decompressed_data.pop(); // remove the EOF

  Ok(decompressed_data)
}

#[test]
fn sherlock_e2e() {
  let sherlock_bytes = SHERLOCK.bytes().into_iter().collect::<Vec<u8>>();
  let compressed = encode(&sherlock_bytes).unwrap();
  let decompressed = decode(&compressed).unwrap();

  assert_eq!(sherlock_bytes.len(), decompressed.len());
  sherlock_bytes
    .iter()
    .zip(decompressed.iter())
    .enumerate()
    .for_each(|(idx, (a, b))| {
      assert_eq!(a, b, "Found mismatch {} != {} at index {}", a, b, idx);
    });
}
