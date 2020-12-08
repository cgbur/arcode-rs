use crate::util::source_model::SourceModel;

pub struct Range {
  high: u64,
  low: u64,
  half: u64,
  one_quarter_mark: u64,
  three_quarter_mark: u64,
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
      three_quarter_mark: (high / 4) * 3,
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

  /// scaling the upper half is a left shift, to avoid overflow we minus a 1/2 first
  pub fn scale_upper_half(&mut self) {
    self.low = (self.low - self.half) << 1;
    self.high = (self.high - self.half) << 1;
  }
  /// subtract a quarter is the same as shifting out the second most significant bit
  pub fn scale_middle_half(&mut self) {
    self.low = (self.low - self.one_quarter_mark) << 1;
    self.high = (self.high - self.one_quarter_mark) << 1;
  }
  /// scaling the bottom half in a left shift
  pub fn scale_bottom_half(&mut self) {
    self.low <<= 1;
    self.high <<= 1;
  }

  /// returns (low, high)
  pub fn calculate_range(&self, symbol: u32, source_model: &SourceModel) -> (u64, u64) {
    let new_width = self.high - self.low;
    let probability = source_model.probability(symbol);
    ((self.low + (new_width as f64 * probability.0) as u64),
     (self.low + (new_width as f64 * probability.1) as u64))
  }

  pub fn update_range(&mut self, low_high: (u64, u64)) {
    self.low = low_high.0;
    self.high = low_high.1;
  }

  pub fn half(&self) -> u64 {
    self.half
  }
  pub fn quarter(&self) -> u64 {
    self.one_quarter_mark
  }
}

#[cfg(test)]
mod tests {
  use crate::util::range::Range;
  use crate::util::source_model_builder::SourceModelBuilder;

  #[test]
  fn constructor() {
    let range = Range::new(5);
    assert_eq!(range.high, 32);
    assert_eq!(range.one_quarter_mark, range.high / 4);
    assert_eq!(range.half, range.high / 2);
    assert_eq!(range.three_quarter_mark, range.high - range.one_quarter_mark);

    assert_eq!(range.half, range.half());
    assert_eq!(range.one_quarter_mark, range.quarter());
  }

  #[test]
  fn calculate_range() {

    let model = SourceModelBuilder::new()
        .num_symbols(3).build();

    let range = Range::new(8);
    assert_eq!(range.calculate_range(0, &model), (0, 85));
    assert_eq!(range.calculate_range(1, &model), (85, 170));
    assert_eq!(range.calculate_range(2, &model), (170, 256));
  }

  #[test]
  fn test_range() {
    let model = SourceModelBuilder::new()
        .num_symbols(3).build();

    let mut range = Range::new(8);
    range.update_range(range.calculate_range(0, &model));
    assert_eq!(range.in_bottom_half(), true);
    assert_eq!(range.in_upper_half(), false);
    assert_eq!(range.in_middle_half(), false);
    assert_eq!(range.in_bottom_quarter(), true);

    let model = SourceModelBuilder::new()
        .num_symbols(3).build();

    let mut range = Range::new(8);
    range.update_range(range.calculate_range(2, &model));
    assert_eq!(range.in_bottom_half(), false);
    assert_eq!(range.in_upper_half(), true);
    assert_eq!(range.in_middle_half(), false);
    assert_eq!(range.in_bottom_quarter(), false);

    let model = SourceModelBuilder::new()
        .num_symbols(100).build();

    let mut range = Range::new(12);
    range.update_range(range.calculate_range(50, &model));
    assert_eq!(range.in_bottom_half(), false);
    assert_eq!(range.in_upper_half(), false);
    assert_eq!(range.in_middle_half(), true);
    assert_eq!(range.in_bottom_quarter(), false);
  }
}