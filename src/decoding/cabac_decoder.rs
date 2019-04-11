use std::io::Read;

use bitbit::reader::Bit;
use bitbit::BitReader;
use std::error::Error;
use crate::util::source_model::SourceModel;
use super::decoder::ArithmeticDecoder;

pub struct CABACDecoder {
    models: Vec<SourceModel>
}

impl CABACDecoder {
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

    pub fn decode<R: Read, B: Bit>(&mut self, decoder: &mut ArithmeticDecoder, input: &mut BitReader<R, B>) -> Result<u32, Box<Error>> {
        let mut value: u32 = 0;
        for model in self.models.iter_mut() {
            let sym = decoder.decode(model, input)?;
            model.add_symbol(sym);
            value = value * 2 + sym;
        }
        Ok(value)
    }
}