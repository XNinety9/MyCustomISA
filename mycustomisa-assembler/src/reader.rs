use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_lines(path: &str) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");

    let reader = BufReader::new(file);

    let lines = reader
        .lines()
        .collect::<Result<_, _>>()
        .expect("Could not read file");

    lines
}