mod builder;
mod common;
mod encoder;
mod parser;
mod reader;
mod writer;

use std::env;

const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const MAGENTA: &str = "\x1b[95m";
const BLUE: &str = "\x1b[94m";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("{}\x1b[91merror:{} usage: assembler <input.asm>", BOLD, RST);
        std::process::exit(1);
    }

    let path = &args[1];

    let lines = reader::read_lines(path);
    println!("{}▸ reader{}    {}{}{} lines read", CYAN, RST, BOLD, lines.len(), RST);

    let parsed = parser::parse(&lines);
    println!("{}▸ parser{}    {}{}{} parsed lines", BLUE, RST, BOLD, parsed.len(), RST);

    let symbols = builder::build(&parsed);
    println!("{}▸ builder{}   {}{}{} symbols found", YELLOW, RST, BOLD, symbols.len(), RST);
    for (name, addr) in &symbols {
        println!("    {}{:<16}{} {}→{}  {}0x{:04X}{}", DIM, name, RST, DIM, RST, YELLOW, addr, RST);
    }

    let words = encoder::encode(&parsed, &symbols);
    println!("{}▸ encoder{}   {}{}{} words encoded", MAGENTA, RST, BOLD, words.len(), RST);

    let output_path = path.replace(".asm", ".bin");
    writer::write(&output_path, &words);
    println!("{}▸ writer{}    written to {}{}{}", GREEN, RST, BOLD, output_path, RST);
}
