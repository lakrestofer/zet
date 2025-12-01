use zet::core::parser::FrontMatterFormat;

pub mod index;
pub mod init;
pub mod parse;

use crate::app::preamble::*;
use zet::preamble::*;

pub fn handle_command(command: Command, root: Option<PathBuf>) -> Result<()> {
    match command {
        Command::Init { root, force } => init::handle_command(root, force)?,
        command => {
            let config = zet::config::Config {
                root: zet::core::paths::resolve_root(root)?,
                front_matter_format: FrontMatterFormat::Toml,
            };

            log::debug!("root: {:?}", config.root);

            match command {
                Command::Parse { path } => parse::handle_command(config, path)?,
                Command::Index => index::handle_command(config)?,
                Command::Lsp => todo!(),
                Command::Format => todo!(),
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}
