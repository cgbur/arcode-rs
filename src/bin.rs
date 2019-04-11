use arithmetic_rs::util::source_model::SourceModel;
use arithmetic_rs::encoding::encoder::ArithmeticEncoder;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use bitbit::{BitReader, MSB, BitWriter};
use std::time::Instant;
use arithmetic_rs::decoding::decoder::ArithmeticDecoder;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    comp_decomp()?;

    Ok(())
}

fn comp_decomp() -> Result<(), Box<dyn Error>> {
    let num_symbols = 257;
    let precision = 44;
    let input_path = "./new.txt";
    let compressed_path = "./compressed.dat";
    let uncompressed_path = "./decompressed.txt";

    let mut source_model = SourceModel::new(num_symbols, 256);
    let mut encoder = ArithmeticEncoder::new(precision);

    let num_bytes = fs::metadata(input_path).unwrap().len();
    let mut byte_break = (num_bytes as f64 / 8f64) as u64;
    if byte_break < 2 {byte_break = 1};


    if true {
        let mut models = Vec::with_capacity(source_model.len() as usize);
        for i in 0..source_model.len() {
            models.push(SourceModel::new(num_symbols, 256));
        }
        let input_file = File::open(input_path)?;
        let mut buffer_input = BufReader::new(input_file);
        let mut output_file = File::create(compressed_path)?;
        let mut buffered_output = BufWriter::new(output_file);
        let mut input: BitReader<&mut BufReader<File>, MSB> = BitReader::new(&mut buffer_input);
        let mut out_writer = BitWriter::new(&mut buffered_output);
        let num_bytes = fs::metadata(input_path).unwrap().len();
        let encode_start = Instant::now();

        let mut current_model = &mut models[0];
        for x in 0..num_bytes {
            if x % byte_break == 0 {
                println!("{:.1}%", (x as f64 / num_bytes as f64) * 100.0);
            }
            let symbol: u32 = input.read_byte().unwrap() as u32;
            encoder.encode(symbol, current_model, &mut out_writer)?;
            current_model.add_symbol(symbol);
            current_model = &mut models[symbol as usize];
        }
        encoder.encode(current_model.get_eof(), current_model, &mut out_writer);
        encoder.finish_encode( &mut out_writer)?;
        out_writer.pad_to_byte()?;
        let finished = encode_start.elapsed().as_millis();
        println!("{:.2}Mbps", ((num_bytes as f64 * 8.0) / (finished as f64 / 1000.0)) / 1048576 as f64);
        buffered_output.flush()?;
    }

    let compressed_bytes = fs::metadata(compressed_path).unwrap().len();
    println!("input: {}, output: {}, compression ratio {}",
             num_bytes,
             compressed_bytes,
             num_bytes as f64 / compressed_bytes as f64);

    //decode
    if true {
        let input_file = File::open(compressed_path)?;
        let mut buffer_input = BufReader::new(input_file);
        let mut output_file = File::create(uncompressed_path)?;
        let mut buffered_output = BufWriter::new(output_file);
        let mut input: BitReader<_, MSB> = BitReader::new(&mut buffer_input);
        let mut out_writer = BitWriter::new(&mut buffered_output);
        source_model = SourceModel::new(num_symbols, 256);

        let mut models = Vec::with_capacity(source_model.len() as usize);
        for i in 0..source_model.len() {
            models.push(SourceModel::new(num_symbols, 256));
        }

        let mut decoder = ArithmeticDecoder::new(precision);
        let mut x = 0;
        let decode_start = Instant::now();

        let mut current_model = &mut models[0];

        while !decoder.is_finished() {
            if x % byte_break == 0 {
                println!("{:.1}%", (x as f64 / num_bytes as f64) * 100.0);
            }
            let sym = decoder.decode(current_model, &mut input)?;
//        let sym = c_decoder.decode(&mut decoder, &source_model, &mut input);
            if sym != current_model.get_eof() { out_writer.write_byte(sym as u8)?; }
            current_model.add_symbol(sym);
            current_model = &mut models[sym as usize];
            x += 1;
        }
        let finished = decode_start.elapsed().as_millis();
        println!("{:.2}Mbps", ((num_bytes as f64 * 8.0) / (finished as f64 / 1000.0)) / 1048576 as f64);

        buffered_output.flush()?;
    }

//
    let input1 = fs::File::open(input_path)?;
    let input2 = fs::File::open(uncompressed_path)?;
    let mut i1: BitReader<_, MSB> = BitReader::new(BufReader::new(input1));
    let mut i2: BitReader<_, MSB> = BitReader::new(BufReader::new(input2));
    if fs::metadata(uncompressed_path).unwrap().len() == fs::metadata(input_path).unwrap().len() {
        println!("files are same length");
        let mut matching = true;
        for i in 0..num_bytes {
            if i1.read_byte()? != i2.read_byte()? {
                matching = false;
            }
        }
        println!("Matching? {}", matching);
    } else {
        println!("files not same length")
    }
    Ok(())
}