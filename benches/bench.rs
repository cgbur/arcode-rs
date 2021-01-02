use arcode::decode::decoder::ArithmeticDecoder;
use arcode::encode::encoder::ArithmeticEncoder;
use arcode::util::source_model_builder::{EOFKind, SourceModelBuilder};
use bitbit::{BitReader, BitWriter, MSB};
use byte_unit::Byte;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::io::Cursor;

mod sherlock;

fn encode(data: &[u8]) {
  let mut model = SourceModelBuilder::new()
    .num_bits(8)
    .eof(EOFKind::EndAddOne)
    .build();

  let compressed = Cursor::new(vec![]);
  let mut compressed_writer = BitWriter::new(compressed);

  let mut encoder = ArithmeticEncoder::new(48);

  for &sym in data {
    encoder
      .encode(sym as u32, &model, &mut compressed_writer)
      .unwrap();
    model.update_symbol(sym as u32);
  }
  encoder
    .encode(model.eof(), &model, &mut compressed_writer)
    .unwrap();
  encoder.finish_encode(&mut compressed_writer).unwrap();
}

fn encode_return(data: &[u8]) -> Vec<u8> {
  let mut model = SourceModelBuilder::new()
    .num_bits(8)
    .eof(EOFKind::EndAddOne)
    .build();

  let compressed = Cursor::new(vec![]);
  let mut compressed_writer = BitWriter::new(compressed);

  let mut encoder = ArithmeticEncoder::new(48);

  for &sym in data {
    encoder
      .encode(sym as u32, &model, &mut compressed_writer)
      .unwrap();
    model.update_symbol(sym as u32);
  }
  encoder
    .encode(model.eof(), &model, &mut compressed_writer)
    .unwrap();
  encoder.finish_encode(&mut compressed_writer).unwrap();

  compressed_writer.get_ref().get_ref().clone()
}

fn decode(data: &[u8]) {
  let mut model = SourceModelBuilder::new()
    .num_bits(8)
    .eof(EOFKind::EndAddOne)
    .build();

  let mut input_reader = BitReader::<_, MSB>::new(data);
  let mut decoder = ArithmeticDecoder::new(48);
  let mut decompressed_data = vec![];

  while !decoder.finished() {
    let sym = decoder.decode(&model, &mut input_reader).unwrap();
    model.update_symbol(sym);
    decompressed_data.push(sym as u8);
  }
}

pub fn bench_encode(c: &mut Criterion) {
  let scale = 300;
  let sherlock_bytes = sherlock::SHERLOCK.bytes().into_iter().collect::<Vec<u8>>();
  let sherlock_bytes = sherlock_bytes.repeat(scale);
  let compressed_bytes = encode_return(&sherlock_bytes);

  let byte = Byte::from_bytes(sherlock_bytes.len() as u128);
  let label_encode = byte.get_appropriate_unit(false).to_string();
  let byte = Byte::from_bytes(compressed_bytes.len() as u128);
  let label_decode = byte.get_appropriate_unit(false).to_string();

  c.bench_with_input(
    BenchmarkId::new("encode_single", label_encode),
    &sherlock_bytes,
    |b, data| {
      b.iter(|| encode(data));
    },
  );

  c.bench_with_input(
    BenchmarkId::new("decode_single", label_decode),
    &compressed_bytes,
    |b, data| {
      b.iter(|| decode(data));
    },
  );
}

criterion_group!(benches, bench_encode);
criterion_main!(benches);
