pub struct SourceModel {
  counts: Vec<u32>,
  fenwick_counts: Vec<u32>,
  total_count: u32,
  eof: u32,
}

use fenwick::array::{update, prefix_sum};

impl SourceModel {
  pub fn new_eof(symbols_count: u32, eof_symbol: u32) -> Self {
    let count_vector = vec![1; symbols_count as usize];
    let mut fenwick_counts = vec![0u32; count_vector.len()];

    for i in 0..count_vector.len() {
      update(&mut fenwick_counts, i, 1);
    }

    Self {
      total_count: symbols_count as u32,
      eof: eof_symbol,
      counts: count_vector,
      fenwick_counts,
    }
  }

  pub fn new(symbols_count: u32) -> Self {
    Self::new_eof(symbols_count, symbols_count + 1)
  }

  pub fn new_binary() -> Self {
    Self {
      counts: vec![1, 1],
      fenwick_counts: vec![1, 2],
      total_count: 2,
      eof: 3,
    }
  }

  pub fn from_counts(counts: Vec<u32>, eof: u32) -> Self {
    let mut fenwick_counts = vec![0u32; counts.len()];

    for i in 0..counts.len() {
      update(&mut fenwick_counts, i, counts[i]);
    }

    let total_count = counts.iter().sum();
    Self {
      counts,
      fenwick_counts,
      total_count,
      eof,
    }
  }

  pub fn update_symbol(&mut self, symbol: u32) {
    self.total_count += 1;
    self.counts[symbol as usize] += 1;
    update(&mut self.fenwick_counts, symbol as usize, 1);
  }

  pub fn len(&self) -> u32 {
    self.counts.len() as u32
  }

  pub fn high(&self, index: u32) -> f64 {
    let high: u32 = fenwick::array::prefix_sum(&self.fenwick_counts, index as usize);
    high as f64 / self.total_count as f64
  }

  pub fn low(&self, index: u32) -> f64 {
    let low: u32 = fenwick::array::prefix_sum(&self.fenwick_counts, index as usize) - self.counts[index as usize];
    low as f64 / self.total_count as f64
  }

  pub fn probability(&self, symbol: u32) -> (f64, f64) {
    let total = self.total_count as f64;

    let high = prefix_sum(&self.fenwick_counts, symbol as usize);
    let low = high - self.counts[symbol as usize];
    (low as f64 / total, high as f64 / total)
  }

  fn generate_symbol_vec(num_symbols: u32) -> Vec<u32> {
    (0..num_symbols).collect()
  }

  pub fn eof(&self) -> u32 {
    self.eof
  }
}

#[cfg(test)]
mod tests {
  use crate::util::source_model::SourceModel;

  #[test]
  fn constructor() {
    let model = SourceModel::new_eof(4, 3);
    assert_eq!(3, model.eof());
    assert_eq!(model.probability(0), (0.0, 0.25));
    assert_eq!(model.probability(1), (0.25, 0.5));
    assert_eq!(model.probability(2), (0.5, 0.75));
    assert_eq!(model.probability(3), (0.75, 1.0));
  }

  #[test]
  fn constructor_binary() {
    let binary = SourceModel::new_binary();
    let model = SourceModel::new(2);
    assert_eq!(binary.eof(), model.eof());
    assert_eq!(binary.probability(0), model.probability(0));
    assert_eq!(binary.probability(1), model.probability(1));
  }

  #[test]
  fn constructor_from_counts() {
    let mut model = SourceModel::new_eof(4, 3);
    let counts_model = SourceModel::from_counts(vec![1; 4], 3);
    assert_eq!(3, model.eof());
    assert_eq!(model.probability(0), counts_model.probability(0));
    assert_eq!(model.probability(1), counts_model.probability(1));
    assert_eq!(model.probability(2), counts_model.probability(2));
    assert_eq!(model.probability(3), counts_model.probability(3));

    model.update_symbol(0);
    model.update_symbol(0);
    model.update_symbol(0);
    model.update_symbol(2);
    model.update_symbol(2);

    let counts_model = SourceModel::from_counts(vec![4, 1, 3, 1], 3);
    assert_eq!(model.probability(0), counts_model.probability(0));
    assert_eq!(model.probability(1), counts_model.probability(1));
    assert_eq!(model.probability(2), counts_model.probability(2));
    assert_eq!(model.probability(3), counts_model.probability(3));
  }

  #[test]
  fn probability_min() {
    let model = SourceModel::new_eof(1000, 3);
    assert_eq!(model.probability(0),
               (model.low(0), model.high(0)));
  }

  #[test]
  fn probability_high() {
    let count = 1_000;
    let model = SourceModel::new_eof(count + 1, 3);

    assert_eq!(model.probability(count),
               (model.low(count), model.high(count)));
  }

  #[test]
  fn update_symbols() {
    let mut model = SourceModel::new_eof(4, 3);
    model.update_symbol(2);
    model.update_symbol(2);
    model.update_symbol(2);
    model.update_symbol(3);
    model.update_symbol(1);
    model.update_symbol(3);

    assert_eq!(model.probability(0), (0.0, 0.1));
    assert_eq!(model.probability(1), (0.1, 0.3));
    assert_eq!(model.probability(2), (0.3, 0.7));
    assert_eq!(model.probability(3), (0.7, 1.0));
  }
}
