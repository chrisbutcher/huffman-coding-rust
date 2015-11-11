use std::collections::HashMap;

pub fn hash_map_reducer(hash_maps: Vec<HashMap<char, usize>>) -> HashMap<char, usize> {
  let mut result: HashMap<char, usize> = HashMap::new();

  for hash_map in hash_maps {
    for (key, val) in hash_map.iter() {
      let count = result.entry(*key).or_insert(0);
      *count += *val;
    }
  }

  result
}

pub fn string_to_substrings(input_string: &str, substring_count: usize) -> Vec<&str> {
  let input_string_length = input_string.len();
  let length_per_thread = input_string_length / substring_count;

  let mut substring_offsets = Vec::with_capacity(substring_count);

  for n in 1..substring_count {
    let offset = n * length_per_thread;
    substring_offsets.push(offset);
  }
  substring_offsets.push(input_string_length);

  let mut substrings = vec![];
  let mut from = 0;

  for substring_offset in substring_offsets {
    substrings.push(&input_string[from..substring_offset]);
    from = substring_offset;
  }

  substrings
}
