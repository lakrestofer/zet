use clap::Parser as CliParser;
use pulldown_cmark::Parser;
use std::{fs, path::PathBuf};
use zet::parser::DocumentParser;

#[derive(CliParser)]
struct Cli {
    path: PathBuf,
}

fn main() {
    let Cli { path } = Cli::parse();
    let markdown_input = fs::read_to_string(&path).expect(&format!("Failed to read {:?}", &path));

    let parser = DocumentParser::default();
    let parser = Parser::new_ext(&markdown_input, parser.options).into_offset_iter();

    println!("=== Parsing CLAUDE.md ===\n");
    println!("{}\n", markdown_input);
    println!("=== pulldown-cmark Events ===\n");

    for (event, range) in parser {
        println!("range: {:?}, event: {:?}", range, event);
    }
}
