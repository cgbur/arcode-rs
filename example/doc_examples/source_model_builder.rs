use arcode::util::source_model_builder::{EOFKind, SourceModelBuilder};

fn source_model_example() {
  // create a new model that has symbols 0-256
  // 8 bit values + one EOF marker
  let mut model_with_eof = SourceModelBuilder::new()
    .num_symbols(256)
    .eof(EOFKind::EndAddOne)
    .build();

  // model for 8 bit 0 - 255, if we arent using
  // the EOF flag we can set it to NONE or let it default
  // to none as in the second example below.
  let model_without_eof = SourceModelBuilder::new()
    .num_symbols(256)
    .eof(EOFKind::None)
    .build();
  let model_without_eof = SourceModelBuilder::new().num_symbols(256).build();

  // we can also create a model for 0-255 using num_bits
  let model_8_bit = SourceModelBuilder::new().num_bits(8).build();

  // update the probability of symbol 4.
  model_with_eof.update_symbol(4);
}
