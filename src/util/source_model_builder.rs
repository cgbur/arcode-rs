use crate::util::source_model::SourceModel;
use fenwick::array::{update};
use std::cmp::max;

pub enum EOFKind {
  /// Choose a valid index as the EOF `[0, counts.len())`
  Specify(u32),
  /// index 0
  Start,
  /// index `counts.len()` - 1
  End,
  /// adds an element to `counts` and sets EOF to `counts.len() - 1`
  EndAddOne,
  /// Same as not specifying. Sets to value outside symbol range
  None,
}

/// options in precedence order:
///
/// **As of 0.2.0**: Specify the number symbols
/// excluding the EOF symbol. If you specify an EOF
/// it will automatically increase symbol count by one
/// for the EOF. (or else choose value in range). See
/// the setter for EOF for more detail.
///
/// - counts
///   - eof?
/// - pdf
///   - eof?
///   - scale?
/// - symbol count
///   - eof?
/// - binary - default but can also be explicit
///
///
/// You should only use one of the build paths
///
pub struct SourceModelBuilder {
  counts: Option<Vec<u32>>,
  num_symbols: Option<u32>,
  eof: Option<EOFKind>,
  pdf: Option<Vec<f32>>,
  scale: Option<u32>,
  binary: bool,
}

impl SourceModelBuilder {
  pub fn new() -> Self {
    Self {
      counts: None,
      num_symbols: None,
      eof: None,
      pdf: None,
      scale: None,
      binary: false,
    }
  }

  pub fn num_symbols(&mut self, count: u32) -> &mut Self {
    if self.counts.is_some() {
      assert_eq!(self.num_symbols.unwrap(), self.counts.as_ref().unwrap().len() as u32,
                 "Attempted to set num symbols that is not equal to counts.len");
    }

    self.num_symbols = Some(count);
    self
  }

  /// Constructs new model if you already have counts present.
  /// Implied number of symbols from length of `counts`.
  pub fn counts(&mut self, counts: Vec<u32>) -> &mut Self {
    if self.num_symbols.is_some() {
      assert_eq!(self.num_symbols.unwrap(), counts.len() as u32,
                 "Attempted to set counts vec that had different length than previously set num_symbols");
    }

    assert!(counts.len() > 0);

    self.counts = Some(counts.clone());
    self
  }

  /// - Specify(u32): Choose a valid index as the EOF `[0, counts.len())`
  /// - Start: index 0
  /// - End: index `counts.len()` - 1
  /// - EndAddOne: adds an element to `counts` and sets EOF to `counts.len() - 1`
  /// - None: Same as not specifying. Sets to value outside symbol rangec
  pub fn eof(&mut self, eof: EOFKind) -> &mut Self {
    self.eof = Some(eof);
    self
  }

  /// `value = (p * scale)`
  ///
  /// Therefore besides determining the accuracy, scale is
  /// used to determine the elasticity of the model.
  ///
  /// This method will not panic on negative values or values
  /// greater than 1.0. They dont cause mathematical errors so
  /// its on the user to use probabilities correctly.
  pub fn scale(&mut self, scale: u32) -> &mut Self {
    assert!(scale >= 10);
    self.scale = Some(scale);
    self
  }

  /// constructs a new source_model given a vector
  /// of probabilities where the length is the number
  /// of symbols (min 10). Defaults scale to length of pdf.
  ///
  /// *Open to suggestions for default scale*
  pub fn pdf(&mut self, pdf: Vec<f32>) -> &mut Self {
    self.pdf = Some(pdf);
    self
  }

  /// Constructs new model for encoding 0's and 1's
  pub fn binary(&mut self) -> &mut Self {
    self.binary = true;
    self
  }

  pub fn build(&self) -> SourceModel {
    let mut counts = match &self.counts {
      Some(counts) => counts.clone(),
      None => match &self.pdf {
        Some(pdf) => {
          let scale = self.scale.unwrap_or(max(pdf.len() as u32, 10));
          let scale = scale as f32;

          pdf.iter()
              .map(|p| max((p * scale) as i32, 1))
              .map(|c| c as u32).collect()
        }
        None => match self.num_symbols {
          Some(num_symbols) => vec![1; num_symbols as usize],
          None => match self.binary {
            _ => vec![1, 1],
          }
        }
      }
    };


    let eof = match &self.eof {
      None => counts.len() as u32,
      Some(eof_kind) => {
        match eof_kind {
          EOFKind::Specify(index) => {
            assert!(*index < counts.len() as u32);
            *index
          },
          EOFKind::Start => 0,
          EOFKind::End => counts.len() as u32 - 1,
          EOFKind::EndAddOne => {
            counts.push(1);
            counts.len() as u32 - 1
          }
          EOFKind::None => counts.len() as u32
        }
      }
    };

    let mut fenwick_counts = vec![0u32; counts.len()];

    for i in 0..counts.len() {
      update(&mut fenwick_counts, i, counts[i]);
    }

    let total_count = counts.iter().sum();
    SourceModel::from_values(counts, fenwick_counts, total_count, eof)
  }
}


#[cfg(test)]
mod tests {
  use crate::util::source_model_builder::{SourceModelBuilder, EOFKind};
  use crate::util::source_model::SourceModel;

  fn model_eq(a: &SourceModel, b: &SourceModel) {
    assert_eq!(a.eof(), b.eof(), "EOF not equal");
    assert_eq!(a.counts(), b.counts(), "Counts not equal");
    assert_eq!(a.fenwick_counts(), b.fenwick_counts(), "fenwicks not equal");
    assert_eq!(a.total_count(), b.total_count(), "total not equal");
  }

  #[test]
  fn num_symbols() {
    let sut = SourceModelBuilder::new().num_symbols(4).build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1],
      vec![1, 2, 1, 4],
      4,
      4,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn counts() {
    let sut = SourceModelBuilder::new().counts(
      vec![4, 1, 3, 1]
    ).build();

    let reference = SourceModel::from_values(
      vec![4, 1, 3, 1],
      vec![4, 5, 3, 9],
      9,
      4,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn pdf() {
    let sut = SourceModelBuilder::new().pdf(
      vec![0.4, 0.2, 0.3, 0.1]
    ).build();

    let reference = SourceModel::from_values(
      vec![4, 2, 3, 1],
      vec![4, 6, 3, 10],
      10,
      4,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn pdf_scale() {
    let sut = SourceModelBuilder::new().pdf(
      vec![0.4, 0.2, 0.3, 0.1]
    ).scale(20).build();

    let reference = SourceModel::from_values(
      vec![8, 4, 6, 2],
      vec![8, 12, 6, 20],
      20,
      4,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn pdf_scale_defaults_length() {
    let sut = SourceModelBuilder::new().pdf(
      vec![0.4, 0.2, 0.3, 0.1, 0.4, 0.2, 0.3, 0.4, 0.2, 0.3, 0.4, 0.2, 0.3,0.0,0.0]
    ).build();

    let reference = SourceModel::from_values(
      vec![6, 3, 4, 1, 6, 3, 4, 6, 3, 4, 6, 3, 4, 1, 1],
      vec![6, 9, 4, 14, 6, 9, 4, 33, 3, 7, 6, 16, 4, 5, 1],
      55,
      15,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn binary() {
    let sut = SourceModelBuilder::new().binary().build();

    let reference = SourceModel::from_values(
      vec![1,1],
      vec![1,2],
      2,
      2,
    );

    model_eq(&reference, &sut);
  }


  #[test]
  fn default_binary() {
    let sut = SourceModelBuilder::new().build();

    let reference = SourceModel::from_values(
      vec![1,1],
      vec![1,2],
      2,
      2,
    );

    model_eq(&reference, &sut);
  }



  #[test]
  fn eof_end() {
    let sut = SourceModelBuilder::new()
        .num_symbols(4)
        .eof(EOFKind::End)
        .build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1],
      vec![1, 2, 1, 4],
      4,
      3,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn eof_end_add() {
    let sut = SourceModelBuilder::new()
        .num_symbols(4)
        .eof(EOFKind::EndAddOne)
        .build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1, 1],
      vec![1, 2, 1, 4, 1],
      5,
      4,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn eof_start() {
    let sut = SourceModelBuilder::new()
        .num_symbols(4)
        .eof(EOFKind::Start)
        .build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1],
      vec![1, 2, 1, 4],
      4,
      0,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn eof_specify() {
    let sut = SourceModelBuilder::new()
        .num_symbols(4)
        .eof(EOFKind::Specify(2))
        .build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1],
      vec![1, 2, 1, 4],
      4,
      2,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn eof_none() {
    let sut = SourceModelBuilder::new()
        .num_symbols(4)
        .eof(EOFKind::None)
        .build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1],
      vec![1, 2, 1, 4],
      4,
      4,
    );

    model_eq(&reference, &sut);
  }

  #[test]
  fn eof_default() {
    let sut = SourceModelBuilder::new()
        .num_symbols(4)
        .build();

    let reference = SourceModel::from_values(
      vec![1, 1, 1, 1],
      vec![1, 2, 1, 4],
      4,
      4,
    );

    model_eq(&reference, &sut);
  }
}