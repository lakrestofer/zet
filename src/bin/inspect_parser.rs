use pulldown_cmark::Parser;
use std::fs;
use zet::core::parser::DocumentParserOptions;

fn main() {
    let markdown_input = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    let options = DocumentParserOptions::default();
    let parser = Parser::new_ext(&markdown_input, options.0).into_offset_iter();

    println!("=== Parsing CLAUDE.md ===\n");
    println!("{}\n", markdown_input);
    println!("=== pulldown-cmark Events ===\n");

    for (event, range) in parser {
        println!("range: {:?}, event: {:?}", range, event);
    }
}
