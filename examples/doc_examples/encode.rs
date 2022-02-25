use arcode::bitbit::BitWriter;
use arcode::encode::encoder::ArithmeticEncoder;
use arcode::util::source_model_builder::{EOFKind, SourceModelBuilder};
use std::io::{Cursor, Result};

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
