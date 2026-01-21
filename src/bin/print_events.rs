use clap::Parser as CliParser;
use pulldown_cmark::Parser;
use std::{fs, path::PathBuf};
use zet::core::parser::{DocumentParser, DocumentParserOptions};

#[derive(CliParser)]
struct Cli {
    path: PathBuf,
}

fn main() {
    let Cli { path } = Cli::parse();
    let markdown_input =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {:?}", &path));

    let options = DocumentParserOptions::default();
    let parser = Parser::new_ext(&markdown_input, options.0).into_offset_iter();

    println!("=== Parsing CLAUDE.md ===\n");
    println!("{}\n", markdown_input);
    println!("=== pulldown-cmark Events ===\n");

    for (event, range) in parser {
        println!("range: {:?}, event: {:?}", range, event);
    }
}
