extern crate num;

use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Equal, Greater};
use num::abs;

pub struct CompressionResult {
  bytes: Vec<u8>,
  bits_padded: u8
}

#[derive(Debug)]
struct Node {
  weight: usize,
  data: NodeData
}

#[derive(Debug)]
enum NodeData {
  Children(Links),
  Leaf(char)
}

#[derive(Debug)]
struct Links {
  left: Box<Node>,
  right: Box<Node>
}

impl Ord for Node {
  fn cmp(&self, other: &Node) -> Ordering {
    match self.weight.cmp(&other.weight) {
      Less => Greater,
      Equal => Equal,
      Greater => Less
    }
  }
}

impl PartialOrd for Node {
  fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Node {
}

impl PartialEq for Node {
  fn eq(&self, other: &Node) -> bool {
    self.weight == other.weight
  }
}

fn main() {
  let input_string = "MISSISSIPPI RIVER";
  let huffman_codebook = build_huffman_codebook(&input_string);

  let compressed = compress(&input_string, &huffman_codebook);

  println!("{:?} compressed as {:?}, with {:?} bits padded", input_string, compressed.bytes, compressed.bits_padded);

  let original_size = input_string.len() * 8;
  let compressed_size = compressed.bytes.len() * 8;
  let compression_ratio = compressed_size as f32 / original_size as f32;

  println!("Compressed bytes size {:?}, from {:?}. Ratio: {:?}", compressed_size, original_size, compression_ratio);
}

pub fn build_huffman_codebook(input: &str) -> HashMap<char,String> {
  let character_frequencies = map_chars_to_frequency(input);

  let mut priority_queue = build_priority_queue(&character_frequencies);
  let tree_root = build_tree(&mut priority_queue);

  let mut codebook = HashMap::<char,String>::new();
  build_codebook(&tree_root, &mut codebook, &"");

  codebook
}

pub fn compress(input_string: &str, codebook: &HashMap<char,String>) -> CompressionResult {
  let mut output_bytes: Vec<u8> = Vec::new();

  let mut current_byte: u8 = 0;
  let mut remaining_bits: i8 = 8; // TODO: instead, shift 1 << until it is 0x10000000 (checking with &)

  for ch in input_string.chars() {

    let code = codebook.get(&ch).unwrap();
    let code_length = code.len() as i8;

    if remaining_bits - code_length > 0 {
      current_byte = (current_byte << code_length) | binary_str_to_u8(code);
      remaining_bits -= code_length;

    } else if remaining_bits - code_length == 0 {
      current_byte = (current_byte << code_length) | binary_str_to_u8(code);
      output_bytes.push(current_byte);

      current_byte = 0;
      remaining_bits = 8;
    } else {
      let absolute_difference = (code_length - abs(remaining_bits - code_length)) as usize;

      let code_first = &code[0..absolute_difference];
      let code_first_length = code_first.len();

      current_byte = (current_byte << code_first_length) | binary_str_to_u8(code_first);
      output_bytes.push(current_byte);

      current_byte = 0;
      remaining_bits = 8;

      let code_rest = &code[absolute_difference..];
      let code_rest_length = code_rest.len() as i8;

      remaining_bits -= code_rest_length;

      current_byte = (current_byte << code_rest_length) | binary_str_to_u8(code_rest);
    }
  }

  if remaining_bits != 8 {
    current_byte = current_byte << remaining_bits;
    output_bytes.push(current_byte);
  }

  CompressionResult {
    bytes: output_bytes,
    bits_padded: (remaining_bits as u8)
  }
}

// TODO store the u8 version of all codes in the codebook, rather than recomputing each time
fn binary_str_to_u8(binary_str: &str) -> u8 {
  let mut result = 0u8;
  for ch in binary_str.chars() {
    if ch == '0' {
      result = (result << 1) | 0;
    } else {
      result = (result << 1) | 1;
    }
  }

  result
}

fn map_chars_to_frequency(input_string: &str) -> HashMap<char, usize> {
  let mut chars_to_frequency = HashMap::new();

  for ch in input_string.chars() {
    let count = chars_to_frequency.entry(ch).or_insert(0);
    *count += 1;
  }

  chars_to_frequency
}

fn build_priority_queue(character_frequencies: &HashMap<char, usize>) -> BinaryHeap<Node> {
  let mut priority_queue = BinaryHeap::<Node>::new();
  for (ch, freq) in character_frequencies.iter() {
    let node = Node {
      weight: *freq,
      data: NodeData::Leaf(*ch)
    };

    priority_queue.push(node);
  }

  priority_queue
}

fn build_tree(priority_queue: &mut BinaryHeap<Node>) -> Node {
  while priority_queue.len() > 1 {
    let popped_1 = priority_queue.pop().unwrap();
    let popped_2 = priority_queue.pop().unwrap();

    let combined_node = Node {
      weight: popped_1.weight + popped_2.weight,
      data: NodeData::Children(Links {
        left: Box::new(popped_1),
        right: Box::new(popped_2),
      })
    };
    priority_queue.push(combined_node);
  }

  priority_queue.pop().unwrap()
}

fn build_codebook(tree: &Node, codebook: &mut HashMap<char,String>, start_str: &str) {
  match tree.data {
    NodeData::Children(ref children) => {
      build_codebook(&children.left, codebook, &(start_str.to_string() + "0"));
      build_codebook(&children.right, codebook, &(start_str.to_string() + "1"));
    },
    NodeData::Leaf(ch) => {
      let insert_string = if start_str == "" {
        "0".to_string()
      } else {
        start_str.to_string()
      };

      codebook.insert(ch, insert_string);
    }
  }
}

#[test]
fn test_main() {
  // TODO
  main()
}

#[test]
fn test_compress() {
  let mut huffman_codebook = HashMap::<char,String>::new();
  huffman_codebook.insert('S', "00".to_string());
  huffman_codebook.insert('V', "0110".to_string());
  huffman_codebook.insert('E', "0111".to_string());
  huffman_codebook.insert('I', "11".to_string());
  huffman_codebook.insert('R', "010".to_string());
  huffman_codebook.insert('M', "1010".to_string());
  huffman_codebook.insert(' ', "1011".to_string());
  huffman_codebook.insert('P', "100".to_string());

  // 172      48       228      237      108      232          [decimal]
  // 10101100 00110000 11100100 11101101 01101100 11101000      [binary]
  // M   I S  S I S S  I P  P   I _   R   I V   E    R  ^(2 padded bits)

  let result = compress("MISSISSIPPI RIVER", &huffman_codebook);
  let expected = CompressionResult{
    bytes: vec![172u8, 48, 228, 237, 108, 232],
    bits_padded: 2u8
  };

  assert_eq!(expected.bytes, result.bytes);
}

#[test]
fn test_map_chars_to_frequency() {
  let result = map_chars_to_frequency("MISSISSIPPI RIVER");
  assert_eq!(5, result[&'I']);
  assert_eq!(4, result[&'S']);
  assert_eq!(2, result[&'P']);
  assert_eq!(2, result[&'R']);
  assert_eq!(1, result[&'M']);
  assert_eq!(1, result[&'V']);
  assert_eq!(1, result[&'E']);
  assert_eq!(1, result[&' ']);
}

#[test]
fn test_build_huffman_codebook() {
  let huffman_codebook = build_huffman_codebook("MISSISSIPPI RIVER");
  assert_eq!(2, huffman_codebook[&'I'].len());
  assert_eq!(2, huffman_codebook[&'S'].len());
  assert_eq!(3, huffman_codebook[&'P'].len());
  assert_eq!(3, huffman_codebook[&'R'].len());
  assert_eq!(4, huffman_codebook[&'M'].len());
  assert_eq!(4, huffman_codebook[&'V'].len());
  assert_eq!(4, huffman_codebook[&'E'].len());
  assert_eq!(4, huffman_codebook[&' '].len());
}

#[test]
fn test_build_huffman_codebook_with_one_letter() {
  let huffman_codebook = build_huffman_codebook("AAAAA");
  assert_eq!(1, huffman_codebook[&'A'].len());
}
