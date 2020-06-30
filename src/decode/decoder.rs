use std::io::{Read, ErrorKind, Error};
use bitbit::reader::Bit;
use bitbit::BitReader;
use crate::util::range::Range;
use crate::util::source_model::SourceModel;

pub struct ArithmeticDecoder {
    range: Range,
    precision: u64,
    first_time: bool,
    input_buffer: u64,
    finished: bool,
}

impl ArithmeticDecoder {
    /// # Arguments
    /// `precision` is the [bit precision](https://en.wikipedia.org/wiki/Arithmetic_coding#Precision_and_renormalization)
    /// that the decoder should use. If the
    /// precision is too low than symbols will not be able to be differentiated.
    pub fn new(precision: u64) -> Self {
        Self {
            range: Range::new(precision),
            precision,
            first_time: true,
            input_buffer: 0,
            finished: false,
        }
    }

    pub fn decode<R: Read, B: Bit>(
        &mut self, source_model:
        &SourceModel, bit_source:
        &mut BitReader<R, B>) -> Result<u32, Error> {
        if self.first_time {
            for _i in 0..self.precision {
                self.input_buffer = (self.input_buffer << 1) | self.bit(bit_source)?;
            }
            self.first_time = false;
        }

        let symbol: u32;
        let mut low_high: (u64, u64);
        let mut sym_idx_low_high = (0, source_model.len());
        loop {
            let sym_idx_mid = (sym_idx_low_high.0 + sym_idx_low_high.1) / 2;
            low_high = self.range.calculate_range(sym_idx_mid, source_model);
            if low_high.0 <= self.input_buffer && self.input_buffer < low_high.1 {
                symbol = sym_idx_mid;
                break;
            } else if self.input_buffer >= low_high.1 {
                sym_idx_low_high.0 = sym_idx_mid + 1;
            } else {
                sym_idx_low_high.1 = sym_idx_mid - 1;
            }
        }
        if symbol == source_model.eof() {
            self.set_finished();
            return Ok(symbol);
        }

        self.range.update_range(low_high);

        while self.range.in_bottom_half() || self.range.in_upper_half() {
            if self.range.in_bottom_half() {
                self.range.scale_bottom_half();
                self.input_buffer = (2 * self.input_buffer) | self.bit(bit_source)?;
            } else if self.range.in_upper_half() {
                self.range.scale_upper_half();
                self.input_buffer = (2 * (self.input_buffer - self.range.half())) | self.bit(bit_source)?;
            }
        }

        while self.range.in_middle_half() {
            self.range.scale_middle_half();
            self.input_buffer = (2 * (self.input_buffer - self.range.quarter())) | self.bit(bit_source)?;
        }
        Ok(symbol)
    }


    fn bit<R: Read, B: Bit>(&mut self, source: &mut BitReader<R, B>)
                            -> Result<u64, Error> {
        match source.read_bit() {
            Ok(res) => Ok(res as u64),
            Err(_e) => {
                if self.precision == 0 {
                    return Err(Error::new(ErrorKind::UnexpectedEof,
                                          "EOF has been read $PRECISION times and \n\
                                          EOF symbol has not been decoded.\n\
                                           Did you forget to encode the EOF symbol?"));
                }
                self.precision -= 1;
                Ok(0)
            }
        }
    }
    pub fn set_finished(&mut self) {
        self.finished = true;
    }
    pub fn finished(&self) -> bool { self.finished }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::util::source_model::SourceModel;
    use bitbit::{BitReader, MSB};
    use crate::decode::decoder::ArithmeticDecoder;

    #[test]
    fn e2e() {
        let input = Cursor::new(vec![184, 96, 208]);
        let mut source_model = SourceModel::new_eof(10, 9);
        let mut output = Vec::new();
        let mut in_reader: BitReader<_, MSB> = BitReader::new(input);

        let mut decoder = ArithmeticDecoder::new(30);
        while !decoder.finished() {
            let sym = decoder.decode(&source_model, &mut in_reader).unwrap();
            source_model.update_symbol(sym);
            if sym != source_model.eof() { output.push(sym) };
        }
        assert_eq!(output, &[7, 2, 2, 2, 7]);
    }
}