//! Note this example is using multiple models to change
//! the prediction on a per symbol basis. For a cleaner
//! example look at the integration test found in `tests/`

use std::{
    error::Error,
    fs,
    fs::File,
    io::{BufReader, BufWriter, Write},
    time::Instant,
};

use arcode::{ArithmeticDecoder, ArithmeticEncoder, EOFKind, Model};
use bitbit::{BitReader, BitWriter, MSB};

mod doc_examples;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Make sure to run with --release");
    comp_decomp()?;
    Ok(())
}

fn comp_decomp() -> Result<(), Box<dyn Error>> {
    let num_symbols = 257;
    let precision = 42;
    let input_path = "./examples/sherlock.txt";
    let compressed_path = "./examples/compressed.dat";
    let uncompressed_path = "./examples/decompressed.txt";

    let num_bytes = fs::metadata(input_path).unwrap().len();
    let mut byte_break = (num_bytes as f64 / 8f64) as u64;
    if byte_break < 2 {
        byte_break = 1;
    };

    let mut encoder = ArithmeticEncoder::new(precision);

    let mut models = Vec::with_capacity(num_symbols);
    for _i in 0..num_symbols {
        // models.push(SourceModel::new(num_symbols as u32, 256));
        models.push(
            Model::builder()
                .num_symbols(num_symbols as u32)
                .eof(EOFKind::End)
                .build(),
        );
    }
    let input_file = File::open(input_path)?;
    let mut buffer_input = BufReader::new(input_file);
    let output_file = File::create(compressed_path)?;
    let mut buffered_output = BufWriter::new(output_file);
    let mut input: BitReader<_, MSB> = BitReader::new(&mut buffer_input);
    let mut out_writer = BitWriter::new(&mut buffered_output);
    let encode_start = Instant::now();

    let num_bytes = fs::metadata(input_path).unwrap().len();
    let mut current_model = &mut models[0];
    for x in 0..num_bytes {
        if x % byte_break == 0 {
            println!("{:.1}%", (x as f64 / num_bytes as f64) * 100.0);
        }
        let symbol = input.read_byte().unwrap().into();
        encoder.encode(symbol, current_model, &mut out_writer)?;
        current_model.update_symbol(symbol);
        current_model = &mut models[symbol as usize];
    }
    encoder.encode(current_model.eof(), current_model, &mut out_writer)?;
    encoder.finish_encode(&mut out_writer)?;
    out_writer.pad_to_byte()?;
    let finished = encode_start.elapsed().as_millis();
    println!(
        "{:.2}Mbps",
        ((num_bytes as f64 * 8.0) / (finished as f64 / 1000.0)) / 1_000_000_f64
    );
    buffered_output.flush()?;

    let compressed_bytes = fs::metadata(compressed_path).unwrap().len();

    // decode
    let mut decoder = ArithmeticDecoder::new(precision);

    let input_file = File::open(compressed_path)?;
    let mut buffer_input = BufReader::new(input_file);
    let output_file = File::create(uncompressed_path)?;
    let mut buffered_output = BufWriter::new(output_file);
    let mut input: BitReader<_, MSB> = BitReader::new(&mut buffer_input);
    let mut out_writer = BitWriter::new(&mut buffered_output);

    let mut models = Vec::with_capacity(num_symbols);
    for _i in 0..num_symbols {
        models.push(
            Model::builder()
                .num_symbols(num_symbols as u32)
                .eof(EOFKind::End)
                .build(),
        );
    }

    let mut x = 0;
    let decode_start = Instant::now();

    let mut current_model = &mut models[0];

    while !decoder.finished() {
        if x % byte_break == 0 {
            println!("{:.1}%", (x as f64 / num_bytes as f64) * 100.0);
        }
        let sym = decoder.decode(current_model, &mut input)?;
        if sym != current_model.eof() {
            out_writer.write_byte(sym as u8)?;
        }
        current_model.update_symbol(sym);
        current_model = &mut models[sym as usize];
        x += 1;
    }
    let finished = decode_start.elapsed().as_millis();
    println!(
        "{:.2}Mbps",
        ((num_bytes as f64 * 8.0) / (finished as f64 / 1000.0)) / 1_000_000_f64
    );

    buffered_output.flush()?;

    //
    let input1 = fs::File::open(input_path)?;
    let input2 = fs::File::open(uncompressed_path)?;
    let mut i1: BitReader<_, MSB> = BitReader::new(BufReader::new(input1));
    let mut i2: BitReader<_, MSB> = BitReader::new(BufReader::new(input2));
    if fs::metadata(uncompressed_path).unwrap().len() == fs::metadata(input_path).unwrap().len() {
        println!("files are same length");
        let mut matching = true;
        for _i in 0..num_bytes {
            if i1.read_byte()? != i2.read_byte()? {
                matching = false;
            }
        }
        println!("Matching? {}", matching);
        if matching {
            println!(
                "input: {}, output: {}, compression ratio {}",
                num_bytes,
                compressed_bytes,
                num_bytes as f64 / compressed_bytes as f64
            );
        }
    } else {
        println!("files not same length");
    }

    Ok(())
}
