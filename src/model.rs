use fenwick::array::{prefix_sum, update};

mod builder;
pub use builder::{Builder, EOFKind};

/// Symbol table for the encoder/decoder.
/// Used to store the probabilities as a vector of counts
/// (number of occurrences). Uniform would be every symbol has
/// a count of 0.
pub struct Model {
    counts: Vec<u32>,
    fenwick_counts: Vec<u32>,
    total_count: u32,
    eof: u32,
    num_symbols: u32,
}

impl Model {
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// For loading a saved model. Use the
    /// [`Builder`] for
    /// more options.
    pub fn from_values(
        counts: Vec<u32>,
        fenwick_counts: Vec<u32>,
        total_count: u32,
        eof: u32,
    ) -> Self {
        Self {
            num_symbols: counts.len() as u32,
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

    pub const fn num_symbols(&self) -> u32 {
        self.num_symbols
    }

    pub fn high(&self, index: u32) -> f64 {
        let high = fenwick::array::prefix_sum(&self.fenwick_counts, index as usize);
        f64::from(high) / f64::from(self.total_count)
    }

    pub fn low(&self, index: u32) -> f64 {
        let low = fenwick::array::prefix_sum(&self.fenwick_counts, index as usize)
            - self.counts[index as usize];
        f64::from(low) / f64::from(self.total_count)
    }

    pub fn probability(&self, symbol: u32) -> (f64, f64) {
        let total = f64::from(self.total_count);

        let high = prefix_sum(&self.fenwick_counts, symbol as usize);
        let low = high - self.counts[symbol as usize];

        (f64::from(low) / total, f64::from(high) / total)
    }

    pub const fn eof(&self) -> u32 {
        self.eof
    }

    pub const fn counts(&self) -> &Vec<u32> {
        &self.counts
    }

    pub const fn fenwick_counts(&self) -> &Vec<u32> {
        &self.fenwick_counts
    }

    pub const fn total_count(&self) -> u32 {
        self.total_count
    }
}

#[cfg(test)]
mod tests {
    use super::{EOFKind, Model};

    #[test]
    fn constructor() {
        let model = Model::builder().num_symbols(4).eof(EOFKind::End).build();

        assert_eq!(3, model.eof());
        assert_eq!(model.probability(0), (0.0, 0.25));
        assert_eq!(model.probability(1), (0.25, 0.5));
        assert_eq!(model.probability(2), (0.5, 0.75));
        assert_eq!(model.probability(3), (0.75, 1.0));
    }

    #[test]
    fn constructor_new() {
        let model = Model::builder().num_symbols(4).build();
        assert_eq!(4, model.eof());
        assert_eq!(model.probability(0), (0.0, 0.25));
        assert_eq!(model.probability(1), (0.25, 0.5));
        assert_eq!(model.probability(2), (0.5, 0.75));
        assert_eq!(model.probability(3), (0.75, 1.0));
    }

    #[test]
    fn constructor_binary() {
        let binary = Model::builder().binary().build();
        let model = Model::builder().num_symbols(2).build();

        assert_eq!(binary.eof(), model.eof());
        assert_eq!(binary.probability(0), model.probability(0));
        assert_eq!(binary.probability(1), model.probability(1));
    }

    #[test]
    fn constructor_from_counts() {
        let mut model = Model::builder().num_symbols(4).eof(EOFKind::End).build();

        let counts_model = Model::builder()
            .counts(vec![1; 4])
            .eof(EOFKind::End)
            .build();

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

        let counts_model = Model::builder()
            .counts(vec![4, 1, 3, 1])
            .eof(EOFKind::End)
            .build();
        assert_eq!(model.probability(0), counts_model.probability(0));
        assert_eq!(model.probability(1), counts_model.probability(1));
        assert_eq!(model.probability(2), counts_model.probability(2));
        assert_eq!(model.probability(3), counts_model.probability(3));
    }

    #[test]
    fn constructor_from_pdf() {
        let mut model = Model::builder().num_symbols(4).eof(EOFKind::End).build();

        let pdf_model = Model::builder()
            .pdf(vec![0.25f32; 4])
            .eof(EOFKind::End)
            .build();

        assert_eq!(3, model.eof());
        assert_eq!(model.probability(0), pdf_model.probability(0));
        assert_eq!(model.probability(1), pdf_model.probability(1));
        assert_eq!(model.probability(2), pdf_model.probability(2));
        assert_eq!(model.probability(3), pdf_model.probability(3));

        model.update_symbol(0);
        model.update_symbol(0);
        model.update_symbol(0);
        model.update_symbol(1);
        model.update_symbol(2);
        model.update_symbol(2);

        let pdf_model = Model::builder()
            .pdf(vec![0.4, 0.2, 0.3, 0.1])
            .eof(EOFKind::End)
            .build();

        assert_eq!(model.probability(0), pdf_model.probability(0));
        assert_eq!(model.probability(1), pdf_model.probability(1));
        assert_eq!(model.probability(2), pdf_model.probability(2));
        assert_eq!(model.probability(3), pdf_model.probability(3));
    }

    #[test]
    fn probability_min() {
        let model = Model::builder().num_symbols(2315).build();
        assert_eq!(model.probability(0), (model.low(0), model.high(0)));
    }

    #[test]
    fn probability_high() {
        let count = 1_000;

        let model = Model::builder().num_symbols(count + 1).build();

        assert_eq!(
            model.probability(count),
            (model.low(count), model.high(count))
        );
    }

    #[test]
    fn update_symbols() {
        let mut model = Model::builder().num_symbols(4).eof(EOFKind::End).build();

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
