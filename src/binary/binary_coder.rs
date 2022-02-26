use crate::decode::decoder::ArithmeticDecoder;
use crate::encode::encoder::ArithmeticEncoder;
use crate::util::source_model::SourceModel;
use crate::util::source_model_builder::SourceModelBuilder;
use bitbit::reader::Bit;
use bitbit::{BitReader, BitWriter};
use std::io::{Error, Read, Write};

pub struct BinaryCoder {
    models: Vec<SourceModel>,
}

impl BinaryCoder {
    pub fn new_from_max(max_value: u32) -> Self {
        let bit_width = 32 - max_value.leading_zeros();
        Self::new(bit_width)
    }

    pub fn new(bit_width: u32) -> Self {
        let mut models: Vec<SourceModel> = Vec::with_capacity(bit_width as usize);
        for _i in 0..bit_width {
            models.push(SourceModelBuilder::new().binary().build());
        }
        Self { models }
    }

    pub fn from_values(models: Vec<SourceModel>) -> Self {
        Self { models }
    }

    pub fn encode<W: Write>(
        &mut self,
        encoder: &mut ArithmeticEncoder,
        output: &mut BitWriter<W>,
        value: u32,
    ) -> Result<(), Error> {
        for i in 0..self.models.len() {
            let symbol = (value >> (self.models.len() - i - 1) as u32) & 0x1;
            encoder.encode(symbol, &self.models[i], output)?;
            self.models[i].update_symbol(symbol);
        }
        Ok(())
    }

    pub fn decode<R: Read, B: Bit>(
        &mut self,
        decoder: &mut ArithmeticDecoder,
        input: &mut BitReader<R, B>,
    ) -> Result<u32, Error> {
        let mut value: u32 = 0;
        for model in self.models.iter_mut() {
            let sym = decoder.decode(model, input)?;
            model.update_symbol(sym);
            value = value * 2 + sym;
        }
        Ok(value)

    }

    pub fn models(&self) -> &[SourceModel] {
        &self.models
    }
}
