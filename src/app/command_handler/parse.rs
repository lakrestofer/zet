use std::io::BufWriter;
use std::io::Write;

use zet::core::parser::FrontMatterFormat;
use zet::core::parser::FrontMatterParser;

use crate::app::preamble::*;
use zet::preamble::*;

pub fn handle_command(
    front_matter_format: FrontMatterFormat,
    pretty_print: bool,
    path: PathBuf,
) -> Result<()> {
    log::debug!("parsing {:?}", path);

    let frontmatter_parser = FrontMatterParser::new(front_matter_format);
    let content_parser = zet::core::parser::DocumentParser::new();

    let document = std::fs::read_to_string(path)?;

    let (frontmatter, content) =
        zet::core::parser::parse(frontmatter_parser, content_parser, document)?;

    let frontmatter = serde_json::to_value(frontmatter)?;
    let content = serde_json::to_value(content)?;
    let mut res = serde_json::Map::new();
    res.insert("frontmatter".into(), frontmatter);
    res.insert("content".into(), content);

    let out = BufWriter::new(std::io::stdout());

    if pretty_print {
        serde_json::to_writer_pretty(out, &res)?;
    } else {
        serde_json::to_writer(out, &res)?;
    }

    Ok(())
}
