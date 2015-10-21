mod huffman;

fn main() {
  let input_string = "MISSISSIPPI RIVER";
  let huffman_codebook = huffman::build_huffman_codebook(&input_string);

  let compressed = huffman::compress(&input_string, &huffman_codebook);

  println!("{:?} compressed as {:?}, with {:?} bits padded", input_string, compressed.bytes, compressed.bits_padded);

  let original_size = input_string.len() * 8;
  let compressed_size = compressed.bytes.len() * 8;
  let compression_ratio = compressed_size as f32 / original_size as f32;

  println!("Compressed bytes size {:?}, from {:?}. Ratio: {:?}", compressed_size, original_size, compression_ratio);
}

#[test]
fn test_main() {
  main();
}
