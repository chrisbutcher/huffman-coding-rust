extern crate num;

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use codebook;

const PROGRESS_FULL_BYTE: u8 = 255u8;

#[derive(Debug)]
pub struct CompressionResult {
  pub bytes: Vec<u8>,
  pub bits_padded: u8
}

// TODO: Output file layout. Header indicating number of blocks, offsets to them, offset to dictionary
// [header][dictionary][block0][block1][block2][block3]

// Why take in an Arc (atomic reference counted) codebook? https://www.reddit.com/r/rust/comments/2w75wr/how_do_i_read_immutable_vector_inside_a_spawned/coocmn7
pub fn parallel_compress(input_string: &str, codebook: Arc<codebook::Codebook>, thread_count: usize) -> Vec<CompressionResult> {
  let substrings = parallel_divide_work(thread_count, &input_string);
  let mut threads = vec![];

  for substring in substrings {
    let owned_codebook = codebook.clone();
    threads.push(thread::spawn(move || {
      compress(&substring, &owned_codebook)
    }));
  }

  let mut results = vec![];
  for thread in threads {
    let result = thread.join().unwrap();

    results.push(result);
  }

  results
}

pub fn compress(input_string: &str, codebook: &codebook::Codebook) -> CompressionResult {
  let mut output_bytes: Vec<u8> = Vec::new();
  let mut byte_buffer: u8 = 0u8;
  let mut progress_byte: u8 = 0u8;
  let mut bits_padded = 0;

  for ch in input_string.chars() {
    let code = codebook.character_map.get(&ch).unwrap();
    for code_ch in code.chars() {
      let new_bit = if code_ch == '0' { 0 } else { 1 };
      byte_buffer = (byte_buffer << 1) | new_bit;
      progress_byte = (progress_byte << 1) | 1;

      if progress_byte == PROGRESS_FULL_BYTE {
        let byte_to_push = byte_buffer;
        output_bytes.push(byte_to_push);
        progress_byte = 0u8;
      }
    }
  }

  if progress_byte != 0u8 && progress_byte.leading_zeros() > 0 {
    bits_padded = 8 - (8 - progress_byte.leading_zeros()) % 8;
    for _ in 0..bits_padded {
      byte_buffer = (byte_buffer << 1) | 0;
    }

    output_bytes.push(byte_buffer);
  }

  CompressionResult {
    bytes: output_bytes,
    bits_padded: (bits_padded as u8)
  }
}

fn parallel_divide_work(thread_count: usize, input_string: &str) -> Vec<String> {
  let thread_count = thread_count;
  let input_string_length = input_string.len();
  let length_per_thread = input_string_length / thread_count;

  let mut substring_offsets = Vec::with_capacity(thread_count);

  for cpu in 1..thread_count {
    let offset = cpu * length_per_thread;
    substring_offsets.push(offset);
  }
  substring_offsets.push(input_string_length);

  let mut substrings = vec![];
  let mut from = 0;

  for substring_offset in substring_offsets {
    substrings.push(input_string[from..substring_offset].to_string().to_owned());
    from = substring_offset;
  }

  substrings
}

#[test]
fn test_compress() {
  let mut char_map = HashMap::<char,String>::new();
  char_map.insert('S', "00".to_string());
  char_map.insert('V', "0110".to_string());
  char_map.insert('E', "0111".to_string());
  char_map.insert('I', "11".to_string());
  char_map.insert('R', "010".to_string());
  char_map.insert('M', "1010".to_string());
  char_map.insert(' ', "1011".to_string());
  char_map.insert('P', "100".to_string());

  let codebook = codebook::Codebook { character_map: char_map };

  // 172      48       228      237      108      232          [decimal]
  // 10101100 00110000 11100100 11101101 01101100 11101000      [binary]
  // M   I S  S I S S  I P  P   I _   R   I V   E    R  ^(2 padded bits)

  let result = compress("MISSISSIPPI RIVER", &codebook);
  let expected = CompressionResult{
    bytes: vec![172u8, 48, 228, 237, 108, 232],
    bits_padded: 2u8
  };

  assert_eq!(expected.bytes, result.bytes);
}

#[test]
fn test_parallel_compress() {
  let mut char_map = HashMap::<char,String>::new();
  char_map.insert('S', "00".to_string());
  char_map.insert('V', "0110".to_string());
  char_map.insert('E', "0111".to_string());
  char_map.insert('I', "11".to_string());
  char_map.insert('R', "010".to_string());
  char_map.insert('M', "1010".to_string());
  char_map.insert(' ', "1011".to_string());
  char_map.insert('P', "100".to_string());

  let codebook = codebook::Codebook { character_map: char_map };
  let check_codebook = codebook.clone();

  let parallel_results = parallel_compress("MISSISSIPPI RIVER", Arc::new(codebook), 4);

  let expecteds = vec![compress("MISS", &check_codebook),
                       compress("ISSI", &check_codebook),
                       compress("PPI ", &check_codebook),
                       compress("RIVER", &check_codebook)];

  let mut i = 0;
  for expected in expecteds {
      assert_eq!(expected.bytes, parallel_results[i].bytes);
      assert_eq!(expected.bits_padded, parallel_results[i].bits_padded);
      i = i + 1;
  }
}
