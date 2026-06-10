use std::fs::File;
use std::io::{BufWriter, Write};

pub fn write(path: &str, words: &[u16]) {
    let file = File::create(path).expect(&format!("Cannot create output file: {}", path));
    let mut writer = BufWriter::new(file);

    for word in words {
        writer.write_all(&word.to_be_bytes()).expect("Write failed");
    }
}