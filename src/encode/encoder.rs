use crate::util::range::Range;
use crate::util::source_model::SourceModel;
use bitbit::BitWriter;
use std::io::{Error, Write};

pub struct ArithmeticEncoder {
  _precision: u64,
  pending_bit_count: u32,
  range: Range,
}

impl ArithmeticEncoder {
  /// # Arguments
  /// `precision` is the [bit precision](https://en.wikipedia.org/wiki/Arithmetic_coding#Precision_and_renormalization)
  /// that the encoder should use. If the
  /// precision is too low than symbols will not be able to be differentiated.
  pub fn new(precision: u64) -> Self {
    Self {
      _precision: precision,
      pending_bit_count: 0,
      range: Range::new(precision),
    }
  }

  pub fn encode<T: Write>(
    &mut self,
    symbol: u32,
    source_model: &SourceModel,
    output: &mut BitWriter<T>,
  ) -> Result<(), Error> {
    let low_high = self.range.calculate_range(symbol, &source_model);
    self.range.update_range(low_high);

    while self.range.in_bottom_half() || self.range.in_upper_half() {
      if self.range.in_bottom_half() {
        self.range.scale_bottom_half();
        self.emit(false, output)?;
      } else if self.range.in_upper_half() {
        self.range.scale_upper_half();
        self.emit(true, output)?;
      }
    }

    while self.range.in_middle_half() {
      self.pending_bit_count += 1;
      self.range.scale_middle_half();
    }

    Ok(())
  }

  fn emit<T: Write>(&mut self, bit: bool, output: &mut BitWriter<T>) -> Result<(), Error> {
    output.write_bit(bit)?;

    while self.pending_bit_count > 0 {
      output.write_bit(!bit)?;
      self.pending_bit_count -= 1;
    }

    Ok(())
  }

  pub fn finish_encode<T: Write>(&mut self, output: &mut BitWriter<T>) -> Result<(), Error> {
    self.pending_bit_count += 1;

    if self.range.in_bottom_quarter() {
      self.emit(false, output)?;
    } else {
      self.emit(true, output)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::encode::encoder::ArithmeticEncoder;
  use crate::util::source_model_builder::{EOFKind, SourceModelBuilder};
  use bitbit::BitWriter;
  use std::io::Cursor;

  #[test]
  fn e2e() {
    let mut encoder = ArithmeticEncoder::new(30);
    let mut source_model = SourceModelBuilder::new()
      .num_symbols(10)
      .eof(EOFKind::End)
      .build();
    let mut output = Cursor::new(vec![]);
    let mut out_writer = BitWriter::new(&mut output);
    let to_encode: [u32; 5] = [7, 2, 2, 2, 7];
    for x in &to_encode {
      encoder
        .encode(*x, &source_model, &mut out_writer)
        .unwrap();
      source_model.update_symbol(*x);
    }
    encoder
      .encode(source_model.eof(), &source_model, &mut out_writer)
      .unwrap();
    encoder.finish_encode(&mut out_writer).unwrap();
    out_writer.pad_to_byte().unwrap();
    assert_eq!(output.get_ref(), &[184, 96, 208]);
  }
}
