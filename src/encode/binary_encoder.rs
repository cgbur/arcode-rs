use std::io::Write;
use bitbit::BitWriter;
use std::error::Error;
use crate::util::source_model::SourceModel;
use super::encoder::ArithmeticEncoder;

pub struct BinaryEncoder {
  models: Vec<SourceModel>
}

impl BinaryEncoder {
  pub fn new(max_value: u32) -> Self {
    let bit_width = 32 - max_value.leading_zeros();
    let mut models: Vec<SourceModel> = Vec::with_capacity(bit_width as usize);
    for _i in 0..bit_width {
      models.push(SourceModel::new_binary());
    }
    Self {
      models
    }
  }

  pub fn encode<W: Write>(&mut self, encoder: &mut ArithmeticEncoder, output: &mut BitWriter<W>, value: u32)
                          -> Result<(), Box<Error>> {
    for i in 0..self.models.len() {
      let symbol = (value >> (self.models.len() - i - 1)) & 0x1;
      encoder.encode(symbol, &self.models[i], output)?;
      self.models[i].update_symbol(symbol);
    }
    Ok(())
  }
}