use std::io::BufWriter;
use std::io::Write;

use pulldown_cmark::Parser;
use zet::core::parser::DocumentParser;
use zet::core::parser::FrontMatterFormat;
use zet::core::parser::FrontMatterParser;

use crate::app::preamble::*;
use zet::preamble::*;

/// Instead of producing an ast, this command simply outputs the event stream as
/// returned by pulldown_cmark
pub fn handle_command(front_matter_format: FrontMatterFormat, path: PathBuf) -> Result<()> {
    log::debug!("parsing {:?}", path);

    let frontmatter_parser = FrontMatterParser::new(front_matter_format);

    let document = std::fs::read_to_string(path)?;

    let (_, content) = frontmatter_parser.parse(document);

    let options = DocumentParser::default().options;
    let mut parser = Parser::new_ext(&content, options).into_offset_iter();

    let mut out = BufWriter::new(std::io::stdout());

    for (event, range) in parser {
        writeln!(out, "{:?}: {:?}", range, event)?;
    }

    Ok(())
}
