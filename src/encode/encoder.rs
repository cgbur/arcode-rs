use std::io::{Write, Error};
use bitbit::BitWriter;
use crate::util::range::Range;
use crate::util::source_model::SourceModel;


pub struct ArithmeticEncoder {
    _precision: u64,
    pending_bit_count: u32,
    range: Range,
}

impl ArithmeticEncoder {
    pub fn new(precision: u64) -> Self {
        Self {
            _precision: precision,
            pending_bit_count: 0,
            range: Range::new(precision),
        }
    }
    pub fn encode<T: Write>(&mut self, symbol: u32,
                            source_model: &SourceModel,
                            output: &mut BitWriter<T>)
                            -> Result<(), Error> {
        let low_high: (u64, u64) = self.range.calculate_range(symbol, &source_model);
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
    pub fn finish_encode<T: Write>(&mut self, output: &mut BitWriter<T>) -> Result<bool, Error> {
        self.pending_bit_count += 1;
        if self.range.in_bottom_quarter() {
            self.emit(false, output)?;
        } else {
            self.emit(true, output)?;
        }
        Ok(true)
    }
}