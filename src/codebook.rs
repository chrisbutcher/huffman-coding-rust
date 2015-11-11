extern crate num_cpus;
extern crate crossbeam;

use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Equal, Greater};

use util;

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

pub struct Codebook {
  pub character_map: HashMap<char,String>
}

impl Clone for Codebook {
    fn clone(&self) -> Codebook {
      Codebook { character_map: self.character_map.clone() }
    }
}

impl Codebook {
  pub fn new(input_strings: &Vec<&str>) -> Codebook {
    let character_frequencies = parallel_map_chars_to_frequency(&input_strings);
    let mut priority_queue = build_priority_queue(&character_frequencies);
    let tree_root = build_tree(&mut priority_queue);
    let mut character_map = HashMap::<char,String>::new();
    build_codebook(&tree_root, &mut character_map, &"");

    Codebook { character_map: character_map}
  }
}

fn parallel_map_chars_to_frequency(substrings: &Vec<&str>) -> HashMap<char, usize> {
  let mut threads = vec![];

  for substring in substrings {
    crossbeam::scope(|scope| {
      threads.push(scope.spawn(move || {
        map_chars_to_frequency(&substring)
      }));
    });
  }

  let mut results = vec![];
  for thread in threads {
    let result = thread.join();

    results.push(result);
  }

  util::hash_map_reducer(results)
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
  let codebook = Codebook::new(&vec![&"MISSISSIPPI RIVER"]);
  assert_eq!(2, codebook.character_map[&'I'].len());
  assert_eq!(2, codebook.character_map[&'S'].len());
  assert_eq!(3, codebook.character_map[&'P'].len());
  assert_eq!(3, codebook.character_map[&'R'].len());
  assert_eq!(4, codebook.character_map[&'M'].len());
  assert_eq!(4, codebook.character_map[&'V'].len());
  assert_eq!(4, codebook.character_map[&'E'].len());
  assert_eq!(4, codebook.character_map[&' '].len());
}

#[test]
fn test_build_huffman_codebook_with_one_letter() {
  let codebook = Codebook::new(&vec![&"AAAAA"]);
  assert_eq!(1, codebook.character_map[&'A'].len());
}
