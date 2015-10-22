use std::env;
use std::io::prelude::*;
use std::fs::File;

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

  let input_string = if args.len() > 1 {
    read_file_to_string(&args[1])
  } else {
    "MISSISSIPPI RIVER".to_string()
  };

  println!("Building codebook");
  let huffman_codebook = huffman::build_huffman_codebook(&input_string);

  println!("Compressing");
  let compressed = huffman::compress(&input_string, &huffman_codebook);

  let original_size = input_string.len() * 8;
  let compressed_size = compressed.bytes.len() * 8;
  let compression_ratio = compressed_size as f32 / original_size as f32;

  println!("Compressed bytes size {:?}, from {:?}. Ratio: {:?}", compressed_size, original_size, compression_ratio);
}

#[test]
fn test_main() {
  main();
}
