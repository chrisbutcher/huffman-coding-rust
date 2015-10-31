extern crate num_cpus;
extern crate getopts;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use getopts::Options;

mod huffman;

fn read_file_to_string (filename: &str) -> String {
  let mut input_string = String::new();

  let mut file = match File::open(filename) {
    Ok(f) => f,
    Err(_) => { panic!("Cannot open input file") }
  };

  let _ = file.read_to_string(&mut input_string);
  input_string
}

fn main() {
  let args: Vec<String> = env::args().collect();

  let mut opts = Options::new();
  opts.optopt("f", "", "set input file name", "FILENAME");
  opts.optflag("p", "parallel", "compress file with multiple threads");

  let option_matches = match opts.parse(&args[1..]) {
      Ok(m) => { m }
      Err(f) => { panic!(f.to_string()) }
  };

  let input_string = if option_matches.opt_present("f") {
    let input_filename = option_matches.opt_str("f").unwrap().to_string();
    read_file_to_string(&*input_filename)
  } else {
    "MISSISSIPPI RIVER".to_string()
  };

  println!("Building codebook");
  let huffman_codebook = huffman::build_huffman_codebook(&input_string);

  if option_matches.opt_present("p") {
    println!("Compressing in parallel");
    let compressed_set = huffman::parallel_compress(&input_string, &huffman_codebook, num_cpus::get());
    println!("Done!");
  } else {
    println!("Compressing");
    let compressed = huffman::compress(&input_string, &huffman_codebook);
    let original_size = input_string.len() * 8;
    let compressed_size = compressed.bytes.len() * 8;
    let compression_ratio = compressed_size as f32 / original_size as f32;

    println!("Compressed bytes size {:?}, from {:?}. Ratio: {:?}", compressed_size, original_size, compression_ratio);
  }
}

#[test]
fn test_main() {
  main();
}
