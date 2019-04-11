pub struct SourceModel {
    symbols: Vec<u32>,
    counts: Vec<u32>,
    fenwick_counts: Vec<u32>,
    total_count: u32,
    eof: u32,
}

use fenwick::array::{update, prefix_sum};

impl SourceModel {
    pub fn new(symbols_count: u32, eof_symbol: u32) -> Self {
        let symbol_vector = Self::generate_symbol_vec(symbols_count);
        let count_vector = vec![1; symbol_vector.len()];
        let mut fenwick_counts = vec![0u32; count_vector.len()];

        for i in 0..count_vector.len() {
            update(&mut fenwick_counts, i, 1);
        }

        Self {
            total_count: symbol_vector.len() as u32,
            eof: eof_symbol,
            symbols: symbol_vector,
            counts: count_vector,
            fenwick_counts,
        }
    }
    pub fn new_binary() -> Self {
        Self {
            symbols: vec![0, 1],
            counts: vec![1, 1],
            fenwick_counts: vec![1, 2],
            total_count: 2,
            eof: 3,
        }
    }

    pub fn add_symbol(&mut self, symbol: u32) {
        self.total_count += 1;
        self.counts[symbol as usize] += 1;
        update(&mut self.fenwick_counts, symbol as usize, 1);
    }

    pub fn len(&self) -> u32 {
        self.symbols.len() as u32
    }

    pub fn get_high(&self, index: u32) -> f64 {
        if index == self.len() - 1 { 1.0 } else { self.get_low(index + 1) }
    }

    pub fn get_low(&self, index: u32) -> f64 {
        let low: u32 = fenwick::array::prefix_sum(&self.fenwick_counts, index as usize) - self.counts[index as usize];
        low as f64 / self.total_count as f64
    }

    pub fn get_probability(&self, symbol: u32) -> (f64, f64) {
        let symbol_count = self.counts[symbol as usize];
        let total = self.total_count as f64;

        let low: u32 =
            prefix_sum(&self.fenwick_counts, symbol as usize) - symbol_count;
        (low as f64 / total,
         if symbol == self.len() {
            1.0
        } else {
            (low + symbol_count) as f64 / total
        })
    }

    fn generate_symbol_vec(num_symbols: u32) -> Vec<u32> {
        (0..num_symbols).collect()
    }
    pub fn get_eof(&self)-> u32{
        self.eof
    }
}


