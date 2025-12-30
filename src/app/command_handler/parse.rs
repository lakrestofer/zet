use std::io::BufWriter;
use std::io::Write;

use zet::{config::Config, core::parser::FrontMatterParser};

use crate::app::preamble::*;
use zet::preamble::*;

pub fn handle_command(config: Config, path: PathBuf) -> Result<()> {
    log::debug!("parsing {:?}", path);

    let frontmatter_parser = FrontMatterParser::new(config.front_matter_format);
    let content_parser = zet::core::parser::DocumentParser::new();

    let document = std::fs::read_to_string(path)?;

    let (frontmatter, content) =
        zet::core::parser::parse(frontmatter_parser, content_parser, document)?;

    let frontmatter = serde_json::to_value(frontmatter)?;
    let content = serde_json::to_value(content)?;
    let mut res = serde_json::Map::new();
    res.insert("frontmatter".into(), frontmatter);
    res.insert("content".into(), content);

    let res = serde_json::to_string(&res)?;

    let mut out = BufWriter::new(std::io::stdout());
    write!(out, "{}", res)?;

    Ok(())
}
