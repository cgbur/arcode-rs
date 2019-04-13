use crate::util::source_model::SourceModel;

pub struct Range {
    pub high: u64,
    pub low: u64,
    pub half: u64,
    pub one_quarter_mark: u64,
    pub three_quarter_mark: u64
}

impl Range {
    pub fn new(precision: u64) -> Self {
        assert_eq!(precision < 64, true);
        let high: u64 = 1 << precision;
        Self {
            high,
            low: 0,
            half: high / 2,
            one_quarter_mark: high / 4,
            three_quarter_mark: (high / 4) * 3
        }
    }
    pub fn in_bottom_half(&self) -> bool {
        self.high < self.half
    }
    pub fn in_upper_half(&self) -> bool {
        self.low > self.half
    }
    pub fn in_middle_half(&self) -> bool {
        self.low > self.one_quarter_mark && self.high < self.three_quarter_mark
    }
    pub fn in_bottom_quarter(&self) -> bool {
        self.low <= self.one_quarter_mark
    }

    //scaling the upper half is a left shift, to avoid overflow we minus a 1/2 first
    pub fn scale_upper_half(&mut self) {
        self.low = (self.low - self.half) << 1;
        self.high = (self.high - self.half) << 1;
    }
    //subtract a quarter is the same as shifting out the second most significant bit
    pub fn scale_middle_half(&mut self) {
        self.low = (self.low - self.one_quarter_mark) << 1;
        self.high = (self.high - self.one_quarter_mark) << 1;
    }
    //scaling the bottom half in a left shift
    pub fn scale_bottom_half(&mut self) {
        self.low <<= 1;
        self.high <<= 1;
    }


    //returns (low, high)
    pub fn calculate_range(&mut self, symbol: u32, source_model: &SourceModel) -> (u64, u64) {
        let new_width = self.high - self.low;
        let probability = source_model.get_probability(symbol);
        ((self.low + (new_width as f64 * probability.0) as u64),
         (self.low + (new_width as f64 * probability.1) as u64))
    }

    pub fn update_range(&mut self, low_high: (u64, u64)) {
        self.low = low_high.0;
        self.high = low_high.1;
    }
}