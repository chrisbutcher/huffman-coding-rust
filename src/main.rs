// External crates, via Cargo.toml
extern crate num_cpus;
extern crate getopts;

// https://doc.rust-lang.org/getopts/getopts/index.html

// From std library
use std::env;
use std::io::prelude::*;
use std::fs::File;
use getopts::Options;

// Internal modules
mod codebook;
mod compress;
mod util;

fn read_file_to_string (filename: &str) -> String {
  let mut input_string = String::new();

  // match on Result, if I/O succeeded, access via Ok() which converts to Options
  // Results succeed (Ok) or fail (Err)
  // Options have values (Some) or do not (None)

  let mut file = match File::open(filename) {
    Ok(f) => f,
    Err(_) => { panic!("Cannot open input file") } // '_' for when a variable is returned, but not used, same as in go
  };

  // Giving a method a mutable reference (unused return value is number of bytes read)
  let _ = file.read_to_string(&mut input_string);
  input_string
}

fn print_summary(compression_results: Vec<compress::CompressionResult>, original_size: usize) {
  println!("Done! Threads used: {}", compression_results.len());
  let compressed_size = compression_results.iter().fold(0, |acc, ref result|  acc + result.bytes.len());
  let compression_ratio = compressed_size as f32 / original_size as f32;
  println!("Compressed bytes size {:?}, from {:?}. Ratio: {:?}", compressed_size, original_size, compression_ratio);
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
    let input_filename = match option_matches.opt_str("f") {
      Some(filename) => filename.to_string(),
      None => { panic!("-f option used but no file name specified!")  },
    };
    read_file_to_string(&*input_filename)
  } else {
    "MISSISSIPPI RIVER".to_string()
  };

  let num_threads = if option_matches.opt_present("p") { num_cpus::get() } else { 1 };

  let input_substrings = util::string_to_substrings(&input_string, num_threads);
  let huffman_codebook = codebook::Codebook::new(&input_substrings);
  let compression_results = compress::parallel_compress(&input_substrings, &huffman_codebook);

  print_summary(compression_results, input_string.len());
}

// Running main in test, just to avoid test warning
#[test]
fn test_main() {
  main();
}
