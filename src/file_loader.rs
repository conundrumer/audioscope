use std::io::prelude::*;
use std::fs::File;

pub fn load_from_file(filename: &str) -> String {
    let mut file = File::open(filename).unwrap_or_else(|e| panic!("couldn't open file {}: {}", filename, e));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|e| panic!("couldn't read file {}: {}", filename, e));
    contents
}
