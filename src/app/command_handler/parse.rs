use zet::{config::Config, core::parser::FrontMatterParser};

use crate::app::preamble::*;
use zet::preamble::*;

pub fn handle_command(config: Config, path: PathBuf) -> Result<()> {
    log::debug!("parsing {:?}", path);

    let frontmatter_parser = FrontMatterParser::new(config.front_matter_format);
    let content_parser = zet::core::parser::DocumentParser::new();

    let document = std::fs::read_to_string(path)?;

    _ = zet::core::parser::parse(frontmatter_parser, content_parser, document)?;

    Ok(())
}
