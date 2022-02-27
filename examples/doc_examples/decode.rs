use std::io::{Cursor, Result};

use arcode::{
    bitbit::{BitReader, MSB},
    ArithmeticDecoder, EOFKind, Model,
};

/// Decompresses the data
fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let mut model = Model::builder().num_bits(8).eof(EOFKind::EndAddOne).build();

    let mut input_reader = BitReader::<_, MSB>::new(data);
    let mut decoder = ArithmeticDecoder::new(48);
    let mut decompressed_data = vec![];

    while !decoder.finished() {
        let sym = decoder.decode(&model, &mut input_reader)?;
        model.update_symbol(sym);
        decompressed_data.push(sym as u8);
    }

    decompressed_data.pop(); // remove the EOF

    Ok(decompressed_data)
}
