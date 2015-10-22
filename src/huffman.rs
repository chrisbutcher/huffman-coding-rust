extern crate num;

use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Equal, Greater};

const PROGRESS_FULL_BYTE: u8 = 255u8;

pub struct CompressionResult {
  pub bytes: Vec<u8>,
  pub bits_padded: u8
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
  let mut byte_buffer: u8 = 0u8;
  let mut progress_byte: u8 = 0u8;
  let mut bits_padded = 0;

  for ch in input_string.chars() {
    let code = codebook.get(&ch).unwrap();
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

  if progress_byte.leading_zeros() > 0 {
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
    let node = Node { weight: *freq, data: NodeData::Leaf(*ch) };
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
